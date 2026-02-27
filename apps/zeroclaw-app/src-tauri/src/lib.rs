#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::{Context, Result};
use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use chrono::Utc;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::collections::{BTreeMap, HashMap};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex;
use zeroclaw_core::{
    channel_add, channel_bind_telegram, channel_remove, channels_list, cost_summary,
    create_pairing_bundle, cron_add, cron_list, cron_pause, cron_remove, cron_resume,
    migrate_openclaw, protocol_handshake as core_protocol_handshake, providers_catalog,
    refresh_models, response_cache_stats, run_channel_doctor, run_doctor, run_service_lifecycle,
    status_report, AccessPlan, AccessState, ActionPolicyDecision, ActionPolicyRequest,
    ActionReceipt, AdaptiveSecretVault, AgentRuntime, ApprovalRequest, BackgroundCapabilities,
    ChannelSummary, ControlPlaneState, ControlPlaneStore, CostSummaryReport, CronJobSummary,
    IntegrationPermissionContract, IntegrationRecord, IntegrationRegistry,
    IntegrationRegistryStore, JsonlLogSink, LocalAgentRuntime, LogLine, LogSink, LogSinkConfig,
    McpConnectorConfig, McpConnectorInstallRequest, McpConnectorRecord, McpConnectorRegistry,
    McpConnectorStore, OperationResult, PairingBundle, PairingRequest, PairingTransport,
    PlatformBackground, ProfileManager, ProfileRecord, ProfilesIndex, ProviderDescriptor,
    PurgeSummary, ResponseCacheStatsReport, RetentionPolicy, RuntimeStartConfig, SecretVault,
    ServiceLifecycleAction, SkillInstallRequest, SkillRecord, SkillsRegistry, SkillsRegistryStore,
    StatusReport, WorkspaceView,
};

struct RuntimeSlot {
    runtime: Option<Arc<LocalAgentRuntime>>,
    log_sink: Option<Arc<JsonlLogSink>>,
    profile_id: Option<String>,
}

impl RuntimeSlot {
    fn new() -> Self {
        Self {
            runtime: None,
            log_sink: None,
            profile_id: None,
        }
    }
}

struct AppController {
    profile_manager: ProfileManager,
    app_root: PathBuf,
    vault: Arc<dyn SecretVault>,
    runtime_slot: Mutex<RuntimeSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct IntegrationCatalogEntry {
    name: String,
    description: String,
    category: String,
    status: String,
    setup_hint: String,
}

impl AppController {
    fn new() -> Result<Self> {
        let root = ProfileManager::default_root()?;
        let profile_manager = ProfileManager::new(root.clone());
        profile_manager.ensure_layout()?;

        let vault: Arc<dyn SecretVault> = Arc::new(AdaptiveSecretVault::new(&root)?);

        let controller = Self {
            profile_manager,
            app_root: root,
            vault,
            runtime_slot: Mutex::new(RuntimeSlot::new()),
        };

        controller.ensure_default_profile()?;
        Ok(controller)
    }

    fn ensure_default_profile(&self) -> Result<()> {
        let index = self.profile_manager.load_index()?;
        if index.profiles.is_empty() {
            let _ = self.profile_manager.create_profile("default")?;
        }
        Ok(())
    }

    fn active_profile_fallback(&self) -> Result<ProfileRecord> {
        if let Some(active) = self.profile_manager.get_active_profile()? {
            return Ok(active);
        }

        let created = self.profile_manager.create_profile("default")?;
        Ok(created)
    }

    fn control_plane_store_for_profile(&self, profile_id: &str) -> Result<ControlPlaneStore> {
        let workspace = self
            .profile_manager
            .workspace_for_profile(profile_id)
            .with_context(|| format!("failed to resolve profile workspace for '{profile_id}'"))?;
        Ok(ControlPlaneStore::for_workspace(&workspace.root_dir))
    }
}

const PROFILE_SETUP_FILE: &str = ".right-hand-profile.json";
const RBAC_FILE: &str = ".right-hand-rbac.json";
const CLIENT_CONNECTION_FILE: &str = ".right-hand-client-connection.json";
const ROLLOUT_STATE_FILE: &str = ".right-hand-rollout.json";
const AUDIT_LOG_FILE: &str = ".right-hand-audit.jsonl";
const OUTCOMES_FILE: &str = ".right-hand-outcomes.json";
const POLICY_PROFILE_FILE: &str = ".right-hand-policy-profile.json";
const AUDIT_REMOTE_FILE: &str = ".right-hand-audit-remote.json";
const BILLING_STATE_FILE: &str = ".right-hand-billing.json";
const WORKFLOW_BOARD_FILE: &str = ".right-hand-workflow-board.json";
const COMPLIANCE_PROFILE_FILE: &str = ".right-hand-compliance-profile.json";

fn default_orchestrator_mode() -> String {
    "single_orchestrator".to_string()
}

fn setup_default_temperature() -> f64 {
    0.7
}

fn setup_default_agent_max_tool_iterations() -> u32 {
    10
}

fn setup_default_agent_max_history_messages() -> u32 {
    50
}

fn setup_default_agent_tool_dispatcher() -> String {
    "auto".to_string()
}

fn setup_default_skills_prompt_injection_mode() -> String {
    "full".to_string()
}

fn default_enable_tool_connectors() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SetupWorkspaceMode {
    #[serde(alias = "personal", alias = "org")]
    Workspace,
}

impl Default for SetupWorkspaceMode {
    fn default() -> Self {
        Self::Workspace
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum DeploymentMode {
    Host,
    Client,
}

fn default_deployment_mode() -> DeploymentMode {
    if cfg!(any(target_os = "android", target_os = "ios")) {
        DeploymentMode::Client
    } else {
        DeploymentMode::Host
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WorkspaceRole {
    #[serde(alias = "owner")]
    Admin,
    Manager,
    #[serde(alias = "operator")]
    User,
    #[serde(alias = "viewer")]
    Observer,
}

fn default_workspace_role() -> WorkspaceRole {
    WorkspaceRole::Admin
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SubscriptionTier {
    #[serde(alias = "starter")]
    Basic,
    #[serde(alias = "pro")]
    Professional,
    Enterprise,
}

fn default_subscription_tier() -> SubscriptionTier {
    SubscriptionTier::Professional
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DelegateAgentSetup {
    provider: String,
    model: String,
    #[serde(default)]
    system_prompt: Option<String>,
    #[serde(default)]
    temperature: Option<f64>,
    #[serde(default)]
    max_depth: Option<u32>,
    #[serde(default)]
    agentic: bool,
    #[serde(default)]
    allowed_tools: Vec<String>,
    #[serde(default)]
    max_iterations: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ProfileSetupState {
    user_display_name: String,
    agent_name: String,
    #[serde(default)]
    workspace_mode: SetupWorkspaceMode,
    #[serde(default = "default_deployment_mode")]
    deployment_mode: DeploymentMode,
    #[serde(default = "default_workspace_role")]
    workspace_role: WorkspaceRole,
    #[serde(default = "default_subscription_tier")]
    subscription_tier: SubscriptionTier,
    #[serde(default = "default_orchestrator_mode")]
    orchestrator_mode: String,
    provider: String,
    model: String,
    #[serde(default)]
    api_url: Option<String>,
    #[serde(default = "setup_default_temperature")]
    default_temperature: f64,
    memory_backend: String,
    #[serde(default)]
    runtime_reasoning_enabled: Option<bool>,
    #[serde(default)]
    agent_compact_context: bool,
    #[serde(default)]
    agent_parallel_tools: bool,
    #[serde(default = "setup_default_agent_max_tool_iterations")]
    agent_max_tool_iterations: u32,
    #[serde(default = "setup_default_agent_max_history_messages")]
    agent_max_history_messages: u32,
    #[serde(default = "setup_default_agent_tool_dispatcher")]
    agent_tool_dispatcher: String,
    #[serde(default = "setup_default_skills_prompt_injection_mode")]
    skills_prompt_injection_mode: String,
    #[serde(default)]
    skills_open_enabled: bool,
    #[serde(default)]
    skills_open_dir: Option<String>,
    #[serde(
        default = "default_enable_tool_connectors",
        alias = "enable_mcp_connectors"
    )]
    enable_tool_connectors: bool,
    #[serde(default)]
    delegate_agents: BTreeMap<String, DelegateAgentSetup>,
    has_provider_key: bool,
    provider_key_id: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ProfileSetupPayload {
    user_display_name: String,
    agent_name: String,
    #[serde(default)]
    workspace_mode: SetupWorkspaceMode,
    #[serde(default = "default_deployment_mode")]
    deployment_mode: DeploymentMode,
    #[serde(default = "default_workspace_role")]
    workspace_role: WorkspaceRole,
    #[serde(default = "default_subscription_tier")]
    subscription_tier: SubscriptionTier,
    #[serde(default = "default_orchestrator_mode")]
    orchestrator_mode: String,
    provider: String,
    model: String,
    #[serde(default)]
    api_url: Option<String>,
    #[serde(default = "setup_default_temperature")]
    default_temperature: f64,
    memory_backend: String,
    #[serde(default)]
    runtime_reasoning_enabled: Option<bool>,
    #[serde(default)]
    agent_compact_context: bool,
    #[serde(default)]
    agent_parallel_tools: bool,
    #[serde(default = "setup_default_agent_max_tool_iterations")]
    agent_max_tool_iterations: u32,
    #[serde(default = "setup_default_agent_max_history_messages")]
    agent_max_history_messages: u32,
    #[serde(default = "setup_default_agent_tool_dispatcher")]
    agent_tool_dispatcher: String,
    #[serde(default = "setup_default_skills_prompt_injection_mode")]
    skills_prompt_injection_mode: String,
    #[serde(default)]
    skills_open_enabled: bool,
    #[serde(default)]
    skills_open_dir: Option<String>,
    #[serde(
        default = "default_enable_tool_connectors",
        alias = "enable_mcp_connectors"
    )]
    enable_tool_connectors: bool,
    #[serde(default)]
    delegate_agents: BTreeMap<String, DelegateAgentSetup>,
    api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AuthProfileSummary {
    id: String,
    provider: String,
    profile_name: String,
    kind: String,
    active: bool,
    account_id: Option<String>,
    workspace_id: Option<String>,
    expires_at: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct MemoryEntrySummary {
    id: String,
    key: String,
    category: String,
    timestamp: String,
    session_id: Option<String>,
    score: Option<f64>,
    content_preview: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct CommandSurfaceCapability {
    family: String,
    supported: bool,
    coverage: String,
    note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct DeploymentCapabilities {
    platform: String,
    supports_host: bool,
    supports_client: bool,
    configured_mode: DeploymentMode,
    effective_mode: DeploymentMode,
    workspace_mode: SetupWorkspaceMode,
    workspace_role: WorkspaceRole,
    subscription_tier: SubscriptionTier,
    note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct HostConnectionState {
    connected: bool,
    endpoint: Option<String>,
    transport: Option<String>,
    pairing_token_hint: Option<String>,
    connected_at: Option<String>,
    updated_at: String,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct HostConnectPayload {
    invite_payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RbacUserRecord {
    user_id: String,
    display_name: String,
    role: WorkspaceRole,
    active: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RbacRegistry {
    version: u32,
    users: Vec<RbacUserRecord>,
    updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RolloutRing {
    Pilot,
    Group,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ReleaseDescriptor {
    release_id: String,
    version: String,
    checksum_sha256: String,
    signature: Option<String>,
    sbom_checksum_sha256: Option<String>,
    ring: RolloutRing,
    staged_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RolloutState {
    version: u32,
    current_release: Option<ReleaseDescriptor>,
    previous_release: Option<ReleaseDescriptor>,
    staged_release: Option<ReleaseDescriptor>,
    signature_required: bool,
    trusted_signers: Vec<String>,
    last_verified_signer: Option<String>,
    last_promoted_at: Option<String>,
    last_verification_error: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RolloutStageRequest {
    release_id: String,
    version: String,
    checksum_sha256: String,
    signature: Option<String>,
    sbom_checksum_sha256: Option<String>,
    ring: RolloutRing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RolloutSigningPolicyRequest {
    signature_required: bool,
    trusted_signers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PolicyProfileTemplate {
    template_id: String,
    display_name: String,
    description: String,
    allowed_providers: Vec<String>,
    allowed_transports: Vec<String>,
    allow_public_bind: bool,
    require_pairing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct PolicyProfileState {
    template_id: String,
    applied_at: String,
    allowed_providers: Vec<String>,
    allowed_transports: Vec<String>,
    allow_public_bind: bool,
    require_pairing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AuditEvent {
    id: String,
    timestamp: String,
    actor_id: String,
    actor_role: String,
    action: String,
    resource: String,
    destination: String,
    result: String,
    reason: String,
    receipt_id: String,
    approval_id: Option<String>,
    prev_hash: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AuditLogVerification {
    valid: bool,
    entries: usize,
    last_hash: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AuditRemoteSinkState {
    version: u32,
    enabled: bool,
    endpoint: Option<String>,
    sink_kind: String,
    auth_secret_id: Option<String>,
    verify_tls: bool,
    batch_size: usize,
    last_synced_hash: Option<String>,
    last_synced_at: Option<String>,
    last_error: Option<String>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct AuditRemoteConfigureRequest {
    enabled: bool,
    endpoint: Option<String>,
    sink_kind: Option<String>,
    auth_secret_id: Option<String>,
    verify_tls: Option<bool>,
    batch_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct AuditRemoteSyncResult {
    endpoint: String,
    sink_kind: String,
    events_sent: usize,
    first_hash: Option<String>,
    last_hash: Option<String>,
    synced_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BillingEntitlementStatus {
    Active,
    Grace,
    Expired,
    Unverified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct BillingEntitlement {
    tier: SubscriptionTier,
    status: BillingEntitlementStatus,
    verified: bool,
    source: String,
    account_id: Option<String>,
    entitlement_id: Option<String>,
    receipt_id: Option<String>,
    expires_at: Option<String>,
    last_verified_at: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct BillingState {
    version: u32,
    backend_url: Option<String>,
    auth_secret_id: Option<String>,
    enforce_verification: bool,
    entitlement: BillingEntitlement,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct BillingConfigRequest {
    backend_url: Option<String>,
    auth_secret_id: Option<String>,
    enforce_verification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct BillingReceiptVerifyRequest {
    receipt_payload: String,
    platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct BillingVerificationResponse {
    valid: bool,
    tier: Option<SubscriptionTier>,
    status: Option<BillingEntitlementStatus>,
    account_id: Option<String>,
    entitlement_id: Option<String>,
    receipt_id: Option<String>,
    expires_at: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WorkflowTaskStatus {
    Pending,
    InProgress,
    Done,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum WorkflowTaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowTaskRecord {
    id: String,
    title: String,
    description: Option<String>,
    status: WorkflowTaskStatus,
    priority: WorkflowTaskPriority,
    owner: Option<String>,
    workspace_scope: String,
    runtime_task_id: Option<String>,
    agent_id: Option<String>,
    skill_id: Option<String>,
    tool_id: Option<String>,
    tags: Vec<String>,
    risk_score: f64,
    related_receipt_id: Option<String>,
    created_at: String,
    updated_at: String,
    started_at: Option<String>,
    completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowBoardState {
    version: u32,
    tasks: Vec<WorkflowTaskRecord>,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowBoardSummary {
    total: usize,
    pending: usize,
    in_progress: usize,
    done: usize,
    failed: usize,
    blocked: usize,
    high_risk_open: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowBoardView {
    summary: WorkflowBoardSummary,
    tasks: Vec<WorkflowTaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowTaskUpsertRequest {
    id: Option<String>,
    title: String,
    description: Option<String>,
    status: Option<WorkflowTaskStatus>,
    priority: Option<WorkflowTaskPriority>,
    owner: Option<String>,
    runtime_task_id: Option<String>,
    agent_id: Option<String>,
    skill_id: Option<String>,
    tool_id: Option<String>,
    tags: Option<Vec<String>>,
    risk_score: Option<f64>,
    related_receipt_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct WorkflowTaskMoveRequest {
    task_id: String,
    status: WorkflowTaskStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ComplianceProfileTemplate {
    template_id: String,
    display_name: String,
    description: String,
    industry: String,
    standards: Vec<String>,
    recommended_policy_template: Option<String>,
    minimum_tier: SubscriptionTier,
    require_signed_release: bool,
    require_remote_audit: bool,
    require_billing_verification: bool,
    require_pairing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct ComplianceProfileState {
    template_id: String,
    applied_at: String,
    industry: String,
    standards: Vec<String>,
    recommended_policy_template: Option<String>,
    minimum_tier: SubscriptionTier,
    require_signed_release: bool,
    require_remote_audit: bool,
    require_billing_verification: bool,
    require_pairing: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct ComplianceControlCheck {
    control_id: String,
    label: String,
    framework: String,
    required: bool,
    satisfied: bool,
    evidence: Option<String>,
    recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct CompliancePosture {
    template_id: Option<String>,
    standards: Vec<String>,
    compliant: bool,
    generated_at: String,
    checks: Vec<ComplianceControlCheck>,
    missing_controls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum OutcomeStatus {
    Solved,
    Partial,
    Unsolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct OutcomeRecord {
    id: String,
    timestamp: String,
    title: String,
    status: OutcomeStatus,
    impact_score: f64,
    owner: Option<String>,
    related_receipt_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct OutcomeUpsertRequest {
    title: String,
    status: OutcomeStatus,
    impact_score: f64,
    owner: Option<String>,
    related_receipt_id: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct OutcomeSummary {
    total: usize,
    solved: usize,
    partial: usize,
    unsolved: usize,
    solved_rate: f64,
    avg_impact_score: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct MissionControlSummary {
    deployment: DeploymentCapabilities,
    rollout: RolloutState,
    rbac_users: usize,
    audit: AuditLogVerification,
    audit_remote: AuditRemoteSinkState,
    billing: BillingState,
    workflow: WorkflowBoardSummary,
    compliance: CompliancePosture,
    outcomes: OutcomeSummary,
    approvals_pending: usize,
    receipts_total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct EvidenceExportSummary {
    output_dir: String,
    files: Vec<String>,
}

impl Default for HostConnectionState {
    fn default() -> Self {
        Self {
            connected: false,
            endpoint: None,
            transport: None,
            pairing_token_hint: None,
            connected_at: None,
            updated_at: Utc::now().to_rfc3339(),
            last_error: None,
        }
    }
}

impl Default for RbacRegistry {
    fn default() -> Self {
        Self {
            version: 1,
            users: Vec::new(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for RolloutState {
    fn default() -> Self {
        Self {
            version: 1,
            current_release: None,
            previous_release: None,
            staged_release: None,
            signature_required: false,
            trusted_signers: vec![],
            last_verified_signer: None,
            last_promoted_at: None,
            last_verification_error: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for AuditRemoteSinkState {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: false,
            endpoint: None,
            sink_kind: "siem".to_string(),
            auth_secret_id: None,
            verify_tls: true,
            batch_size: 200,
            last_synced_hash: None,
            last_synced_at: None,
            last_error: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for BillingState {
    fn default() -> Self {
        Self {
            version: 1,
            backend_url: None,
            auth_secret_id: None,
            enforce_verification: false,
            entitlement: BillingEntitlement {
                tier: default_subscription_tier(),
                status: BillingEntitlementStatus::Unverified,
                verified: false,
                source: "setup".to_string(),
                account_id: None,
                entitlement_id: None,
                receipt_id: None,
                expires_at: None,
                last_verified_at: None,
                last_error: None,
            },
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for WorkflowBoardState {
    fn default() -> Self {
        Self {
            version: 1,
            tasks: Vec::new(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for PolicyProfileState {
    fn default() -> Self {
        Self {
            template_id: "general".to_string(),
            applied_at: Utc::now().to_rfc3339(),
            allowed_providers: Vec::new(),
            allowed_transports: vec![
                "lan".to_string(),
                "tailscale".to_string(),
                "cloudflare".to_string(),
                "ngrok".to_string(),
            ],
            allow_public_bind: false,
            require_pairing: true,
        }
    }
}

fn setup_profile_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(PROFILE_SETUP_FILE)
}

fn rbac_registry_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(RBAC_FILE)
}

fn client_connection_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(CLIENT_CONNECTION_FILE)
}

fn rollout_state_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(ROLLOUT_STATE_FILE)
}

fn audit_log_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(AUDIT_LOG_FILE)
}

fn outcomes_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(OUTCOMES_FILE)
}

fn policy_profile_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(POLICY_PROFILE_FILE)
}

fn audit_remote_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(AUDIT_REMOTE_FILE)
}

fn billing_state_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(BILLING_STATE_FILE)
}

fn workflow_board_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(WORKFLOW_BOARD_FILE)
}

fn compliance_profile_path(workspace_dir: &Path) -> PathBuf {
    workspace_dir.join(COMPLIANCE_PROFILE_FILE)
}

fn save_json_pretty<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory for {}", path.display()))?;
    }
    let payload = serde_json::to_string_pretty(value)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    std::fs::write(path, payload).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn load_json_or_default<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed = serde_json::from_str::<T>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(parsed)
}

fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

fn read_audit_events(path: &Path) -> Result<Vec<AuditEvent>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file =
        std::fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("failed to read line from {}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let event = serde_json::from_str::<AuditEvent>(&line)
            .with_context(|| format!("failed to parse audit event line in {}", path.display()))?;
        events.push(event);
    }
    Ok(events)
}

fn append_audit_event(path: &Path, mut event: AuditEvent) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create audit directory {}", parent.display()))?;
    }
    let events = read_audit_events(path)?;
    let prev_hash = events
        .last()
        .map(|entry| entry.hash.clone())
        .unwrap_or_else(|| "genesis".to_string());
    event.prev_hash = prev_hash.clone();
    let unsigned = serde_json::json!({
        "id": event.id,
        "timestamp": event.timestamp,
        "actor_id": event.actor_id,
        "actor_role": event.actor_role,
        "action": event.action,
        "resource": event.resource,
        "destination": event.destination,
        "result": event.result,
        "reason": event.reason,
        "receipt_id": event.receipt_id,
        "approval_id": event.approval_id,
        "prev_hash": prev_hash,
    });
    event.hash = sha256_hex(serde_json::to_string(&unsigned)?.as_bytes());

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to append {}", path.display()))?;
    let line = serde_json::to_string(&event)?;
    writeln!(file, "{line}")
        .with_context(|| format!("failed to write audit event to {}", path.display()))?;
    Ok(())
}

fn verify_audit_log(path: &Path) -> Result<AuditLogVerification> {
    let events = read_audit_events(path)?;
    if events.is_empty() {
        return Ok(AuditLogVerification {
            valid: true,
            entries: 0,
            last_hash: None,
            error: None,
        });
    }

    let mut prev_hash = "genesis".to_string();
    for event in &events {
        if event.prev_hash != prev_hash {
            return Ok(AuditLogVerification {
                valid: false,
                entries: events.len(),
                last_hash: Some(prev_hash),
                error: Some(format!("chain mismatch at event {}", event.id)),
            });
        }
        let unsigned = serde_json::json!({
            "id": event.id,
            "timestamp": event.timestamp,
            "actor_id": event.actor_id,
            "actor_role": event.actor_role,
            "action": event.action,
            "resource": event.resource,
            "destination": event.destination,
            "result": event.result,
            "reason": event.reason,
            "receipt_id": event.receipt_id,
            "approval_id": event.approval_id,
            "prev_hash": event.prev_hash,
        });
        let expected = sha256_hex(serde_json::to_string(&unsigned)?.as_bytes());
        if expected != event.hash {
            return Ok(AuditLogVerification {
                valid: false,
                entries: events.len(),
                last_hash: Some(prev_hash),
                error: Some(format!("hash mismatch at event {}", event.id)),
            });
        }
        prev_hash = event.hash.clone();
    }

    Ok(AuditLogVerification {
        valid: true,
        entries: events.len(),
        last_hash: Some(prev_hash),
        error: None,
    })
}

fn current_platform_label() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else {
        "unknown"
    }
}

fn platform_supports_host_mode() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    ))
}

fn platform_supports_client_mode() -> bool {
    cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows",
        target_os = "android",
        target_os = "ios"
    ))
}

fn validate_deployment_mode(mode: DeploymentMode) -> Result<()> {
    match mode {
        DeploymentMode::Host if !platform_supports_host_mode() => {
            anyhow::bail!(
                "deployment_mode=host is not supported on {} (supported: linux/macos/windows)",
                current_platform_label()
            );
        }
        DeploymentMode::Client if !platform_supports_client_mode() => {
            anyhow::bail!(
                "deployment_mode=client is not supported on {}",
                current_platform_label()
            );
        }
        _ => {}
    }
    Ok(())
}

fn effective_deployment_mode(configured: DeploymentMode) -> DeploymentMode {
    if configured == DeploymentMode::Host && !platform_supports_host_mode() {
        DeploymentMode::Client
    } else if configured == DeploymentMode::Client && !platform_supports_client_mode() {
        default_deployment_mode()
    } else {
        configured
    }
}

fn deployment_mode_label(mode: DeploymentMode) -> &'static str {
    match mode {
        DeploymentMode::Host => "host",
        DeploymentMode::Client => "client",
    }
}

fn normalize_actor_role(role: Option<String>) -> String {
    let raw = role.unwrap_or_else(|| "admin".to_string());
    let lowered = raw.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "owner" | "admin" => "owner".to_string(),
        "manager" => "admin".to_string(),
        "operator" | "user" => "operator".to_string(),
        "viewer" | "observer" => "viewer".to_string(),
        "" => "owner".to_string(),
        _ => lowered,
    }
}

fn normalize_approver_role(role: &str) -> String {
    let lowered = role.trim().to_ascii_lowercase();
    match lowered.as_str() {
        "owner" | "admin" => "owner".to_string(),
        "manager" => "admin".to_string(),
        "" => "owner".to_string(),
        _ => lowered,
    }
}

fn next_rollout_ring(ring: RolloutRing) -> RolloutRing {
    match ring {
        RolloutRing::Pilot => RolloutRing::Group,
        RolloutRing::Group => RolloutRing::All,
        RolloutRing::All => RolloutRing::All,
    }
}

fn rollout_state_load(workspace_dir: &Path) -> Result<RolloutState> {
    let mut state: RolloutState = load_json_or_default(&rollout_state_path(workspace_dir))?;
    if state.signature_required {
        let has_valid_signer = state
            .trusted_signers
            .iter()
            .enumerate()
            .any(|(index, entry)| parse_signer_entry(entry, index).is_ok());
        if !has_valid_signer {
            state.signature_required = false;
            state.trusted_signers.clear();
            state.last_verification_error = Some(
                "legacy signer configuration detected; signing policy reset and requires reconfiguration"
                    .to_string(),
            );
        }
    }
    Ok(state)
}

fn rollout_state_save(workspace_dir: &Path, state: &RolloutState) -> Result<()> {
    save_json_pretty(&rollout_state_path(workspace_dir), state)
}

fn decode_base64_flexible(raw: &str) -> Result<Vec<u8>> {
    let trimmed = raw.trim();
    BASE64_STANDARD
        .decode(trimmed)
        .or_else(|_| URL_SAFE_NO_PAD.decode(trimmed))
        .with_context(|| "failed to decode base64 payload")
}

fn validate_sha256_hex(raw: &str, field: &str) -> Result<()> {
    if raw.len() != 64 || !raw.chars().all(|ch| ch.is_ascii_hexdigit()) {
        anyhow::bail!("{field} must be a lowercase/uppercase 64-char SHA-256 hex string");
    }
    Ok(())
}

fn parse_signer_entry(raw: &str, index: usize) -> Result<(String, [u8; 32])> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("trusted_signers[{}] is empty", index);
    }
    let (key_id, key_b64) = if let Some((left, right)) = trimmed.split_once(':') {
        (left.trim().to_string(), right.trim().to_string())
    } else {
        (format!("signer-{}", index + 1), trimmed.to_string())
    };
    if key_id.is_empty() {
        anyhow::bail!("trusted_signers[{}] key id is empty", index);
    }
    let bytes = decode_base64_flexible(&key_b64)
        .with_context(|| format!("trusted_signers[{}] key is not valid base64", index))?;
    let key: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("trusted_signers[{}] key must decode to 32 bytes", index))?;
    Ok((key_id, key))
}

fn parse_signature_value(raw: &str) -> Result<(Option<String>, [u8; 64])> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        anyhow::bail!("signature is empty");
    }

    if let Some((left, right)) = trimmed.split_once(':') {
        if let Ok(bytes) = decode_base64_flexible(right) {
            let sig: [u8; 64] = bytes
                .as_slice()
                .try_into()
                .map_err(|_| anyhow::anyhow!("signature must decode to 64 bytes"))?;
            let key_hint = left.trim();
            return Ok(((!key_hint.is_empty()).then(|| key_hint.to_string()), sig));
        }
    }

    let bytes = decode_base64_flexible(trimmed)?;
    let sig: [u8; 64] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| anyhow::anyhow!("signature must decode to 64 bytes"))?;
    Ok((None, sig))
}

fn release_signing_payload(release: &ReleaseDescriptor) -> String {
    format!(
        "release_id={}\nversion={}\nchecksum_sha256={}\nsbom_checksum_sha256={}\nring={}",
        release.release_id,
        release.version,
        release.checksum_sha256,
        release.sbom_checksum_sha256.as_deref().unwrap_or(""),
        format!("{:?}", release.ring).to_lowercase()
    )
}

fn verify_release_signature(rollout: &RolloutState, release: &ReleaseDescriptor) -> Result<String> {
    validate_sha256_hex(&release.checksum_sha256, "checksum_sha256")?;
    if let Some(sbom_checksum) = release.sbom_checksum_sha256.as_deref() {
        validate_sha256_hex(sbom_checksum, "sbom_checksum_sha256")?;
    }

    if !rollout.signature_required {
        return Ok("signature_not_required".to_string());
    }

    let signature_raw = release
        .signature
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("release signature is required but missing"))?;
    let (key_hint, signature_bytes) = parse_signature_value(signature_raw)?;
    let message = release_signing_payload(release);
    let signature = Signature::from_bytes(&signature_bytes);

    if rollout.trusted_signers.is_empty() {
        anyhow::bail!("signature_required=true but trusted_signers is empty");
    }

    for (index, signer_entry) in rollout.trusted_signers.iter().enumerate() {
        let (key_id, key_bytes) = parse_signer_entry(signer_entry, index)?;
        if let Some(hint) = key_hint.as_deref() {
            if hint != key_id {
                continue;
            }
        }
        let verifying_key = VerifyingKey::from_bytes(&key_bytes)
            .with_context(|| format!("trusted signer '{key_id}' has invalid key material"))?;
        if verifying_key.verify(message.as_bytes(), &signature).is_ok() {
            return Ok(key_id);
        }
    }

    anyhow::bail!("release signature verification failed for staged release")
}

fn sanitize_sink_kind(raw: Option<String>) -> String {
    match raw
        .unwrap_or_else(|| "siem".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "object_lock" | "object-lock" => "object_lock".to_string(),
        _ => "siem".to_string(),
    }
}

fn audit_remote_load(workspace_dir: &Path) -> Result<AuditRemoteSinkState> {
    load_json_or_default(&audit_remote_path(workspace_dir))
}

fn audit_remote_save(workspace_dir: &Path, state: &AuditRemoteSinkState) -> Result<()> {
    save_json_pretty(&audit_remote_path(workspace_dir), state)
}

fn setup_tier_from_workspace(workspace_dir: &Path) -> SubscriptionTier {
    let path = setup_profile_path(workspace_dir);
    if !path.exists() {
        return default_subscription_tier();
    }
    match std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<ProfileSetupState>(&raw).ok())
    {
        Some(setup) => setup.subscription_tier,
        None => default_subscription_tier(),
    }
}

fn billing_state_load(workspace_dir: &Path) -> Result<BillingState> {
    let mut state: BillingState = load_json_or_default(&billing_state_path(workspace_dir))?;
    if state.version == 0 {
        state.version = 1;
    }
    state.entitlement.tier = if matches!(
        state.entitlement.tier,
        SubscriptionTier::Basic | SubscriptionTier::Professional | SubscriptionTier::Enterprise
    ) {
        state.entitlement.tier
    } else {
        setup_tier_from_workspace(workspace_dir)
    };
    Ok(state)
}

fn billing_state_save(workspace_dir: &Path, state: &BillingState) -> Result<()> {
    save_json_pretty(&billing_state_path(workspace_dir), state)
}

fn tier_rank(tier: SubscriptionTier) -> u8 {
    match tier {
        SubscriptionTier::Basic => 1,
        SubscriptionTier::Professional => 2,
        SubscriptionTier::Enterprise => 3,
    }
}

fn ensure_entitlement_for_feature(
    workspace_dir: &Path,
    minimum_tier: SubscriptionTier,
    feature: &str,
) -> std::result::Result<(), String> {
    let billing = billing_state_load(workspace_dir)
        .map_err(|e| format!("failed to load billing state for entitlement check: {e}"))?;
    if billing.enforce_verification && !billing.entitlement.verified {
        return Err(format!(
            "billing entitlement is not verified for feature '{}' (verification required)",
            feature
        ));
    }
    if billing.enforce_verification
        && matches!(
            billing.entitlement.status,
            BillingEntitlementStatus::Expired | BillingEntitlementStatus::Unverified
        )
    {
        return Err(format!(
            "billing entitlement status '{}' blocks feature '{}'",
            format!("{:?}", billing.entitlement.status).to_lowercase(),
            feature
        ));
    }
    if tier_rank(billing.entitlement.tier) < tier_rank(minimum_tier) {
        return Err(format!(
            "feature '{}' requires '{}' tier (current: '{}')",
            feature,
            format!("{:?}", minimum_tier).to_lowercase(),
            format!("{:?}", billing.entitlement.tier).to_lowercase()
        ));
    }
    Ok(())
}

fn rbac_registry_load(workspace_dir: &Path) -> Result<RbacRegistry> {
    let mut registry: RbacRegistry = load_json_or_default(&rbac_registry_path(workspace_dir))?;
    if !registry
        .users
        .iter()
        .any(|user| matches!(user.role, WorkspaceRole::Admin))
    {
        let now = Utc::now().to_rfc3339();
        registry.users.push(RbacUserRecord {
            user_id: "local-admin".to_string(),
            display_name: "Local Admin".to_string(),
            role: WorkspaceRole::Admin,
            active: true,
            created_at: now.clone(),
            updated_at: now,
        });
    }
    registry.updated_at = Utc::now().to_rfc3339();
    Ok(registry)
}

fn rbac_registry_save(workspace_dir: &Path, registry: &RbacRegistry) -> Result<()> {
    save_json_pretty(&rbac_registry_path(workspace_dir), registry)
}

fn outcomes_load(workspace_dir: &Path) -> Result<Vec<OutcomeRecord>> {
    load_json_or_default(&outcomes_path(workspace_dir))
}

fn outcomes_save(workspace_dir: &Path, outcomes: &[OutcomeRecord]) -> Result<()> {
    save_json_pretty(&outcomes_path(workspace_dir), outcomes)
}

fn summarize_outcomes(outcomes: &[OutcomeRecord]) -> OutcomeSummary {
    let total = outcomes.len();
    let solved = outcomes
        .iter()
        .filter(|item| matches!(item.status, OutcomeStatus::Solved))
        .count();
    let partial = outcomes
        .iter()
        .filter(|item| matches!(item.status, OutcomeStatus::Partial))
        .count();
    let unsolved = outcomes
        .iter()
        .filter(|item| matches!(item.status, OutcomeStatus::Unsolved))
        .count();
    let solved_rate = if total == 0 {
        0.0
    } else {
        solved as f64 / total as f64
    };
    let avg_impact_score = if total == 0 {
        0.0
    } else {
        outcomes.iter().map(|item| item.impact_score).sum::<f64>() / total as f64
    };

    OutcomeSummary {
        total,
        solved,
        partial,
        unsolved,
        solved_rate,
        avg_impact_score,
    }
}

fn workflow_board_load(workspace_dir: &Path) -> Result<WorkflowBoardState> {
    load_json_or_default(&workflow_board_path(workspace_dir))
}

fn workflow_board_save(workspace_dir: &Path, board: &WorkflowBoardState) -> Result<()> {
    save_json_pretty(&workflow_board_path(workspace_dir), board)
}

fn summarize_workflow_tasks(tasks: &[WorkflowTaskRecord]) -> WorkflowBoardSummary {
    let mut pending = 0usize;
    let mut in_progress = 0usize;
    let mut done = 0usize;
    let mut failed = 0usize;
    let mut blocked = 0usize;
    let mut high_risk_open = 0usize;

    for task in tasks {
        match task.status {
            WorkflowTaskStatus::Pending => pending += 1,
            WorkflowTaskStatus::InProgress => in_progress += 1,
            WorkflowTaskStatus::Done => done += 1,
            WorkflowTaskStatus::Failed => failed += 1,
            WorkflowTaskStatus::Blocked => blocked += 1,
        }
        if matches!(
            task.status,
            WorkflowTaskStatus::Pending
                | WorkflowTaskStatus::InProgress
                | WorkflowTaskStatus::Blocked
        ) && task.risk_score >= 70.0
        {
            high_risk_open += 1;
        }
    }

    WorkflowBoardSummary {
        total: tasks.len(),
        pending,
        in_progress,
        done,
        failed,
        blocked,
        high_risk_open,
    }
}

fn compliance_profile_catalog() -> Vec<ComplianceProfileTemplate> {
    vec![
        ComplianceProfileTemplate {
            template_id: "general_baseline".to_string(),
            display_name: "General Baseline".to_string(),
            description: "General 2026-ready governance baseline for most organizations."
                .to_string(),
            industry: "general".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
            ],
            recommended_policy_template: Some("general".to_string()),
            minimum_tier: SubscriptionTier::Professional,
            require_signed_release: true,
            require_remote_audit: false,
            require_billing_verification: false,
            require_pairing: true,
        },
        ComplianceProfileTemplate {
            template_id: "ai_act_nist_strict".to_string(),
            display_name: "AI Act + NIST Strict".to_string(),
            description:
                "Strict baseline aligning AI oversight, auditable operations, and signed deployments."
                    .to_string(),
            industry: "cross_industry".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
                "NIST SP 800-53 Rev.5".to_string(),
            ],
            recommended_policy_template: Some("general".to_string()),
            minimum_tier: SubscriptionTier::Enterprise,
            require_signed_release: true,
            require_remote_audit: true,
            require_billing_verification: true,
            require_pairing: true,
        },
        ComplianceProfileTemplate {
            template_id: "finance_fintech".to_string(),
            display_name: "Finance / Fintech".to_string(),
            description: "Financial-sector constraints with stricter network/provider controls."
                .to_string(),
            industry: "finance".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
                "ISO/IEC 27001:2022".to_string(),
                "SOC 2".to_string(),
                "DORA".to_string(),
            ],
            recommended_policy_template: Some("finance_strict".to_string()),
            minimum_tier: SubscriptionTier::Enterprise,
            require_signed_release: true,
            require_remote_audit: true,
            require_billing_verification: true,
            require_pairing: true,
        },
        ComplianceProfileTemplate {
            template_id: "healthcare_pharma".to_string(),
            display_name: "Healthcare / Pharma".to_string(),
            description:
                "Healthcare controls prioritizing auditable access, private transport, and traceability."
                    .to_string(),
            industry: "healthcare".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
                "ISO/IEC 27001:2022".to_string(),
                "ISO/IEC 42001:2023".to_string(),
                "HIPAA".to_string(),
            ],
            recommended_policy_template: Some("healthcare_strict".to_string()),
            minimum_tier: SubscriptionTier::Enterprise,
            require_signed_release: true,
            require_remote_audit: true,
            require_billing_verification: true,
            require_pairing: true,
        },
        ComplianceProfileTemplate {
            template_id: "tech_cloud_web3_ai".to_string(),
            display_name: "Tech / Cloud / Web3 / AI".to_string(),
            description: "Fast-moving technical organizations with strict software supply controls."
                .to_string(),
            industry: "tech".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
                "ISO/IEC 42001:2023".to_string(),
                "SOC 2".to_string(),
            ],
            recommended_policy_template: Some("general".to_string()),
            minimum_tier: SubscriptionTier::Professional,
            require_signed_release: true,
            require_remote_audit: false,
            require_billing_verification: true,
            require_pairing: true,
        },
        ComplianceProfileTemplate {
            template_id: "government_us_eu".to_string(),
            display_name: "Government (US/EU)".to_string(),
            description:
                "Government posture prioritizing zero-public ingress, immutable evidence, and strict approvals."
                    .to_string(),
            industry: "government".to_string(),
            standards: vec![
                "EU AI Act".to_string(),
                "NIST AI RMF 1.0".to_string(),
                "NIST CSF 2.0".to_string(),
                "NIST SP 800-53 Rev.5".to_string(),
                "ISO/IEC 27001:2022".to_string(),
            ],
            recommended_policy_template: Some("gov_zero_public".to_string()),
            minimum_tier: SubscriptionTier::Enterprise,
            require_signed_release: true,
            require_remote_audit: true,
            require_billing_verification: true,
            require_pairing: true,
        },
    ]
}

fn compliance_profile_load(workspace_dir: &Path) -> Result<Option<ComplianceProfileState>> {
    let path = compliance_profile_path(workspace_dir);
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let state = serde_json::from_str::<ComplianceProfileState>(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(state))
}

fn compliance_profile_save(workspace_dir: &Path, state: &ComplianceProfileState) -> Result<()> {
    save_json_pretty(&compliance_profile_path(workspace_dir), state)
}

fn compliance_posture_evaluate(workspace_dir: &Path) -> Result<CompliancePosture> {
    let profile = compliance_profile_load(workspace_dir)?;
    let rollout = rollout_state_load(workspace_dir)?;
    let audit_verify = verify_audit_log(&audit_log_path(workspace_dir))?;
    let audit_remote = audit_remote_load(workspace_dir)?;
    let billing = billing_state_load(workspace_dir)?;
    let rbac = rbac_registry_load(workspace_dir)?;
    let workflow = workflow_board_load(workspace_dir)?;
    let outcomes = outcomes_load(workspace_dir)?;
    let policy = policy_profile_load(workspace_dir)?;

    let mut checks: Vec<ComplianceControlCheck> = Vec::new();

    let has_admin = rbac
        .users
        .iter()
        .any(|user| matches!(user.role, WorkspaceRole::Admin) && user.active);
    let has_observer = rbac
        .users
        .iter()
        .any(|user| matches!(user.role, WorkspaceRole::Observer) && user.active);
    checks.push(ComplianceControlCheck {
        control_id: "governance.rbac_separation".to_string(),
        label: "RBAC role separation".to_string(),
        framework: "NIST AI RMF / EU AI Act".to_string(),
        required: true,
        satisfied: has_admin && has_observer,
        evidence: Some(format!(
            "active_roles={{admin:{},observer:{}}}",
            has_admin, has_observer
        )),
        recommendation: Some(
            "Ensure at least one active observer for independent oversight.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "assurance.signed_rollout".to_string(),
        label: "Signed release rollout".to_string(),
        framework: "NIST CSF / Software supply chain".to_string(),
        required: profile
            .as_ref()
            .map(|item| item.require_signed_release)
            .unwrap_or(false),
        satisfied: rollout.signature_required && !rollout.trusted_signers.is_empty(),
        evidence: Some(format!(
            "signature_required={},trusted_signers={}",
            rollout.signature_required,
            rollout.trusted_signers.len()
        )),
        recommendation: Some(
            "Enable signature_required and configure trusted signer public keys.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "audit.local_hash_chain".to_string(),
        label: "Tamper-evident local audit chain".to_string(),
        framework: "EU AI Act / NIST AI RMF".to_string(),
        required: true,
        satisfied: audit_verify.valid,
        evidence: Some(format!(
            "entries={},last_hash={}",
            audit_verify.entries,
            audit_verify.last_hash.as_deref().unwrap_or("none")
        )),
        recommendation: Some(
            "Investigate audit chain mismatches before rollout promotion.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "audit.remote_append_only".to_string(),
        label: "Remote append-only audit sink".to_string(),
        framework: "NIST CSF / SOC2".to_string(),
        required: profile
            .as_ref()
            .map(|item| item.require_remote_audit)
            .unwrap_or(false),
        satisfied: audit_remote.enabled && audit_remote.endpoint.is_some(),
        evidence: Some(format!(
            "enabled={},endpoint={}",
            audit_remote.enabled,
            audit_remote.endpoint.as_deref().unwrap_or("none")
        )),
        recommendation: Some(
            "Configure SIEM/object-lock endpoint and run audit_remote_sync regularly.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "billing.entitlement_verification".to_string(),
        label: "Entitlement verification".to_string(),
        framework: "Operational governance".to_string(),
        required: profile
            .as_ref()
            .map(|item| item.require_billing_verification)
            .unwrap_or(false),
        satisfied: !billing.enforce_verification || billing.entitlement.verified,
        evidence: Some(format!(
            "enforce_verification={},verified={},status={}",
            billing.enforce_verification,
            billing.entitlement.verified,
            format!("{:?}", billing.entitlement.status).to_lowercase()
        )),
        recommendation: Some(
            "Enable backend receipt verification for enterprise posture.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "operations.workflow_tracking".to_string(),
        label: "Workflow tracking in mission control".to_string(),
        framework: "NIST AI RMF (Manage/Monitor)".to_string(),
        required: true,
        satisfied: !workflow.tasks.is_empty(),
        evidence: Some(format!("tasks={}", workflow.tasks.len())),
        recommendation: Some(
            "Track runtime and agent work items in the workflow board.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "operations.outcome_measurement".to_string(),
        label: "Outcome measurement".to_string(),
        framework: "NIST AI RMF (Measure)".to_string(),
        required: true,
        satisfied: !outcomes.is_empty(),
        evidence: Some(format!("outcomes={}", outcomes.len())),
        recommendation: Some(
            "Record solved/partial/unsolved outcomes to prove value and control.".to_string(),
        ),
    });

    checks.push(ComplianceControlCheck {
        control_id: "network.pairing_and_transport".to_string(),
        label: "Pairing and transport restrictions".to_string(),
        framework: "EU AI Act / Zero trust".to_string(),
        required: profile
            .as_ref()
            .map(|item| item.require_pairing)
            .unwrap_or(false),
        satisfied: policy
            .as_ref()
            .map(|item| item.require_pairing)
            .unwrap_or(false),
        evidence: Some(format!(
            "policy_profile={}",
            policy
                .as_ref()
                .map(|item| item.template_id.clone())
                .unwrap_or_else(|| "none".to_string())
        )),
        recommendation: Some(
            "Apply an industry policy profile with strict pairing and transport rules.".to_string(),
        ),
    });

    let missing_controls = checks
        .iter()
        .filter(|item| item.required && !item.satisfied)
        .map(|item| item.control_id.clone())
        .collect::<Vec<_>>();

    Ok(CompliancePosture {
        template_id: profile.as_ref().map(|item| item.template_id.clone()),
        standards: profile
            .as_ref()
            .map(|item| item.standards.clone())
            .unwrap_or_default(),
        compliant: missing_controls.is_empty(),
        generated_at: Utc::now().to_rfc3339(),
        checks,
        missing_controls,
    })
}

fn policy_profile_catalog() -> Vec<PolicyProfileTemplate> {
    vec![
        PolicyProfileTemplate {
            template_id: "general".to_string(),
            display_name: "General".to_string(),
            description: "Balanced defaults for most organizations.".to_string(),
            allowed_providers: vec![],
            allowed_transports: vec![
                "lan".to_string(),
                "tailscale".to_string(),
                "cloudflare".to_string(),
                "ngrok".to_string(),
            ],
            allow_public_bind: false,
            require_pairing: true,
        },
        PolicyProfileTemplate {
            template_id: "finance_strict".to_string(),
            display_name: "Finance Strict".to_string(),
            description: "No public tunnels, strict provider allowlist, explicit pairing only."
                .to_string(),
            allowed_providers: vec!["openai".to_string(), "anthropic".to_string()],
            allowed_transports: vec!["lan".to_string(), "tailscale".to_string()],
            allow_public_bind: false,
            require_pairing: true,
        },
        PolicyProfileTemplate {
            template_id: "healthcare_strict".to_string(),
            display_name: "Healthcare Strict".to_string(),
            description: "Private transport only, pairing mandatory, provider allowlist."
                .to_string(),
            allowed_providers: vec!["openai".to_string(), "anthropic".to_string()],
            allowed_transports: vec!["lan".to_string(), "tailscale".to_string()],
            allow_public_bind: false,
            require_pairing: true,
        },
        PolicyProfileTemplate {
            template_id: "gov_zero_public".to_string(),
            display_name: "Gov Zero Public".to_string(),
            description: "No public ingress or public tunnels. LAN-only by default.".to_string(),
            allowed_providers: vec!["openai".to_string()],
            allowed_transports: vec!["lan".to_string()],
            allow_public_bind: false,
            require_pairing: true,
        },
    ]
}

fn policy_profile_load(workspace_dir: &Path) -> Result<Option<PolicyProfileState>> {
    let path = policy_profile_path(workspace_dir);
    if !path.exists() {
        return Ok(None);
    }
    let state = load_json_or_default::<PolicyProfileState>(&path)?;
    Ok(Some(state))
}

fn policy_profile_save(workspace_dir: &Path, state: &PolicyProfileState) -> Result<()> {
    save_json_pretty(&policy_profile_path(workspace_dir), state)
}

fn trim_or_none(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_skills_prompt_injection_mode(
    raw: &str,
) -> Result<zeroclaw::config::schema::SkillsPromptInjectionMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "full" => Ok(zeroclaw::config::schema::SkillsPromptInjectionMode::Full),
        "compact" => Ok(zeroclaw::config::schema::SkillsPromptInjectionMode::Compact),
        _ => anyhow::bail!("unsupported skills_prompt_injection_mode '{raw}'"),
    }
}

fn skills_prompt_injection_mode_to_string(
    mode: zeroclaw::config::schema::SkillsPromptInjectionMode,
) -> String {
    match mode {
        zeroclaw::config::schema::SkillsPromptInjectionMode::Full => "full".to_string(),
        zeroclaw::config::schema::SkillsPromptInjectionMode::Compact => "compact".to_string(),
    }
}

fn normalize_tool_names(raw: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for item in raw {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        if output.iter().any(|existing: &String| existing == trimmed) {
            continue;
        }
        output.push(trimmed.to_string());
    }
    output
}

fn delegate_agents_from_config(cfg: &zeroclaw::Config) -> BTreeMap<String, DelegateAgentSetup> {
    let mut agents = BTreeMap::new();
    for (name, agent) in &cfg.agents {
        agents.insert(
            name.clone(),
            DelegateAgentSetup {
                provider: agent.provider.clone(),
                model: agent.model.clone(),
                system_prompt: trim_or_none(agent.system_prompt.clone()),
                temperature: agent.temperature,
                max_depth: Some(agent.max_depth),
                agentic: agent.agentic,
                allowed_tools: agent.allowed_tools.clone(),
                max_iterations: Some(agent.max_iterations),
            },
        );
    }
    agents
}

fn delegate_agents_to_config(
    delegate_agents: BTreeMap<String, DelegateAgentSetup>,
) -> Result<HashMap<String, zeroclaw::config::schema::DelegateAgentConfig>> {
    let mut agents = HashMap::new();

    for (raw_name, setup) in delegate_agents {
        let name = raw_name.trim();
        if name.is_empty() {
            continue;
        }
        if !name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            anyhow::bail!(
                "invalid delegate agent name '{name}'. use only letters, numbers, '-' or '_'"
            );
        }

        let provider = setup.provider.trim();
        if provider.is_empty() {
            anyhow::bail!("delegate agent '{name}' is missing provider");
        }
        let model = setup.model.trim();
        if model.is_empty() {
            anyhow::bail!("delegate agent '{name}' is missing model");
        }
        if let Some(temperature) = setup.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                anyhow::bail!(
                    "delegate agent '{name}' has invalid temperature '{}'; expected 0.0..=2.0",
                    temperature
                );
            }
        }
        let max_iterations = setup.max_iterations.unwrap_or(10).max(1);
        let allowed_tools = normalize_tool_names(setup.allowed_tools);

        agents.insert(
            name.to_string(),
            zeroclaw::config::schema::DelegateAgentConfig {
                provider: provider.to_string(),
                model: model.to_string(),
                system_prompt: trim_or_none(setup.system_prompt),
                api_key: None,
                temperature: setup.temperature,
                max_depth: setup.max_depth.unwrap_or(3).max(1),
                agentic: setup.agentic,
                allowed_tools,
                max_iterations,
            },
        );
    }

    Ok(agents)
}

fn parse_memory_category(raw: &str) -> zeroclaw::memory::MemoryCategory {
    match raw.trim().to_ascii_lowercase().as_str() {
        "core" => zeroclaw::memory::MemoryCategory::Core,
        "daily" => zeroclaw::memory::MemoryCategory::Daily,
        "conversation" => zeroclaw::memory::MemoryCategory::Conversation,
        other => zeroclaw::memory::MemoryCategory::Custom(other.to_string()),
    }
}

fn truncate_preview(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let mut preview = value.chars().take(max_chars).collect::<String>();
    preview.push_str("...");
    preview
}

async fn load_or_init_profile_config(
    config_path: &Path,
    workspace_dir: &Path,
) -> Result<zeroclaw::Config> {
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create config directory for {}",
                config_path.display()
            )
        })?;
    }
    std::fs::create_dir_all(workspace_dir).with_context(|| {
        format!(
            "failed to create workspace directory {}",
            workspace_dir.display()
        )
    })?;

    if config_path.exists() {
        let data = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let mut cfg: zeroclaw::Config =
            toml::from_str(&data).context("failed to parse profile config")?;
        cfg.config_path = config_path.to_path_buf();
        cfg.workspace_dir = workspace_dir.to_path_buf();
        cfg.apply_env_overrides();
        return Ok(cfg);
    }

    let mut cfg = zeroclaw::Config::default();
    cfg.config_path = config_path.to_path_buf();
    cfg.workspace_dir = workspace_dir.to_path_buf();
    cfg.save()
        .await
        .context("failed to initialize profile config")?;
    Ok(cfg)
}

fn derive_setup_state(
    workspace_dir: &Path,
    cfg: &zeroclaw::Config,
    profile_id: &str,
    state: &State<'_, AppController>,
) -> Result<ProfileSetupState> {
    let provider_from_config = cfg
        .default_provider
        .clone()
        .unwrap_or_else(|| "openrouter".to_string());
    let model_from_config = cfg
        .default_model
        .clone()
        .unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string());
    let key_id = format!("provider.{}.api_key", provider_from_config);
    let has_key = state
        .vault
        .get_secret(profile_id, &key_id)
        .ok()
        .flatten()
        .is_some();

    let profile_path = setup_profile_path(workspace_dir);
    if profile_path.exists() {
        let raw = std::fs::read_to_string(&profile_path)
            .with_context(|| format!("failed to read {}", profile_path.display()))?;
        let mut parsed: ProfileSetupState =
            serde_json::from_str(&raw).context("failed to parse profile setup file")?;
        parsed.provider = provider_from_config.clone();
        parsed.model = model_from_config.clone();
        parsed.api_url = trim_or_none(cfg.api_url.clone());
        parsed.default_temperature = cfg.default_temperature;
        parsed.memory_backend = cfg.memory.backend.clone();
        parsed.runtime_reasoning_enabled = cfg.runtime.reasoning_enabled;
        parsed.agent_compact_context = cfg.agent.compact_context;
        parsed.agent_parallel_tools = cfg.agent.parallel_tools;
        parsed.agent_max_tool_iterations = cfg.agent.max_tool_iterations as u32;
        parsed.agent_max_history_messages = cfg.agent.max_history_messages as u32;
        parsed.agent_tool_dispatcher = if cfg.agent.tool_dispatcher.trim().is_empty() {
            setup_default_agent_tool_dispatcher()
        } else {
            cfg.agent.tool_dispatcher.clone()
        };
        parsed.skills_prompt_injection_mode =
            skills_prompt_injection_mode_to_string(cfg.skills.prompt_injection_mode);
        parsed.skills_open_enabled = cfg.skills.open_skills_enabled;
        parsed.skills_open_dir = trim_or_none(cfg.skills.open_skills_dir.clone());
        parsed.provider_key_id = format!("provider.{}.api_key", parsed.provider);
        parsed.has_provider_key = state
            .vault
            .get_secret(profile_id, &parsed.provider_key_id)
            .ok()
            .flatten()
            .is_some();
        if parsed.orchestrator_mode.trim().is_empty() {
            parsed.orchestrator_mode = default_orchestrator_mode();
        }
        parsed.delegate_agents = delegate_agents_from_config(cfg);
        return Ok(parsed);
    }

    Ok(ProfileSetupState {
        user_display_name: "Operator".into(),
        agent_name: "Right Hand".into(),
        workspace_mode: SetupWorkspaceMode::Workspace,
        deployment_mode: default_deployment_mode(),
        workspace_role: default_workspace_role(),
        subscription_tier: default_subscription_tier(),
        orchestrator_mode: default_orchestrator_mode(),
        provider: provider_from_config,
        model: model_from_config,
        api_url: trim_or_none(cfg.api_url.clone()),
        default_temperature: cfg.default_temperature,
        memory_backend: cfg.memory.backend.clone(),
        runtime_reasoning_enabled: cfg.runtime.reasoning_enabled,
        agent_compact_context: cfg.agent.compact_context,
        agent_parallel_tools: cfg.agent.parallel_tools,
        agent_max_tool_iterations: cfg.agent.max_tool_iterations as u32,
        agent_max_history_messages: cfg.agent.max_history_messages as u32,
        agent_tool_dispatcher: if cfg.agent.tool_dispatcher.trim().is_empty() {
            setup_default_agent_tool_dispatcher()
        } else {
            cfg.agent.tool_dispatcher.clone()
        },
        skills_prompt_injection_mode: skills_prompt_injection_mode_to_string(
            cfg.skills.prompt_injection_mode,
        ),
        skills_open_enabled: cfg.skills.open_skills_enabled,
        skills_open_dir: trim_or_none(cfg.skills.open_skills_dir.clone()),
        enable_tool_connectors: default_enable_tool_connectors(),
        delegate_agents: delegate_agents_from_config(cfg),
        has_provider_key: has_key,
        provider_key_id: key_id,
        updated_at: Utc::now().to_rfc3339(),
    })
}

fn setup_tool_connectors_enabled(workspace_dir: &Path) -> Result<bool> {
    let path = setup_profile_path(workspace_dir);
    if !path.exists() {
        return Ok(default_enable_tool_connectors());
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: ProfileSetupState =
        serde_json::from_str(&raw).context("failed to parse profile setup file")?;
    Ok(parsed.enable_tool_connectors)
}

fn ensure_tool_connectors_enabled(workspace_dir: &Path) -> std::result::Result<(), String> {
    let enabled = setup_tool_connectors_enabled(workspace_dir)
        .map_err(|e| format!("failed to read setup tool connector policy: {e}"))?;
    if !enabled {
        return Err(
            "tool connectors are disabled in setup; enable 'Tool Connectors (MCP)' first"
                .to_string(),
        );
    }
    Ok(())
}

#[tauri::command]
fn protocol_handshake() -> zeroclaw_core::ProtocolHandshake {
    core_protocol_handshake()
}

fn evaluate_policy_gate(
    profile_id: &str,
    state: &State<'_, AppController>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    action: &str,
    resource: &str,
    destination: &str,
    approval_id: Option<String>,
) -> std::result::Result<ActionPolicyDecision, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let store = state
        .control_plane_store_for_profile(profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    let actor_id_value = actor_id.unwrap_or_else(|| "local-user".into());
    let actor_role_value = normalize_actor_role(actor_role);
    let request = ActionPolicyRequest {
        actor_id: actor_id_value.clone(),
        actor_role: actor_role_value.clone(),
        action: action.to_string(),
        resource: resource.to_string(),
        destination: destination.to_string(),
        approval_id,
        occurred_at: Some(Utc::now().to_rfc3339()),
        context: BTreeMap::new(),
    };
    let decision = store
        .evaluate_action(request)
        .map_err(|e| format!("failed to evaluate action policy: {e}"))?;

    let result = if decision.allowed {
        "allowed"
    } else if decision.requires_approval {
        "pending_approval"
    } else {
        "denied"
    };
    let event = AuditEvent {
        id: format!("audit-{}", Utc::now().timestamp_micros()),
        timestamp: Utc::now().to_rfc3339(),
        actor_id: actor_id_value,
        actor_role: actor_role_value,
        action: action.to_string(),
        resource: resource.to_string(),
        destination: destination.to_string(),
        result: result.to_string(),
        reason: decision.reason.clone(),
        receipt_id: decision.receipt_id.clone(),
        approval_id: decision.approval_id.clone(),
        prev_hash: String::new(),
        hash: String::new(),
    };
    append_audit_event(&audit_log_path(&workspace.root_dir), event)
        .map_err(|e| format!("failed to append audit event: {e}"))?;

    if decision.requires_approval {
        let approval = decision.approval_id.clone().unwrap_or_default();
        return Err(format!(
            "action requires approval (approval_id: {}, receipt_id: {})",
            approval, decision.receipt_id
        ));
    }
    if !decision.allowed {
        return Err(format!(
            "action denied by policy: {} (receipt_id: {})",
            decision.reason, decision.receipt_id
        ));
    }

    Ok(decision)
}

#[tauri::command]
fn profiles_list(state: State<'_, AppController>) -> std::result::Result<ProfilesIndex, String> {
    state
        .profile_manager
        .load_index()
        .map_err(|e| format!("failed to list profiles: {e}"))
}

#[tauri::command]
fn profiles_create(
    display_name: String,
    state: State<'_, AppController>,
) -> std::result::Result<ProfileRecord, String> {
    state
        .profile_manager
        .create_profile(&display_name)
        .map_err(|e| format!("failed to create profile: {e}"))
}

#[tauri::command]
fn profiles_switch(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<ProfileRecord, String> {
    state
        .profile_manager
        .switch_active_profile(&profile_id)
        .map_err(|e| format!("failed to switch profile: {e}"))
}

#[tauri::command]
async fn profile_setup_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<ProfileSetupState, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;
    derive_setup_state(&workspace.root_dir, &cfg, &profile_id, &state)
        .map_err(|e| format!("failed to derive setup state: {e}"))
}

#[tauri::command]
async fn profile_setup_save(
    profile_id: String,
    payload: ProfileSetupPayload,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<ProfileSetupState, String> {
    validate_deployment_mode(payload.deployment_mode)
        .map_err(|e| format!("invalid deployment mode for this platform: {e}"))?;

    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "profile.setup",
        &format!("profile:{profile_id}"),
        "local",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let mut cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;
    if let Some(policy) = policy_profile_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load policy profile: {e}"))?
    {
        if !policy.allowed_providers.is_empty()
            && !policy
                .allowed_providers
                .iter()
                .any(|provider| provider.eq_ignore_ascii_case(payload.provider.trim()))
        {
            return Err(format!(
                "provider '{}' is not allowed by policy profile '{}'",
                payload.provider, policy.template_id
            ));
        }
        cfg.gateway.allow_public_bind = policy.allow_public_bind;
        cfg.gateway.require_pairing = policy.require_pairing;
    }

    let provider = payload.provider.trim();
    if provider.is_empty() {
        return Err("provider must not be empty".to_string());
    }
    let model = payload.model.trim();
    if model.is_empty() {
        return Err("model must not be empty".to_string());
    }
    if !(0.0..=2.0).contains(&payload.default_temperature) {
        return Err(format!(
            "default_temperature '{}' is invalid; expected 0.0..=2.0",
            payload.default_temperature
        ));
    }
    let skills_prompt_injection_mode =
        parse_skills_prompt_injection_mode(&payload.skills_prompt_injection_mode)
            .map_err(|e| format!("failed to parse skills_prompt_injection_mode: {e}"))?;

    cfg.default_provider = Some(provider.to_string());
    cfg.default_model = Some(model.to_string());
    cfg.api_url = trim_or_none(payload.api_url.clone());
    cfg.default_temperature = payload.default_temperature;
    cfg.memory.backend = payload.memory_backend.clone();
    cfg.agents = delegate_agents_to_config(payload.delegate_agents.clone())
        .map_err(|e| format!("failed to configure delegate agents: {e}"))?;
    cfg.runtime.reasoning_enabled = payload.runtime_reasoning_enabled;
    cfg.agent.compact_context = payload.agent_compact_context;
    cfg.agent.parallel_tools = payload.agent_parallel_tools;
    cfg.agent.max_tool_iterations = payload.agent_max_tool_iterations.max(1) as usize;
    cfg.agent.max_history_messages = payload.agent_max_history_messages.max(1) as usize;
    cfg.agent.tool_dispatcher = if payload.agent_tool_dispatcher.trim().is_empty() {
        setup_default_agent_tool_dispatcher()
    } else {
        payload.agent_tool_dispatcher.trim().to_string()
    };
    cfg.skills.prompt_injection_mode = skills_prompt_injection_mode;
    cfg.skills.open_skills_enabled = payload.skills_open_enabled;
    cfg.skills.open_skills_dir = trim_or_none(payload.skills_open_dir.clone());
    cfg.autonomy.workspace_only = true;
    cfg.gateway.require_pairing = true;
    cfg.gateway.allow_public_bind = false;
    cfg.save()
        .await
        .map_err(|e| format!("failed to save profile config: {e}"))?;

    if let Some(raw_api_key) = payload.api_key.as_deref() {
        let trimmed = raw_api_key.trim();
        if !trimmed.is_empty() {
            let key_id = format!("provider.{}.api_key", payload.provider);
            state
                .vault
                .set_secret(&profile_id, &key_id, trimmed)
                .map_err(|e| format!("failed to store provider API key: {e}"))?;
        }
    }

    let persisted = ProfileSetupState {
        user_display_name: payload.user_display_name,
        agent_name: payload.agent_name,
        workspace_mode: payload.workspace_mode,
        deployment_mode: payload.deployment_mode,
        workspace_role: payload.workspace_role,
        subscription_tier: payload.subscription_tier,
        orchestrator_mode: if payload.orchestrator_mode.trim().is_empty() {
            default_orchestrator_mode()
        } else {
            payload.orchestrator_mode
        },
        provider: provider.to_string(),
        model: model.to_string(),
        api_url: trim_or_none(payload.api_url),
        default_temperature: payload.default_temperature,
        memory_backend: payload.memory_backend,
        runtime_reasoning_enabled: payload.runtime_reasoning_enabled,
        agent_compact_context: payload.agent_compact_context,
        agent_parallel_tools: payload.agent_parallel_tools,
        agent_max_tool_iterations: payload.agent_max_tool_iterations.max(1),
        agent_max_history_messages: payload.agent_max_history_messages.max(1),
        agent_tool_dispatcher: if payload.agent_tool_dispatcher.trim().is_empty() {
            setup_default_agent_tool_dispatcher()
        } else {
            payload.agent_tool_dispatcher.trim().to_string()
        },
        skills_prompt_injection_mode: if payload.skills_prompt_injection_mode.trim().is_empty() {
            setup_default_skills_prompt_injection_mode()
        } else {
            payload.skills_prompt_injection_mode.trim().to_string()
        },
        skills_open_enabled: payload.skills_open_enabled,
        skills_open_dir: trim_or_none(payload.skills_open_dir),
        enable_tool_connectors: payload.enable_tool_connectors,
        delegate_agents: payload.delegate_agents,
        has_provider_key: false,
        provider_key_id: String::new(),
        updated_at: Utc::now().to_rfc3339(),
    };

    let path = setup_profile_path(&workspace.root_dir);
    let json = serde_json::to_string_pretty(&persisted)
        .map_err(|e| format!("failed to serialize profile setup state: {e}"))?;
    std::fs::write(&path, json).map_err(|e| {
        format!(
            "failed to write profile setup state {}: {e}",
            path.display()
        )
    })?;

    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .set_paid_plan(AccessPlan::Org)
        .map_err(|e| format!("failed to enforce workspace plan: {e}"))?;
    store
        .set_active_view(WorkspaceView::Org)
        .map_err(|e| format!("failed to enforce workspace view: {e}"))?;

    let mut billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state during setup save: {e}"))?;
    if !billing.entitlement.verified {
        billing.entitlement.tier = persisted.subscription_tier;
        billing.entitlement.status = BillingEntitlementStatus::Unverified;
        billing.entitlement.source = "setup".to_string();
        billing.entitlement.last_error = None;
    }
    billing.updated_at = Utc::now().to_rfc3339();
    billing_state_save(&workspace.root_dir, &billing)
        .map_err(|e| format!("failed to persist billing state during setup save: {e}"))?;

    derive_setup_state(&workspace.root_dir, &cfg, &profile_id, &state)
        .map_err(|e| format!("failed to derive setup state: {e}"))
}

#[tauri::command]
async fn deployment_capabilities(
    profile_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<DeploymentCapabilities, String> {
    deployment_capabilities_inner(profile_id, &state).await
}

async fn deployment_capabilities_inner(
    profile_id: Option<String>,
    state: &State<'_, AppController>,
) -> std::result::Result<DeploymentCapabilities, String> {
    let supports_host = platform_supports_host_mode();
    let supports_client = platform_supports_client_mode();
    let mut configured_mode = default_deployment_mode();
    let mut workspace_mode = SetupWorkspaceMode::Workspace;
    let mut workspace_role = default_workspace_role();
    let mut subscription_tier = default_subscription_tier();

    let resolved_profile = if let Some(id) = profile_id {
        Some(id)
    } else {
        state
            .profile_manager
            .get_active_profile()
            .map_err(|e| format!("failed to resolve active profile: {e}"))?
            .map(|profile| profile.id)
    };

    if let Some(id) = resolved_profile {
        let workspace = state
            .profile_manager
            .workspace_for_profile(&id)
            .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
        let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
            .await
            .map_err(|e| format!("failed to load profile config: {e}"))?;
        let setup = derive_setup_state(&workspace.root_dir, &cfg, &id, &state)
            .map_err(|e| format!("failed to derive setup state: {e}"))?;
        configured_mode = setup.deployment_mode;
        workspace_mode = setup.workspace_mode;
        workspace_role = setup.workspace_role;
        subscription_tier = setup.subscription_tier;
    }

    let effective_mode = effective_deployment_mode(configured_mode);
    let note = if configured_mode != effective_mode {
        format!(
            "configured mode '{}' is not supported on {}. effective mode is '{}'",
            deployment_mode_label(configured_mode),
            current_platform_label(),
            deployment_mode_label(effective_mode)
        )
    } else if effective_mode == DeploymentMode::Host {
        "host mode runs local runtime on this device; use client mode for lightweight access"
            .to_string()
    } else {
        "client mode is optimized for approvals/alerts/status/chat and delegated actions"
            .to_string()
    };

    Ok(DeploymentCapabilities {
        platform: current_platform_label().to_string(),
        supports_host,
        supports_client,
        configured_mode,
        effective_mode,
        workspace_mode,
        workspace_role,
        subscription_tier,
        note,
    })
}

#[tauri::command]
fn policy_profiles_list() -> Vec<PolicyProfileTemplate> {
    policy_profile_catalog()
}

#[tauri::command]
fn policy_profile_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Option<PolicyProfileState>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    policy_profile_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load policy profile: {e}"))
}

#[tauri::command]
async fn policy_profile_apply(
    profile_id: String,
    template_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<PolicyProfileState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "policy.apply",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "policy_profile_apply",
    )?;

    let template = policy_profile_catalog()
        .into_iter()
        .find(|item| item.template_id == template_id)
        .ok_or_else(|| format!("unknown policy template '{template_id}'"))?;
    let profile = PolicyProfileState {
        template_id: template.template_id,
        applied_at: Utc::now().to_rfc3339(),
        allowed_providers: template.allowed_providers,
        allowed_transports: template.allowed_transports,
        allow_public_bind: template.allow_public_bind,
        require_pairing: template.require_pairing,
    };
    policy_profile_save(&workspace.root_dir, &profile)
        .map_err(|e| format!("failed to persist policy profile: {e}"))?;

    let mut cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;
    cfg.gateway.allow_public_bind = profile.allow_public_bind;
    cfg.gateway.require_pairing = profile.require_pairing;
    cfg.save()
        .await
        .map_err(|e| format!("failed to save policy-applied profile config: {e}"))?;
    Ok(profile)
}

#[tauri::command]
fn compliance_profiles_list() -> Vec<ComplianceProfileTemplate> {
    compliance_profile_catalog()
}

#[tauri::command]
fn compliance_profile_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Option<ComplianceProfileState>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    compliance_profile_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load compliance profile: {e}"))
}

#[tauri::command]
fn compliance_posture_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<CompliancePosture, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    compliance_posture_evaluate(&workspace.root_dir)
        .map_err(|e| format!("failed to evaluate compliance posture: {e}"))
}

#[tauri::command]
async fn compliance_profile_apply(
    profile_id: String,
    template_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<ComplianceProfileState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "compliance.apply",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let template = compliance_profile_catalog()
        .into_iter()
        .find(|item| item.template_id == template_id)
        .ok_or_else(|| format!("unknown compliance template '{template_id}'"))?;

    ensure_entitlement_for_feature(
        &workspace.root_dir,
        template.minimum_tier,
        "compliance_profile_apply",
    )?;

    let profile = ComplianceProfileState {
        template_id: template.template_id.clone(),
        applied_at: Utc::now().to_rfc3339(),
        industry: template.industry,
        standards: template.standards,
        recommended_policy_template: template.recommended_policy_template.clone(),
        minimum_tier: template.minimum_tier,
        require_signed_release: template.require_signed_release,
        require_remote_audit: template.require_remote_audit,
        require_billing_verification: template.require_billing_verification,
        require_pairing: template.require_pairing,
    };
    compliance_profile_save(&workspace.root_dir, &profile)
        .map_err(|e| format!("failed to persist compliance profile: {e}"))?;

    if let Some(policy_template_id) = profile.recommended_policy_template.as_deref() {
        if let Some(policy_template) = policy_profile_catalog()
            .into_iter()
            .find(|item| item.template_id == policy_template_id)
        {
            let policy = PolicyProfileState {
                template_id: policy_template.template_id,
                applied_at: Utc::now().to_rfc3339(),
                allowed_providers: policy_template.allowed_providers,
                allowed_transports: policy_template.allowed_transports,
                allow_public_bind: policy_template.allow_public_bind,
                require_pairing: policy_template.require_pairing,
            };
            policy_profile_save(&workspace.root_dir, &policy).map_err(|e| {
                format!("failed to persist policy profile from compliance template: {e}")
            })?;

            let mut cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
                .await
                .map_err(|e| format!("failed to load profile config: {e}"))?;
            cfg.gateway.allow_public_bind = policy.allow_public_bind;
            cfg.gateway.require_pairing = policy.require_pairing;
            cfg.save()
                .await
                .map_err(|e| format!("failed to save policy-applied profile config: {e}"))?;
        }
    }

    if profile.require_signed_release {
        let mut rollout = rollout_state_load(&workspace.root_dir)
            .map_err(|e| format!("failed to load rollout state: {e}"))?;
        rollout.signature_required = true;
        if rollout.trusted_signers.is_empty() {
            rollout.last_verification_error = Some(
                "compliance profile requires signed rollout; configure trusted_signers".to_string(),
            );
        }
        rollout.updated_at = Utc::now().to_rfc3339();
        rollout_state_save(&workspace.root_dir, &rollout)
            .map_err(|e| format!("failed to save rollout state: {e}"))?;
    }

    if profile.require_billing_verification {
        let mut billing = billing_state_load(&workspace.root_dir)
            .map_err(|e| format!("failed to load billing state: {e}"))?;
        billing.enforce_verification = true;
        billing.updated_at = Utc::now().to_rfc3339();
        billing_state_save(&workspace.root_dir, &billing)
            .map_err(|e| format!("failed to save billing state: {e}"))?;
    }

    if profile.require_remote_audit {
        let mut remote = audit_remote_load(&workspace.root_dir)
            .map_err(|e| format!("failed to load remote audit sink state: {e}"))?;
        if !remote.enabled || remote.endpoint.is_none() {
            remote.last_error = Some(
                "compliance profile requires remote audit sink; set endpoint and enable sync"
                    .to_string(),
            );
            remote.updated_at = Utc::now().to_rfc3339();
            audit_remote_save(&workspace.root_dir, &remote)
                .map_err(|e| format!("failed to save remote audit sink state: {e}"))?;
        }
    }

    Ok(profile)
}

#[tauri::command]
fn host_connection_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<HostConnectionState, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    load_json_or_default(&client_connection_path(&workspace.root_dir))
        .map_err(|e| format!("failed to load host connection state: {e}"))
}

#[tauri::command]
fn client_connect_host(
    profile_id: String,
    payload: HostConnectPayload,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<HostConnectionState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "host.connect",
        &format!("profile:{profile_id}"),
        "network",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let now = Utc::now().to_rfc3339();
    let parsed = serde_json::from_str::<PairingBundle>(&payload.invite_payload)
        .map_err(|e| format!("invalid invite payload: expected pairing bundle json ({e})"))?;
    let token_hint = if parsed.access_token.len() > 10 {
        format!("{}...", &parsed.access_token[..10])
    } else {
        parsed.access_token.clone()
    };
    let state_value = HostConnectionState {
        connected: true,
        endpoint: Some(parsed.endpoint),
        transport: Some(format!("{:?}", parsed.transport).to_lowercase()),
        pairing_token_hint: Some(token_hint),
        connected_at: Some(now.clone()),
        updated_at: now,
        last_error: None,
    };
    save_json_pretty(&client_connection_path(&workspace.root_dir), &state_value)
        .map_err(|e| format!("failed to persist host connection: {e}"))?;
    Ok(state_value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RbacUserUpsertRequest {
    user_id: String,
    display_name: String,
    role: WorkspaceRole,
    active: bool,
}

#[tauri::command]
fn rbac_users_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<RbacRegistry, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let registry = rbac_registry_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rbac registry: {e}"))?;
    rbac_registry_save(&workspace.root_dir, &registry)
        .map_err(|e| format!("failed to persist normalized rbac registry: {e}"))?;
    Ok(registry)
}

#[tauri::command]
fn rbac_user_upsert(
    profile_id: String,
    request: RbacUserUpsertRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<RbacRegistry, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "rbac.manage",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "rbac_user_upsert",
    )?;
    let mut registry = rbac_registry_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rbac registry: {e}"))?;
    let now = Utc::now().to_rfc3339();
    if let Some(user) = registry
        .users
        .iter_mut()
        .find(|item| item.user_id == request.user_id)
    {
        user.display_name = request.display_name;
        user.role = request.role;
        user.active = request.active;
        user.updated_at = now.clone();
    } else {
        registry.users.push(RbacUserRecord {
            user_id: request.user_id,
            display_name: request.display_name,
            role: request.role,
            active: request.active,
            created_at: now.clone(),
            updated_at: now.clone(),
        });
    }
    registry.updated_at = now;
    rbac_registry_save(&workspace.root_dir, &registry)
        .map_err(|e| format!("failed to persist rbac registry: {e}"))?;
    Ok(registry)
}

#[tauri::command]
fn rollout_state_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<RolloutState, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))
}

#[tauri::command]
fn rollout_stage_release(
    profile_id: String,
    request: RolloutStageRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<RolloutState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "release.stage",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "rollout_stage_release",
    )?;
    validate_sha256_hex(&request.checksum_sha256, "checksum_sha256")
        .map_err(|e| format!("invalid rollout checksum: {e}"))?;
    if let Some(sbom_checksum) = request.sbom_checksum_sha256.as_deref() {
        validate_sha256_hex(sbom_checksum, "sbom_checksum_sha256")
            .map_err(|e| format!("invalid rollout sbom checksum: {e}"))?;
    }
    let mut rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;
    if let Some(signature) = request.signature.as_deref() {
        parse_signature_value(signature)
            .map_err(|e| format!("invalid rollout signature payload: {e}"))?;
    }
    rollout.staged_release = Some(ReleaseDescriptor {
        release_id: request.release_id,
        version: request.version,
        checksum_sha256: request.checksum_sha256,
        signature: request.signature,
        sbom_checksum_sha256: request.sbom_checksum_sha256,
        ring: request.ring,
        staged_at: Utc::now().to_rfc3339(),
    });
    rollout.updated_at = Utc::now().to_rfc3339();
    rollout_state_save(&workspace.root_dir, &rollout)
        .map_err(|e| format!("failed to persist rollout state: {e}"))?;
    Ok(rollout)
}

#[tauri::command]
fn rollout_set_signing_policy(
    profile_id: String,
    request: RolloutSigningPolicyRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<RolloutState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "release.signing_policy",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "rollout_set_signing_policy",
    )?;
    if request.signature_required && request.trusted_signers.is_empty() {
        return Err("signature_required=true requires at least one trusted signer".to_string());
    }
    for (index, entry) in request.trusted_signers.iter().enumerate() {
        parse_signer_entry(entry, index)
            .map_err(|e| format!("invalid trusted signer configuration: {e}"))?;
    }

    let mut rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;
    rollout.signature_required = request.signature_required;
    rollout.trusted_signers = request.trusted_signers;
    rollout.last_verification_error = None;
    rollout.updated_at = Utc::now().to_rfc3339();
    rollout_state_save(&workspace.root_dir, &rollout)
        .map_err(|e| format!("failed to persist rollout signing policy: {e}"))?;
    Ok(rollout)
}

#[tauri::command]
fn rollout_promote(
    profile_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<RolloutState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "release.promote",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "rollout_promote",
    )?;
    let mut rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;

    if let Some(staged) = rollout.staged_release.take() {
        match verify_release_signature(&rollout, &staged) {
            Ok(signer) => {
                rollout.last_verified_signer = Some(signer);
                rollout.last_verification_error = None;
            }
            Err(error) => {
                rollout.last_verification_error = Some(error.to_string());
                rollout.updated_at = Utc::now().to_rfc3339();
                rollout_state_save(&workspace.root_dir, &rollout).map_err(|e| {
                    format!("failed to persist rollout verification error state: {e}")
                })?;
                return Err(format!(
                    "staged release failed signature verification: {error}"
                ));
            }
        }
        rollout.previous_release = rollout.current_release.take();
        rollout.current_release = Some(staged);
    } else if let Some(current) = rollout.current_release.as_mut() {
        current.ring = next_rollout_ring(current.ring);
    } else {
        return Err("no staged or current release available to promote".to_string());
    }

    rollout.last_promoted_at = Some(Utc::now().to_rfc3339());
    rollout.updated_at = Utc::now().to_rfc3339();
    rollout_state_save(&workspace.root_dir, &rollout)
        .map_err(|e| format!("failed to persist rollout state: {e}"))?;
    Ok(rollout)
}

#[tauri::command]
fn rollout_rollback(
    profile_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<RolloutState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "release.rollback",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "rollout_rollback",
    )?;
    let mut rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;
    let previous = rollout
        .previous_release
        .clone()
        .ok_or_else(|| "no previous release found for rollback".to_string())?;
    rollout.staged_release = rollout.current_release.take();
    rollout.current_release = Some(previous);
    rollout.updated_at = Utc::now().to_rfc3339();
    rollout_state_save(&workspace.root_dir, &rollout)
        .map_err(|e| format!("failed to persist rollout state: {e}"))?;
    Ok(rollout)
}

#[tauri::command]
fn audit_log_list(
    profile_id: String,
    limit: Option<usize>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<AuditEvent>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut events = read_audit_events(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to read audit log: {e}"))?;
    let max = limit.unwrap_or(300);
    if events.len() > max {
        events = events.split_off(events.len().saturating_sub(max));
    }
    Ok(events)
}

#[tauri::command]
fn audit_log_verify(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<AuditLogVerification, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    verify_audit_log(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to verify audit log: {e}"))
}

#[tauri::command]
fn audit_log_export(
    profile_id: String,
    output_path: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<String, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Professional,
        "audit_log_export",
    )?;
    let events = read_audit_events(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to read audit log: {e}"))?;
    let default_path = workspace.logs_dir.join(format!(
        "audit-log-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let path = output_path.map(PathBuf::from).unwrap_or(default_path);
    save_json_pretty(&path, &events).map_err(|e| format!("failed to export audit log: {e}"))?;
    Ok(path.display().to_string())
}

#[tauri::command]
fn audit_remote_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<AuditRemoteSinkState, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut remote = audit_remote_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load remote audit state: {e}"))?;
    remote.updated_at = Utc::now().to_rfc3339();
    audit_remote_save(&workspace.root_dir, &remote)
        .map_err(|e| format!("failed to persist remote audit state: {e}"))?;
    Ok(remote)
}

#[tauri::command]
fn audit_remote_configure(
    profile_id: String,
    request: AuditRemoteConfigureRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<AuditRemoteSinkState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "audit.remote.configure",
        &format!("profile:{profile_id}"),
        "network",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Enterprise,
        "audit_remote_configure",
    )?;

    let mut remote = audit_remote_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load remote audit state: {e}"))?;
    let endpoint = request
        .endpoint
        .as_deref()
        .map(str::trim)
        .and_then(|value| (!value.is_empty()).then(|| value.to_string()));
    if request.enabled {
        let endpoint_value = endpoint
            .as_deref()
            .ok_or_else(|| "enabled remote audit sink requires endpoint".to_string())?;
        if !endpoint_value.starts_with("https://") {
            return Err("remote audit sink endpoint must use https://".to_string());
        }
    }

    remote.enabled = request.enabled;
    remote.endpoint = endpoint;
    remote.sink_kind = sanitize_sink_kind(request.sink_kind);
    remote.auth_secret_id = request
        .auth_secret_id
        .and_then(|value| (!value.trim().is_empty()).then(|| value));
    remote.verify_tls = request.verify_tls.unwrap_or(true);
    remote.batch_size = request
        .batch_size
        .unwrap_or(remote.batch_size)
        .clamp(1, 5000);
    remote.updated_at = Utc::now().to_rfc3339();
    audit_remote_save(&workspace.root_dir, &remote)
        .map_err(|e| format!("failed to persist remote audit state: {e}"))?;
    Ok(remote)
}

#[tauri::command]
async fn audit_remote_sync(
    profile_id: String,
    limit: Option<usize>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<AuditRemoteSyncResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "audit.remote.sync",
        &format!("profile:{profile_id}"),
        "network",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Enterprise,
        "audit_remote_sync",
    )?;
    let mut remote = audit_remote_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load remote audit state: {e}"))?;
    if !remote.enabled {
        return Err("remote audit sink is disabled".to_string());
    }
    let endpoint = remote
        .endpoint
        .clone()
        .ok_or_else(|| "remote audit sink endpoint is missing".to_string())?;

    let events = read_audit_events(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to read audit log for remote sync: {e}"))?;
    let start_index = match remote.last_synced_hash.as_deref() {
        Some(last_hash) => events
            .iter()
            .position(|item| item.hash == last_hash)
            .map(|index| index + 1)
            .unwrap_or(0),
        None => 0,
    };
    let max = limit.unwrap_or(remote.batch_size).clamp(1, 5000);
    let mut pending = events.into_iter().skip(start_index).collect::<Vec<_>>();
    if pending.len() > max {
        pending.truncate(max);
    }

    if pending.is_empty() {
        let now = Utc::now().to_rfc3339();
        return Ok(AuditRemoteSyncResult {
            endpoint,
            sink_kind: remote.sink_kind,
            events_sent: 0,
            first_hash: None,
            last_hash: remote.last_synced_hash,
            synced_at: now,
        });
    }

    let verification = verify_audit_log(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to verify audit log before remote sync: {e}"))?;
    let payload = serde_json::json!({
        "format": "right-hand-audit-remote-v1",
        "profile_id": profile_id,
        "synced_at": Utc::now().to_rfc3339(),
        "sink_kind": remote.sink_kind,
        "verification": verification,
        "events": pending,
    });

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(!remote.verify_tls)
        .build()
        .map_err(|e| format!("failed to construct remote audit client: {e}"))?;
    let mut request_builder = client
        .post(&endpoint)
        .header(CONTENT_TYPE, "application/json")
        .json(&payload);
    if let Some(secret_id) = remote.auth_secret_id.as_deref() {
        let token = state
            .vault
            .get_secret(&profile_id, secret_id)
            .map_err(|e| format!("failed to read remote audit auth secret '{secret_id}': {e}"))?
            .ok_or_else(|| format!("missing remote audit auth secret '{secret_id}'"))?;
        request_builder = request_builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("failed to sync remote audit events: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        remote.last_error = Some(format!(
            "remote sink rejected request: status={} body={}",
            status,
            truncate_preview(&body, 240)
        ));
        remote.updated_at = Utc::now().to_rfc3339();
        audit_remote_save(&workspace.root_dir, &remote)
            .map_err(|e| format!("failed to persist remote audit sync failure: {e}"))?;
        return Err(format!("remote sink rejected request with status {status}"));
    }

    let now = Utc::now().to_rfc3339();
    let first_hash = payload["events"]
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("hash"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    let last_hash = payload["events"]
        .as_array()
        .and_then(|items| items.last())
        .and_then(|item| item.get("hash"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    remote.last_synced_hash = last_hash.clone();
    remote.last_synced_at = Some(now.clone());
    remote.last_error = None;
    remote.updated_at = now.clone();
    audit_remote_save(&workspace.root_dir, &remote)
        .map_err(|e| format!("failed to persist remote audit sync state: {e}"))?;

    Ok(AuditRemoteSyncResult {
        endpoint,
        sink_kind: remote.sink_kind,
        events_sent: payload["events"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or(0),
        first_hash,
        last_hash,
        synced_at: now,
    })
}

#[tauri::command]
fn billing_state_get(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<BillingState, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state: {e}"))?;
    if !billing.entitlement.verified {
        billing.entitlement.tier = setup_tier_from_workspace(&workspace.root_dir);
    }
    billing.updated_at = Utc::now().to_rfc3339();
    billing_state_save(&workspace.root_dir, &billing)
        .map_err(|e| format!("failed to persist normalized billing state: {e}"))?;
    Ok(billing)
}

#[tauri::command]
fn billing_config_set(
    profile_id: String,
    request: BillingConfigRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<BillingState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "billing.configure",
        &format!("profile:{profile_id}"),
        "network",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state: {e}"))?;

    let backend_url = request
        .backend_url
        .as_deref()
        .map(str::trim)
        .and_then(|value| (!value.is_empty()).then(|| value.to_string()));
    if let Some(url) = backend_url.as_deref() {
        if !(url.starts_with("https://") || url.starts_with("http://127.0.0.1")) {
            return Err(
                "billing backend url must use https:// (or http://127.0.0.1 for local dev)"
                    .to_string(),
            );
        }
    }

    billing.backend_url = backend_url;
    billing.auth_secret_id = request
        .auth_secret_id
        .and_then(|value| (!value.trim().is_empty()).then(|| value));
    billing.enforce_verification = request.enforce_verification;
    billing.updated_at = Utc::now().to_rfc3339();
    billing_state_save(&workspace.root_dir, &billing)
        .map_err(|e| format!("failed to persist billing state: {e}"))?;
    Ok(billing)
}

#[tauri::command]
async fn billing_verify_receipt(
    profile_id: String,
    request: BillingReceiptVerifyRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<BillingState, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "billing.verify",
        &format!("profile:{profile_id}"),
        "network",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state: {e}"))?;
    let backend_url = billing
        .backend_url
        .clone()
        .ok_or_else(|| "billing backend_url is not configured".to_string())?;
    if request.receipt_payload.trim().is_empty() {
        return Err("receipt_payload is required".to_string());
    }

    let expected_tier = setup_tier_from_workspace(&workspace.root_dir);
    let payload = serde_json::json!({
        "profile_id": profile_id,
        "expected_tier": expected_tier,
        "receipt_payload": request.receipt_payload,
        "platform": request.platform,
    });
    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| format!("failed to construct billing verification client: {e}"))?;
    let mut request_builder = client
        .post(&backend_url)
        .header(CONTENT_TYPE, "application/json")
        .json(&payload);
    if let Some(secret_id) = billing.auth_secret_id.as_deref() {
        let token = state
            .vault
            .get_secret(&profile_id, secret_id)
            .map_err(|e| format!("failed to read billing auth secret '{secret_id}': {e}"))?
            .ok_or_else(|| format!("missing billing auth secret '{secret_id}'"))?;
        request_builder = request_builder.header(AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = request_builder
        .send()
        .await
        .map_err(|e| format!("failed to call billing verification backend: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        billing.entitlement.verified = false;
        billing.entitlement.status = BillingEntitlementStatus::Unverified;
        billing.entitlement.last_error = Some(format!(
            "billing backend rejected request: status={} body={}",
            status,
            truncate_preview(&body, 240)
        ));
        billing.entitlement.last_verified_at = Some(Utc::now().to_rfc3339());
        billing.updated_at = Utc::now().to_rfc3339();
        billing_state_save(&workspace.root_dir, &billing)
            .map_err(|e| format!("failed to persist billing failure state: {e}"))?;
        return Err(format!(
            "billing verification backend rejected request: {status}"
        ));
    }

    let verification = response
        .json::<BillingVerificationResponse>()
        .await
        .map_err(|e| format!("failed to parse billing verification response: {e}"))?;
    let now = Utc::now().to_rfc3339();
    billing.entitlement.source = "backend".to_string();
    billing.entitlement.last_verified_at = Some(now.clone());
    billing.entitlement.account_id = verification.account_id;
    billing.entitlement.entitlement_id = verification.entitlement_id;
    billing.entitlement.receipt_id = verification.receipt_id;
    billing.entitlement.expires_at = verification.expires_at;
    if verification.valid {
        billing.entitlement.tier = verification.tier.unwrap_or(expected_tier);
        billing.entitlement.status = verification
            .status
            .unwrap_or(BillingEntitlementStatus::Active);
        billing.entitlement.verified = true;
        billing.entitlement.last_error = None;
    } else {
        billing.entitlement.tier = verification.tier.unwrap_or(expected_tier);
        billing.entitlement.status = verification
            .status
            .unwrap_or(BillingEntitlementStatus::Unverified);
        billing.entitlement.verified = false;
        billing.entitlement.last_error = Some(
            verification
                .reason
                .unwrap_or_else(|| "billing receipt verification failed".to_string()),
        );
    }
    billing.updated_at = now;
    billing_state_save(&workspace.root_dir, &billing)
        .map_err(|e| format!("failed to persist billing verification state: {e}"))?;
    Ok(billing)
}

#[tauri::command]
fn workflow_board_get(
    profile_id: String,
    limit: Option<usize>,
    state: State<'_, AppController>,
) -> std::result::Result<WorkflowBoardView, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let board = workflow_board_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load workflow board: {e}"))?;
    let max = limit.unwrap_or(400);
    let tasks = board.tasks.into_iter().take(max).collect::<Vec<_>>();
    Ok(WorkflowBoardView {
        summary: summarize_workflow_tasks(&tasks),
        tasks,
    })
}

#[tauri::command]
fn workflow_task_upsert(
    profile_id: String,
    request: WorkflowTaskUpsertRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<WorkflowTaskRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "workflow.task_upsert",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut board = workflow_board_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load workflow board: {e}"))?;
    let now = Utc::now().to_rfc3339();
    let risk_score = request.risk_score.unwrap_or(50.0).clamp(0.0, 100.0);

    if let Some(task_id) = request.id.as_deref() {
        if let Some(index) = board.tasks.iter().position(|item| item.id == task_id) {
            {
                let task = &mut board.tasks[index];
                task.title = request.title;
                task.description = request.description;
                if let Some(status) = request.status {
                    if matches!(status, WorkflowTaskStatus::InProgress) && task.started_at.is_none()
                    {
                        task.started_at = Some(now.clone());
                    }
                    if matches!(
                        status,
                        WorkflowTaskStatus::Done | WorkflowTaskStatus::Failed
                    ) {
                        task.completed_at = Some(now.clone());
                    } else {
                        task.completed_at = None;
                    }
                    task.status = status;
                }
                if let Some(priority) = request.priority {
                    task.priority = priority;
                }
                task.owner = request.owner;
                task.runtime_task_id = request.runtime_task_id;
                task.agent_id = request.agent_id;
                task.skill_id = request.skill_id;
                task.tool_id = request.tool_id;
                task.tags = request
                    .tags
                    .unwrap_or_default()
                    .into_iter()
                    .map(|item| item.trim().to_string())
                    .filter(|item| !item.is_empty())
                    .collect();
                task.risk_score = risk_score;
                task.related_receipt_id = request.related_receipt_id;
                task.updated_at = now.clone();
            }
            let record = board.tasks[index].clone();
            board.updated_at = now;
            workflow_board_save(&workspace.root_dir, &board)
                .map_err(|e| format!("failed to persist workflow board: {e}"))?;
            return Ok(record);
        }
        return Err(format!("workflow task '{task_id}' was not found"));
    }

    let status = request.status.unwrap_or(WorkflowTaskStatus::Pending);
    let started_at = matches!(status, WorkflowTaskStatus::InProgress).then(|| now.clone());
    let completed_at = matches!(
        status,
        WorkflowTaskStatus::Done | WorkflowTaskStatus::Failed
    )
    .then(|| now.clone());
    let record = WorkflowTaskRecord {
        id: format!("task-{}", Utc::now().timestamp_micros()),
        title: request.title,
        description: request.description,
        status,
        priority: request.priority.unwrap_or(WorkflowTaskPriority::Medium),
        owner: request.owner,
        workspace_scope: profile_id,
        runtime_task_id: request.runtime_task_id,
        agent_id: request.agent_id,
        skill_id: request.skill_id,
        tool_id: request.tool_id,
        tags: request
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect(),
        risk_score,
        related_receipt_id: request.related_receipt_id,
        created_at: now.clone(),
        updated_at: now.clone(),
        started_at,
        completed_at,
    };
    board.tasks.insert(0, record.clone());
    board.tasks.truncate(4000);
    board.updated_at = now;
    workflow_board_save(&workspace.root_dir, &board)
        .map_err(|e| format!("failed to persist workflow board: {e}"))?;
    Ok(record)
}

#[tauri::command]
fn workflow_task_move(
    profile_id: String,
    request: WorkflowTaskMoveRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<WorkflowTaskRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "workflow.task_move",
        &format!("task:{}", request.task_id),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut board = workflow_board_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load workflow board: {e}"))?;
    let now = Utc::now().to_rfc3339();
    let task = board
        .tasks
        .iter_mut()
        .find(|item| item.id == request.task_id)
        .ok_or_else(|| format!("workflow task '{}' was not found", request.task_id))?;
    task.status = request.status;
    task.updated_at = now.clone();
    if matches!(task.status, WorkflowTaskStatus::InProgress) && task.started_at.is_none() {
        task.started_at = Some(now.clone());
    }
    if matches!(
        task.status,
        WorkflowTaskStatus::Done | WorkflowTaskStatus::Failed
    ) {
        task.completed_at = Some(now.clone());
    } else {
        task.completed_at = None;
    }
    let record = task.clone();
    board.updated_at = now;
    workflow_board_save(&workspace.root_dir, &board)
        .map_err(|e| format!("failed to persist workflow board: {e}"))?;
    Ok(record)
}

#[tauri::command]
fn outcomes_record(
    profile_id: String,
    request: OutcomeUpsertRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OutcomeRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "outcomes.record",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let mut outcomes =
        outcomes_load(&workspace.root_dir).map_err(|e| format!("failed to load outcomes: {e}"))?;
    let record = OutcomeRecord {
        id: format!("outcome-{}", Utc::now().timestamp_micros()),
        timestamp: Utc::now().to_rfc3339(),
        title: request.title,
        status: request.status,
        impact_score: request.impact_score.clamp(0.0, 100.0),
        owner: request.owner,
        related_receipt_id: request.related_receipt_id,
        notes: request.notes,
    };
    outcomes.insert(0, record.clone());
    outcomes_save(&workspace.root_dir, &outcomes)
        .map_err(|e| format!("failed to persist outcomes: {e}"))?;
    Ok(record)
}

#[tauri::command]
fn outcomes_list(
    profile_id: String,
    limit: Option<usize>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<OutcomeRecord>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let outcomes =
        outcomes_load(&workspace.root_dir).map_err(|e| format!("failed to load outcomes: {e}"))?;
    let max = limit.unwrap_or(200);
    Ok(outcomes.into_iter().take(max).collect())
}

#[tauri::command]
fn outcomes_summary(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<OutcomeSummary, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let outcomes =
        outcomes_load(&workspace.root_dir).map_err(|e| format!("failed to load outcomes: {e}"))?;
    Ok(summarize_outcomes(&outcomes))
}

#[tauri::command]
async fn mission_control_summary(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<MissionControlSummary, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let deployment = deployment_capabilities_inner(Some(profile_id.clone()), &state).await?;
    let rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;
    let audit_remote = audit_remote_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load remote audit sink state: {e}"))?;
    let billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state: {e}"))?;
    let workflow = workflow_board_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load workflow board: {e}"))?;
    let compliance = compliance_posture_evaluate(&workspace.root_dir)
        .map_err(|e| format!("failed to evaluate compliance posture: {e}"))?;
    let rbac = rbac_registry_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rbac registry: {e}"))?;
    let audit = verify_audit_log(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to verify audit log: {e}"))?;
    let outcomes =
        outcomes_load(&workspace.root_dir).map_err(|e| format!("failed to load outcomes: {e}"))?;
    let control = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?
        .get_state()
        .map_err(|e| format!("failed to load control-plane state: {e}"))?;

    Ok(MissionControlSummary {
        deployment,
        rollout,
        rbac_users: rbac.users.len(),
        audit,
        audit_remote,
        billing,
        workflow: summarize_workflow_tasks(&workflow.tasks),
        compliance,
        outcomes: summarize_outcomes(&outcomes),
        approvals_pending: control
            .approvals
            .iter()
            .filter(|item| item.status == zeroclaw_core::ApprovalStatus::Pending)
            .count(),
        receipts_total: control.receipts.len(),
    })
}

#[tauri::command]
async fn evidence_export(
    profile_id: String,
    output_dir: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<EvidenceExportSummary, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_entitlement_for_feature(
        &workspace.root_dir,
        SubscriptionTier::Enterprise,
        "evidence_export",
    )?;
    let dir = output_dir.map(PathBuf::from).unwrap_or_else(|| {
        workspace
            .logs_dir
            .join(format!("evidence-{}", Utc::now().format("%Y%m%d-%H%M%S")))
    });
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("failed to create evidence directory {}: {e}", dir.display()))?;

    let audit_events = read_audit_events(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to read audit log: {e}"))?;
    let audit_verify = verify_audit_log(&audit_log_path(&workspace.root_dir))
        .map_err(|e| format!("failed to verify audit log: {e}"))?;
    let rollout = rollout_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rollout state: {e}"))?;
    let audit_remote = audit_remote_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load remote audit sink state: {e}"))?;
    let billing = billing_state_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load billing state: {e}"))?;
    let workflow = workflow_board_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load workflow board: {e}"))?;
    let compliance_profile = compliance_profile_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load compliance profile: {e}"))?;
    let compliance_posture = compliance_posture_evaluate(&workspace.root_dir)
        .map_err(|e| format!("failed to evaluate compliance posture: {e}"))?;
    let rbac = rbac_registry_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load rbac registry: {e}"))?;
    let outcomes =
        outcomes_load(&workspace.root_dir).map_err(|e| format!("failed to load outcomes: {e}"))?;
    let deployment = deployment_capabilities_inner(Some(profile_id.clone()), &state).await?;
    let control = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?
        .get_state()
        .map_err(|e| format!("failed to load control-plane state: {e}"))?;
    let mission = MissionControlSummary {
        deployment: deployment.clone(),
        rollout: rollout.clone(),
        rbac_users: rbac.users.len(),
        audit: audit_verify.clone(),
        audit_remote: audit_remote.clone(),
        billing: billing.clone(),
        workflow: summarize_workflow_tasks(&workflow.tasks),
        compliance: compliance_posture.clone(),
        outcomes: summarize_outcomes(&outcomes),
        approvals_pending: control
            .approvals
            .iter()
            .filter(|item| item.status == zeroclaw_core::ApprovalStatus::Pending)
            .count(),
        receipts_total: control.receipts.len(),
    };
    let handshake = core_protocol_handshake();

    let audit_path = dir.join("audit-log.json");
    let verify_path = dir.join("audit-verify.json");
    let rollout_path = dir.join("rollout-state.json");
    let rbac_path = dir.join("rbac-users.json");
    let outcomes_path = dir.join("outcomes.json");
    let audit_remote_path = dir.join("audit-remote-state.json");
    let billing_path = dir.join("billing-state.json");
    let workflow_path = dir.join("workflow-board.json");
    let compliance_profile_path = dir.join("compliance-profile.json");
    let compliance_posture_path = dir.join("compliance-posture.json");
    let mission_path = dir.join("mission-summary.json");
    let version_path = dir.join("version-manifest.json");
    let sbom_path = dir.join("sbom-manifest.json");
    let incident_path = dir.join("incident-playbook.md");

    save_json_pretty(&audit_path, &audit_events)
        .map_err(|e| format!("failed to write audit export: {e}"))?;
    save_json_pretty(&verify_path, &audit_verify)
        .map_err(|e| format!("failed to write audit verification: {e}"))?;
    save_json_pretty(&rollout_path, &rollout)
        .map_err(|e| format!("failed to write rollout export: {e}"))?;
    save_json_pretty(&rbac_path, &rbac).map_err(|e| format!("failed to write rbac export: {e}"))?;
    save_json_pretty(&outcomes_path, &outcomes)
        .map_err(|e| format!("failed to write outcomes export: {e}"))?;
    save_json_pretty(&audit_remote_path, &audit_remote)
        .map_err(|e| format!("failed to write remote audit state export: {e}"))?;
    save_json_pretty(&billing_path, &billing)
        .map_err(|e| format!("failed to write billing state export: {e}"))?;
    save_json_pretty(&workflow_path, &workflow)
        .map_err(|e| format!("failed to write workflow board export: {e}"))?;
    save_json_pretty(&compliance_profile_path, &compliance_profile)
        .map_err(|e| format!("failed to write compliance profile export: {e}"))?;
    save_json_pretty(&compliance_posture_path, &compliance_posture)
        .map_err(|e| format!("failed to write compliance posture export: {e}"))?;
    save_json_pretty(&mission_path, &mission)
        .map_err(|e| format!("failed to write mission summary export: {e}"))?;

    let version_manifest = serde_json::json!({
        "app_name": "right-hand-app",
        "app_version": env!("CARGO_PKG_VERSION"),
        "exported_at": Utc::now().to_rfc3339(),
        "profile_id": profile_id,
        "deployment": deployment,
        "protocol_handshake": handshake
    });
    save_json_pretty(&version_path, &version_manifest)
        .map_err(|e| format!("failed to write version manifest: {e}"))?;

    let candidate_files = vec![
        PathBuf::from("Cargo.lock"),
        PathBuf::from("apps/zeroclaw-app/package-lock.json"),
        PathBuf::from("apps/zeroclaw-app/src-tauri/Cargo.lock"),
        PathBuf::from("apps/zeroclaw-app/src-tauri/tauri.conf.json"),
    ];
    let mut sbom_components = Vec::new();
    for candidate in candidate_files {
        let absolute = std::env::current_dir()
            .map_err(|e| format!("failed to resolve cwd for sbom manifest: {e}"))?
            .join(&candidate);
        if absolute.exists() {
            let bytes = std::fs::read(&absolute).map_err(|e| {
                format!(
                    "failed to read {} for sbom manifest: {e}",
                    absolute.display()
                )
            })?;
            sbom_components.push(serde_json::json!({
                "path": candidate.display().to_string(),
                "sha256": sha256_hex(&bytes),
                "bytes": bytes.len(),
            }));
        }
    }
    let sbom_manifest = serde_json::json!({
        "generated_at": Utc::now().to_rfc3339(),
        "format": "right-hand-sbom-manifest-v1",
        "components": sbom_components
    });
    save_json_pretty(&sbom_path, &sbom_manifest)
        .map_err(|e| format!("failed to write sbom manifest: {e}"))?;

    let incident_pack = r#"# Security Incident + Vulnerability Reporting Pack

## Security Contact
- Email: security@example.com
- PGP: to-be-configured

## Operational SLA Targets (Template)
- Initial acknowledgment: <= 24h
- Triage complete: <= 72h
- Customer update cadence: every 24h until mitigation

## CRA/EU-ready Workflow (Template)
1. Detect incident/vulnerability.
2. Preserve immutable audit evidence package.
3. Classify severity and affected releases/endpoints.
4. Contain and rollback staged release if needed.
5. Notify impacted customers and regulators per legal obligations.
6. Publish remediation and verification evidence.
"#;
    std::fs::write(&incident_path, incident_pack)
        .map_err(|e| format!("failed to write incident workflow pack: {e}"))?;

    let files = vec![
        audit_path.display().to_string(),
        verify_path.display().to_string(),
        rollout_path.display().to_string(),
        rbac_path.display().to_string(),
        outcomes_path.display().to_string(),
        audit_remote_path.display().to_string(),
        billing_path.display().to_string(),
        workflow_path.display().to_string(),
        compliance_profile_path.display().to_string(),
        compliance_posture_path.display().to_string(),
        mission_path.display().to_string(),
        version_path.display().to_string(),
        sbom_path.display().to_string(),
        incident_path.display().to_string(),
    ];
    Ok(EvidenceExportSummary {
        output_dir: dir.display().to_string(),
        files,
    })
}

#[tauri::command]
fn control_plane_state(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<ControlPlaneState, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .get_state()
        .map_err(|e| format!("failed to load control-plane state: {e}"))
}

#[tauri::command]
fn access_state(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<AccessState, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .get_state()
        .map(|state| state.access_state)
        .map_err(|e| format!("failed to load access state: {e}"))
}

#[tauri::command]
fn access_start_trial(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<AccessState, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .set_paid_plan(AccessPlan::Org)
        .map_err(|e| format!("failed to enforce org workspace plan: {e}"))
}

#[tauri::command]
fn access_set_plan(
    profile_id: String,
    _plan: AccessPlan,
    state: State<'_, AppController>,
) -> std::result::Result<AccessState, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .set_paid_plan(AccessPlan::Org)
        .map_err(|e| format!("failed to enforce org workspace plan: {e}"))
}

#[tauri::command]
fn access_set_view(
    profile_id: String,
    _view: WorkspaceView,
    state: State<'_, AppController>,
) -> std::result::Result<AccessState, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .set_active_view(WorkspaceView::Org)
        .map_err(|e| format!("failed to enforce org workspace view: {e}"))
}

#[tauri::command]
fn policy_evaluate(
    profile_id: String,
    mut request: ActionPolicyRequest,
    state: State<'_, AppController>,
) -> std::result::Result<ActionPolicyDecision, String> {
    request.actor_role = normalize_actor_role(Some(request.actor_role.clone()));
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .evaluate_action(request)
        .map_err(|e| format!("failed to evaluate action policy: {e}"))
}

#[tauri::command]
fn approvals_list(
    profile_id: String,
    pending_only: Option<bool>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<ApprovalRequest>, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .list_approvals(pending_only.unwrap_or(false))
        .map_err(|e| format!("failed to list approvals: {e}"))
}

#[tauri::command]
fn approvals_resolve(
    profile_id: String,
    approval_id: String,
    approver_role: String,
    approved: bool,
    reason: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<ApprovalRequest, String> {
    let normalized_approver_role = normalize_approver_role(&approver_role);
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .resolve_approval(&approval_id, &normalized_approver_role, approved, reason)
        .map_err(|e| format!("failed to resolve approval: {e}"))
}

#[tauri::command]
fn receipts_list(
    profile_id: String,
    limit: Option<usize>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<ActionReceipt>, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .list_receipts(limit.unwrap_or(200))
        .map_err(|e| format!("failed to list receipts: {e}"))
}

#[tauri::command]
fn retention_set(
    profile_id: String,
    receipts_days: u32,
    approvals_days: u32,
    state: State<'_, AppController>,
) -> std::result::Result<RetentionPolicy, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .set_retention(receipts_days, approvals_days)
        .map_err(|e| format!("failed to update retention policy: {e}"))
}

#[tauri::command]
fn retention_purge(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<PurgeSummary, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    store
        .purge_by_retention()
        .map_err(|e| format!("failed to purge by retention policy: {e}"))
}

#[tauri::command]
fn receipts_export(
    profile_id: String,
    output_path: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<String, String> {
    let store = state
        .control_plane_store_for_profile(&profile_id)
        .map_err(|e| format!("failed to open control-plane store: {e}"))?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let default_path = workspace.logs_dir.join(format!(
        "receipts-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let output_path = output_path.map(PathBuf::from).unwrap_or(default_path);
    let exported = store
        .export_receipts(&output_path)
        .map_err(|e| format!("failed to export receipts: {e}"))?;
    Ok(exported.display().to_string())
}

#[tauri::command]
async fn runtime_start(
    profile_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    app: AppHandle,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "runtime.start",
        &format!("profile:{profile_id}"),
        "local",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;
    let setup = derive_setup_state(&workspace.root_dir, &cfg, &profile_id, &state)
        .map_err(|e| format!("failed to derive setup state: {e}"))?;
    if effective_deployment_mode(setup.deployment_mode) != DeploymentMode::Host {
        return Err(
            "runtime_start is disabled for deployment_mode=client; switch profile setup to host on desktop"
                .to_string(),
        );
    }
    validate_deployment_mode(DeploymentMode::Host)
        .map_err(|e| format!("runtime host mode is unavailable: {e}"))?;

    let previous_runtime = {
        let mut slot = state.runtime_slot.lock().await;
        let runtime = slot.runtime.take();
        slot.log_sink = None;
        slot.profile_id = None;
        runtime
    };

    if let Some(runtime) = previous_runtime {
        runtime
            .stop("switching runtime profile")
            .await
            .map_err(|e| format!("failed to stop existing runtime before restart: {e}"))?;
    }

    let sink = Arc::new(
        JsonlLogSink::new(LogSinkConfig::new(workspace.logs_dir.clone()))
            .map_err(|e| format!("failed to initialize profile logs: {e}"))?,
    );
    let runtime = Arc::new(LocalAgentRuntime::new(sink.clone()));

    let start = RuntimeStartConfig {
        profile_id: profile_id.clone(),
        config_path: workspace.config_path,
        workspace_dir: workspace.root_dir,
    };

    runtime
        .start(start)
        .await
        .map_err(|e| format!("failed to start runtime: {e}"))?;

    let mut rx = runtime.subscribe_events();
    tauri::async_runtime::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let _ = app.emit("runtime-event", event);
                }
                Err(error) => {
                    let _ = app.emit("runtime-event-error", error.to_string());
                    break;
                }
            }
        }
    });

    let mut slot = state.runtime_slot.lock().await;
    slot.runtime = Some(runtime);
    slot.log_sink = Some(sink);
    slot.profile_id = Some(profile_id);
    Ok(())
}

#[tauri::command]
async fn runtime_stop(
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    reason: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let profile_id = {
        let slot = state.runtime_slot.lock().await;
        slot.profile_id.clone()
    }
    .or_else(|| {
        state
            .profile_manager
            .get_active_profile()
            .ok()
            .flatten()
            .map(|p| p.id)
    })
    .ok_or_else(|| "missing profile for runtime stop policy check".to_string())?;

    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "runtime.stop",
        &format!("profile:{profile_id}"),
        "local",
        approval_id,
    )?;

    let runtime = {
        let mut slot = state.runtime_slot.lock().await;
        let runtime = slot.runtime.take();
        slot.log_sink = None;
        slot.profile_id = None;
        runtime
    };

    if let Some(runtime) = runtime {
        runtime
            .stop(reason.as_deref().unwrap_or("user requested stop"))
            .await
            .map_err(|e| format!("failed to stop runtime: {e}"))?;
    }

    Ok(())
}

#[tauri::command]
async fn runtime_send_message(
    message: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<String, String> {
    let (runtime, profile_id) = {
        let slot = state.runtime_slot.lock().await;
        (slot.runtime.clone(), slot.profile_id.clone())
    };
    let runtime = runtime.ok_or_else(|| "runtime is not started".to_string())?;
    let profile_id = profile_id.ok_or_else(|| "missing active profile id".to_string())?;

    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "runtime.send_message",
        &format!("profile:{profile_id}"),
        "provider",
        approval_id,
    )?;

    runtime
        .send_user_message(&message)
        .await
        .map_err(|e| format!("failed to send message: {e}"))
}

#[tauri::command]
async fn runtime_state(state: State<'_, AppController>) -> std::result::Result<String, String> {
    let runtime = {
        let slot = state.runtime_slot.lock().await;
        slot.runtime.clone()
    };

    if let Some(runtime) = runtime {
        return Ok(runtime.state().as_str().to_string());
    }

    Ok("stopped".to_string())
}

#[tauri::command]
async fn logs_tail(
    limit: Option<usize>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<LogLine>, String> {
    let sink = {
        let slot = state.runtime_slot.lock().await;
        slot.log_sink.clone()
    }
    .ok_or_else(|| "runtime is not started".to_string())?;

    sink.tail(limit.unwrap_or(200))
        .map_err(|e| format!("failed to tail logs: {e}"))
}

#[tauri::command]
async fn logs_export_diagnostics(
    output_path: Option<String>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<String, String> {
    let (sink, profile_id) = {
        let slot = state.runtime_slot.lock().await;
        (slot.log_sink.clone(), slot.profile_id.clone())
    };

    let sink = sink.ok_or_else(|| "runtime is not started".to_string())?;
    let profile_id = profile_id.ok_or_else(|| "missing active profile id".to_string())?;

    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "logs.export",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve active profile workspace: {e}"))?;

    let default_path = workspace.logs_dir.join(format!(
        "diagnostics-{}.jsonl",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let output_path = output_path
        .map(PathBuf::from)
        .unwrap_or(default_path)
        .to_path_buf();

    let exported = sink
        .export_diagnostics_bundle(&output_path)
        .map_err(|e| format!("failed to export diagnostics bundle: {e}"))?;

    Ok(exported.display().to_string())
}

#[tauri::command]
fn secret_set(
    profile_id: String,
    key: String,
    value: String,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    state
        .vault
        .set_secret(&profile_id, &key, &value)
        .map_err(|e| format!("failed to save secret: {e}"))
}

#[tauri::command]
fn secret_get(
    profile_id: String,
    key: String,
    state: State<'_, AppController>,
) -> std::result::Result<Option<String>, String> {
    state
        .vault
        .get_secret(&profile_id, &key)
        .map_err(|e| format!("failed to read secret: {e}"))
}

#[tauri::command]
fn secret_exists(
    profile_id: String,
    key: String,
    state: State<'_, AppController>,
) -> std::result::Result<bool, String> {
    state
        .vault
        .get_secret(&profile_id, &key)
        .map(|value| value.is_some())
        .map_err(|e| format!("failed to read secret existence: {e}"))
}

#[tauri::command]
fn secret_delete(
    profile_id: String,
    key: String,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    state
        .vault
        .delete_secret(&profile_id, &key)
        .map_err(|e| format!("failed to delete secret: {e}"))
}

#[tauri::command]
fn secret_backend(state: State<'_, AppController>) -> String {
    state.vault.backend_name().to_string()
}

#[tauri::command]
fn integration_install(
    profile_id: String,
    contract: IntegrationPermissionContract,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<IntegrationRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "integration.install",
        &format!("integration:{}", contract.integration_id),
        contract
            .data_destinations
            .first()
            .map_or("local", std::string::String::as_str),
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    IntegrationRegistryStore::for_workspace(&workspace.root_dir)
        .install(contract)
        .map_err(|e| format!("failed to install integration: {e}"))
}

#[tauri::command]
fn integration_enable(
    profile_id: String,
    integration_id: String,
    approved: bool,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<IntegrationRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "integration.enable",
        &format!("integration:{integration_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    IntegrationRegistryStore::for_workspace(&workspace.root_dir)
        .enable(&integration_id, approved)
        .map_err(|e| format!("failed to enable integration: {e}"))
}

#[tauri::command]
fn integration_disable(
    profile_id: String,
    integration_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<IntegrationRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "integration.disable",
        &format!("integration:{integration_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    IntegrationRegistryStore::for_workspace(&workspace.root_dir)
        .disable(&integration_id)
        .map_err(|e| format!("failed to disable integration: {e}"))
}

#[tauri::command]
fn integration_remove(
    profile_id: String,
    integration_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "integration.remove",
        &format!("integration:{integration_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    IntegrationRegistryStore::for_workspace(&workspace.root_dir)
        .remove(&integration_id)
        .map_err(|e| format!("failed to remove integration: {e}"))
}

#[tauri::command]
fn integration_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<IntegrationRegistry, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    IntegrationRegistryStore::for_workspace(&workspace.root_dir)
        .load()
        .map_err(|e| format!("failed to list integrations: {e}"))
}

#[tauri::command]
fn skills_install(
    profile_id: String,
    request: SkillInstallRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<SkillRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "skills.install",
        &format!("skill:{}", request.skill_id),
        request
            .contract
            .data_destinations
            .first()
            .map_or("local", std::string::String::as_str),
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    SkillsRegistryStore::for_workspace(&workspace.root_dir)
        .install(request)
        .map_err(|e| format!("failed to install skill: {e}"))
}

#[tauri::command]
fn skills_enable(
    profile_id: String,
    skill_id: String,
    approved: bool,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<SkillRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "skills.enable",
        &format!("skill:{skill_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    SkillsRegistryStore::for_workspace(&workspace.root_dir)
        .enable(&skill_id, approved)
        .map_err(|e| format!("failed to enable skill: {e}"))
}

#[tauri::command]
fn skills_disable(
    profile_id: String,
    skill_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<SkillRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "skills.disable",
        &format!("skill:{skill_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    SkillsRegistryStore::for_workspace(&workspace.root_dir)
        .disable(&skill_id)
        .map_err(|e| format!("failed to disable skill: {e}"))
}

#[tauri::command]
fn skills_remove(
    profile_id: String,
    skill_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "skills.remove",
        &format!("skill:{skill_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    SkillsRegistryStore::for_workspace(&workspace.root_dir)
        .remove(&skill_id)
        .map_err(|e| format!("failed to remove skill: {e}"))
}

#[tauri::command]
fn skills_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<SkillsRegistry, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    SkillsRegistryStore::for_workspace(&workspace.root_dir)
        .load()
        .map_err(|e| format!("failed to list skills: {e}"))
}

#[tauri::command]
fn mcp_install(
    profile_id: String,
    request: McpConnectorInstallRequest,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<McpConnectorRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "mcp.install",
        &format!("mcp:{}", request.connector_id),
        request
            .contract
            .data_destinations
            .first()
            .map_or("local", std::string::String::as_str),
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_tool_connectors_enabled(&workspace.root_dir)?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .install(request)
        .map_err(|e| format!("failed to install MCP connector: {e}"))
}

#[tauri::command]
fn mcp_update_config(
    profile_id: String,
    connector_id: String,
    config: McpConnectorConfig,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<McpConnectorRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "mcp.update_config",
        &format!("mcp:{connector_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_tool_connectors_enabled(&workspace.root_dir)?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .update_config(&connector_id, config)
        .map_err(|e| format!("failed to update MCP connector config: {e}"))
}

#[tauri::command]
fn mcp_enable(
    profile_id: String,
    connector_id: String,
    approved: bool,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<McpConnectorRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "mcp.enable",
        &format!("mcp:{connector_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_tool_connectors_enabled(&workspace.root_dir)?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .enable(&connector_id, approved)
        .map_err(|e| format!("failed to enable MCP connector: {e}"))
}

#[tauri::command]
fn mcp_disable(
    profile_id: String,
    connector_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<McpConnectorRecord, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "mcp.disable",
        &format!("mcp:{connector_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_tool_connectors_enabled(&workspace.root_dir)?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .disable(&connector_id)
        .map_err(|e| format!("failed to disable MCP connector: {e}"))
}

#[tauri::command]
fn mcp_remove(
    profile_id: String,
    connector_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "mcp.remove",
        &format!("mcp:{connector_id}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    ensure_tool_connectors_enabled(&workspace.root_dir)?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .remove(&connector_id)
        .map_err(|e| format!("failed to remove MCP connector: {e}"))
}

#[tauri::command]
fn mcp_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<McpConnectorRegistry, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    McpConnectorStore::for_workspace(&workspace.root_dir)
        .load()
        .map_err(|e| format!("failed to list MCP connectors: {e}"))
}

#[tauri::command]
fn pairing_create_bundle(
    profile_id: String,
    transport: String,
    endpoint: Option<String>,
    expires_in_minutes: Option<u32>,
    state: State<'_, AppController>,
) -> std::result::Result<PairingBundle, String> {
    let transport = parse_transport(&transport)?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    if let Some(policy) = policy_profile_load(&workspace.root_dir)
        .map_err(|e| format!("failed to load policy profile: {e}"))?
    {
        let transport_name = match transport {
            PairingTransport::Lan => "lan",
            PairingTransport::Tailscale => "tailscale",
            PairingTransport::CloudflareTunnel => "cloudflare",
            PairingTransport::NgrokTunnel => "ngrok",
        };
        if !policy.allowed_transports.is_empty()
            && !policy
                .allowed_transports
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(transport_name))
        {
            return Err(format!(
                "transport '{}' is blocked by policy profile '{}'",
                transport_name, policy.template_id
            ));
        }
    }

    let endpoint = endpoint.unwrap_or_else(|| match transport {
        PairingTransport::Lan => "http://127.0.0.1:8080".into(),
        PairingTransport::Tailscale => "https://zeroclaw-hub.tailnet.ts.net".into(),
        PairingTransport::CloudflareTunnel => "https://zeroclaw-hub.example.com".into(),
        PairingTransport::NgrokTunnel => "https://zeroclaw-hub.ngrok-free.app".into(),
    });

    create_pairing_bundle(PairingRequest {
        hub_device: format!("hub-{profile_id}"),
        endpoint,
        transport,
        expires_in_minutes: expires_in_minutes.unwrap_or(15),
    })
    .map_err(|e| format!("failed to create pairing bundle: {e}"))
}

#[tauri::command]
fn pairing_snapshot_sync_placeholder() -> String {
    "Encrypted snapshot sync is intentionally a placeholder for later implementation.".into()
}

#[tauri::command]
fn background_capabilities() -> BackgroundCapabilities {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        return zeroclaw_core::DesktopBackgroundAdapter::new(PathBuf::new(), PathBuf::new())
            .capabilities();
    }
    #[cfg(target_os = "android")]
    {
        return zeroclaw_core::AndroidBackgroundAdapter.capabilities();
    }
    #[cfg(target_os = "ios")]
    {
        return zeroclaw_core::IosBackgroundAdapter.capabilities();
    }
    #[allow(unreachable_code)]
    BackgroundCapabilities {
        supports_always_on: false,
        requires_ongoing_notification: false,
        best_effort_only: true,
    }
}

#[tauri::command]
fn background_enable(
    profile_id: Option<String>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let profile = resolve_profile_record(profile_id, &state)?;
    let _decision = evaluate_policy_gate(
        &profile.id,
        &state,
        actor_id,
        actor_role,
        "background.enable",
        &format!("profile:{}", profile.id),
        "local",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile.id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let adapter = background_adapter_for_workspace(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to initialize background adapter: {e}"))?;

    adapter
        .enable_background_mode()
        .map_err(|e| format!("failed to enable background mode: {e}"))
}

#[tauri::command]
fn background_disable(
    profile_id: Option<String>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<(), String> {
    let profile = resolve_profile_record(profile_id, &state)?;
    let _decision = evaluate_policy_gate(
        &profile.id,
        &state,
        actor_id,
        actor_role,
        "background.disable",
        &format!("profile:{}", profile.id),
        "local",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile.id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let adapter = background_adapter_for_workspace(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to initialize background adapter: {e}"))?;

    adapter
        .disable_background_mode()
        .map_err(|e| format!("failed to disable background mode: {e}"))
}

#[tauri::command]
fn operations_status(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<StatusReport, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    status_report(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to collect status report: {e}"))
}

#[tauri::command]
fn operations_doctor(
    profile_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "doctor.run",
        &format!("profile:{profile_id}"),
        "local",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    run_doctor(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to run doctor: {e}"))
}

#[tauri::command]
async fn operations_channel_doctor(
    profile_id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "channel.doctor",
        &format!("profile:{profile_id}"),
        "local",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    run_channel_doctor(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to run channel doctor: {e}"))
}

#[tauri::command]
fn operations_channels_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<ChannelSummary>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    channels_list(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to list channels: {e}"))
}

#[tauri::command]
async fn operations_channel_add(
    profile_id: String,
    channel_type: String,
    config_json: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "channel.add",
        &format!("channel:{channel_type}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    channel_add(
        &workspace.config_path,
        &workspace.root_dir,
        channel_type,
        config_json,
    )
    .await
    .map_err(|e| format!("failed to add channel: {e}"))
}

#[tauri::command]
async fn operations_channel_remove(
    profile_id: String,
    name: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "channel.remove",
        &format!("channel:{name}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    channel_remove(&workspace.config_path, &workspace.root_dir, name)
        .await
        .map_err(|e| format!("failed to remove channel: {e}"))
}

#[tauri::command]
async fn operations_channel_bind_telegram(
    profile_id: String,
    identity: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "channel.bind_telegram",
        &format!("channel:telegram:{identity}"),
        "integration",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    channel_bind_telegram(&workspace.config_path, &workspace.root_dir, identity)
        .await
        .map_err(|e| format!("failed to bind telegram identity: {e}"))
}

#[tauri::command]
fn operations_cron_list(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<CronJobSummary>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cron_list(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to list cron jobs: {e}"))
}

#[tauri::command]
fn operations_cron_add(
    profile_id: String,
    expression: String,
    command: String,
    tz: Option<String>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "cron.add",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cron_add(
        &workspace.config_path,
        &workspace.root_dir,
        expression,
        command,
        tz,
    )
    .map_err(|e| format!("failed to add cron job: {e}"))
}

#[tauri::command]
fn operations_cron_remove(
    profile_id: String,
    id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "cron.remove",
        &format!("cron:{id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cron_remove(&workspace.config_path, &workspace.root_dir, id)
        .map_err(|e| format!("failed to remove cron job: {e}"))
}

#[tauri::command]
fn operations_cron_pause(
    profile_id: String,
    id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "cron.pause",
        &format!("cron:{id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cron_pause(&workspace.config_path, &workspace.root_dir, id)
        .map_err(|e| format!("failed to pause cron job: {e}"))
}

#[tauri::command]
fn operations_cron_resume(
    profile_id: String,
    id: String,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "cron.resume",
        &format!("cron:{id}"),
        "workspace",
        approval_id,
    )?;
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cron_resume(&workspace.config_path, &workspace.root_dir, id)
        .map_err(|e| format!("failed to resume cron job: {e}"))
}

#[tauri::command]
fn operations_providers(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<ProviderDescriptor>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    providers_catalog(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to list providers: {e}"))
}

fn integration_setup_hint(name: &str) -> &'static str {
    match name {
        "Telegram" => "Create token in BotFather, then configure Telegram channel.",
        "Discord" => "Create bot token and message-content intent, then configure Discord channel.",
        "Slack" => "Create Slack app token + signing secret, then configure Slack channel.",
        "Webhooks" => "Set webhook endpoint/secret and route events to gateway.",
        "WhatsApp" => "Configure Meta Cloud API webhook and verify token.",
        "Signal" => "Install signal-cli and configure sender/allowlist.",
        "iMessage" => "macOS only; configure AppleScript bridge permissions.",
        "Matrix" => "Configure homeserver/user/device credentials for Matrix.",
        _ => "Use onboarding/docs to configure credentials and channel/provider settings.",
    }
}

#[tauri::command]
async fn operations_integrations_catalog(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<IntegrationCatalogEntry>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;

    Ok(zeroclaw::all_integrations()
        .into_iter()
        .map(|entry| {
            let status = match (entry.status_fn)(&cfg) {
                zeroclaw::IntegrationStatus::Active => "active",
                zeroclaw::IntegrationStatus::Available => "available",
                zeroclaw::IntegrationStatus::ComingSoon => "coming_soon",
            }
            .to_string();

            IntegrationCatalogEntry {
                name: entry.name.to_string(),
                description: entry.description.to_string(),
                category: entry.category.label().to_string(),
                status,
                setup_hint: integration_setup_hint(entry.name).to_string(),
            }
        })
        .collect())
}

#[tauri::command]
fn operations_models_refresh(
    profile_id: String,
    provider: Option<String>,
    force: Option<bool>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let provider_name = provider.clone().unwrap_or_else(|| "default".into());
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "models.refresh",
        &format!("provider:{provider_name}"),
        "provider",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    refresh_models(
        &workspace.config_path,
        &workspace.root_dir,
        provider,
        force.unwrap_or(false),
    )
    .map_err(|e| format!("failed to refresh models: {e}"))
}

#[tauri::command]
fn operations_service(
    profile_id: String,
    action: ServiceLifecycleAction,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let policy_action = match &action {
        ServiceLifecycleAction::Install => "service.install",
        ServiceLifecycleAction::Start => "service.start",
        ServiceLifecycleAction::Stop => "service.stop",
        ServiceLifecycleAction::Status => "service.status",
        ServiceLifecycleAction::Uninstall => "service.uninstall",
    };

    if !matches!(&action, ServiceLifecycleAction::Status) {
        let _decision = evaluate_policy_gate(
            &profile_id,
            &state,
            actor_id,
            actor_role,
            policy_action,
            &format!("profile:{profile_id}"),
            "local",
            approval_id,
        )?;
    }

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    run_service_lifecycle(&workspace.config_path, &workspace.root_dir, action)
        .map_err(|e| format!("failed to run service action: {e}"))
}

#[tauri::command]
fn operations_config_schema() -> std::result::Result<serde_json::Value, String> {
    let schema = schemars::schema_for!(zeroclaw::Config);
    serde_json::to_value(&schema).map_err(|e| format!("failed to encode config schema: {e}"))
}

#[tauri::command]
async fn operations_auth_profiles(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<AuthProfileSummary>, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;
    let auth = zeroclaw::auth::AuthService::from_config(&cfg);
    let data = auth
        .load_profiles()
        .await
        .map_err(|e| format!("failed to load auth profiles: {e}"))?;

    let mut summaries = data
        .profiles
        .values()
        .map(|profile| {
            let active = data
                .active_profiles
                .get(&profile.provider)
                .map(|value| value == &profile.id)
                .unwrap_or(false);
            let kind = match profile.kind {
                zeroclaw::auth::profiles::AuthProfileKind::OAuth => "oauth",
                zeroclaw::auth::profiles::AuthProfileKind::Token => "token",
            };
            AuthProfileSummary {
                id: profile.id.clone(),
                provider: profile.provider.clone(),
                profile_name: profile.profile_name.clone(),
                kind: kind.to_string(),
                active,
                account_id: profile.account_id.clone(),
                workspace_id: profile.workspace_id.clone(),
                expires_at: profile.token_set.as_ref().and_then(|tokens| {
                    tokens.expires_at.as_ref().map(chrono::DateTime::to_rfc3339)
                }),
                updated_at: profile.updated_at.to_rfc3339(),
            }
        })
        .collect::<Vec<_>>();
    summaries.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then(a.profile_name.cmp(&b.profile_name))
    });
    Ok(summaries)
}

#[tauri::command]
async fn operations_memory_list(
    profile_id: String,
    category: Option<String>,
    session_id: Option<String>,
    limit: Option<usize>,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<Vec<MemoryEntrySummary>, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "memory.list",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    let cfg = load_or_init_profile_config(&workspace.config_path, &workspace.root_dir)
        .await
        .map_err(|e| format!("failed to load profile config: {e}"))?;

    let memory =
        zeroclaw::memory::create_memory(&cfg.memory, &workspace.root_dir, cfg.api_key.as_deref())
            .map_err(|e| format!("failed to initialize memory backend: {e}"))?;

    let category_filter = category
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(parse_memory_category);

    let mut entries = memory
        .list(category_filter.as_ref(), session_id.as_deref())
        .await
        .map_err(|e| format!("failed to list memory entries: {e}"))?;

    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let max = limit.unwrap_or(200).clamp(1, 1000);
    entries.truncate(max);

    Ok(entries
        .into_iter()
        .map(|entry| MemoryEntrySummary {
            id: entry.id,
            key: entry.key,
            category: entry.category.to_string(),
            timestamp: entry.timestamp,
            session_id: entry.session_id,
            score: entry.score,
            content_preview: truncate_preview(&entry.content, 160),
        })
        .collect())
}

#[tauri::command]
fn operations_command_surface() -> Vec<CommandSurfaceCapability> {
    vec![
        CommandSurfaceCapability {
            family: "onboard".into(),
            supported: true,
            coverage: "core".into(),
            note: "setup wizard and quick onboarding".into(),
        },
        CommandSurfaceCapability {
            family: "agent".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "runtime start/stop/message in command center".into(),
        },
        CommandSurfaceCapability {
            family: "gateway".into(),
            supported: true,
            coverage: "core".into(),
            note: "managed through runtime/service controls".into(),
        },
        CommandSurfaceCapability {
            family: "daemon".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "desktop background mode with service lifecycle".into(),
        },
        CommandSurfaceCapability {
            family: "service".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "install/start/stop/status/uninstall exposed".into(),
        },
        CommandSurfaceCapability {
            family: "doctor".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "doctor and channel doctor operations available".into(),
        },
        CommandSurfaceCapability {
            family: "status".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "status summary and live runtime state available".into(),
        },
        CommandSurfaceCapability {
            family: "cron".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "list/add/remove/pause/resume exposed".into(),
        },
        CommandSurfaceCapability {
            family: "models".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "model catalog refresh available".into(),
        },
        CommandSurfaceCapability {
            family: "providers".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "provider catalog exposed".into(),
        },
        CommandSurfaceCapability {
            family: "channel".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "list/add/remove/bind telegram exposed".into(),
        },
        CommandSurfaceCapability {
            family: "integrations".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "integration catalog and permission contract exposed".into(),
        },
        CommandSurfaceCapability {
            family: "integration_catalog".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "built-in integration catalog across categories exposed with setup hints".into(),
        },
        CommandSurfaceCapability {
            family: "skills".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "skill install/enable/disable/remove exposed".into(),
        },
        CommandSurfaceCapability {
            family: "tool_connectors".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "MCP connector install/enable/disable with explicit setup opt-in".into(),
        },
        CommandSurfaceCapability {
            family: "migrate".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "OpenClaw memory migration is available in operations panel".into(),
        },
        CommandSurfaceCapability {
            family: "auth".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "sanitized auth profile visibility exposed".into(),
        },
        CommandSurfaceCapability {
            family: "hardware".into(),
            supported: true,
            coverage: "core".into(),
            note: "hardware commands available in core CLI/runtime".into(),
        },
        CommandSurfaceCapability {
            family: "peripheral".into(),
            supported: true,
            coverage: "core".into(),
            note: "peripheral tool chain available in core".into(),
        },
        CommandSurfaceCapability {
            family: "memory".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "memory listing surfaced for setup and diagnostics".into(),
        },
        CommandSurfaceCapability {
            family: "config".into(),
            supported: true,
            coverage: "core + ui".into(),
            note: "config schema exported for validation/help".into(),
        },
        CommandSurfaceCapability {
            family: "rollout".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "staged release, ring promotion, and rollback controls exposed".into(),
        },
        CommandSurfaceCapability {
            family: "rbac".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "central role assignments for admin/manager/user/observer exposed".into(),
        },
        CommandSurfaceCapability {
            family: "audit".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "tamper-evident audit chain verification/export exposed".into(),
        },
        CommandSurfaceCapability {
            family: "audit_remote".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "remote append-only audit sink configuration and sync exposed".into(),
        },
        CommandSurfaceCapability {
            family: "outcomes".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "business outcome records and solved-rate metrics exposed".into(),
        },
        CommandSurfaceCapability {
            family: "workflow".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "kanban-style task tracking by workspace/agent/skill/tool exposed".into(),
        },
        CommandSurfaceCapability {
            family: "compliance".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "AI Act/NIST/industry compliance profile and posture checks exposed".into(),
        },
        CommandSurfaceCapability {
            family: "billing".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "tier entitlement verification and backend receipt checks exposed".into(),
        },
        CommandSurfaceCapability {
            family: "evidence".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "one-click evidence export pack for compliance operations".into(),
        },
        CommandSurfaceCapability {
            family: "completions".into(),
            supported: true,
            coverage: "wrapper + ui".into(),
            note: "shell completion text can be generated in-app using the local zeroclaw binary"
                .into(),
        },
    ]
}

#[tauri::command]
fn operations_cost_summary(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<CostSummaryReport, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    cost_summary(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to collect cost summary: {e}"))
}

#[tauri::command]
fn operations_response_cache_stats(
    profile_id: String,
    state: State<'_, AppController>,
) -> std::result::Result<ResponseCacheStatsReport, String> {
    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;
    response_cache_stats(&workspace.config_path, &workspace.root_dir)
        .map_err(|e| format!("failed to collect response cache stats: {e}"))
}

#[tauri::command]
async fn operations_migrate_openclaw(
    profile_id: String,
    source: Option<String>,
    dry_run: bool,
    actor_id: Option<String>,
    actor_role: Option<String>,
    approval_id: Option<String>,
    state: State<'_, AppController>,
) -> std::result::Result<OperationResult, String> {
    let _decision = evaluate_policy_gate(
        &profile_id,
        &state,
        actor_id,
        actor_role,
        "migrate.openclaw",
        &format!("profile:{profile_id}"),
        "workspace",
        approval_id,
    )?;

    let workspace = state
        .profile_manager
        .workspace_for_profile(&profile_id)
        .map_err(|e| format!("failed to resolve profile workspace: {e}"))?;

    let source = source.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(PathBuf::from(trimmed))
        }
    });

    migrate_openclaw(&workspace.config_path, &workspace.root_dir, source, dry_run)
        .await
        .map_err(|e| format!("failed to migrate OpenClaw memory: {e}"))
}

fn normalized_completion_shell(raw: &str) -> std::result::Result<&'static str, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "bash" => Ok("bash"),
        "zsh" => Ok("zsh"),
        "fish" => Ok("fish"),
        "powershell" | "pwsh" => Ok("powershell"),
        "elvish" => Ok("elvish"),
        value => Err(format!("unsupported shell '{value}'")),
    }
}

#[tauri::command]
fn operations_generate_shell_completions(
    shell: String,
    binary_path: Option<String>,
    app: AppHandle,
) -> std::result::Result<String, String> {
    let shell = normalized_completion_shell(&shell)?;
    let binary = resolve_zeroclaw_binary(binary_path.as_deref(), &app)?;
    let output = Command::new(&binary)
        .arg("completions")
        .arg(shell)
        .output()
        .map_err(|error| {
            format!(
                "failed to execute '{} completions {shell}': {error}",
                binary.display()
            )
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "zeroclaw completions command failed with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| format!("completion output was not valid utf-8: {error}"))
}

fn zeroclaw_binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "zeroclaw.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "zeroclaw"
    }
}

fn resolve_zeroclaw_binary(
    binary_path: Option<&str>,
    app: &AppHandle,
) -> std::result::Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(raw) = binary_path {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed));
        }
    }
    if let Some(raw) = env::var_os("ZEROCLAW_BIN") {
        candidates.push(PathBuf::from(raw));
    }

    let binary_name = zeroclaw_binary_name();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("bin").join(binary_name));
        candidates.push(resource_dir.join("binaries").join(binary_name));
        candidates.push(resource_dir.join(binary_name));
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            candidates.push(parent.join(binary_name));
            candidates.push(parent.join("bin").join(binary_name));
            candidates.push(parent.join("binaries").join(binary_name));
        }
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    if let Some(from_path) = lookup_binary_in_path(binary_name) {
        return Ok(from_path);
    }

    Err(
        "failed to locate zeroclaw binary. set completion binary path in UI, set ZEROCLAW_BIN, or package sidecar binary under app resources/bin"
            .to_string(),
    )
}

fn lookup_binary_in_path(binary_name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn parse_transport(raw: &str) -> std::result::Result<PairingTransport, String> {
    match raw.to_ascii_lowercase().as_str() {
        "lan" => Ok(PairingTransport::Lan),
        "tailscale" => Ok(PairingTransport::Tailscale),
        "cloudflare" | "cloudflare_tunnel" => Ok(PairingTransport::CloudflareTunnel),
        "ngrok" | "ngrok_tunnel" => Ok(PairingTransport::NgrokTunnel),
        _ => Err(format!("unknown transport '{raw}'")),
    }
}

fn resolve_profile_record(
    profile_id: Option<String>,
    state: &State<'_, AppController>,
) -> std::result::Result<ProfileRecord, String> {
    match profile_id {
        Some(id) => state
            .profile_manager
            .switch_active_profile(&id)
            .map_err(|e| format!("failed to resolve profile '{id}': {e}")),
        None => state
            .active_profile_fallback()
            .map_err(|e| format!("failed to resolve active profile: {e}")),
    }
}

fn background_adapter_for_workspace(
    config_path: &Path,
    workspace_dir: &Path,
) -> Result<Box<dyn PlatformBackground>> {
    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    {
        return Ok(Box::new(zeroclaw_core::DesktopBackgroundAdapter::new(
            config_path.to_path_buf(),
            workspace_dir.to_path_buf(),
        )));
    }
    #[cfg(target_os = "android")]
    {
        let _ = (config_path, workspace_dir);
        return Ok(Box::new(zeroclaw_core::AndroidBackgroundAdapter));
    }
    #[cfg(target_os = "ios")]
    {
        let _ = (config_path, workspace_dir);
        return Ok(Box::new(zeroclaw_core::IosBackgroundAdapter));
    }
    #[allow(unreachable_code)]
    {
        let _ = (config_path, workspace_dir);
        anyhow::bail!("background adapter is not supported on this target")
    }
}

pub fn run() {
    let controller = AppController::new().unwrap_or_else(|error| {
        panic!("failed to initialize app controller: {error}");
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(controller)
        .setup(|app| {
            let state = app.state::<AppController>();
            let _ = app.emit("app-root", state.app_root.display().to_string());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            protocol_handshake,
            profiles_list,
            profiles_create,
            profiles_switch,
            profile_setup_get,
            profile_setup_save,
            deployment_capabilities,
            policy_profiles_list,
            policy_profile_get,
            policy_profile_apply,
            compliance_profiles_list,
            compliance_profile_get,
            compliance_profile_apply,
            compliance_posture_get,
            host_connection_get,
            client_connect_host,
            rbac_users_list,
            rbac_user_upsert,
            rollout_state_get,
            rollout_stage_release,
            rollout_set_signing_policy,
            rollout_promote,
            rollout_rollback,
            audit_log_list,
            audit_log_verify,
            audit_log_export,
            audit_remote_get,
            audit_remote_configure,
            audit_remote_sync,
            billing_state_get,
            billing_config_set,
            billing_verify_receipt,
            workflow_board_get,
            workflow_task_upsert,
            workflow_task_move,
            outcomes_record,
            outcomes_list,
            outcomes_summary,
            mission_control_summary,
            evidence_export,
            control_plane_state,
            access_state,
            access_start_trial,
            access_set_plan,
            access_set_view,
            policy_evaluate,
            approvals_list,
            approvals_resolve,
            receipts_list,
            receipts_export,
            retention_set,
            retention_purge,
            runtime_start,
            runtime_stop,
            runtime_send_message,
            runtime_state,
            logs_tail,
            logs_export_diagnostics,
            secret_set,
            secret_get,
            secret_exists,
            secret_delete,
            secret_backend,
            integration_install,
            integration_enable,
            integration_disable,
            integration_remove,
            integration_list,
            skills_install,
            skills_enable,
            skills_disable,
            skills_remove,
            skills_list,
            mcp_install,
            mcp_update_config,
            mcp_enable,
            mcp_disable,
            mcp_remove,
            mcp_list,
            pairing_create_bundle,
            pairing_snapshot_sync_placeholder,
            operations_status,
            operations_doctor,
            operations_channel_doctor,
            operations_channels_list,
            operations_channel_add,
            operations_channel_remove,
            operations_channel_bind_telegram,
            operations_providers,
            operations_integrations_catalog,
            operations_models_refresh,
            operations_cron_list,
            operations_cron_add,
            operations_cron_remove,
            operations_cron_pause,
            operations_cron_resume,
            operations_service,
            operations_config_schema,
            operations_auth_profiles,
            operations_memory_list,
            operations_migrate_openclaw,
            operations_command_surface,
            operations_cost_summary,
            operations_response_cache_stats,
            operations_generate_shell_completions,
            background_capabilities,
            background_enable,
            background_disable
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")
        .unwrap();
}
