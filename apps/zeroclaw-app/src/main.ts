type ProtocolHandshake = {
  core_protocol_version: string;
  event_schema_version: number;
  config_schema_version: number;
};

type ProfileRecord = {
  id: string;
  display_name: string;
  workspace_dir: string;
  created_at: string;
  updated_at: string;
};

type ProfilesIndex = {
  version: number;
  active_profile: string | null;
  profiles: ProfileRecord[];
};

type PermissionContract = {
  integration_id: string;
  can_access: string[];
  can_do: string[];
  data_destinations: string[];
};

type IntegrationRecord = {
  integration_id: string;
  installed_at: string;
  enabled: boolean;
  enabled_at: string | null;
  contract: PermissionContract;
};

type IntegrationRegistry = { records: IntegrationRecord[] };

type SkillInstallRequest = {
  skill_id: string;
  display_name: string;
  source: string;
  version: string;
  manifest_markdown?: string;
  contract: PermissionContract;
};

type SkillRecord = {
  skill_id: string;
  display_name: string;
  source: string;
  version: string;
  installed_at: string;
  enabled: boolean;
  enabled_at: string | null;
  skill_dir: string;
  contract: PermissionContract;
};

type SkillsRegistry = { records: SkillRecord[] };

type McpConnectorConfig = {
  transport: string;
  endpoint?: string;
  command?: string;
  args: string[];
  env_secret_ids: string[];
  timeout_secs?: number;
};

type McpConnectorInstallRequest = {
  connector_id: string;
  display_name: string;
  config: McpConnectorConfig;
  contract: PermissionContract;
};

type McpConnectorRecord = {
  connector_id: string;
  display_name: string;
  installed_at: string;
  updated_at: string;
  enabled: boolean;
  enabled_at: string | null;
  config: McpConnectorConfig;
  contract: PermissionContract;
};

type McpConnectorRegistry = { records: McpConnectorRecord[] };

type LogLine = {
  timestamp: string;
  level: string;
  component: string;
  message: string;
};

type PairingBundle = {
  pairing_id: string;
  hub_device: string;
  qr_payload: string;
  endpoint: string;
  transport: string;
  access_token: string;
  created_at: string;
  expires_at: string;
  snapshot_sync_mode: string;
  notes: string;
};

type WorkspaceView = "personal" | "org";
type AccessPlan = "trial" | "personal" | "org";
type SetupWorkspaceMode = "workspace";
type DeploymentMode = "host" | "client";
type WorkspaceRole = "admin" | "manager" | "user" | "observer";
type SubscriptionTier = "basic" | "professional" | "enterprise";

type AccessState = {
  plan: AccessPlan;
  active_view: WorkspaceView;
  trial_started_at: string | null;
  trial_expires_at: string | null;
  updated_at: string;
};

type PolicyRule = {
  id: string;
  actor_roles: string[];
  actions: string[];
  resources: string[];
  destinations: string[];
  require_approval: boolean;
  enabled: boolean;
};

type ActionPolicyRequest = {
  actor_id: string;
  actor_role: string;
  action: string;
  resource: string;
  destination: string;
  approval_id?: string;
  occurred_at?: string;
  context?: Record<string, unknown>;
};

type ActionPolicyDecision = {
  allowed: boolean;
  requires_approval: boolean;
  reason: string;
  approval_id: string | null;
  receipt_id: string;
};

type ApprovalStatus = "pending" | "approved" | "rejected";
type ReceiptResult = "allowed" | "denied" | "pending_approval";

type ActionReceipt = {
  id: string;
  timestamp: string;
  actor_id: string;
  actor_role: string;
  action: string;
  resource: string;
  destination: string;
  result: ReceiptResult;
  reason: string;
  context: Record<string, unknown>;
};

type ApprovalRequest = {
  id: string;
  created_at: string;
  actor_id: string;
  actor_role: string;
  action: string;
  resource: string;
  destination: string;
  status: ApprovalStatus;
  decided_by: string | null;
  decided_at: string | null;
  reason: string | null;
  context: Record<string, unknown>;
};

type RetentionPolicy = {
  receipts_days: number;
  approvals_days: number;
};

type PurgeSummary = {
  removed_receipts: number;
  removed_approvals: number;
};

type StatusReport = {
  config_path: string;
  workspace_dir: string;
  default_provider: string | null;
  default_model: string | null;
  temperature: number;
  gateway_host: string;
  gateway_port: number;
  channels: Record<string, boolean>;
  peripherals_enabled: boolean;
  peripheral_boards: number;
};

type ProviderDescriptor = {
  name: string;
  display_name: string;
  aliases: string[];
  local: boolean;
  active: boolean;
};

type IntegrationCatalogEntry = {
  name: string;
  description: string;
  category: string;
  status: "active" | "available" | "coming_soon" | string;
  setup_hint: string;
};

type ServiceLifecycleAction = "install" | "start" | "stop" | "status" | "uninstall";

type OperationResult = {
  operation: string;
  ok: boolean;
  detail: string;
};

type DelegateAgentSetup = {
  provider: string;
  model: string;
  system_prompt?: string;
  temperature?: number;
  max_depth?: number;
  agentic?: boolean;
  allowed_tools?: string[];
  max_iterations?: number;
};

type ProfileSetupState = {
  user_display_name: string;
  agent_name: string;
  workspace_mode: SetupWorkspaceMode;
  deployment_mode: DeploymentMode;
  workspace_role: WorkspaceRole;
  subscription_tier: SubscriptionTier;
  orchestrator_mode: string;
  provider: string;
  model: string;
  api_url?: string;
  default_temperature: number;
  memory_backend: string;
  runtime_reasoning_enabled?: boolean;
  agent_compact_context: boolean;
  agent_parallel_tools: boolean;
  agent_max_tool_iterations: number;
  agent_max_history_messages: number;
  agent_tool_dispatcher: string;
  skills_prompt_injection_mode: "full" | "compact" | string;
  skills_open_enabled: boolean;
  skills_open_dir?: string;
  enable_tool_connectors: boolean;
  delegate_agents: Record<string, DelegateAgentSetup>;
  has_provider_key: boolean;
  provider_key_id: string;
  updated_at: string;
};

type ProfileSetupPayload = {
  user_display_name: string;
  agent_name: string;
  workspace_mode: SetupWorkspaceMode;
  deployment_mode: DeploymentMode;
  workspace_role: WorkspaceRole;
  subscription_tier: SubscriptionTier;
  orchestrator_mode: string;
  provider: string;
  model: string;
  api_url?: string;
  default_temperature: number;
  memory_backend: string;
  runtime_reasoning_enabled?: boolean;
  agent_compact_context: boolean;
  agent_parallel_tools: boolean;
  agent_max_tool_iterations: number;
  agent_max_history_messages: number;
  agent_tool_dispatcher: string;
  skills_prompt_injection_mode: "full" | "compact" | string;
  skills_open_enabled: boolean;
  skills_open_dir?: string;
  enable_tool_connectors: boolean;
  delegate_agents: Record<string, DelegateAgentSetup>;
  api_key?: string;
};

type ChannelSummary = {
  channel_type: string;
  configured: boolean;
};

type CronJobSummary = {
  id: string;
  schedule: string;
  command: string;
  enabled: boolean;
  next_run: string;
  last_run?: string | null;
  last_status?: string | null;
};

type ControlPlaneState = {
  version: number;
  access_state: AccessState;
  policy_rules: PolicyRule[];
  retention: RetentionPolicy;
  receipts: ActionReceipt[];
  approvals: ApprovalRequest[];
};

type ModelCostSummary = {
  model: string;
  request_count: number;
  total_tokens: number;
  total_cost_usd: number;
};

type CostSummaryReport = {
  enabled: boolean;
  total_cost_usd: number;
  daily_cost_usd: number;
  monthly_cost_usd: number;
  total_tokens: number;
  request_count: number;
  by_model: ModelCostSummary[];
};

type ResponseCacheStatsReport = {
  enabled: boolean;
  ttl_minutes: number;
  max_entries: number;
  entries: number;
  hits: number;
  tokens_saved: number;
};

type DeploymentCapabilities = {
  platform: string;
  supports_host: boolean;
  supports_client: boolean;
  configured_mode: DeploymentMode;
  effective_mode: DeploymentMode;
  workspace_mode: SetupWorkspaceMode;
  workspace_role: WorkspaceRole;
  subscription_tier: SubscriptionTier;
  note: string;
};

type HostConnectionState = {
  connected: boolean;
  endpoint: string | null;
  transport: string | null;
  pairing_token_hint: string | null;
  connected_at: string | null;
  updated_at: string;
  last_error: string | null;
};

type HostConnectPayload = {
  invite_payload: string;
};

type RbacUserRecord = {
  user_id: string;
  display_name: string;
  role: WorkspaceRole;
  active: boolean;
  created_at: string;
  updated_at: string;
};

type RbacRegistry = {
  version: number;
  users: RbacUserRecord[];
  updated_at: string;
};

type RolloutRing = "pilot" | "group" | "all";

type ReleaseDescriptor = {
  release_id: string;
  version: string;
  checksum_sha256: string;
  signature?: string | null;
  sbom_checksum_sha256?: string | null;
  ring: RolloutRing;
  staged_at: string;
};

type RolloutState = {
  version: number;
  current_release: ReleaseDescriptor | null;
  previous_release: ReleaseDescriptor | null;
  staged_release: ReleaseDescriptor | null;
  signature_required: boolean;
  trusted_signers: string[];
  last_verified_signer?: string | null;
  last_promoted_at?: string | null;
  last_verification_error?: string | null;
  updated_at: string;
};

type RolloutStageRequest = {
  release_id: string;
  version: string;
  checksum_sha256: string;
  signature?: string;
  sbom_checksum_sha256?: string;
  ring: RolloutRing;
};

type RolloutSigningPolicyRequest = {
  signature_required: boolean;
  trusted_signers: string[];
};

type PolicyProfileTemplate = {
  template_id: string;
  display_name: string;
  description: string;
  allowed_providers: string[];
  allowed_transports: string[];
  allow_public_bind: boolean;
  require_pairing: boolean;
};

type PolicyProfileState = {
  template_id: string;
  applied_at: string;
  allowed_providers: string[];
  allowed_transports: string[];
  allow_public_bind: boolean;
  require_pairing: boolean;
};

type AuditEvent = {
  id: string;
  timestamp: string;
  actor_id: string;
  actor_role: string;
  action: string;
  resource: string;
  destination: string;
  result: string;
  reason: string;
  receipt_id: string;
  approval_id: string | null;
  prev_hash: string;
  hash: string;
};

type AuditLogVerification = {
  valid: boolean;
  entries: number;
  last_hash: string | null;
  error: string | null;
};

type AuditRemoteSinkState = {
  version: number;
  enabled: boolean;
  endpoint: string | null;
  sink_kind: string;
  auth_secret_id: string | null;
  verify_tls: boolean;
  batch_size: number;
  last_synced_hash: string | null;
  last_synced_at: string | null;
  last_error: string | null;
  updated_at: string;
};

type AuditRemoteConfigureRequest = {
  enabled: boolean;
  endpoint?: string;
  sink_kind?: string;
  auth_secret_id?: string;
  verify_tls?: boolean;
  batch_size?: number;
};

type AuditRemoteSyncResult = {
  endpoint: string;
  sink_kind: string;
  events_sent: number;
  first_hash: string | null;
  last_hash: string | null;
  synced_at: string;
};

type BillingEntitlementStatus = "active" | "grace" | "expired" | "unverified";

type BillingEntitlement = {
  tier: SubscriptionTier;
  status: BillingEntitlementStatus;
  verified: boolean;
  source: string;
  account_id: string | null;
  entitlement_id: string | null;
  receipt_id: string | null;
  expires_at: string | null;
  last_verified_at: string | null;
  last_error: string | null;
};

type BillingState = {
  version: number;
  backend_url: string | null;
  auth_secret_id: string | null;
  enforce_verification: boolean;
  entitlement: BillingEntitlement;
  updated_at: string;
};

type BillingConfigRequest = {
  backend_url?: string;
  auth_secret_id?: string;
  enforce_verification: boolean;
};

type BillingReceiptVerifyRequest = {
  receipt_payload: string;
  platform?: string;
};

type WorkflowTaskStatus = "pending" | "in_progress" | "done" | "failed" | "blocked";
type WorkflowTaskPriority = "low" | "medium" | "high" | "critical";

type WorkflowTaskRecord = {
  id: string;
  title: string;
  description: string | null;
  status: WorkflowTaskStatus;
  priority: WorkflowTaskPriority;
  owner: string | null;
  workspace_scope: string;
  runtime_task_id: string | null;
  agent_id: string | null;
  skill_id: string | null;
  tool_id: string | null;
  tags: string[];
  risk_score: number;
  related_receipt_id: string | null;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  completed_at: string | null;
};

type WorkflowBoardSummary = {
  total: number;
  pending: number;
  in_progress: number;
  done: number;
  failed: number;
  blocked: number;
  high_risk_open: number;
};

type WorkflowBoardState = {
  version: number;
  tasks: WorkflowTaskRecord[];
  updated_at: string;
};

type WorkflowBoardView = {
  summary: WorkflowBoardSummary;
  tasks: WorkflowTaskRecord[];
};

type WorkflowTaskUpsertRequest = {
  id?: string;
  title: string;
  description?: string;
  status?: WorkflowTaskStatus;
  priority?: WorkflowTaskPriority;
  owner?: string;
  runtime_task_id?: string;
  agent_id?: string;
  skill_id?: string;
  tool_id?: string;
  tags?: string[];
  risk_score?: number;
  related_receipt_id?: string;
};

type WorkflowTaskMoveRequest = {
  task_id: string;
  status: WorkflowTaskStatus;
};

type ComplianceProfileTemplate = {
  template_id: string;
  display_name: string;
  description: string;
  industry: string;
  standards: string[];
  recommended_policy_template: string | null;
  minimum_tier: SubscriptionTier;
  require_signed_release: boolean;
  require_remote_audit: boolean;
  require_billing_verification: boolean;
  require_pairing: boolean;
};

type ComplianceProfileState = {
  template_id: string;
  applied_at: string;
  industry: string;
  standards: string[];
  recommended_policy_template: string | null;
  minimum_tier: SubscriptionTier;
  require_signed_release: boolean;
  require_remote_audit: boolean;
  require_billing_verification: boolean;
  require_pairing: boolean;
};

type ComplianceControlCheck = {
  control_id: string;
  label: string;
  framework: string;
  required: boolean;
  satisfied: boolean;
  evidence: string | null;
  recommendation: string | null;
};

type CompliancePosture = {
  template_id: string | null;
  standards: string[];
  compliant: boolean;
  generated_at: string;
  checks: ComplianceControlCheck[];
  missing_controls: string[];
};

type OutcomeStatus = "solved" | "partial" | "unsolved";

type OutcomeRecord = {
  id: string;
  timestamp: string;
  title: string;
  status: OutcomeStatus;
  impact_score: number;
  owner?: string | null;
  related_receipt_id?: string | null;
  notes?: string | null;
};

type OutcomeUpsertRequest = {
  title: string;
  status: OutcomeStatus;
  impact_score: number;
  owner?: string;
  related_receipt_id?: string;
  notes?: string;
};

type OutcomeSummary = {
  total: number;
  solved: number;
  partial: number;
  unsolved: number;
  solved_rate: number;
  avg_impact_score: number;
};

type MissionControlSummary = {
  deployment: DeploymentCapabilities;
  rollout: RolloutState;
  rbac_users: number;
  audit: AuditLogVerification;
  audit_remote: AuditRemoteSinkState;
  billing: BillingState;
  workflow: WorkflowBoardSummary;
  compliance: CompliancePosture;
  outcomes: OutcomeSummary;
  approvals_pending: number;
  receipts_total: number;
};

type EvidenceExportSummary = {
  output_dir: string;
  files: string[];
};

const statusEl = must<HTMLParagraphElement>("status");
const runtimeModeEl = must<HTMLParagraphElement>("runtime-mode");
const handshakeOutputEl = must<HTMLPreElement>("handshake-output");
const activityOutputEl = must<HTMLPreElement>("activity-output");
const logsOutputEl = must<HTMLPreElement>("logs-output");
const runtimeOutputEl = must<HTMLPreElement>("runtime-output");
const pairingOutputEl = must<HTMLPreElement>("pairing-output");
const runtimeStateEl = must<HTMLSpanElement>("runtime-state");
const profileSelectEl = must<HTMLSelectElement>("profile-select");
const profileAvatarEl = must<HTMLSpanElement>("profile-avatar");
const profileDisplayNameEl = must<HTMLElement>("profile-display-name");
const profileDisplayMetaEl = must<HTMLElement>("profile-display-meta");
const profileNameEl = must<HTMLInputElement>("profile-name");
const runtimeMessageEl = must<HTMLTextAreaElement>("runtime-message");
const setupOutputEl = must<HTMLPreElement>("setup-output");

const setupUserNameEl = must<HTMLInputElement>("setup-user-name");
const setupAgentNameEl = must<HTMLInputElement>("setup-agent-name");
const setupWorkspaceModeEl = must<HTMLSelectElement>("setup-workspace-mode");
const setupDeploymentModeEl = must<HTMLSelectElement>("setup-deployment-mode");
const setupWorkspaceRoleEl = must<HTMLSelectElement>("setup-workspace-role");
const setupSubscriptionTierEl = must<HTMLSelectElement>("setup-subscription-tier");
const setupOrchestratorModeEl = must<HTMLSelectElement>("setup-orchestrator-mode");
const setupProviderEl = must<HTMLInputElement>("setup-provider");
const setupProviderOptionsEl = must<HTMLDataListElement>("setup-provider-options");
const setupProviderGuidanceEl = must<HTMLElement>("setup-provider-guidance");
const setupModelEl = must<HTMLInputElement>("setup-model");
const setupApiUrlEl = must<HTMLInputElement>("setup-api-url");
const setupDefaultTemperatureEl = must<HTMLInputElement>("setup-default-temperature");
const setupMemoryEl = must<HTMLSelectElement>("setup-memory");
const setupRuntimeReasoningEnabledEl = must<HTMLSelectElement>("setup-runtime-reasoning-enabled");
const setupAgentCompactContextEl = must<HTMLInputElement>("setup-agent-compact-context");
const setupAgentParallelToolsEl = must<HTMLInputElement>("setup-agent-parallel-tools");
const setupAgentMaxToolIterationsEl = must<HTMLInputElement>("setup-agent-max-tool-iterations");
const setupAgentMaxHistoryMessagesEl = must<HTMLInputElement>("setup-agent-max-history-messages");
const setupAgentToolDispatcherEl = must<HTMLInputElement>("setup-agent-tool-dispatcher");
const setupSkillsPromptInjectionModeEl = must<HTMLSelectElement>("setup-skills-prompt-injection-mode");
const setupSkillsOpenEnabledEl = must<HTMLInputElement>("setup-skills-open-enabled");
const setupSkillsOpenDirEl = must<HTMLInputElement>("setup-skills-open-dir");
const setupEnableToolConnectorsEl = must<HTMLInputElement>("setup-enable-tool-connectors");
const setupApiKeyEl = must<HTMLInputElement>("setup-api-key");
const setupDelegateAgentsJsonEl = must<HTMLTextAreaElement>("setup-delegate-agents-json");
const setupAgentsSummaryEl = must<HTMLPreElement>("setup-agents-summary");
const setupSummaryCardEl = must<HTMLElement>("setup-complete-summary");
const setupSummaryTextEl = must<HTMLParagraphElement>("setup-complete-summary-text");
const setupEditorEl = must<HTMLDivElement>("setup-editor");
const setupEditorToggleEl = must<HTMLButtonElement>("setup-editor-toggle");
const setupGoMissionEl = must<HTMLButtonElement>("setup-go-mission");
const connectInvitePayloadEl = must<HTMLTextAreaElement>("connect-invite-payload");
const connectOutputEl = must<HTMLPreElement>("connect-output");

const secretKeyEl = must<HTMLInputElement>("secret-key");
const secretValueEl = must<HTMLInputElement>("secret-value");
const secretOutputEl = must<HTMLPreElement>("secret-output");
const secretKeyTemplateEl = must<HTMLSelectElement>("secret-key-template");
const secretApplyTemplateEl = must<HTMLButtonElement>("secret-apply-template");
const secretAdvancedToggleEl = must<HTMLInputElement>("secret-advanced-toggle");
const secretProviderGuidanceEl = must<HTMLParagraphElement>("secret-provider-guidance");

const metricRuntimeEl = must<HTMLSpanElement>("metric-runtime");
const metricIntegrationsEl = must<HTMLSpanElement>("metric-integrations");
const metricSkillsEl = must<HTMLSpanElement>("metric-skills");
const metricMcpEl = must<HTMLSpanElement>("metric-mcp");
const metricApprovalsEl = must<HTMLSpanElement>("metric-approvals");
const metricReceiptsEl = must<HTMLSpanElement>("metric-receipts");
const metricLogsEl = must<HTMLSpanElement>("metric-log-lines");

const actorIdEl = must<HTMLInputElement>("actor-id");
const actorRoleEl = must<HTMLSelectElement>("actor-role");
const approvalUseIdEl = must<HTMLInputElement>("approval-use-id");
const accessOutputEl = must<HTMLPreElement>("access-output");
const controlPlaneOutputEl = must<HTMLPreElement>("control-plane-output");
const deploymentOutputEl = must<HTMLPreElement>("deployment-output");
const missionOutputEl = must<HTMLPreElement>("mission-output");
const evidenceOutputEl = must<HTMLPreElement>("evidence-output");
const rolloutOutputEl = must<HTMLPreElement>("rollout-output");
const rolloutSigningOutputEl = must<HTMLPreElement>("rollout-signing-output");
const policyOutputEl = must<HTMLPreElement>("policy-output");
const rbacOutputEl = must<HTMLPreElement>("rbac-output");
const auditOutputEl = must<HTMLPreElement>("audit-output");
const auditRemoteOutputEl = must<HTMLPreElement>("audit-remote-output");
const outcomesOutputEl = must<HTMLPreElement>("outcomes-output");
const billingOutputEl = must<HTMLPreElement>("billing-output");

const integrationListEl = must<HTMLDivElement>("integration-list");
const skillsListEl = must<HTMLDivElement>("skills-list");
const mcpListEl = must<HTMLDivElement>("mcp-list");
const approvalsListEl = must<HTMLDivElement>("approvals-list");
const receiptsListEl = must<HTMLDivElement>("receipts-list");

const integrationIdEl = must<HTMLSelectElement>("integration-id");
const integrationCatalogHintEl = must<HTMLParagraphElement>("integration-catalog-hint");
const integrationKeyHintEl = must<HTMLParagraphElement>("integration-key-hint");
const integrationAccessEl = must<HTMLInputElement>("integration-access");
const integrationActionsEl = must<HTMLInputElement>("integration-actions");
const integrationDestinationsEl = must<HTMLInputElement>("integration-destinations");

const skillPresetEl = must<HTMLSelectElement>("skill-preset");
const skillIdEl = must<HTMLInputElement>("skill-id");
const skillNameEl = must<HTMLInputElement>("skill-name");
const skillSourceEl = must<HTMLSelectElement>("skill-source");
const skillVersionEl = must<HTMLInputElement>("skill-version");
const skillAccessEl = must<HTMLInputElement>("skill-access");
const skillActionsEl = must<HTMLInputElement>("skill-actions");
const skillDestinationsEl = must<HTMLInputElement>("skill-destinations");
const skillManifestEl = must<HTMLTextAreaElement>("skill-manifest");

const mcpPresetEl = must<HTMLSelectElement>("mcp-preset");
const mcpIdEl = must<HTMLInputElement>("mcp-id");
const mcpNameEl = must<HTMLInputElement>("mcp-name");
const mcpTransportEl = must<HTMLSelectElement>("mcp-transport");
const mcpEndpointEl = must<HTMLInputElement>("mcp-endpoint");
const mcpCommandEl = must<HTMLInputElement>("mcp-command");
const mcpArgsEl = must<HTMLInputElement>("mcp-args");
const mcpEnvSecretIdsEl = must<HTMLInputElement>("mcp-env-secret-ids");
const mcpTimeoutEl = must<HTMLInputElement>("mcp-timeout");
const mcpAccessEl = must<HTMLInputElement>("mcp-access");
const mcpActionsEl = must<HTMLInputElement>("mcp-actions");
const mcpDestinationsEl = must<HTMLInputElement>("mcp-destinations");
const mcpUpdateIdEl = must<HTMLInputElement>("mcp-update-id");
const toolsLifecycleOutputEl = must<HTMLPreElement>("tools-lifecycle-output");

const pairingTransportEl = must<HTMLSelectElement>("pairing-transport");
const pairingEndpointEl = must<HTMLInputElement>("pairing-endpoint");
const pairingExpiresEl = must<HTMLInputElement>("pairing-expires");

const rolloutReleaseIdEl = must<HTMLInputElement>("rollout-release-id");
const rolloutVersionEl = must<HTMLInputElement>("rollout-version");
const rolloutChecksumEl = must<HTMLInputElement>("rollout-checksum");
const rolloutSignatureEl = must<HTMLInputElement>("rollout-signature");
const rolloutSbomChecksumEl = must<HTMLInputElement>("rollout-sbom-checksum");
const rolloutRingEl = must<HTMLSelectElement>("rollout-ring");
const rolloutSignatureRequiredEl = must<HTMLInputElement>("rollout-signature-required");
const rolloutTrustedSignersEl = must<HTMLTextAreaElement>("rollout-trusted-signers");
const policyTemplateIdEl = must<HTMLSelectElement>("policy-template-id");

const rbacUserIdEl = must<HTMLInputElement>("rbac-user-id");
const rbacDisplayNameEl = must<HTMLInputElement>("rbac-display-name");
const rbacRoleEl = must<HTMLSelectElement>("rbac-role");
const rbacActiveEl = must<HTMLInputElement>("rbac-active");

const outcomeTitleEl = must<HTMLInputElement>("outcome-title");
const outcomeStatusEl = must<HTMLSelectElement>("outcome-status");
const outcomeImpactEl = must<HTMLInputElement>("outcome-impact");
const outcomeOwnerEl = must<HTMLInputElement>("outcome-owner");
const outcomeReceiptIdEl = must<HTMLInputElement>("outcome-receipt-id");
const outcomeNotesEl = must<HTMLTextAreaElement>("outcome-notes");
const auditRemoteEnabledEl = must<HTMLInputElement>("audit-remote-enabled");
const auditRemoteEndpointEl = must<HTMLInputElement>("audit-remote-endpoint");
const auditRemoteKindEl = must<HTMLSelectElement>("audit-remote-kind");
const auditRemoteAuthSecretEl = must<HTMLInputElement>("audit-remote-auth-secret");
const auditRemoteVerifyTlsEl = must<HTMLInputElement>("audit-remote-verify-tls");
const auditRemoteBatchSizeEl = must<HTMLInputElement>("audit-remote-batch-size");
const billingBackendUrlEl = must<HTMLInputElement>("billing-backend-url");
const billingAuthSecretEl = must<HTMLInputElement>("billing-auth-secret");
const billingEnforceVerificationEl = must<HTMLInputElement>("billing-enforce-verification");
const billingReceiptPayloadEl = must<HTMLTextAreaElement>("billing-receipt-payload");
const billingPlatformEl = must<HTMLInputElement>("billing-platform");
const complianceTemplateIdEl = must<HTMLSelectElement>("compliance-template-id");
const complianceOutputEl = must<HTMLPreElement>("compliance-output");
const workflowOutputEl = must<HTMLPreElement>("workflow-output");
const workflowTaskIdEl = must<HTMLInputElement>("workflow-task-id");
const workflowTaskTitleEl = must<HTMLInputElement>("workflow-task-title");
const workflowTaskDescriptionEl = must<HTMLTextAreaElement>("workflow-task-description");
const workflowTaskStatusEl = must<HTMLSelectElement>("workflow-task-status");
const workflowTaskPriorityEl = must<HTMLSelectElement>("workflow-task-priority");
const workflowTaskOwnerEl = must<HTMLInputElement>("workflow-task-owner");
const workflowTaskRuntimeTaskIdEl = must<HTMLInputElement>("workflow-task-runtime-task-id");
const workflowTaskAgentIdEl = must<HTMLInputElement>("workflow-task-agent-id");
const workflowTaskSkillIdEl = must<HTMLInputElement>("workflow-task-skill-id");
const workflowTaskToolIdEl = must<HTMLInputElement>("workflow-task-tool-id");
const workflowTaskTagsEl = must<HTMLInputElement>("workflow-task-tags");
const workflowTaskRiskScoreEl = must<HTMLInputElement>("workflow-task-risk-score");
const workflowTaskReceiptIdEl = must<HTMLInputElement>("workflow-task-receipt-id");
const workflowMoveTaskIdEl = must<HTMLInputElement>("workflow-move-task-id");
const workflowMoveStatusEl = must<HTMLSelectElement>("workflow-move-status");
const workflowViewBoardBtnEl = must<HTMLButtonElement>("workflow-view-board");
const workflowViewListBtnEl = must<HTMLButtonElement>("workflow-view-list");
const workflowBoardViewEl = must<HTMLDivElement>("workflow-board-view");
const workflowListViewEl = must<HTMLDivElement>("workflow-list-view");
const workflowListBodyEl = must<HTMLTableSectionElement>("workflow-list-body");
const workflowDragHintEl = must<HTMLDivElement>("workflow-drag-hint");
const workflowColPendingEl = must<HTMLDivElement>("workflow-col-pending");
const workflowColInProgressEl = must<HTMLDivElement>("workflow-col-in-progress");
const workflowColDoneEl = must<HTMLDivElement>("workflow-col-done");
const workflowColFailedEl = must<HTMLDivElement>("workflow-col-failed");
const workflowColBlockedEl = must<HTMLDivElement>("workflow-col-blocked");
const workflowCountPendingEl = must<HTMLElement>("workflow-count-pending");
const workflowCountInProgressEl = must<HTMLElement>("workflow-count-in-progress");
const workflowCountDoneEl = must<HTMLElement>("workflow-count-done");
const workflowCountFailedEl = must<HTMLElement>("workflow-count-failed");
const workflowCountBlockedEl = must<HTMLElement>("workflow-count-blocked");

const approvalIdEl = must<HTMLInputElement>("approval-id");
const approvalRoleEl = must<HTMLSelectElement>("approval-role");
const approvalApprovedEl = must<HTMLSelectElement>("approval-approved");
const approvalReasonEl = must<HTMLInputElement>("approval-reason");
const retentionReceiptsDaysEl = must<HTMLInputElement>("retention-receipts-days");
const retentionApprovalsDaysEl = must<HTMLInputElement>("retention-approvals-days");
const operationsOutputEl = must<HTMLPreElement>("operations-output");
const providersOutputEl = must<HTMLPreElement>("providers-output");
const modelProviderEl = must<HTMLInputElement>("model-provider");
const modelForceEl = must<HTMLInputElement>("model-force");
const serviceActionEl = must<HTMLSelectElement>("service-action");
const runtimeMessageInlineEl = must<HTMLTextAreaElement>("runtime-message-inline");
const channelsOutputEl = must<HTMLPreElement>("channels-output");
const channelTypeEl = must<HTMLSelectElement>("channel-type");
const channelConfigEl = must<HTMLTextAreaElement>("channel-config-json");
const channelRemoveEl = must<HTMLInputElement>("channel-remove-name");
const channelTelegramIdentityEl = must<HTMLInputElement>("channel-telegram-identity");
const cronListOutputEl = must<HTMLPreElement>("cron-list-output");
const cronExpressionEl = must<HTMLInputElement>("cron-expression");
const cronCommandEl = must<HTMLInputElement>("cron-command");
const cronTimezoneEl = must<HTMLInputElement>("cron-timezone");
const cronJobIdEl = must<HTMLInputElement>("cron-job-id");
const completionBinaryPathEl = must<HTMLInputElement>("completion-binary-path");
const themeToggleEl = must<HTMLButtonElement>("theme-toggle");
const helpDialogEl = must<HTMLDialogElement>("help-dialog");
const helpTitleEl = must<HTMLElement>("help-title");
const helpContentEl = must<HTMLParagraphElement>("help-content");
const helpTriggerEls = Array.from(document.querySelectorAll<HTMLButtonElement>(".help-trigger"));
const missionWorkspacesListEl = must<HTMLDivElement>("mission-workspaces-list");
const missionWorkspaceCreateNameEl = must<HTMLInputElement>("mission-workspace-create-name");
const shellEl = document.querySelector<HTMLElement>(".shell");
if (!shellEl) {
  throw new Error("Missing shell root");
}
const setupStepperEl = document.querySelector<HTMLElement>(".setup-stepper");
if (!setupStepperEl) {
  throw new Error("Missing setup stepper");
}
const routeLinks = Array.from(document.querySelectorAll<HTMLButtonElement>("[data-route-target]"));
const routePages = Array.from(document.querySelectorAll<HTMLElement>("[data-route]"));
const stepperProgressEl = must<HTMLParagraphElement>("stepper-progress");
const stepperPrevEl = must<HTMLButtonElement>("stepper-prev");
const stepperNextEl = must<HTMLButtonElement>("stepper-next");
const setupGateNoteEl = must<HTMLParagraphElement>("setup-gate-note");
const stepperChips = Array.from(document.querySelectorAll<HTMLButtonElement>(".step-chip"));
const openSettingsEl = must<HTMLButtonElement>("open-settings");
const openToolsEl = must<HTMLButtonElement>("open-tools");
const openMissionEl = must<HTMLButtonElement>("open-mission");
const missionKpiComplianceEl = must<HTMLElement>("mission-kpi-compliance");
const missionKpiOutcomeEl = must<HTMLElement>("mission-kpi-outcome");
const missionKpiRolloutEl = must<HTMLElement>("mission-kpi-rollout");
const missionKpiComplianceViewEl = must<HTMLElement>("mission-kpi-compliance-view");
const missionKpiOutcomeViewEl = must<HTMLElement>("mission-kpi-outcome-view");
const missionKpiRolloutViewEl = must<HTMLElement>("mission-kpi-rollout-view");
const missionProgressComplianceEl = must<HTMLElement>("mission-progress-compliance");
const missionProgressOutcomesEl = must<HTMLElement>("mission-progress-outcomes");
const missionProgressRolloutEl = must<HTMLElement>("mission-progress-rollout");
const missionHudWorkspaceEl = must<HTMLElement>("mission-hud-workspace");
const missionHudRuntimeEl = must<HTMLElement>("mission-hud-runtime");
const missionHudRoleEl = must<HTMLElement>("mission-hud-role");
const missionHudApprovalsEl = must<HTMLElement>("mission-hud-approvals");
const missionHudOpenTasksEl = must<HTMLElement>("mission-hud-open-tasks");
const missionHudReceiptsEl = must<HTMLElement>("mission-hud-receipts");
const setupLockStatusEl = must<HTMLParagraphElement>("setup-lock-status");
const setupCompleteInitialBtnEl = must<HTMLButtonElement>("setup-complete-initial");

let integrations: IntegrationRecord[] = [];
let skills: SkillRecord[] = [];
let mcpConnectors: McpConnectorRecord[] = [];
let integrationCatalog: IntegrationCatalogEntry[] = [];
let approvals: ApprovalRequest[] = [];
let receipts: ActionReceipt[] = [];
let currentAccessState: AccessState | null = null;
let currentControlPlane: ControlPlaneState | null = null;
let currentSetupState: ProfileSetupState | null = null;
let currentRolloutState: RolloutState | null = null;
let currentRbacRegistry: RbacRegistry | null = null;
let currentAuditVerification: AuditLogVerification | null = null;
let currentOutcomeSummary: OutcomeSummary | null = null;
let currentAuditRemoteState: AuditRemoteSinkState | null = null;
let currentBillingState: BillingState | null = null;
let currentWorkflowBoard: WorkflowBoardView | null = null;
let currentCompliancePosture: CompliancePosture | null = null;
let currentComplianceProfile: ComplianceProfileState | null = null;
let setupEditorExpanded = false;
let channelSummaries: ChannelSummary[] = [];
let cronJobs: CronJobSummary[] = [];
let currentRoute = "profile";
type WorkflowViewMode = "board" | "list";
let workflowViewMode: WorkflowViewMode = "board";
let draggingWorkflowTaskId: string | null = null;
const guidedStepRoutes = ["profile", "keys", "safety", "channels", "runtime", "mission"] as const;
type GuidedRoute = (typeof guidedStepRoutes)[number];
const onboardingRequiredRoutes = ["profile", "keys", "safety", "channels"] as const;
const lockedDuringOnboardingRoutes = ["runtime", "mission", "deployment"] as const;
const onboardingLockedControlIds = [
  "runtime-start",
  "runtime-stop",
  "runtime-refresh",
  "runtime-refresh-inline",
  "runtime-message-form",
  "runtime-message-inline-form",
  "mission-refresh",
  "mission-runtime-status",
  "mission-runtime-doctor",
  "mission-runtime-start",
  "mission-runtime-stop",
  "mission-workspaces-refresh",
  "mission-workspace-create-form",
  "workflow-task-form",
  "workflow-refresh",
  "workflow-move",
  "outcome-form",
  "outcomes-refresh",
  "rollout-stage-form",
  "rollout-promote",
  "rollout-rollback",
  "rollout-refresh",
  "policy-apply",
  "policy-refresh",
  "compliance-apply",
  "compliance-refresh",
  "rbac-user-form",
  "rbac-refresh",
  "audit-refresh",
  "evidence-export",
  "audit-remote-config-form",
  "audit-remote-sync",
  "audit-remote-refresh",
  "billing-config-form",
  "billing-verify-form",
  "billing-refresh",
] as const;
const THEME_STORAGE_KEY = "right-hand.theme";
const COMPLETION_BINARY_PATH_STORAGE_KEY = "right-hand.completion-binary-path";
const WORKFLOW_VIEW_MODE_STORAGE_KEY = "right-hand.workflow-view-mode";
const ONBOARDING_STATE_STORAGE_KEY = "right-hand.onboarding-complete.v1";

let onboardingCompletionByProfile: Record<string, boolean> = {};

const helpContentByTopic: Record<string, { title: string; body: string }> = {
  setup: {
    title: "Setup",
    body:
      "Set workspace owner, role, deployment mode, provider, model, and memory baseline. This is the required first step.",
  },
  keys: {
    title: "Keys",
    body:
      "Store provider/integration secrets by key ID in secure storage. Pick provider presets, save keys once, then reference key IDs only.",
  },
  safety: {
    title: "Safety",
    body:
      "Set action identity, approval decisions, retention rules, and evidence controls for auditable operations.",
  },
  tools: {
    title: "Team Tools",
    body:
      "Install channels, integrations, skills, and MCP connectors as draft first, validate, then publish.",
  },
  runtime: {
    title: "Runtime Ops",
    body:
      "Run health, diagnostics, model refresh, scheduler, migration, and service actions from the app.",
  },
  mission: {
    title: "Mission Control",
    body:
      "Manage runtime/workspaces, rollout, RBAC, audit evidence, outcomes, and workflow board/list in one place.",
  },
  hud: {
    title: "Control HUD",
    body:
      "Always-on snapshot: workspace, runtime, role, approvals, tasks, receipts, compliance, outcomes, and rollout ring.",
  },
};

const skillPresets: Record<
  string,
  {
    skill_id: string;
    display_name: string;
    source: string;
    version: string;
    can_access: string[];
    can_do: string[];
    data_destinations: string[];
    manifest: string;
  }
> = {
  "incident-response": {
    skill_id: "incident-response",
    display_name: "Incident Response",
    source: "catalog",
    version: "1.0.0",
    can_access: ["logs", "alerts", "workflow"],
    can_do: ["triage", "escalate", "evidence_export"],
    data_destinations: ["local", "siem"],
    manifest: "Respond to production incidents, create triage tasks, and preserve audit evidence.",
  },
  "compliance-review": {
    skill_id: "compliance-review",
    display_name: "Compliance Review",
    source: "catalog",
    version: "1.0.0",
    can_access: ["policy", "audit", "billing"],
    can_do: ["review", "recommend", "evidence_export"],
    data_destinations: ["local"],
    manifest: "Check compliance posture and generate remediation recommendations.",
  },
  "change-management": {
    skill_id: "change-management",
    display_name: "Change Management",
    source: "catalog",
    version: "1.0.0",
    can_access: ["workflow", "rollout", "rbac"],
    can_do: ["plan", "approve", "rollback"],
    data_destinations: ["local", "siem"],
    manifest: "Manage controlled rollout and change approvals for production tasks.",
  },
  "finops-analysis": {
    skill_id: "finops-analysis",
    display_name: "FinOps Analysis",
    source: "catalog",
    version: "1.0.0",
    can_access: ["cost_summary", "usage", "models"],
    can_do: ["analyze", "optimize", "report"],
    data_destinations: ["local"],
    manifest: "Analyze model spend, detect anomalies, and recommend cost optimizations.",
  },
  "healthcare-safety": {
    skill_id: "healthcare-safety",
    display_name: "Healthcare Safety",
    source: "catalog",
    version: "1.0.0",
    can_access: ["workflow", "audit", "policy"],
    can_do: ["review", "flag_risk", "escalate"],
    data_destinations: ["local", "siem"],
    manifest: "Assist healthcare safety workflows with risk-first escalation rules.",
  },
};

const mcpPresets: Record<
  string,
  {
    connector_id: string;
    display_name: string;
    transport: string;
    endpoint?: string;
    command?: string;
    args: string[];
    env_secret_ids: string[];
    timeout_secs: number;
    can_access: string[];
    can_do: string[];
    data_destinations: string[];
  }
> = {
  filesystem: {
    connector_id: "filesystem",
    display_name: "Filesystem Connector",
    transport: "stdio",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-filesystem"],
    env_secret_ids: [],
    timeout_secs: 30,
    can_access: ["workspace_files"],
    can_do: ["read", "write"],
    data_destinations: ["local"],
  },
  github: {
    connector_id: "github",
    display_name: "GitHub Connector",
    transport: "sse",
    endpoint: "https://mcp.github.com/sse",
    env_secret_ids: ["provider.github.token"],
    args: [],
    timeout_secs: 30,
    can_access: ["repos", "issues", "prs"],
    can_do: ["list", "comment", "update_status"],
    data_destinations: ["github"],
  },
  postgres: {
    connector_id: "postgres",
    display_name: "Postgres Connector",
    transport: "stdio",
    command: "npx",
    args: ["-y", "@modelcontextprotocol/server-postgres"],
    env_secret_ids: ["db.postgres.url"],
    timeout_secs: 30,
    can_access: ["database"],
    can_do: ["query"],
    data_destinations: ["local"],
  },
  slack: {
    connector_id: "slack",
    display_name: "Slack Connector",
    transport: "sse",
    endpoint: "https://mcp.slack.com/sse",
    env_secret_ids: ["provider.slack.token"],
    args: [],
    timeout_secs: 30,
    can_access: ["channels", "messages"],
    can_do: ["read", "post"],
    data_destinations: ["slack"],
  },
  http: {
    connector_id: "http-api",
    display_name: "HTTP API Connector",
    transport: "websocket",
    endpoint: "wss://mcp.example.com/ws",
    env_secret_ids: ["provider.api.token"],
    args: [],
    timeout_secs: 30,
    can_access: ["api"],
    can_do: ["request"],
    data_destinations: ["trusted_api"],
  },
};

const PROVIDERS_REFERENCE_URL = "https://zeroclawlabs.ai/docs/reference/providers";
const OPENAI_COMPATIBLE_PROVIDER_IDS = new Set<string>([
  "venice",
  "vercel",
  "cloudflare",
  "moonshot",
  "kimi-code",
  "synthetic",
  "opencode",
  "zai",
  "glm",
  "minimax",
  "qianfan",
  "doubao",
  "qwen",
  "groq",
  "mistral",
  "xai",
  "deepseek",
  "together",
  "fireworks",
  "perplexity",
  "cohere",
  "nvidia",
  "ovhcloud",
]);
const PROVIDERS_WITH_NON_API_KEY_AUTH = new Set<string>(["bedrock", "openai-codex"]);
const PROVIDER_KEY_DOCS: Record<string, { key_url: string; privacy_url?: string }> = {
  openrouter: {
    key_url: "https://openrouter.ai/keys",
    privacy_url: "https://openrouter.ai/privacy",
  },
  openai: {
    key_url: "https://platform.openai.com/api-keys",
    privacy_url: "https://platform.openai.com/docs/guides/your-data",
  },
  anthropic: {
    key_url: "https://console.anthropic.com/settings/keys",
    privacy_url: "https://www.anthropic.com/legal/privacy",
  },
  gemini: {
    key_url: "https://aistudio.google.com/app/apikey",
    privacy_url: "https://ai.google.dev/gemini-api/docs/data-usage",
  },
  groq: {
    key_url: "https://console.groq.com/keys",
    privacy_url: "https://console.groq.com/docs/data-privacy",
  },
  deepseek: {
    key_url: "https://platform.deepseek.com/api_keys",
    privacy_url: "https://platform.deepseek.com/privacy-policy",
  },
  together: {
    key_url: "https://api.together.xyz/settings/api-keys",
    privacy_url: "https://www.together.ai/privacy-policy",
  },
  fireworks: {
    key_url: "https://fireworks.ai/account/api-keys",
    privacy_url: "https://fireworks.ai/privacy-policy",
  },
  mistral: {
    key_url: "https://console.mistral.ai/api-keys/",
    privacy_url: "https://mistral.ai/terms#privacy-policy",
  },
  cohere: {
    key_url: "https://dashboard.cohere.com/api-keys",
    privacy_url: "https://cohere.com/privacy",
  },
};
const INTEGRATION_KEY_TEMPLATES = [
  { key_id: "provider.github.token", label: "GitHub Token (integrations/tools)" },
  { key_id: "provider.slack.token", label: "Slack Token (integrations/tools)" },
];
const DEFAULT_PROVIDER_CATALOG: ProviderDescriptor[] = [
  { name: "openrouter", display_name: "OpenRouter", aliases: [], local: false, active: true },
  { name: "anthropic", display_name: "Anthropic", aliases: [], local: false, active: false },
  { name: "openai", display_name: "OpenAI", aliases: [], local: false, active: false },
  {
    name: "openai-codex",
    display_name: "OpenAI Codex (OAuth)",
    aliases: ["openai_codex", "codex"],
    local: false,
    active: false,
  },
  { name: "ollama", display_name: "Ollama", aliases: [], local: true, active: false },
  {
    name: "gemini",
    display_name: "Google Gemini",
    aliases: ["google", "google-gemini"],
    local: false,
    active: false,
  },
  { name: "venice", display_name: "Venice", aliases: [], local: false, active: false },
  { name: "vercel", display_name: "Vercel AI Gateway", aliases: ["vercel-ai"], local: false, active: false },
  {
    name: "cloudflare",
    display_name: "Cloudflare AI",
    aliases: ["cloudflare-ai"],
    local: false,
    active: false,
  },
  { name: "moonshot", display_name: "Moonshot", aliases: ["kimi"], local: false, active: false },
  {
    name: "kimi-code",
    display_name: "Kimi Code",
    aliases: ["kimi_coding", "kimi_for_coding"],
    local: false,
    active: false,
  },
  { name: "synthetic", display_name: "Synthetic", aliases: [], local: false, active: false },
  { name: "opencode", display_name: "OpenCode Zen", aliases: ["opencode-zen"], local: false, active: false },
  { name: "zai", display_name: "Z.AI", aliases: ["z.ai"], local: false, active: false },
  { name: "glm", display_name: "GLM (Zhipu)", aliases: ["zhipu"], local: false, active: false },
  {
    name: "minimax",
    display_name: "MiniMax",
    aliases: [
      "minimax-intl",
      "minimax-io",
      "minimax-global",
      "minimax-cn",
      "minimaxi",
      "minimax-oauth",
      "minimax-oauth-cn",
      "minimax-portal",
      "minimax-portal-cn",
    ],
    local: false,
    active: false,
  },
  { name: "bedrock", display_name: "Amazon Bedrock", aliases: ["aws-bedrock"], local: false, active: false },
  { name: "qianfan", display_name: "Qianfan (Baidu)", aliases: ["baidu"], local: false, active: false },
  {
    name: "doubao",
    display_name: "Doubao (Volcengine)",
    aliases: ["volcengine", "ark", "doubao-cn"],
    local: false,
    active: false,
  },
  {
    name: "qwen",
    display_name: "Qwen (DashScope / Qwen Code OAuth)",
    aliases: [
      "dashscope",
      "qwen-intl",
      "dashscope-intl",
      "qwen-us",
      "dashscope-us",
      "qwen-code",
      "qwen-oauth",
      "qwen_oauth",
    ],
    local: false,
    active: false,
  },
  { name: "groq", display_name: "Groq", aliases: [], local: false, active: false },
  { name: "mistral", display_name: "Mistral", aliases: [], local: false, active: false },
  { name: "xai", display_name: "xAI (Grok)", aliases: ["grok"], local: false, active: false },
  { name: "deepseek", display_name: "DeepSeek", aliases: [], local: false, active: false },
  {
    name: "together",
    display_name: "Together AI",
    aliases: ["together-ai"],
    local: false,
    active: false,
  },
  {
    name: "fireworks",
    display_name: "Fireworks AI",
    aliases: ["fireworks-ai"],
    local: false,
    active: false,
  },
  { name: "perplexity", display_name: "Perplexity", aliases: [], local: false, active: false },
  { name: "cohere", display_name: "Cohere", aliases: [], local: false, active: false },
  {
    name: "copilot",
    display_name: "GitHub Copilot",
    aliases: ["github-copilot"],
    local: false,
    active: false,
  },
  { name: "lmstudio", display_name: "LM Studio", aliases: ["lm-studio"], local: true, active: false },
  {
    name: "llamacpp",
    display_name: "llama.cpp server",
    aliases: ["llama.cpp"],
    local: true,
    active: false,
  },
  {
    name: "nvidia",
    display_name: "NVIDIA NIM",
    aliases: ["nvidia-nim", "build.nvidia.com"],
    local: false,
    active: false,
  },
  {
    name: "ovhcloud",
    display_name: "OVHcloud AI Endpoints",
    aliases: ["ovh"],
    local: false,
    active: false,
  },
];
let providerCatalogCache: ProviderDescriptor[] = [...DEFAULT_PROVIDER_CATALOG];
let secretKeyTemplateIds = new Set<string>();

const isTauriRuntime =
  typeof window !== "undefined" &&
  Boolean((window as Record<string, unknown>).__TAURI_INTERNALS__);
const browserMock = createBrowserMock();

function must<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) {
    throw new Error(`Missing required element #${id}`);
  }
  return node as T;
}

function parseCsv(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
}

function listToCsv(values: string[]): string {
  return values.join(", ");
}

function toTitleCase(raw: string): string {
  return raw
    .split(/[_\s-]+/)
    .filter((part) => part.length > 0)
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join(" ");
}

function keyForIntegration(integrationName: string): string {
  const normalized = integrationName.toLowerCase().replace(/[^a-z0-9._-]+/g, "_");
  return `provider.${normalized}.api_key`;
}

function canonicalProviderId(providerName: string): string {
  const normalized = providerName.toLowerCase().replace(/[^a-z0-9._-]+/g, "_");
  const matched = providerCatalogCache.find(
    (provider) =>
      provider.name === normalized ||
      provider.aliases.some((alias) => alias.toLowerCase() === normalized),
  );
  return matched?.name || normalized;
}

function keyForProvider(providerName: string): string {
  const normalized = canonicalProviderId(providerName);
  return `provider.${normalized}.api_key`;
}

function escapeHtml(raw: string): string {
  return raw
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function providerFromKeyId(keyId: string): string | null {
  const match = keyId.trim().match(/^provider\.([a-z0-9._-]+)\.api_key$/i);
  if (!match || !match[1]) {
    return null;
  }
  return canonicalProviderId(match[1]);
}

function providerOrderWeight(name: string): number {
  switch (name) {
    case "openrouter":
      return 0;
    case "openai":
      return 1;
    case "anthropic":
      return 2;
    case "gemini":
      return 3;
    case "ollama":
      return 4;
    default:
      return OPENAI_COMPATIBLE_PROVIDER_IDS.has(name) ? 10 : 20;
  }
}

function sortedProviders(providers: ProviderDescriptor[]): ProviderDescriptor[] {
  return [...providers].sort((a, b) => {
    const weightDelta = providerOrderWeight(a.name) - providerOrderWeight(b.name);
    if (weightDelta !== 0) {
      return weightDelta;
    }
    return a.display_name.localeCompare(b.display_name);
  });
}

function keyDocsForProvider(providerName: string): { keyUrl: string; privacyUrl: string } {
  const doc = PROVIDER_KEY_DOCS[providerName];
  return {
    keyUrl: doc?.key_url || PROVIDERS_REFERENCE_URL,
    privacyUrl: doc?.privacy_url || PROVIDERS_REFERENCE_URL,
  };
}

function applyProviderCatalogPresets(providers: ProviderDescriptor[]) {
  providerCatalogCache = providers.length > 0 ? [...providers] : [...DEFAULT_PROVIDER_CATALOG];

  setupProviderOptionsEl.replaceChildren();
  for (const provider of sortedProviders(providerCatalogCache)) {
    const option = document.createElement("option");
    option.value = provider.name;
    option.label = `${provider.display_name}${provider.local ? " (local)" : ""}`;
    setupProviderOptionsEl.appendChild(option);
  }

  const preservedTemplate = secretKeyTemplateEl.value.trim();
  secretKeyTemplateEl.replaceChildren();
  const placeholder = document.createElement("option");
  placeholder.value = "";
  placeholder.textContent = "Select provider preset";
  secretKeyTemplateEl.appendChild(placeholder);

  secretKeyTemplateIds = new Set<string>();
  for (const provider of sortedProviders(providerCatalogCache)) {
    const providerName = provider.name.toLowerCase();
    if (PROVIDERS_WITH_NON_API_KEY_AUTH.has(providerName)) {
      continue;
    }
    const keyId = keyForProvider(providerName);
    const option = document.createElement("option");
    const compatibility = OPENAI_COMPATIBLE_PROVIDER_IDS.has(providerName)
      ? "OpenAI-compatible"
      : provider.local
        ? "local / optional key"
        : "native";
    option.value = keyId;
    option.textContent = `${provider.display_name} (${provider.name}) - ${compatibility}`;
    secretKeyTemplateEl.appendChild(option);
    secretKeyTemplateIds.add(keyId);
  }

  const divider = document.createElement("option");
  divider.value = "";
  divider.disabled = true;
  divider.textContent = "──────── Integrations ────────";
  secretKeyTemplateEl.appendChild(divider);

  for (const template of INTEGRATION_KEY_TEMPLATES) {
    const option = document.createElement("option");
    option.value = template.key_id;
    option.textContent = template.label;
    secretKeyTemplateEl.appendChild(option);
    secretKeyTemplateIds.add(template.key_id);
  }

  if (preservedTemplate && secretKeyTemplateIds.has(preservedTemplate)) {
    secretKeyTemplateEl.value = preservedTemplate;
  } else {
    secretKeyTemplateEl.value = "";
  }
}

function renderProviderGuidance(providerNameRaw: string | null | undefined) {
  const raw = String(providerNameRaw || "").trim();
  const providerName = raw ? canonicalProviderId(raw) : "";
  if (!providerName) {
    setupProviderGuidanceEl.innerHTML =
      'Pick a provider preset. Key ID and key setup guidance will auto-populate in Keys.';
    return;
  }
  const keyId = keyForProvider(providerName);
  const provider = providerCatalogCache.find((entry) => entry.name === providerName);
  const providerLabel = provider?.display_name || toTitleCase(providerName);
  const { keyUrl, privacyUrl } = keyDocsForProvider(providerName);
  const compatibility = OPENAI_COMPATIBLE_PROVIDER_IDS.has(providerName)
    ? "OpenAI-compatible"
    : provider?.local
      ? "local provider"
      : "native provider";
  setupProviderGuidanceEl.innerHTML =
    `${escapeHtml(providerLabel)} (${escapeHtml(compatibility)}). ` +
    `Key ID: <code>${escapeHtml(keyId)}</code>. ` +
    `<a href="${escapeHtml(keyUrl)}" target="_blank" rel="noreferrer noopener">Get key</a> · ` +
    `<a href="${escapeHtml(privacyUrl)}" target="_blank" rel="noreferrer noopener">Data policy</a>`;
}

function renderSecretKeyGuidance(keyIdRaw: string | null | undefined) {
  const keyId = String(keyIdRaw || "").trim();
  if (!keyId) {
    secretProviderGuidanceEl.innerHTML =
      'Select a provider preset to auto-fill key ID and open provider key setup docs.';
    return;
  }
  const providerName = providerFromKeyId(keyId);
  if (!providerName) {
    if (keyId === "provider.github.token") {
      secretProviderGuidanceEl.innerHTML =
        'GitHub token for integrations/tools. <a href="https://github.com/settings/tokens" target="_blank" rel="noreferrer noopener">Create token</a>.';
      return;
    }
    if (keyId === "provider.slack.token") {
      secretProviderGuidanceEl.innerHTML =
        'Slack token for integrations/tools. <a href="https://api.slack.com/apps" target="_blank" rel="noreferrer noopener">Create Slack app token</a>.';
      return;
    }
    secretProviderGuidanceEl.innerHTML =
      `Custom key ID <code>${escapeHtml(keyId)}</code>. Use only when you know the integration/provider expects this exact key ID.`;
    return;
  }

  const provider = providerCatalogCache.find((entry) => entry.name === providerName);
  const providerLabel = provider?.display_name || toTitleCase(providerName);
  const { keyUrl, privacyUrl } = keyDocsForProvider(providerName);
  const compatibility = OPENAI_COMPATIBLE_PROVIDER_IDS.has(providerName)
    ? "OpenAI-compatible"
    : provider?.local
      ? "local"
      : "native";
  secretProviderGuidanceEl.innerHTML =
    `<strong>${escapeHtml(providerLabel)}</strong> (${escapeHtml(compatibility)}). ` +
    `<a href="${escapeHtml(keyUrl)}" target="_blank" rel="noreferrer noopener">Get key</a> · ` +
    `<a href="${escapeHtml(privacyUrl)}" target="_blank" rel="noreferrer noopener">Review data policy</a>.`;
}

async function refreshProviderCatalogPresets(profileId?: string) {
  let providers = [...DEFAULT_PROVIDER_CATALOG];
  if (profileId) {
    try {
      const loaded = await invokeCommand<ProviderDescriptor[]>("operations_providers", { profileId });
      if (loaded.length > 0) {
        providers = loaded;
      }
    } catch {
      // keep defaults when runtime/provider listing is unavailable
    }
  }
  applyProviderCatalogPresets(providers);
  renderProviderGuidance(setupProviderEl.value);
  renderSecretKeyGuidance(secretKeyEl.value);
}

function syncSecretKeyEditingMode() {
  const advanced = secretAdvancedToggleEl.checked;
  secretKeyEl.readOnly = !advanced;
  secretKeyEl.classList.toggle("input-readonly", !advanced);

  if (!advanced) {
    const preset = secretKeyTemplateEl.value.trim();
    if (preset) {
      secretKeyEl.value = preset;
    } else if (!secretKeyEl.value.trim()) {
      const provider = setupProviderEl.value.trim() || currentSetupState.provider || "openrouter";
      const derived = keyForProvider(provider);
      secretKeyEl.value = derived;
      if (secretKeyTemplateIds.has(derived)) {
        secretKeyTemplateEl.value = derived;
      }
    }
  }
  renderSecretKeyGuidance(secretKeyEl.value);
}

function resolveSecretKeyIdForActions(): string {
  if (secretAdvancedToggleEl.checked) {
    const key = secretKeyEl.value.trim();
    if (!key) {
      throw new Error("Enter custom key ID, or disable Advanced and choose a preset.");
    }
    return key;
  }

  const preset = secretKeyTemplateEl.value.trim();
  if (preset) {
    secretKeyEl.value = preset;
    return preset;
  }

  const provider = setupProviderEl.value.trim() || currentSetupState.provider || "";
  if (provider) {
    const derived = keyForProvider(provider);
    secretKeyEl.value = derived;
    if (secretKeyTemplateIds.has(derived)) {
      secretKeyTemplateEl.value = derived;
    }
    return derived;
  }

  throw new Error("Select a provider key preset before saving/checking keys.");
}

function loadOnboardingCompletionState(): Record<string, boolean> {
  try {
    const raw = window.localStorage.getItem(ONBOARDING_STATE_STORAGE_KEY);
    if (!raw) {
      return {};
    }
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return {};
    }
    const map = parsed as Record<string, unknown>;
    return Object.fromEntries(
      Object.entries(map).map(([key, value]) => [key, Boolean(value)]),
    );
  } catch {
    return {};
  }
}

function saveOnboardingCompletionState() {
  window.localStorage.setItem(
    ONBOARDING_STATE_STORAGE_KEY,
    JSON.stringify(onboardingCompletionByProfile),
  );
}

function markOnboardingCompletion(profileId: string, complete: boolean) {
  onboardingCompletionByProfile[profileId] = complete;
  saveOnboardingCompletionState();
}

function isOnboardingCompleteForProfile(profileId: string | null): boolean {
  if (!profileId) {
    return false;
  }
  return Boolean(onboardingCompletionByProfile[profileId]);
}

function isOnboardingLocked(): boolean {
  const profileId = activeProfileId();
  return !isSetupReady(currentSetupState) || !isOnboardingCompleteForProfile(profileId);
}

function isOnboardingRoute(route: string): route is (typeof onboardingRequiredRoutes)[number] {
  return onboardingRequiredRoutes.includes(route as (typeof onboardingRequiredRoutes)[number]);
}

function isRouteLockedByOnboarding(route: string): boolean {
  if (!isOnboardingLocked()) {
    return false;
  }
  return lockedDuringOnboardingRoutes.includes(route as (typeof lockedDuringOnboardingRoutes)[number]);
}

function normalizeRouteForOnboarding(route: string): string {
  if (isRouteLockedByOnboarding(route)) {
    return "safety";
  }
  return route;
}

function suggestIntegrationContract(entry: IntegrationCatalogEntry): PermissionContract {
  const category = entry.category.toLowerCase();
  const isCommunication = ["messaging", "communication", "chat"].includes(category);
  const isData = ["database", "storage", "docs"].includes(category);
  const isObservability = ["monitoring", "security", "devops"].includes(category);

  const canAccess = isCommunication
    ? ["messages"]
    : isData
      ? ["records"]
      : isObservability
        ? ["events", "logs"]
        : ["metadata"];
  const canDo = isCommunication
    ? ["send", "read"]
    : isData
      ? ["read"]
      : isObservability
        ? ["query", "report"]
        : ["execute"];
  const dataDestinations = [entry.name];

  return {
    integration_id: entry.name,
    can_access: canAccess,
    can_do: canDo,
    data_destinations: dataDestinations,
  };
}

function openHelp(topic: string) {
  const content = helpContentByTopic[topic] || {
    title: "Help",
    body: "No help content available for this section yet.",
  };
  helpTitleEl.textContent = content.title;
  helpContentEl.textContent = content.body;
  if (typeof helpDialogEl.showModal !== "function") {
    window.alert(`${content.title}\n\n${content.body}`);
    return;
  }
  if (!helpDialogEl.open) {
    helpDialogEl.showModal();
  }
}

function parseDelegateAgentsJson(raw: string): Record<string, DelegateAgentSetup> {
  const trimmed = raw.trim();
  if (!trimmed) {
    return {};
  }
  const parsed = JSON.parse(trimmed) as unknown;
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    throw new Error("Delegate agents JSON must be an object keyed by agent name.");
  }
  return parsed as Record<string, DelegateAgentSetup>;
}

function renderDelegateAgentsSummary(agents: Record<string, DelegateAgentSetup>) {
  const names = Object.keys(agents);
  if (names.length === 0) {
    setupAgentsSummaryEl.textContent =
      "No delegate agents configured. Add specialized agents (marketing/coding/finance/coaching) when needed.";
    return;
  }

  const summary = names
    .sort()
    .map((name) => {
      const agent = agents[name];
      const depth = typeof agent.max_depth === "number" ? agent.max_depth : 3;
      const iterations = typeof agent.max_iterations === "number" ? agent.max_iterations : 10;
      const tools = Array.isArray(agent.allowed_tools) && agent.allowed_tools.length > 0
        ? agent.allowed_tools.join("|")
        : "*";
      return `${name}: ${agent.provider}/${agent.model} (max_depth=${depth}, agentic=${Boolean(agent.agentic)}, max_iterations=${iterations}, allowed_tools=${tools})`;
    });
  setupAgentsSummaryEl.textContent = summary.join("\n");
}

function isSetupReady(setup: ProfileSetupState | null | undefined): boolean {
  if (!setup) return false;
  return Boolean(setup.provider && setup.model && (setup.has_provider_key || setup.deployment_mode === "client"));
}

function refreshSetupGateUi() {
  const locked = isOnboardingLocked();
  const profileId = activeProfileId();
  const ready = isSetupReady(currentSetupState);
  const complete = isOnboardingCompleteForProfile(profileId);

  if (!ready) {
    setupLockStatusEl.textContent =
      "Setup is incomplete. Save setup and key first (host), or save client baseline.";
  } else if (!complete) {
    setupLockStatusEl.textContent =
      "Setup is ready. Confirm unlock to enable Runtime Ops and Mission Control.";
  } else {
    setupLockStatusEl.textContent =
      "Initial setup is complete. Runtime Ops and Mission Control are unlocked for this workspace.";
  }

  setupCompleteInitialBtnEl.disabled = !ready || complete;
  setupCompleteInitialBtnEl.textContent = complete ? "Initial Setup Complete" : "Complete Initial Setup";
  setupGoMissionEl.disabled = locked;
  setupGoMissionEl.title = locked
    ? "Complete Initial Setup in Safety to unlock Mission Control."
    : "Open Mission Control";
  openMissionEl.disabled = locked;
  openMissionEl.classList.toggle("is-locked-control", locked);
  openMissionEl.dataset.tip = locked ? "Locked until setup is complete" : "Mission";
  setupStepperEl.classList.toggle("is-complete", !locked);

  for (const link of routeLinks) {
    const target = link.dataset.routeTarget;
    if (!target) {
      continue;
    }
    const blocked = isRouteLockedByOnboarding(target);
    link.disabled = blocked;
    link.classList.toggle("is-locked", blocked);
    if (blocked) {
      link.title = "Complete Initial Setup in Safety to unlock this step.";
      link.setAttribute("aria-disabled", "true");
    } else {
      link.removeAttribute("title");
      link.removeAttribute("aria-disabled");
    }
  }

  const lockMessage = "Complete Initial Setup in Safety to unlock advanced operations.";
  for (const id of onboardingLockedControlIds) {
    const node = document.getElementById(id);
    if (!node) {
      continue;
    }

    if (node instanceof HTMLButtonElement) {
      node.disabled = locked;
      node.classList.toggle("is-locked-control", locked);
      if (locked) {
        node.title = lockMessage;
      } else {
        node.removeAttribute("title");
      }
      continue;
    }

    if (node instanceof HTMLFormElement) {
      node.classList.toggle("is-locked-control", locked);
      for (const element of Array.from(node.elements)) {
        if (
          element instanceof HTMLButtonElement ||
          element instanceof HTMLInputElement ||
          element instanceof HTMLSelectElement ||
          element instanceof HTMLTextAreaElement
        ) {
          element.disabled = locked;
          if (locked) {
            element.title = lockMessage;
          } else {
            element.removeAttribute("title");
          }
        }
      }
    }
  }

  setupGateNoteEl.classList.toggle("is-hidden", !locked);
  if (locked) {
    setupGateNoteEl.textContent =
      "Finish Setup + Keys and confirm unlock in Safety to use Runtime Ops and Mission Control.";
  } else {
    setupGateNoteEl.textContent = "";
  }

  if (!locked) {
    return;
  }
  if (!isOnboardingRoute(currentRoute)) {
    setActiveRoute("safety");
  }
}

function refreshSetupSummaryCard() {
  const ready = isSetupReady(currentSetupState);
  setupSummaryCardEl.classList.toggle("is-hidden", !ready);
  shellEl.classList.toggle("is-stepper-compact", ready && currentRoute !== "profile");
  if (!ready) {
    setupEditorEl.classList.remove("is-hidden");
    setupEditorExpanded = true;
    setupEditorToggleEl.textContent = "Hide setup";
    return;
  }

  const setup = currentSetupState!;
  setupSummaryTextEl.textContent =
    `${setup.workspace_role} · ${setup.subscription_tier} · ${setup.provider}/${setup.model} · deployment=${setup.deployment_mode}`;
  if (currentRoute !== "profile") {
    setupEditorEl.classList.add("is-hidden");
    setupEditorToggleEl.textContent = "Edit setup";
    return;
  }

  if (setupEditorExpanded) {
    setupEditorEl.classList.remove("is-hidden");
    setupEditorToggleEl.textContent = "Hide setup";
  } else {
    setupEditorEl.classList.add("is-hidden");
    setupEditorToggleEl.textContent = "Edit setup";
  }
}

function applyTheme(theme: "light" | "dark") {
  document.body.dataset.theme = theme;
  const nextLabel = theme === "dark" ? "Switch to light mode" : "Switch to dark mode";
  const glyph = theme === "dark" ? "☀" : "◐";
  themeToggleEl.innerHTML = `<span aria-hidden="true">${glyph}</span><span class="sr-only">${nextLabel}</span>`;
  themeToggleEl.setAttribute("title", nextLabel);
  themeToggleEl.setAttribute("aria-label", nextLabel);
  themeToggleEl.dataset.tip = nextLabel;
  window.localStorage.setItem(THEME_STORAGE_KEY, theme);
}

function nowIso(): string {
  return new Date().toISOString();
}

function appendActivity(message: string, payload?: unknown) {
  const prefix = `[${new Date().toLocaleTimeString()}] ${message}`;
  const body = payload ? `${prefix}\n${JSON.stringify(payload, null, 2)}\n` : `${prefix}\n`;
  activityOutputEl.textContent = `${body}${activityOutputEl.textContent || ""}`.slice(0, 20000);
}

function parseApprovalFromError(raw: string): string | null {
  const match = raw.match(/approval_id:\s*([a-z0-9-]+)/i);
  return match ? match[1] : null;
}

async function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriRuntime) {
    return browserMock.invoke(command, args) as Promise<T>;
  }
  const core = await import("@tauri-apps/api/core");
  return core.invoke<T>(command, args);
}

function actorContext() {
  return {
    actorId: actorIdEl.value.trim() || "local-user",
    actorRole: actorRoleEl.value.trim() || "admin",
    approvalId: approvalUseIdEl.value.trim() || undefined,
  };
}

function activeProfileId(): string | null {
  const value = profileSelectEl.value.trim();
  return value.length > 0 ? value : null;
}

function ensureProfileId(): string {
  const profileId = activeProfileId();
  if (!profileId) {
    throw new Error("No active profile selected");
  }
  return profileId;
}

async function invokeGuarded<T>(
  command: string,
  args: Record<string, unknown>,
  label: string,
): Promise<T> {
  const { actorId, actorRole, approvalId } = actorContext();
  try {
    const mergedArgs = {
      ...args,
      actorId,
      actorRole,
      approvalId,
    };
    return await invokeCommand<T>(command, mergedArgs);
  } catch (error) {
    const message = String(error);
    const approvalIdHint = parseApprovalFromError(message);
    if (approvalIdHint) {
      appendActivity(`${label} queued for approval`, { approval_id: approvalIdHint });
    }
    throw error;
  }
}

function applyViewScope() {
  const view = currentAccessState?.active_view || "org";
  const scoped = document.querySelectorAll<HTMLElement>("[data-view-scope]");
  for (const panel of scoped) {
    const scope = panel.dataset.viewScope;
    const matches =
      !scope ||
      scope === "shared" ||
      scope
        .split(",")
        .map((value) => value.trim())
        .includes(view);
    if (matches) {
      panel.classList.remove("is-hidden");
    } else {
      panel.classList.add("is-hidden");
    }
  }
}

function setActiveRoute(route: string) {
  const effectiveRoute = normalizeRouteForOnboarding(route);
  currentRoute = effectiveRoute;
  shellEl.dataset.route = effectiveRoute;
  for (const page of routePages) {
    page.classList.toggle("is-hidden", page.dataset.route !== effectiveRoute);
  }
  for (const link of routeLinks) {
    link.classList.toggle("is-active", link.dataset.routeTarget === effectiveRoute);
  }
  for (const chip of stepperChips) {
    chip.classList.toggle("is-active", chip.dataset.routeTarget === effectiveRoute);
  }
  const stepIndex = guidedStepRoutes.indexOf(effectiveRoute as GuidedRoute);
  const maxUnlockedStepIndex = isOnboardingLocked() ? guidedStepRoutes.indexOf("channels") : guidedStepRoutes.length - 1;
  if (stepIndex >= 0) {
    stepperProgressEl.textContent = `Step ${stepIndex + 1} of ${guidedStepRoutes.length}`;
    stepperPrevEl.disabled = stepIndex === 0;
    stepperNextEl.disabled = stepIndex >= maxUnlockedStepIndex;
  } else {
    stepperProgressEl.textContent = "Advanced panel";
    stepperPrevEl.disabled = true;
    stepperNextEl.disabled = true;
  }

  shellEl.classList.add("is-focus-mode");
  refreshSetupSummaryCard();
  refreshSetupGateUi();
}

function refreshDashboardMetrics() {
  metricRuntimeEl.textContent = runtimeStateEl.textContent?.replace("State: ", "") || "unknown";
  metricIntegrationsEl.textContent = String(integrations.length);
  metricSkillsEl.textContent = String(skills.length);
  metricMcpEl.textContent = String(mcpConnectors.length);
  metricApprovalsEl.textContent = String(approvals.filter((item) => item.status === "pending").length);
  metricReceiptsEl.textContent = String(receipts.length);
  try {
    const parsed = JSON.parse(logsOutputEl.textContent || "[]") as LogLine[];
    metricLogsEl.textContent = String(parsed.length);
  } catch {
    metricLogsEl.textContent = "0";
  }
}

function setProgressBar(el: HTMLElement, percent: number) {
  const clamped = Math.max(0, Math.min(100, percent));
  el.style.width = `${clamped}%`;
}

function updateMissionKpis(summary: MissionControlSummary) {
  const checks = summary.compliance?.checks || [];
  const satisfied = checks.filter((item) => item.satisfied).length;
  const compliancePercent = checks.length > 0 ? Math.round((satisfied / checks.length) * 100) : 0;
  const outcomePercent = Math.round((summary.outcomes?.solved_rate || 0) * 100);
  const ring = summary.rollout?.current_release?.ring || "pilot";
  const ringPercent = ring === "pilot" ? 34 : ring === "group" ? 67 : 100;

  missionKpiComplianceEl.textContent = `${compliancePercent}%`;
  missionKpiOutcomeEl.textContent = `${outcomePercent}%`;
  missionKpiRolloutEl.textContent = ring;
  missionKpiComplianceViewEl.textContent = `${compliancePercent}%`;
  missionKpiOutcomeViewEl.textContent = `${outcomePercent}%`;
  missionKpiRolloutViewEl.textContent = ring;
  setProgressBar(missionProgressComplianceEl, compliancePercent);
  setProgressBar(missionProgressOutcomesEl, outcomePercent);
  setProgressBar(missionProgressRolloutEl, ringPercent);
}

function updateMissionHud(summary?: MissionControlSummary) {
  const selectedProfile = profileSelectEl.selectedOptions[0]?.textContent || activeProfileId() || "-";
  const runtimeState = runtimeStateEl.textContent?.replace("State:", "").trim() || "unknown";
  missionHudWorkspaceEl.textContent = selectedProfile;
  missionHudRuntimeEl.textContent = runtimeState;
  missionHudRoleEl.textContent = currentSetupState?.workspace_role || "-";
  missionHudApprovalsEl.textContent = summary ? String(summary.approvals_pending) : "0";
  missionHudReceiptsEl.textContent = summary ? String(summary.receipts_total) : "0";

  if (summary) {
    const openTasks = summary.workflow.pending + summary.workflow.in_progress + summary.workflow.blocked;
    missionHudOpenTasksEl.textContent = String(openTasks);
  } else {
    missionHudOpenTasksEl.textContent = "0";
  }
}

function refreshProfileIdentity() {
  const selectedLabel = profileSelectEl.selectedOptions[0]?.textContent?.trim() || "";
  const selectedName = selectedLabel.includes("(")
    ? selectedLabel.split("(")[0].trim()
    : selectedLabel || "Profile";
  const userName = currentSetupState?.user_display_name?.trim() || "";
  const displayName = userName || selectedName;
  const profileId = activeProfileId() || "-";
  const avatarChar = displayName.charAt(0).toUpperCase() || "?";
  profileDisplayNameEl.textContent = displayName;
  profileDisplayMetaEl.textContent = profileId;
  profileAvatarEl.textContent = avatarChar;
}

async function reloadRuntimeIfRunning(reason: string) {
  const profileId = ensureProfileId();
  const runtimeState = await invokeCommand<string>("runtime_state");
  if (/stopped|not started|idle/i.test(runtimeState)) {
    return;
  }
  await invokeGuarded("runtime_stop", { reason }, "runtime.stop");
  await invokeGuarded("runtime_start", { profileId }, "runtime.start");
  await refreshRuntimeState();
}

async function runToolingValidation(label: string) {
  const profileId = ensureProfileId();
  const [doctor, channelDoctor, status] = await Promise.all([
    invokeGuarded<OperationResult>("operations_doctor", { profileId }, "doctor.run"),
    invokeGuarded<OperationResult>("operations_channel_doctor", { profileId }, "channel.doctor"),
    invokeCommand<StatusReport>("operations_status", { profileId }),
  ]);
  toolsLifecycleOutputEl.textContent = JSON.stringify({ label, doctor, channel_doctor: channelDoctor, status }, null, 2);
  appendActivity(`Validation complete: ${label}`);
}

function renderChannelSummaries() {
  if (channelSummaries.length === 0) {
    channelsOutputEl.textContent = "No channels configured.";
    return;
  }
  channelsOutputEl.textContent = JSON.stringify(channelSummaries, null, 2);
}

function renderCronJobs() {
  if (cronJobs.length === 0) {
    cronListOutputEl.textContent = "No cron jobs configured.";
    return;
  }
  cronListOutputEl.textContent = JSON.stringify(cronJobs, null, 2);
}

async function refreshHandshake() {
  const handshake = await invokeCommand<ProtocolHandshake>("protocol_handshake");
  statusEl.textContent = `Core protocol ${handshake.core_protocol_version} | event schema ${handshake.event_schema_version}`;
  runtimeModeEl.textContent = isTauriRuntime
    ? "Runtime mode: Tauri native bridge"
    : "Runtime mode: web preview (mock command bridge)";
  handshakeOutputEl.textContent = JSON.stringify(handshake, null, 2);
}

async function refreshDeploymentCapabilities() {
  const profileId = activeProfileId();
  const capabilities = await invokeCommand<DeploymentCapabilities>("deployment_capabilities", {
    profileId: profileId || undefined,
  });
  runtimeModeEl.textContent = `Runtime mode: ${capabilities.effective_mode} (${capabilities.platform})`;
  deploymentOutputEl.textContent = JSON.stringify(capabilities, null, 2);
}

async function refreshHostConnection() {
  const profileId = ensureProfileId();
  const connection = await invokeCommand<HostConnectionState>("host_connection_get", { profileId });
  connectOutputEl.textContent = JSON.stringify(connection, null, 2);
}

async function refreshRolloutState() {
  const profileId = ensureProfileId();
  currentRolloutState = await invokeCommand<RolloutState>("rollout_state_get", { profileId });
  rolloutOutputEl.textContent = JSON.stringify(currentRolloutState, null, 2);
  rolloutSigningOutputEl.textContent = JSON.stringify(
    {
      signature_required: currentRolloutState.signature_required,
      trusted_signers: currentRolloutState.trusted_signers,
      last_verified_signer: currentRolloutState.last_verified_signer || null,
      last_promoted_at: currentRolloutState.last_promoted_at || null,
      last_verification_error: currentRolloutState.last_verification_error || null,
    },
    null,
    2,
  );
  rolloutSignatureRequiredEl.checked = Boolean(currentRolloutState.signature_required);
  rolloutTrustedSignersEl.value = currentRolloutState.trusted_signers.join("\n");
}

async function refreshPolicyProfile() {
  const profileId = ensureProfileId();
  const [active, templates] = await Promise.all([
    invokeCommand<PolicyProfileState | null>("policy_profile_get", { profileId }),
    invokeCommand<PolicyProfileTemplate[]>("policy_profiles_list"),
  ]);
  policyOutputEl.textContent = JSON.stringify(
    {
      active,
      templates,
    },
    null,
    2,
  );
}

async function refreshRbacRegistry() {
  const profileId = ensureProfileId();
  currentRbacRegistry = await invokeCommand<RbacRegistry>("rbac_users_list", { profileId });
  rbacOutputEl.textContent = JSON.stringify(currentRbacRegistry, null, 2);
}

async function refreshAuditVerification() {
  const profileId = ensureProfileId();
  currentAuditVerification = await invokeCommand<AuditLogVerification>("audit_log_verify", { profileId });
  const recentEvents = await invokeCommand<AuditEvent[]>("audit_log_list", { profileId, limit: 20 });
  auditOutputEl.textContent = JSON.stringify(
    {
      verification: currentAuditVerification,
      recent_events: recentEvents,
    },
    null,
    2,
  );
}

async function refreshAuditRemoteState() {
  const profileId = ensureProfileId();
  currentAuditRemoteState = await invokeCommand<AuditRemoteSinkState>("audit_remote_get", { profileId });
  auditRemoteOutputEl.textContent = JSON.stringify(currentAuditRemoteState, null, 2);
  auditRemoteEnabledEl.checked = currentAuditRemoteState.enabled;
  auditRemoteEndpointEl.value = currentAuditRemoteState.endpoint || "";
  auditRemoteKindEl.value = currentAuditRemoteState.sink_kind || "siem";
  auditRemoteAuthSecretEl.value = currentAuditRemoteState.auth_secret_id || "";
  auditRemoteVerifyTlsEl.checked = currentAuditRemoteState.verify_tls;
  auditRemoteBatchSizeEl.value = String(currentAuditRemoteState.batch_size || 200);
}

async function refreshOutcomes() {
  const profileId = ensureProfileId();
  const [summary, records] = await Promise.all([
    invokeCommand<OutcomeSummary>("outcomes_summary", { profileId }),
    invokeCommand<OutcomeRecord[]>("outcomes_list", { profileId, limit: 100 }),
  ]);
  currentOutcomeSummary = summary;
  outcomesOutputEl.textContent = JSON.stringify(
    {
      summary,
      records,
    },
    null,
    2,
  );
}

async function refreshBillingState() {
  const profileId = ensureProfileId();
  currentBillingState = await invokeCommand<BillingState>("billing_state_get", { profileId });
  billingOutputEl.textContent = JSON.stringify(currentBillingState, null, 2);
  billingBackendUrlEl.value = currentBillingState.backend_url || "";
  billingAuthSecretEl.value = currentBillingState.auth_secret_id || "";
  billingEnforceVerificationEl.checked = currentBillingState.enforce_verification;
}

async function refreshComplianceState() {
  const profileId = ensureProfileId();
  const [templates, profile, posture] = await Promise.all([
    invokeCommand<ComplianceProfileTemplate[]>("compliance_profiles_list"),
    invokeCommand<ComplianceProfileState | null>("compliance_profile_get", { profileId }),
    invokeCommand<CompliancePosture>("compliance_posture_get", { profileId }),
  ]);
  currentComplianceProfile = profile;
  currentCompliancePosture = posture;
  complianceOutputEl.textContent = JSON.stringify(
    {
      active_profile: profile,
      posture,
      templates,
    },
    null,
    2,
  );
}

function setWorkflowViewMode(mode: WorkflowViewMode) {
  workflowViewMode = mode;
  window.localStorage.setItem(WORKFLOW_VIEW_MODE_STORAGE_KEY, mode);
  const isBoard = mode === "board";
  workflowBoardViewEl.classList.toggle("is-hidden", !isBoard);
  workflowListViewEl.classList.toggle("is-hidden", isBoard);
  workflowViewBoardBtnEl.classList.toggle("is-active", isBoard);
  workflowViewListBtnEl.classList.toggle("is-active", !isBoard);
  workflowViewBoardBtnEl.classList.toggle("secondary-btn", !isBoard);
  workflowViewListBtnEl.classList.toggle("secondary-btn", isBoard);
  workflowDragHintEl.textContent = isBoard
    ? "Drag and drop cards in board view to transition status."
    : "Use dropdown + Apply in list view to transition status.";
}

function renderWorkflowKanban(tasks: WorkflowTaskRecord[]) {
  const columns: Array<[WorkflowTaskStatus, HTMLDivElement]> = [
    ["pending", workflowColPendingEl],
    ["in_progress", workflowColInProgressEl],
    ["done", workflowColDoneEl],
    ["failed", workflowColFailedEl],
    ["blocked", workflowColBlockedEl],
  ];
  for (const [, container] of columns) {
    container.innerHTML = "";
  }

  const counts: Record<WorkflowTaskStatus, number> = {
    pending: 0,
    in_progress: 0,
    done: 0,
    failed: 0,
    blocked: 0,
  };

  for (const task of tasks) {
    const entry = columns.find((item) => item[0] === task.status);
    if (!entry) {
      continue;
    }
    counts[task.status] += 1;
    const card = document.createElement("article");
    card.className = "card kanban-card";
    card.draggable = true;
    card.dataset.taskId = task.id;
    card.dataset.status = task.status;
    const tags = task.tags.length > 0 ? task.tags.join(", ") : "-";
    card.innerHTML = `
      <header>
        <strong>${task.title}</strong>
        <span class="pill ${task.status === "done" ? "enabled" : "disabled"}">${task.priority}</span>
      </header>
      <p><strong>ID:</strong> ${task.id}</p>
      <p><strong>Owner:</strong> ${task.owner || "-"}</p>
      <p><strong>Agent/Skill/Tool:</strong> ${task.agent_id || "-"}/${task.skill_id || "-"}/${task.tool_id || "-"}</p>
      <p><strong>Risk:</strong> ${task.risk_score.toFixed(1)}</p>
      <p><strong>Tags:</strong> ${tags}</p>
    `;
    entry[1].appendChild(card);
  }

  workflowCountPendingEl.textContent = String(counts.pending);
  workflowCountInProgressEl.textContent = String(counts.in_progress);
  workflowCountDoneEl.textContent = String(counts.done);
  workflowCountFailedEl.textContent = String(counts.failed);
  workflowCountBlockedEl.textContent = String(counts.blocked);
}

function renderWorkflowList(tasks: WorkflowTaskRecord[]) {
  workflowListBodyEl.innerHTML = "";
  const sorted = [...tasks].sort((a, b) => {
    const statusCompare = a.status.localeCompare(b.status);
    if (statusCompare !== 0) {
      return statusCompare;
    }
    return b.updated_at.localeCompare(a.updated_at);
  });

  for (const task of sorted) {
    const row = document.createElement("tr");
    const moveOptions = ["pending", "in_progress", "done", "failed", "blocked"]
      .map((status) => {
        const selected = status === task.status ? "selected" : "";
        return `<option value="${status}" ${selected}>${status}</option>`;
      })
      .join("");
    row.innerHTML = `
      <td>
        <strong>${task.title}</strong>
        <div class="table-meta">${task.id}</div>
      </td>
      <td><span class="pill ${task.status === "done" ? "enabled" : "disabled"}">${task.status}</span></td>
      <td>${task.priority}</td>
      <td>${task.owner || "-"}</td>
      <td>${task.risk_score.toFixed(1)}</td>
      <td>${task.agent_id || "-"}/${task.skill_id || "-"}/${task.tool_id || "-"}</td>
      <td class="table-actions">
        <select data-kind="workflow-row-status" data-id="${task.id}">
          ${moveOptions}
        </select>
        <button type="button" class="secondary-btn" data-kind="workflow-row-move" data-id="${task.id}">Apply</button>
      </td>
    `;
    workflowListBodyEl.appendChild(row);
  }
}

async function moveWorkflowTask(taskId: string, status: WorkflowTaskStatus, source: string) {
  const profileId = ensureProfileId();
  const request: WorkflowTaskMoveRequest = {
    task_id: taskId.trim(),
    status,
  };
  if (!request.task_id) {
    throw new Error("workflow move requires task id");
  }
  const result = await invokeGuarded<WorkflowTaskRecord>(
    "workflow_task_move",
    { profileId, request },
    "workflow.task_move",
  );
  appendActivity(`Workflow task moved (${source})`, result);
  await Promise.all([refreshWorkflowBoard(), refreshMissionControl(), refreshComplianceState()]);
}

async function refreshWorkflowBoard() {
  const profileId = ensureProfileId();
  currentWorkflowBoard = await invokeCommand<WorkflowBoardView>("workflow_board_get", { profileId, limit: 400 });
  workflowOutputEl.textContent = JSON.stringify(currentWorkflowBoard.summary, null, 2);
  renderWorkflowKanban(currentWorkflowBoard.tasks);
  renderWorkflowList(currentWorkflowBoard.tasks);
}

async function refreshMissionControl() {
  const profileId = ensureProfileId();
  const summary = await invokeCommand<MissionControlSummary>("mission_control_summary", { profileId });
  missionOutputEl.textContent = JSON.stringify(summary, null, 2);
  updateMissionKpis(summary);
  updateMissionHud(summary);
  currentAuditRemoteState = summary.audit_remote;
  currentBillingState = summary.billing;
  currentCompliancePosture = summary.compliance;
  if (!currentWorkflowBoard) {
    currentWorkflowBoard = { summary: summary.workflow, tasks: [] };
  } else {
    currentWorkflowBoard.summary = summary.workflow;
  }
}

function renderIntegrationCatalogSelect() {
  const current = integrationIdEl.value;
  integrationIdEl.innerHTML = "";

  const defaultOption = document.createElement("option");
  defaultOption.value = "";
  defaultOption.textContent = "Choose integration";
  integrationIdEl.appendChild(defaultOption);

  const grouped = new Map<string, IntegrationCatalogEntry[]>();
  for (const entry of integrationCatalog) {
    const key = entry.category || "other";
    if (!grouped.has(key)) {
      grouped.set(key, []);
    }
    grouped.get(key)!.push(entry);
  }

  for (const [category, entries] of [...grouped.entries()].sort(([a], [b]) => a.localeCompare(b))) {
    const group = document.createElement("optgroup");
    group.label = toTitleCase(category);
    for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
      const option = document.createElement("option");
      option.value = entry.name;
      option.textContent = `${entry.name} (${entry.status})`;
      group.appendChild(option);
    }
    integrationIdEl.appendChild(group);
  }

  if (current && integrationCatalog.some((entry) => entry.name === current)) {
    integrationIdEl.value = current;
  }
}

function updateIntegrationHintAndContract() {
  const selected = integrationCatalog.find((entry) => entry.name === integrationIdEl.value);
  if (!selected) {
    integrationCatalogHintEl.textContent =
      "Select an integration to see category, status, and setup guidance.";
    integrationKeyHintEl.textContent =
      "Required key: set provider/integration secret in Keys before publish.";
    return;
  }
  integrationCatalogHintEl.textContent =
    `${toTitleCase(selected.category)} · ${selected.status} · ${selected.description} · Setup: ${selected.setup_hint}`;
  integrationKeyHintEl.textContent = `Recommended key ID: ${keyForIntegration(selected.name)} (set in Keys step)`;

  const contract = suggestIntegrationContract(selected);
  integrationAccessEl.value = listToCsv(contract.can_access);
  integrationActionsEl.value = listToCsv(contract.can_do);
  integrationDestinationsEl.value = listToCsv(contract.data_destinations);
}

async function loadIntegrationCatalog() {
  const profileId = ensureProfileId();
  integrationCatalog = await invokeCommand<IntegrationCatalogEntry[]>("operations_integrations_catalog", { profileId });
  renderIntegrationCatalogSelect();
  updateIntegrationHintAndContract();
}

function applySkillPreset(presetId: string) {
  const preset = skillPresets[presetId];
  if (!preset) {
    return;
  }
  skillIdEl.value = preset.skill_id;
  skillNameEl.value = preset.display_name;
  skillSourceEl.value = preset.source;
  skillVersionEl.value = preset.version;
  skillAccessEl.value = listToCsv(preset.can_access);
  skillActionsEl.value = listToCsv(preset.can_do);
  skillDestinationsEl.value = listToCsv(preset.data_destinations);
  skillManifestEl.value = preset.manifest;
}

function applyMcpPreset(presetId: string) {
  const preset = mcpPresets[presetId];
  if (!preset) {
    return;
  }
  mcpIdEl.value = preset.connector_id;
  mcpNameEl.value = preset.display_name;
  mcpTransportEl.value = preset.transport;
  mcpEndpointEl.value = preset.endpoint || "";
  mcpCommandEl.value = preset.command || "";
  mcpArgsEl.value = listToCsv(preset.args);
  mcpEnvSecretIdsEl.value = listToCsv(preset.env_secret_ids);
  mcpTimeoutEl.value = String(preset.timeout_secs);
  mcpAccessEl.value = listToCsv(preset.can_access);
  mcpActionsEl.value = listToCsv(preset.can_do);
  mcpDestinationsEl.value = listToCsv(preset.data_destinations);
}

async function refreshMissionWorkspaces() {
  const profiles = await invokeCommand<ProfilesIndex>("profiles_list");
  const activeId = profiles.active_profile;
  const setupEntries = await Promise.all(
    profiles.profiles.map(async (profile) => {
      try {
        const setup = await invokeCommand<ProfileSetupState>("profile_setup_get", {
          profileId: profile.id,
        });
        return { profile, setup };
      } catch {
        return { profile, setup: null as ProfileSetupState | null };
      }
    }),
  );

  missionWorkspacesListEl.innerHTML = "";
  for (const { profile, setup } of setupEntries) {
    const card = document.createElement("article");
    card.className = "card workspace-card";
    const isActive = profile.id === activeId;
    const setupLabel = setup
      ? `${setup.deployment_mode} · ${setup.workspace_role} · ${setup.subscription_tier}`
      : "not configured";
    card.innerHTML = `
      <header>
        <strong>${profile.display_name}</strong>
        <span class="pill ${isActive ? "enabled" : "disabled"}">${isActive ? "active" : "inactive"}</span>
      </header>
      <p><strong>Workspace:</strong> ${profile.id}</p>
      <p><strong>Mode:</strong> ${setupLabel}</p>
      <p><strong>Provider:</strong> ${setup?.provider || "-"} / ${setup?.model || "-"}</p>
      <div class="card-actions">
        <button type="button" data-kind="workspace-switch" data-id="${profile.id}">Open</button>
      </div>
    `;
    missionWorkspacesListEl.appendChild(card);
  }
}

async function refreshProfiles() {
  const index = await invokeCommand<ProfilesIndex>("profiles_list");
  if (index.profiles.length === 0) {
    const created = await invokeCommand<ProfileRecord>("profiles_create", {
      displayName: "default",
    });
    appendActivity("Created default profile", created);
    return refreshProfiles();
  }

  profileSelectEl.innerHTML = "";
  for (const profile of index.profiles) {
    const option = document.createElement("option");
    option.value = profile.id;
    option.textContent = `${profile.display_name} (${profile.id})`;
    profileSelectEl.appendChild(option);
  }

  if (index.active_profile) {
    profileSelectEl.value = index.active_profile;
  } else {
    profileSelectEl.value = index.profiles[0].id;
  }
  refreshProfileIdentity();
  updateMissionHud();
}

async function refreshSetupState() {
  const profileId = ensureProfileId();
  const setup = await invokeCommand<ProfileSetupState>("profile_setup_get", { profileId });
  currentSetupState = setup;
  await refreshProviderCatalogPresets(profileId);
  setupUserNameEl.value = setup.user_display_name;
  setupAgentNameEl.value = setup.agent_name;
  setupWorkspaceModeEl.value = setup.workspace_mode;
  setupDeploymentModeEl.value = setup.deployment_mode;
  setupWorkspaceRoleEl.value = setup.workspace_role;
  setupSubscriptionTierEl.value = setup.subscription_tier;
  setupOrchestratorModeEl.value = setup.orchestrator_mode || "single_orchestrator";
  setupProviderEl.value = setup.provider;
  renderProviderGuidance(setup.provider);
  setupModelEl.value = setup.model;
  setupApiUrlEl.value = setup.api_url || "";
  setupDefaultTemperatureEl.value = String(
    Number.isFinite(setup.default_temperature) ? setup.default_temperature : 0.7,
  );
  setupMemoryEl.value = setup.memory_backend;
  setupRuntimeReasoningEnabledEl.value =
    setup.runtime_reasoning_enabled === true
      ? "enabled"
      : setup.runtime_reasoning_enabled === false
        ? "disabled"
        : "auto";
  setupAgentCompactContextEl.checked = Boolean(setup.agent_compact_context);
  setupAgentParallelToolsEl.checked = Boolean(setup.agent_parallel_tools);
  setupAgentMaxToolIterationsEl.value = String(
    Number.isFinite(setup.agent_max_tool_iterations) ? setup.agent_max_tool_iterations : 10,
  );
  setupAgentMaxHistoryMessagesEl.value = String(
    Number.isFinite(setup.agent_max_history_messages) ? setup.agent_max_history_messages : 50,
  );
  setupAgentToolDispatcherEl.value = setup.agent_tool_dispatcher || "auto";
  setupSkillsPromptInjectionModeEl.value =
    setup.skills_prompt_injection_mode === "compact" ? "compact" : "full";
  setupSkillsOpenEnabledEl.checked = Boolean(setup.skills_open_enabled);
  setupSkillsOpenDirEl.value = setup.skills_open_dir || "";
  setupEnableToolConnectorsEl.checked = Boolean(setup.enable_tool_connectors);
  setupDelegateAgentsJsonEl.value = JSON.stringify(setup.delegate_agents || {}, null, 2);
  renderDelegateAgentsSummary(setup.delegate_agents || {});
  if (!actorIdEl.value.trim() || actorIdEl.value.trim() === "local-user") {
    actorIdEl.value = setup.user_display_name;
  }
  if (!actorRoleEl.value.trim()) {
    actorRoleEl.value = setup.workspace_role;
  }
  const resolvedProviderKeyId = setup.provider_key_id || keyForProvider(setup.provider || "openrouter");
  secretKeyEl.value = resolvedProviderKeyId;
  if (secretKeyTemplateIds.has(resolvedProviderKeyId)) {
    secretKeyTemplateEl.value = resolvedProviderKeyId;
  } else {
    secretKeyTemplateEl.value = "";
  }
  syncSecretKeyEditingMode();
  secretOutputEl.textContent = `${resolvedProviderKeyId}: ${setup.has_provider_key ? "set" : "missing"}`;
  setupOutputEl.textContent = JSON.stringify(setup, null, 2);
  if (!isSetupReady(setup) && isOnboardingCompleteForProfile(profileId)) {
    markOnboardingCompletion(profileId, false);
  }
  refreshSetupSummaryCard();
  refreshSetupGateUi();
  refreshProfileIdentity();
  updateMissionHud();
}

async function refreshRuntimeState() {
  const state = await invokeCommand<string>("runtime_state");
  runtimeStateEl.textContent = `State: ${state}`;
  refreshDashboardMetrics();
  updateMissionHud();
}

async function refreshIntegrations() {
  const profileId = ensureProfileId();
  const registry = await invokeCommand<IntegrationRegistry>("integration_list", { profileId });
  integrations = registry.records;
  renderIntegrations();
  refreshDashboardMetrics();
}

async function refreshSkills() {
  const profileId = ensureProfileId();
  const registry = await invokeCommand<SkillsRegistry>("skills_list", { profileId });
  skills = registry.records;
  renderSkills();
  refreshDashboardMetrics();
}

async function refreshMcpConnectors() {
  const profileId = ensureProfileId();
  const registry = await invokeCommand<McpConnectorRegistry>("mcp_list", { profileId });
  mcpConnectors = registry.records;
  renderMcpConnectors();
  refreshDashboardMetrics();
}

async function refreshLogs() {
  try {
    const logs = await invokeCommand<LogLine[]>("logs_tail", { limit: 200 });
    logsOutputEl.textContent = JSON.stringify(logs, null, 2);
  } catch (error) {
    logsOutputEl.textContent = String(error);
  }
  refreshDashboardMetrics();
}

async function refreshAccessAndControlPlane() {
  const profileId = ensureProfileId();
  currentAccessState = await invokeCommand<AccessState>("access_state", { profileId });
  currentControlPlane = await invokeCommand<ControlPlaneState>("control_plane_state", { profileId });
  const accessSummary = JSON.stringify(currentAccessState, null, 2);
  accessOutputEl.textContent = accessSummary;
  controlPlaneOutputEl.textContent = JSON.stringify(
    {
      version: currentControlPlane.version,
      retention: currentControlPlane.retention,
      approvals: currentControlPlane.approvals.length,
      receipts: currentControlPlane.receipts.length,
    },
    null,
    2,
  );
  retentionReceiptsDaysEl.value = String(currentControlPlane.retention.receipts_days);
  retentionApprovalsDaysEl.value = String(currentControlPlane.retention.approvals_days);
  applyViewScope();
}

async function refreshApprovals() {
  const profileId = ensureProfileId();
  approvals = await invokeCommand<ApprovalRequest[]>("approvals_list", {
    profileId,
    pendingOnly: false,
  });
  renderApprovals();
  refreshDashboardMetrics();
}

async function refreshReceipts() {
  const profileId = ensureProfileId();
  receipts = await invokeCommand<ActionReceipt[]>("receipts_list", {
    profileId,
    limit: 200,
  });
  renderReceipts();
  refreshDashboardMetrics();
}

async function refreshChannels() {
  const profileId = ensureProfileId();
  channelSummaries = await invokeCommand<ChannelSummary[]>("operations_channels_list", { profileId });
  renderChannelSummaries();
}

async function refreshCron() {
  const profileId = ensureProfileId();
  cronJobs = await invokeCommand<CronJobSummary[]>("operations_cron_list", { profileId });
  renderCronJobs();
}

function renderIntegrations() {
  integrationListEl.innerHTML = "";
  if (integrations.length === 0) {
    integrationListEl.textContent = "No integrations installed.";
    return;
  }

  for (const record of integrations) {
    const card = document.createElement("article");
    card.className = "card";
    card.innerHTML = `
      <header>
        <strong>${record.integration_id}</strong>
        <span class="pill ${record.enabled ? "enabled" : "disabled"}">${record.enabled ? "enabled" : "disabled"}</span>
      </header>
      <p><strong>Access:</strong> ${record.contract.can_access.join(", ") || "none"}</p>
      <p><strong>Actions:</strong> ${record.contract.can_do.join(", ") || "none"}</p>
      <p><strong>Data destinations:</strong> ${record.contract.data_destinations.join(", ") || "none"}</p>
      <p><strong>Stage:</strong> ${record.enabled ? "published" : "draft"}</p>
      <div class="card-actions">
        <button type="button" data-kind="integration-validate" data-id="${record.integration_id}" class="secondary-btn">Validate</button>
        <button type="button" data-kind="integration-enable" data-id="${record.integration_id}">Publish</button>
        <button type="button" data-kind="integration-disable" data-id="${record.integration_id}">Disable</button>
        <button type="button" data-kind="integration-remove" data-id="${record.integration_id}" class="secondary-btn">Remove</button>
      </div>
    `;
    integrationListEl.appendChild(card);
  }
}

function renderSkills() {
  skillsListEl.innerHTML = "";
  if (skills.length === 0) {
    skillsListEl.textContent = "No skills installed.";
    return;
  }

  for (const record of skills) {
    const card = document.createElement("article");
    card.className = "card";
    card.innerHTML = `
      <header>
        <strong>${record.display_name} (${record.skill_id})</strong>
        <span class="pill ${record.enabled ? "enabled" : "disabled"}">${record.enabled ? "enabled" : "disabled"}</span>
      </header>
      <p><strong>Source:</strong> ${record.source} | <strong>Version:</strong> ${record.version}</p>
      <p><strong>Access:</strong> ${record.contract.can_access.join(", ") || "none"}</p>
      <p><strong>Actions:</strong> ${record.contract.can_do.join(", ") || "none"}</p>
      <p><strong>Data destinations:</strong> ${record.contract.data_destinations.join(", ") || "none"}</p>
      <p><strong>Stage:</strong> ${record.enabled ? "published" : "draft"}</p>
      <div class="card-actions">
        <button type="button" data-kind="skills-validate" data-id="${record.skill_id}" class="secondary-btn">Validate</button>
        <button type="button" data-kind="skills-enable" data-id="${record.skill_id}">Publish</button>
        <button type="button" data-kind="skills-disable" data-id="${record.skill_id}">Disable</button>
        <button type="button" data-kind="skills-remove" data-id="${record.skill_id}">Remove</button>
      </div>
    `;
    skillsListEl.appendChild(card);
  }
}

function renderMcpConnectors() {
  mcpListEl.innerHTML = "";
  if (mcpConnectors.length === 0) {
    mcpListEl.textContent = "No Tool Connectors (MCP) installed.";
    return;
  }

  for (const record of mcpConnectors) {
    const card = document.createElement("article");
    card.className = "card";
    card.innerHTML = `
      <header>
        <strong>${record.display_name} (${record.connector_id})</strong>
        <span class="pill ${record.enabled ? "enabled" : "disabled"}">${record.enabled ? "enabled" : "disabled"}</span>
      </header>
      <p><strong>Transport:</strong> ${record.config.transport}</p>
      <p><strong>Endpoint:</strong> ${record.config.endpoint || "-"}</p>
      <p><strong>Command:</strong> ${record.config.command || "-"}</p>
      <p><strong>Args:</strong> ${record.config.args.join(" ") || "-"}</p>
      <p><strong>Access:</strong> ${record.contract.can_access.join(", ") || "none"}</p>
      <p><strong>Actions:</strong> ${record.contract.can_do.join(", ") || "none"}</p>
      <p><strong>Data destinations:</strong> ${record.contract.data_destinations.join(", ") || "none"}</p>
      <p><strong>Stage:</strong> ${record.enabled ? "published" : "draft"}</p>
      <div class="card-actions">
        <button type="button" data-kind="mcp-validate" data-id="${record.connector_id}" class="secondary-btn">Validate</button>
        <button type="button" data-kind="mcp-enable" data-id="${record.connector_id}">Publish</button>
        <button type="button" data-kind="mcp-disable" data-id="${record.connector_id}">Disable</button>
        <button type="button" data-kind="mcp-remove" data-id="${record.connector_id}">Remove</button>
      </div>
    `;
    mcpListEl.appendChild(card);
  }
}

function renderApprovals() {
  approvalsListEl.innerHTML = "";
  if (approvals.length === 0) {
    approvalsListEl.textContent = "No approval requests.";
    return;
  }

  for (const request of approvals) {
    const card = document.createElement("article");
    card.className = "card";
    card.innerHTML = `
      <header>
        <strong>${request.action}</strong>
        <span class="pill ${request.status === "approved" ? "enabled" : request.status === "pending" ? "disabled" : "disabled"}">${request.status}</span>
      </header>
      <p><strong>ID:</strong> ${request.id}</p>
      <p><strong>Actor:</strong> ${request.actor_id} (${request.actor_role})</p>
      <p><strong>Resource:</strong> ${request.resource}</p>
      <p><strong>Destination:</strong> ${request.destination}</p>
      <p><strong>Decided:</strong> ${request.decided_at || "pending"}</p>
    `;
    approvalsListEl.appendChild(card);
  }
}

function renderReceipts() {
  receiptsListEl.innerHTML = "";
  if (receipts.length === 0) {
    receiptsListEl.textContent = "No action receipts.";
    return;
  }

  for (const receipt of receipts.slice(0, 40)) {
    const card = document.createElement("article");
    card.className = "card";
    card.innerHTML = `
      <header>
        <strong>${receipt.action}</strong>
        <span class="pill ${receipt.result === "allowed" ? "enabled" : "disabled"}">${receipt.result}</span>
      </header>
      <p><strong>ID:</strong> ${receipt.id}</p>
      <p><strong>Actor:</strong> ${receipt.actor_id} (${receipt.actor_role})</p>
      <p><strong>Resource:</strong> ${receipt.resource}</p>
      <p><strong>Destination:</strong> ${receipt.destination}</p>
      <p><strong>Reason:</strong> ${receipt.reason}</p>
      <p><strong>Time:</strong> ${receipt.timestamp}</p>
    `;
    receiptsListEl.appendChild(card);
  }
}

function integrationConsent(record: IntegrationRecord): boolean {
  const prompt = [
    `Enable integration '${record.integration_id}'?`,
    `Can access: ${record.contract.can_access.join(", ") || "none"}`,
    `Can do: ${record.contract.can_do.join(", ") || "none"}`,
    `Data goes to: ${record.contract.data_destinations.join(", ") || "none"}`,
  ].join("\n");
  return window.confirm(prompt);
}

function skillsConsent(record: SkillRecord): boolean {
  const prompt = [
    `Enable skill '${record.display_name}' (${record.skill_id})?`,
    `Can access: ${record.contract.can_access.join(", ") || "none"}`,
    `Can do: ${record.contract.can_do.join(", ") || "none"}`,
    `Data goes to: ${record.contract.data_destinations.join(", ") || "none"}`,
  ].join("\n");
  return window.confirm(prompt);
}

function mcpConsent(record: McpConnectorRecord): boolean {
  const prompt = [
    `Enable Tool Connector (MCP) '${record.display_name}' (${record.connector_id})?`,
    `Can access: ${record.contract.can_access.join(", ") || "none"}`,
    `Can do: ${record.contract.can_do.join(", ") || "none"}`,
    `Data goes to: ${record.contract.data_destinations.join(", ") || "none"}`,
  ].join("\n");
  return window.confirm(prompt);
}

async function handleCardAction(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (!target || target.tagName !== "BUTTON") {
    return;
  }

  const button = target as HTMLButtonElement;
  const kind = button.dataset.kind;
  const id = button.dataset.id;
  if (!kind || !id) {
    return;
  }

  const profileId = ensureProfileId();
  if (kind === "integration-enable") {
    const record = integrations.find((item) => item.integration_id === id);
    if (!record) return;
    if (!integrationConsent(record)) return;
    await invokeGuarded("integration_enable", { profileId, integrationId: id, approved: true }, "integration.enable");
    await Promise.all([refreshIntegrations(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`integration ${id} published`);
    appendActivity(`Integration enabled: ${id}`);
  } else if (kind === "integration-disable") {
    await invokeGuarded("integration_disable", { profileId, integrationId: id }, "integration.disable");
    await Promise.all([refreshIntegrations(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`integration ${id} disabled`);
    appendActivity(`Integration disabled: ${id}`);
  } else if (kind === "integration-remove") {
    await invokeGuarded("integration_remove", { profileId, integrationId: id }, "integration.remove");
    await Promise.all([refreshIntegrations(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`integration ${id} removed`);
    appendActivity(`Integration removed: ${id}`);
  } else if (kind === "integration-validate") {
    await runToolingValidation(`integration:${id}`);
  } else if (kind === "skills-enable") {
    const record = skills.find((item) => item.skill_id === id);
    if (!record) return;
    if (!skillsConsent(record)) return;
    await invokeGuarded("skills_enable", { profileId, skillId: id, approved: true }, "skills.enable");
    await Promise.all([refreshSkills(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`skill ${id} published`);
    appendActivity(`Skill enabled: ${id}`);
  } else if (kind === "skills-disable") {
    await invokeGuarded("skills_disable", { profileId, skillId: id }, "skills.disable");
    await Promise.all([refreshSkills(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`skill ${id} disabled`);
    appendActivity(`Skill disabled: ${id}`);
  } else if (kind === "skills-remove") {
    await invokeGuarded("skills_remove", { profileId, skillId: id }, "skills.remove");
    await Promise.all([refreshSkills(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`skill ${id} removed`);
    appendActivity(`Skill removed: ${id}`);
  } else if (kind === "skills-validate") {
    await runToolingValidation(`skill:${id}`);
  } else if (kind === "mcp-enable") {
    const record = mcpConnectors.find((item) => item.connector_id === id);
    if (!record) return;
    if (!mcpConsent(record)) return;
    await invokeGuarded("mcp_enable", { profileId, connectorId: id, approved: true }, "mcp.enable");
    await Promise.all([refreshMcpConnectors(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`mcp ${id} published`);
    appendActivity(`Tool connector enabled: ${id}`);
  } else if (kind === "mcp-disable") {
    await invokeGuarded("mcp_disable", { profileId, connectorId: id }, "mcp.disable");
    await Promise.all([refreshMcpConnectors(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`mcp ${id} disabled`);
    appendActivity(`Tool connector disabled: ${id}`);
  } else if (kind === "mcp-remove") {
    await invokeGuarded("mcp_remove", { profileId, connectorId: id }, "mcp.remove");
    await Promise.all([refreshMcpConnectors(), refreshApprovals(), refreshReceipts()]);
    await reloadRuntimeIfRunning(`mcp ${id} removed`);
    appendActivity(`Tool connector removed: ${id}`);
  } else if (kind === "mcp-validate") {
    await runToolingValidation(`mcp:${id}`);
  }
}

async function refreshAll() {
  await refreshHandshake();
  await refreshProfiles();
  await Promise.all([
    refreshSetupState(),
    refreshDeploymentCapabilities(),
    refreshHostConnection(),
    refreshRuntimeState(),
    refreshIntegrations(),
    refreshSkills(),
    refreshMcpConnectors(),
    refreshChannels(),
    refreshCron(),
    loadIntegrationCatalog(),
    refreshLogs(),
    refreshAccessAndControlPlane(),
    refreshApprovals(),
    refreshReceipts(),
    refreshRolloutState(),
    refreshPolicyProfile(),
    refreshRbacRegistry(),
    refreshAuditVerification(),
    refreshAuditRemoteState(),
    refreshOutcomes(),
    refreshBillingState(),
    refreshWorkflowBoard(),
    refreshComplianceState(),
    refreshMissionControl(),
    refreshMissionWorkspaces(),
  ]);
}

function bindUiHandlers() {
  for (const link of routeLinks) {
    link.addEventListener("click", () => {
      const target = link.dataset.routeTarget;
      if (!target) return;
      if (isRouteLockedByOnboarding(target)) {
        setActiveRoute("safety");
        return;
      }
      setActiveRoute(target);
    });
  }

  stepperPrevEl.addEventListener("click", () => {
    const idx = guidedStepRoutes.indexOf(currentRoute as (typeof guidedStepRoutes)[number]);
    if (idx <= 0) {
      return;
    }
    setActiveRoute(guidedStepRoutes[idx - 1]);
  });

  stepperNextEl.addEventListener("click", () => {
    const idx = guidedStepRoutes.indexOf(currentRoute as (typeof guidedStepRoutes)[number]);
    const maxUnlockedStepIndex = isOnboardingLocked()
      ? guidedStepRoutes.indexOf("channels")
      : guidedStepRoutes.length - 1;
    if (idx < 0 || idx >= maxUnlockedStepIndex) {
      return;
    }
    setActiveRoute(guidedStepRoutes[idx + 1]);
  });

  themeToggleEl.addEventListener("click", () => {
    const current = (document.body.dataset.theme as "light" | "dark" | undefined) || "light";
    applyTheme(current === "light" ? "dark" : "light");
  });

  openSettingsEl.addEventListener("click", () => {
    setActiveRoute("profile");
  });

  openToolsEl.addEventListener("click", () => {
    setActiveRoute("channels");
  });

  openMissionEl.addEventListener("click", () => {
    setActiveRoute("mission");
  });

  profileSelectEl.addEventListener("change", () => {
    refreshProfileIdentity();
  });

  for (const trigger of helpTriggerEls) {
    trigger.addEventListener("click", () => {
      const topic = trigger.dataset.help || "setup";
      openHelp(topic);
    });
  }

  integrationIdEl.addEventListener("change", () => {
    updateIntegrationHintAndContract();
  });

  skillPresetEl.addEventListener("change", () => {
    applySkillPreset(skillPresetEl.value);
  });

  const onSetupProviderChanged = () => {
    const provider = setupProviderEl.value.trim();
    renderProviderGuidance(provider);
    if (!provider) {
      return;
    }
    const suggested = keyForProvider(provider);
    if (!secretKeyEl.value.trim() || secretKeyEl.value.trim().startsWith("provider.")) {
      secretKeyEl.value = suggested;
    }
    if (secretKeyTemplateIds.has(suggested)) {
      secretKeyTemplateEl.value = suggested;
    }
    syncSecretKeyEditingMode();
  };
  setupProviderEl.addEventListener("change", onSetupProviderChanged);
  setupProviderEl.addEventListener("input", onSetupProviderChanged);

  mcpPresetEl.addEventListener("change", () => {
    applyMcpPreset(mcpPresetEl.value);
  });

  setupEditorToggleEl.addEventListener("click", () => {
    setupEditorExpanded = !setupEditorExpanded;
    setupEditorEl.classList.toggle("is-hidden", !setupEditorExpanded);
    setupEditorToggleEl.textContent = setupEditorExpanded ? "Hide setup" : "Edit setup";
  });

  setupGoMissionEl.addEventListener("click", () => {
    setActiveRoute("mission");
  });

  setupCompleteInitialBtnEl.addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      if (!isSetupReady(currentSetupState)) {
        throw new Error("Setup is not ready yet. Save provider/model and key first.");
      }
      markOnboardingCompletion(profileId, true);
      appendActivity("Initial setup completed", { profile_id: profileId });
      refreshSetupGateUi();
      await Promise.all([refreshMissionControl(), refreshMissionWorkspaces()]);
      setActiveRoute("mission");
    }, "Initial setup completion failed"),
  );

  must<HTMLFormElement>("setup-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const delegateAgents = parseDelegateAgentsJson(setupDelegateAgentsJsonEl.value);
      const defaultTemperature = Number(setupDefaultTemperatureEl.value);
      const runtimeReasoningEnabled =
        setupRuntimeReasoningEnabledEl.value === "enabled"
          ? true
          : setupRuntimeReasoningEnabledEl.value === "disabled"
            ? false
            : undefined;
      const payload: ProfileSetupPayload = {
        user_display_name: setupUserNameEl.value.trim() || "Operator",
        agent_name: setupAgentNameEl.value.trim() || "Right Hand",
        workspace_mode: setupWorkspaceModeEl.value as SetupWorkspaceMode,
        deployment_mode: setupDeploymentModeEl.value as DeploymentMode,
        workspace_role: setupWorkspaceRoleEl.value as WorkspaceRole,
        subscription_tier: setupSubscriptionTierEl.value as SubscriptionTier,
        orchestrator_mode: setupOrchestratorModeEl.value.trim() || "single_orchestrator",
        provider: setupProviderEl.value.trim() || "openrouter",
        model: setupModelEl.value.trim() || "anthropic/claude-sonnet-4",
        api_url: setupApiUrlEl.value.trim() || undefined,
        default_temperature: Number.isFinite(defaultTemperature) ? defaultTemperature : 0.7,
        memory_backend: setupMemoryEl.value.trim() || "sqlite",
        runtime_reasoning_enabled: runtimeReasoningEnabled,
        agent_compact_context: setupAgentCompactContextEl.checked,
        agent_parallel_tools: setupAgentParallelToolsEl.checked,
        agent_max_tool_iterations: Math.max(1, Number(setupAgentMaxToolIterationsEl.value) || 10),
        agent_max_history_messages: Math.max(1, Number(setupAgentMaxHistoryMessagesEl.value) || 50),
        agent_tool_dispatcher: setupAgentToolDispatcherEl.value.trim() || "auto",
        skills_prompt_injection_mode: setupSkillsPromptInjectionModeEl.value || "full",
        skills_open_enabled: setupSkillsOpenEnabledEl.checked,
        skills_open_dir: setupSkillsOpenDirEl.value.trim() || undefined,
        enable_tool_connectors: setupEnableToolConnectorsEl.checked,
        delegate_agents: delegateAgents,
        api_key: setupApiKeyEl.value.trim() || undefined,
      };
      const result = await invokeGuarded<ProfileSetupState>(
        "profile_setup_save",
        { profileId, payload },
        "profile.setup",
      );
      markOnboardingCompletion(profileId, false);
      currentSetupState = result;
      setupApiKeyEl.value = "";
      actorIdEl.value = payload.user_display_name;
      actorRoleEl.value = payload.workspace_role;
      setupOutputEl.textContent = JSON.stringify(result, null, 2);
      renderDelegateAgentsSummary(result.delegate_agents || {});
      appendActivity("Setup saved", result);
      await Promise.all([
        refreshSetupState(),
        refreshDeploymentCapabilities(),
        refreshHostConnection(),
        refreshAccessAndControlPlane(),
        refreshChannels(),
        loadIntegrationCatalog(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionControl(),
        refreshMissionWorkspaces(),
      ]);
      setupEditorExpanded = false;
      setupEditorEl.classList.add("is-hidden");
      await reloadRuntimeIfRunning("setup updated");
      setActiveRoute("safety");
    }, "Setup save failed");
  });

  must<HTMLButtonElement>("tools-validate-runtime").addEventListener("click", () =>
    runSafely(async () => {
      await runToolingValidation("runtime");
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Runtime validation failed"),
  );

  must<HTMLButtonElement>("tools-reload-runtime").addEventListener("click", () =>
    runSafely(async () => {
      await reloadRuntimeIfRunning("manual tooling reload");
      appendActivity("Runtime reload complete");
    }, "Runtime reload failed"),
  );

  must<HTMLButtonElement>("integrations-catalog").addEventListener("click", () =>
    runSafely(async () => {
      await loadIntegrationCatalog();
      toolsLifecycleOutputEl.textContent = JSON.stringify(
        {
          count: integrationCatalog.length,
          categories: [...new Set(integrationCatalog.map((entry) => entry.category))].sort(),
        },
        null,
        2,
      );
      appendActivity("Integration catalog loaded", { count: integrationCatalog.length });
    }, "Integration catalog load failed"),
  );

  must<HTMLButtonElement>("connect-generate-invite").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const bundle = await invokeCommand<PairingBundle>("pairing_create_bundle", {
        profileId,
        transport: "lan",
        expiresInMinutes: 30,
      });
      connectInvitePayloadEl.value = JSON.stringify(bundle, null, 2);
      connectOutputEl.textContent = JSON.stringify(bundle, null, 2);
      appendActivity("Generated host connect invite", bundle);
    }, "Generate connect invite failed"),
  );

  must<HTMLButtonElement>("connect-attach-host").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const payload: HostConnectPayload = {
        invite_payload: connectInvitePayloadEl.value.trim(),
      };
      if (!payload.invite_payload) {
        throw new Error("invite payload is required");
      }
      const result = await invokeGuarded<HostConnectionState>(
        "client_connect_host",
        { profileId, payload },
        "host.connect",
      );
      connectOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Client attached to host", result);
      await Promise.all([refreshHostConnection(), refreshMissionControl()]);
    }, "Attach client to host failed"),
  );

  must<HTMLButtonElement>("connect-refresh-state").addEventListener("click", () =>
    runSafely(refreshHostConnection, "Refresh host connection failed"),
  );

  secretApplyTemplateEl.addEventListener("click", () => {
    const template = secretKeyTemplateEl.value.trim();
    if (!template) {
      syncSecretKeyEditingMode();
      return;
    }
    secretKeyEl.value = template;
    secretOutputEl.textContent = `Selected template: ${template}`;
    syncSecretKeyEditingMode();
  });

  secretKeyTemplateEl.addEventListener("change", () => {
    const template = secretKeyTemplateEl.value.trim();
    if (!template) {
      syncSecretKeyEditingMode();
      return;
    }
    secretKeyEl.value = template;
    syncSecretKeyEditingMode();
  });

  secretKeyEl.addEventListener("input", () => {
    renderSecretKeyGuidance(secretKeyEl.value);
  });

  secretAdvancedToggleEl.addEventListener("change", () => {
    syncSecretKeyEditingMode();
  });

  must<HTMLButtonElement>("secret-save").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const key = resolveSecretKeyIdForActions();
      await invokeCommand("secret_set", {
        profileId,
        key,
        value: secretValueEl.value,
      });
      secretValueEl.value = "";
      const exists = await invokeCommand<boolean>("secret_exists", { profileId, key });
      secretOutputEl.textContent = `${key}: ${exists ? "set" : "missing"}`;
      appendActivity(`Secret saved: ${key}`);
      await refreshSetupState();
    }, "Secret save failed"),
  );

  must<HTMLButtonElement>("secret-check").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const key = resolveSecretKeyIdForActions();
      const exists = await invokeCommand<boolean>("secret_exists", { profileId, key });
      secretOutputEl.textContent = `${key}: ${exists ? "set" : "missing"}`;
    }, "Secret check failed"),
  );

  must<HTMLButtonElement>("secret-delete").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const key = resolveSecretKeyIdForActions();
      await invokeCommand("secret_delete", { profileId, key });
      secretOutputEl.textContent = `${key}: deleted`;
      appendActivity(`Secret deleted: ${key}`);
      await refreshSetupState();
    }, "Secret delete failed"),
  );

  must<HTMLButtonElement>("refresh-handshake").addEventListener("click", () =>
    runSafely(async () => {
      await Promise.all([refreshHandshake(), refreshDeploymentCapabilities()]);
    }, "Refresh handshake failed"),
  );

  must<HTMLButtonElement>("mission-refresh").addEventListener("click", () =>
    runSafely(async () => {
      await Promise.all([
        refreshMissionControl(),
        refreshRolloutState(),
        refreshPolicyProfile(),
        refreshRbacRegistry(),
        refreshAuditVerification(),
        refreshAuditRemoteState(),
        refreshOutcomes(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionWorkspaces(),
      ]);
    }, "Mission refresh failed"),
  );

  must<HTMLButtonElement>("mission-runtime-status").addEventListener("click", () =>
    runSafely(async () => {
      await Promise.all([refreshRuntimeState(), refreshMissionControl()]);
      appendActivity("Mission runtime status refreshed");
    }, "Mission runtime status failed"),
  );

  must<HTMLButtonElement>("mission-runtime-doctor").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const [doctor, channelDoctor, status] = await Promise.all([
        invokeGuarded<OperationResult>("operations_doctor", { profileId }, "doctor.run"),
        invokeGuarded<OperationResult>("operations_channel_doctor", { profileId }, "channel.doctor"),
        invokeCommand<StatusReport>("operations_status", { profileId }),
      ]);
      missionOutputEl.textContent = JSON.stringify(
        {
          mission_runtime_doctor: {
            doctor,
            channel_doctor: channelDoctor,
            status,
          },
        },
        null,
        2,
      );
      await Promise.all([refreshApprovals(), refreshReceipts(), refreshMissionControl()]);
    }, "Mission runtime doctor failed"),
  );

  must<HTMLButtonElement>("mission-runtime-start").addEventListener("click", () =>
    runSafely(async () => {
      await invokeGuarded("runtime_start", { profileId: ensureProfileId() }, "runtime.start");
      await Promise.all([refreshRuntimeState(), refreshMissionControl(), refreshMissionWorkspaces()]);
      appendActivity("Mission runtime started");
    }, "Mission runtime start failed"),
  );

  must<HTMLButtonElement>("mission-runtime-stop").addEventListener("click", () =>
    runSafely(async () => {
      await invokeGuarded("runtime_stop", { reason: "mission-control stop" }, "runtime.stop");
      await Promise.all([refreshRuntimeState(), refreshMissionControl(), refreshMissionWorkspaces()]);
      appendActivity("Mission runtime stopped");
    }, "Mission runtime stop failed"),
  );

  must<HTMLButtonElement>("mission-workspaces-refresh").addEventListener("click", () =>
    runSafely(refreshMissionWorkspaces, "Mission workspace refresh failed"),
  );

  must<HTMLFormElement>("mission-workspace-create-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const name = missionWorkspaceCreateNameEl.value.trim();
      if (!name) {
        return;
      }
      const created = await invokeCommand<ProfileRecord>("profiles_create", { displayName: name });
      missionWorkspaceCreateNameEl.value = "";
      await refreshProfiles();
      profileSelectEl.value = created.id;
      await invokeCommand("profiles_switch", { profileId: created.id });
      await Promise.all([
        refreshSetupState(),
        refreshDeploymentCapabilities(),
        refreshHostConnection(),
        refreshIntegrations(),
        refreshSkills(),
        refreshMcpConnectors(),
        refreshChannels(),
        refreshCron(),
        refreshRuntimeState(),
        refreshLogs(),
        refreshAccessAndControlPlane(),
        refreshApprovals(),
        refreshReceipts(),
        refreshRolloutState(),
        refreshPolicyProfile(),
        refreshRbacRegistry(),
        refreshAuditVerification(),
        refreshAuditRemoteState(),
        refreshOutcomes(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionControl(),
        loadIntegrationCatalog(),
        refreshMissionWorkspaces(),
      ]);
      appendActivity("Workspace created from mission", created);
    }, "Mission workspace create failed");
  });

  must<HTMLButtonElement>("audit-refresh").addEventListener("click", () =>
    runSafely(refreshAuditVerification, "Audit verification failed"),
  );

  must<HTMLButtonElement>("evidence-export").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeCommand<EvidenceExportSummary>("evidence_export", { profileId });
      evidenceOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Evidence export completed", result);
    }, "Evidence export failed"),
  );

  must<HTMLFormElement>("rollout-stage-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: RolloutStageRequest = {
        release_id: rolloutReleaseIdEl.value.trim(),
        version: rolloutVersionEl.value.trim(),
        checksum_sha256: rolloutChecksumEl.value.trim(),
        signature: rolloutSignatureEl.value.trim() || undefined,
        sbom_checksum_sha256: rolloutSbomChecksumEl.value.trim() || undefined,
        ring: rolloutRingEl.value as RolloutRing,
      };
      const result = await invokeGuarded<RolloutState>(
        "rollout_stage_release",
        { profileId, request },
        "release.stage",
      );
      rolloutOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Release staged", result);
      await refreshMissionControl();
    }, "Stage release failed");
  });

  must<HTMLButtonElement>("rollout-promote").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeGuarded<RolloutState>(
        "rollout_promote",
        { profileId },
        "release.promote",
      );
      rolloutOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Release promoted", result);
      await refreshMissionControl();
    }, "Rollout promote failed"),
  );

  must<HTMLButtonElement>("rollout-rollback").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeGuarded<RolloutState>(
        "rollout_rollback",
        { profileId },
        "release.rollback",
      );
      rolloutOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Release rolled back", result);
      await refreshMissionControl();
    }, "Rollout rollback failed"),
  );

  must<HTMLButtonElement>("rollout-refresh").addEventListener("click", () =>
    runSafely(refreshRolloutState, "Rollout refresh failed"),
  );

  must<HTMLFormElement>("rollout-signing-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: RolloutSigningPolicyRequest = {
        signature_required: rolloutSignatureRequiredEl.checked,
        trusted_signers: rolloutTrustedSignersEl.value
          .split("\n")
          .map((item) => item.trim())
          .filter((item) => item.length > 0),
      };
      const result = await invokeGuarded<RolloutState>(
        "rollout_set_signing_policy",
        { profileId, request },
        "release.signing_policy",
      );
      rolloutSigningOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Rollout signing policy updated", result);
      await Promise.all([refreshRolloutState(), refreshMissionControl()]);
    }, "Rollout signing policy update failed");
  });

  must<HTMLButtonElement>("policy-apply").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const templateId = policyTemplateIdEl.value;
      const result = await invokeGuarded<PolicyProfileState>(
        "policy_profile_apply",
        { profileId, templateId },
        "policy.apply",
      );
      policyOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Policy profile applied", result);
      await Promise.all([refreshPolicyProfile(), refreshMissionControl()]);
    }, "Policy profile apply failed"),
  );

  must<HTMLButtonElement>("policy-refresh").addEventListener("click", () =>
    runSafely(refreshPolicyProfile, "Policy profile refresh failed"),
  );

  must<HTMLButtonElement>("compliance-apply").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const templateId = complianceTemplateIdEl.value;
      const result = await invokeGuarded<ComplianceProfileState>(
        "compliance_profile_apply",
        { profileId, templateId },
        "compliance.apply",
      );
      appendActivity("Compliance profile applied", result);
      await Promise.all([
        refreshComplianceState(),
        refreshPolicyProfile(),
        refreshRolloutState(),
        refreshBillingState(),
        refreshAuditRemoteState(),
        refreshMissionControl(),
      ]);
    }, "Compliance profile apply failed"),
  );

  must<HTMLButtonElement>("compliance-refresh").addEventListener("click", () =>
    runSafely(refreshComplianceState, "Compliance posture refresh failed"),
  );

  must<HTMLFormElement>("rbac-user-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request = {
        user_id: rbacUserIdEl.value.trim(),
        display_name: rbacDisplayNameEl.value.trim(),
        role: rbacRoleEl.value as WorkspaceRole,
        active: rbacActiveEl.checked,
      };
      const registry = await invokeGuarded<RbacRegistry>(
        "rbac_user_upsert",
        { profileId, request },
        "rbac.manage",
      );
      rbacOutputEl.textContent = JSON.stringify(registry, null, 2);
      appendActivity("RBAC user upserted", request);
      await refreshMissionControl();
    }, "RBAC update failed");
  });

  must<HTMLButtonElement>("rbac-refresh").addEventListener("click", () =>
    runSafely(refreshRbacRegistry, "RBAC refresh failed"),
  );

  must<HTMLFormElement>("audit-remote-config-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: AuditRemoteConfigureRequest = {
        enabled: auditRemoteEnabledEl.checked,
        endpoint: auditRemoteEndpointEl.value.trim() || undefined,
        sink_kind: auditRemoteKindEl.value.trim() || undefined,
        auth_secret_id: auditRemoteAuthSecretEl.value.trim() || undefined,
        verify_tls: auditRemoteVerifyTlsEl.checked,
        batch_size: Number(auditRemoteBatchSizeEl.value) || 200,
      };
      const result = await invokeGuarded<AuditRemoteSinkState>(
        "audit_remote_configure",
        { profileId, request },
        "audit.remote.configure",
      );
      auditRemoteOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Remote audit sink updated", result);
      await Promise.all([refreshAuditRemoteState(), refreshMissionControl()]);
    }, "Remote audit sink update failed");
  });

  must<HTMLButtonElement>("audit-remote-sync").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeGuarded<AuditRemoteSyncResult>(
        "audit_remote_sync",
        { profileId },
        "audit.remote.sync",
      );
      auditRemoteOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Remote audit sync completed", result);
      await Promise.all([refreshAuditRemoteState(), refreshMissionControl()]);
    }, "Remote audit sync failed"),
  );

  must<HTMLButtonElement>("audit-remote-refresh").addEventListener("click", () =>
    runSafely(refreshAuditRemoteState, "Remote audit state refresh failed"),
  );

  must<HTMLFormElement>("outcome-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: OutcomeUpsertRequest = {
        title: outcomeTitleEl.value.trim(),
        status: outcomeStatusEl.value as OutcomeStatus,
        impact_score: Number(outcomeImpactEl.value),
        owner: outcomeOwnerEl.value.trim() || undefined,
        related_receipt_id: outcomeReceiptIdEl.value.trim() || undefined,
        notes: outcomeNotesEl.value.trim() || undefined,
      };
      const outcome = await invokeGuarded<OutcomeRecord>(
        "outcomes_record",
        { profileId, request },
        "outcomes.record",
      );
      appendActivity("Outcome recorded", outcome);
      await Promise.all([refreshOutcomes(), refreshMissionControl()]);
    }, "Outcome record failed");
  });

  must<HTMLButtonElement>("outcomes-refresh").addEventListener("click", () =>
    runSafely(refreshOutcomes, "Outcomes refresh failed"),
  );

  must<HTMLFormElement>("workflow-task-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: WorkflowTaskUpsertRequest = {
        id: workflowTaskIdEl.value.trim() || undefined,
        title: workflowTaskTitleEl.value.trim(),
        description: workflowTaskDescriptionEl.value.trim() || undefined,
        status: workflowTaskStatusEl.value as WorkflowTaskStatus,
        priority: workflowTaskPriorityEl.value as WorkflowTaskPriority,
        owner: workflowTaskOwnerEl.value.trim() || undefined,
        runtime_task_id: workflowTaskRuntimeTaskIdEl.value.trim() || undefined,
        agent_id: workflowTaskAgentIdEl.value.trim() || undefined,
        skill_id: workflowTaskSkillIdEl.value.trim() || undefined,
        tool_id: workflowTaskToolIdEl.value.trim() || undefined,
        tags: parseCsv(workflowTaskTagsEl.value),
        risk_score: Number(workflowTaskRiskScoreEl.value),
        related_receipt_id: workflowTaskReceiptIdEl.value.trim() || undefined,
      };
      const result = await invokeGuarded<WorkflowTaskRecord>(
        "workflow_task_upsert",
        { profileId, request },
        "workflow.task_upsert",
      );
      appendActivity("Workflow task upserted", result);
      workflowTaskIdEl.value = result.id;
      await Promise.all([refreshWorkflowBoard(), refreshMissionControl(), refreshComplianceState()]);
    }, "Workflow task upsert failed");
  });

  must<HTMLButtonElement>("workflow-move").addEventListener("click", () =>
    runSafely(async () => {
      await moveWorkflowTask(
        workflowMoveTaskIdEl.value.trim(),
        workflowMoveStatusEl.value as WorkflowTaskStatus,
        "manual-id-move",
      );
    }, "Workflow task move failed"),
  );

  workflowViewBoardBtnEl.addEventListener("click", () => {
    setWorkflowViewMode("board");
  });

  workflowViewListBtnEl.addEventListener("click", () => {
    setWorkflowViewMode("list");
  });

  const kanbanColumns = Array.from(document.querySelectorAll<HTMLElement>(".kanban-column[data-status]"));
  for (const column of kanbanColumns) {
    column.addEventListener("dragover", (event) => {
      event.preventDefault();
      column.classList.add("is-drag-over");
    });
    column.addEventListener("dragleave", (event) => {
      const related = event.relatedTarget as Node | null;
      if (!related || !column.contains(related)) {
        column.classList.remove("is-drag-over");
      }
    });
    column.addEventListener("drop", (event) => {
      event.preventDefault();
      column.classList.remove("is-drag-over");
      const targetStatus = column.dataset.status as WorkflowTaskStatus | undefined;
      if (!targetStatus) {
        return;
      }
      const taskId =
        event.dataTransfer?.getData("text/workflow-task-id")?.trim() || draggingWorkflowTaskId || "";
      if (!taskId) {
        return;
      }
      const currentTask = currentWorkflowBoard?.tasks.find((task) => task.id === taskId);
      if (currentTask && currentTask.status === targetStatus) {
        return;
      }
      runSafely(
        async () => moveWorkflowTask(taskId, targetStatus, "drag-drop"),
        "Workflow drag/drop move failed",
      );
    });
  }

  workflowBoardViewEl.addEventListener("dragstart", (event) => {
    const target = event.target as HTMLElement | null;
    const card = target?.closest<HTMLElement>(".kanban-card");
    if (!card) {
      return;
    }
    const taskId = card.dataset.taskId?.trim();
    if (!taskId) {
      return;
    }
    draggingWorkflowTaskId = taskId;
    card.classList.add("is-dragging");
    if (event.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.setData("text/workflow-task-id", taskId);
    }
  });

  workflowBoardViewEl.addEventListener("dragend", (event) => {
    const target = event.target as HTMLElement | null;
    const card = target?.closest<HTMLElement>(".kanban-card");
    if (card) {
      card.classList.remove("is-dragging");
    }
    draggingWorkflowTaskId = null;
    for (const column of kanbanColumns) {
      column.classList.remove("is-drag-over");
    }
  });

  workflowListBodyEl.addEventListener("click", (event) => {
    const target = event.target as HTMLElement | null;
    const button = target?.closest<HTMLButtonElement>('button[data-kind="workflow-row-move"]');
    if (!button) {
      return;
    }
    runSafely(async () => {
      const taskId = button.dataset.id?.trim() || "";
      if (!taskId) {
        throw new Error("workflow list move requires task id");
      }
      const row = button.closest("tr");
      const statusSelect = row?.querySelector<HTMLSelectElement>('select[data-kind="workflow-row-status"]');
      if (!statusSelect) {
        throw new Error("workflow list move status selector not found");
      }
      await moveWorkflowTask(taskId, statusSelect.value as WorkflowTaskStatus, "list-apply");
    }, "Workflow list move failed");
  });

  must<HTMLButtonElement>("workflow-refresh").addEventListener("click", () =>
    runSafely(refreshWorkflowBoard, "Workflow board refresh failed"),
  );

  must<HTMLFormElement>("billing-config-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: BillingConfigRequest = {
        backend_url: billingBackendUrlEl.value.trim() || undefined,
        auth_secret_id: billingAuthSecretEl.value.trim() || undefined,
        enforce_verification: billingEnforceVerificationEl.checked,
      };
      const result = await invokeGuarded<BillingState>(
        "billing_config_set",
        { profileId, request },
        "billing.configure",
      );
      billingOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Billing config updated", result);
      await Promise.all([refreshBillingState(), refreshMissionControl()]);
    }, "Billing config update failed");
  });

  must<HTMLFormElement>("billing-verify-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: BillingReceiptVerifyRequest = {
        receipt_payload: billingReceiptPayloadEl.value.trim(),
        platform: billingPlatformEl.value.trim() || undefined,
      };
      const result = await invokeGuarded<BillingState>(
        "billing_verify_receipt",
        { profileId, request },
        "billing.verify",
      );
      billingOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Billing receipt verified", result);
      await Promise.all([refreshBillingState(), refreshMissionControl()]);
    }, "Billing receipt verification failed");
  });

  must<HTMLButtonElement>("billing-refresh").addEventListener("click", () =>
    runSafely(refreshBillingState, "Billing state refresh failed"),
  );

  must<HTMLButtonElement>("control-plane-refresh").addEventListener("click", () =>
    runSafely(async () => {
      await Promise.all([refreshAccessAndControlPlane(), refreshApprovals(), refreshReceipts()]);
    }, "Control plane refresh failed"),
  );

  must<HTMLButtonElement>("profile-switch").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const switched = await invokeCommand<ProfileRecord>("profiles_switch", { profileId });
      appendActivity("Switched profile", switched);
      await Promise.all([
        refreshSetupState(),
        refreshDeploymentCapabilities(),
        refreshHostConnection(),
        refreshRuntimeState(),
        refreshIntegrations(),
        refreshSkills(),
        refreshMcpConnectors(),
        refreshChannels(),
        refreshCron(),
        loadIntegrationCatalog(),
        refreshLogs(),
        refreshAccessAndControlPlane(),
        refreshApprovals(),
        refreshReceipts(),
        refreshRolloutState(),
        refreshPolicyProfile(),
        refreshRbacRegistry(),
        refreshAuditVerification(),
        refreshAuditRemoteState(),
        refreshOutcomes(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionControl(),
        refreshMissionWorkspaces(),
      ]);
    }, "Profile switch failed"),
  );

  must<HTMLFormElement>("profile-create-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const displayName = profileNameEl.value.trim();
      if (!displayName) {
        return;
      }
      const created = await invokeCommand<ProfileRecord>("profiles_create", { displayName });
      profileNameEl.value = "";
      appendActivity("Created profile", created);
      await refreshProfiles();
      profileSelectEl.value = created.id;
      await invokeCommand("profiles_switch", { profileId: created.id });
      await Promise.all([
        refreshSetupState(),
        refreshDeploymentCapabilities(),
        refreshHostConnection(),
        refreshIntegrations(),
        refreshSkills(),
        refreshMcpConnectors(),
        refreshChannels(),
        refreshCron(),
        loadIntegrationCatalog(),
        refreshRuntimeState(),
        refreshLogs(),
        refreshAccessAndControlPlane(),
        refreshApprovals(),
        refreshReceipts(),
        refreshRolloutState(),
        refreshPolicyProfile(),
        refreshRbacRegistry(),
        refreshAuditVerification(),
        refreshAuditRemoteState(),
        refreshOutcomes(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionControl(),
        refreshMissionWorkspaces(),
      ]);
    }, "Profile creation failed");
  });

  must<HTMLButtonElement>("runtime-start").addEventListener("click", () =>
    runSafely(async () => {
      await invokeGuarded("runtime_start", { profileId: ensureProfileId() }, "runtime.start");
      appendActivity("Runtime started");
      await Promise.all([refreshRuntimeState(), refreshApprovals(), refreshReceipts()]);
    }, "Runtime start failed"),
  );

  must<HTMLButtonElement>("runtime-stop").addEventListener("click", () =>
    runSafely(async () => {
      await invokeGuarded("runtime_stop", { reason: "user requested stop from UI" }, "runtime.stop");
      appendActivity("Runtime stopped");
      await Promise.all([refreshRuntimeState(), refreshApprovals(), refreshReceipts()]);
    }, "Runtime stop failed"),
  );

  must<HTMLButtonElement>("runtime-refresh").addEventListener("click", () =>
    runSafely(refreshRuntimeState, "Runtime refresh failed"),
  );

  must<HTMLButtonElement>("runtime-refresh-inline").addEventListener("click", () =>
    runSafely(refreshRuntimeState, "Runtime refresh failed"),
  );

  must<HTMLFormElement>("runtime-message-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const message = runtimeMessageEl.value.trim();
      if (!message) {
        return;
      }
      const response = await invokeGuarded<string>("runtime_send_message", { message }, "runtime.send_message");
      runtimeOutputEl.textContent = response;
      runtimeMessageEl.value = "";
      appendActivity("Runtime message sent");
      await Promise.all([refreshLogs(), refreshApprovals(), refreshReceipts()]);
    }, "Runtime message failed");
  });

  must<HTMLFormElement>("runtime-message-inline-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const message = runtimeMessageInlineEl.value.trim();
      if (!message) {
        return;
      }
      const response = await invokeGuarded<string>("runtime_send_message", { message }, "runtime.send_message");
      runtimeOutputEl.textContent = response;
      runtimeMessageInlineEl.value = "";
      appendActivity("Runtime message sent");
      await Promise.all([refreshLogs(), refreshApprovals(), refreshReceipts()]);
    }, "Runtime message failed");
  });

  must<HTMLButtonElement>("operations-status").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const report = await invokeCommand<StatusReport>("operations_status", { profileId });
      operationsOutputEl.textContent = JSON.stringify(report, null, 2);
      appendActivity("Status report loaded", report);
    }, "Status report failed"),
  );

  must<HTMLButtonElement>("operations-doctor").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeGuarded<OperationResult>("operations_doctor", { profileId }, "doctor.run");
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Doctor run complete", result);
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Doctor run failed"),
  );

  must<HTMLButtonElement>("operations-channel-doctor").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeGuarded<OperationResult>(
        "operations_channel_doctor",
        { profileId },
        "channel.doctor",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Channel doctor complete", result);
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Channel doctor failed"),
  );

  must<HTMLButtonElement>("operations-auth-profiles").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const profiles = await invokeCommand("operations_auth_profiles", { profileId });
      operationsOutputEl.textContent = JSON.stringify(profiles, null, 2);
      appendActivity("Auth profiles loaded");
    }, "Auth profile list failed"),
  );

  must<HTMLButtonElement>("operations-memory-list").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const entries = await invokeGuarded(
        "operations_memory_list",
        { profileId, limit: 100 },
        "memory.list",
      );
      operationsOutputEl.textContent = JSON.stringify(entries, null, 2);
      appendActivity("Memory entries loaded");
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Memory list failed"),
  );

  must<HTMLButtonElement>("operations-config-schema").addEventListener("click", () =>
    runSafely(async () => {
      const schema = await invokeCommand("operations_config_schema");
      operationsOutputEl.textContent = JSON.stringify(schema, null, 2);
      appendActivity("Config schema loaded");
    }, "Config schema load failed"),
  );

  must<HTMLButtonElement>("operations-command-surface").addEventListener("click", () =>
    runSafely(async () => {
      const surface = await invokeCommand("operations_command_surface");
      operationsOutputEl.textContent = JSON.stringify(surface, null, 2);
      appendActivity("Command surface loaded");
    }, "Command surface load failed"),
  );

  must<HTMLButtonElement>("operations-cost-summary").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const report = await invokeCommand<CostSummaryReport>("operations_cost_summary", { profileId });
      operationsOutputEl.textContent = JSON.stringify(report, null, 2);
      appendActivity("Cost summary loaded", report);
    }, "Cost summary load failed"),
  );

  must<HTMLButtonElement>("operations-response-cache-stats").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const stats = await invokeCommand<ResponseCacheStatsReport>("operations_response_cache_stats", { profileId });
      operationsOutputEl.textContent = JSON.stringify(stats, null, 2);
      appendActivity("Response cache stats loaded", stats);
    }, "Response cache stats load failed"),
  );

  must<HTMLFormElement>("migration-openclaw-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const sourceRaw = must<HTMLInputElement>("migration-openclaw-source").value.trim();
      const dryRun = must<HTMLInputElement>("migration-openclaw-dry-run").checked;
      const result = await invokeGuarded<OperationResult>(
        "operations_migrate_openclaw",
        {
          profileId,
          source: sourceRaw.length > 0 ? sourceRaw : null,
          dryRun,
        },
        "migrate.openclaw",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("OpenClaw migration finished", result);
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "OpenClaw migration failed");
  });

  must<HTMLFormElement>("completions-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const shell = must<HTMLSelectElement>("completion-shell").value;
      const binaryPath = completionBinaryPathEl.value.trim();
      window.localStorage.setItem(COMPLETION_BINARY_PATH_STORAGE_KEY, binaryPath);
      const completion = await invokeCommand<string>("operations_generate_shell_completions", {
        shell,
        binaryPath: binaryPath.length > 0 ? binaryPath : null,
      });
      operationsOutputEl.textContent = completion;
      appendActivity("Shell completion generated", {
        shell,
        binary_path: binaryPath || "(auto-resolve)",
      });
    }, "Shell completion generation failed");
  });

  must<HTMLButtonElement>("operations-providers").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const providers = await invokeCommand<ProviderDescriptor[]>("operations_providers", { profileId });
      applyProviderCatalogPresets(providers);
      renderProviderGuidance(setupProviderEl.value);
      renderSecretKeyGuidance(secretKeyEl.value);
      providersOutputEl.textContent = JSON.stringify(providers, null, 2);
      appendActivity("Provider catalog loaded", { count: providers.length });
    }, "Provider list failed"),
  );

  must<HTMLFormElement>("models-refresh-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const provider = modelProviderEl.value.trim() || undefined;
      const force = modelForceEl.checked;
      const result = await invokeGuarded<OperationResult>(
        "operations_models_refresh",
        { profileId, provider, force },
        "models.refresh",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Model catalog refresh complete", result);
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Model refresh failed");
  });

  must<HTMLFormElement>("service-action-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const action = serviceActionEl.value as ServiceLifecycleAction;
      const label = `service.${action}`;
      const result = await invokeGuarded<OperationResult>(
        "operations_service",
        { profileId, action },
        label,
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Service lifecycle action complete", { action, result });
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Service action failed");
  });

  must<HTMLButtonElement>("channels-refresh").addEventListener("click", () =>
    runSafely(refreshChannels, "Channel list refresh failed"),
  );

  must<HTMLFormElement>("channel-add-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const channelType = channelTypeEl.value.trim();
      const configJson = channelConfigEl.value.trim();
      if (!channelType || !configJson) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_channel_add",
        { profileId, channelType, configJson },
        "channel.add",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Channel added", result);
      await Promise.all([refreshChannels(), refreshApprovals(), refreshReceipts()]);
    }, "Channel add failed");
  });

  must<HTMLFormElement>("channel-remove-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const name = channelRemoveEl.value.trim();
      if (!name) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_channel_remove",
        { profileId, name },
        "channel.remove",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Channel removed", result);
      await Promise.all([refreshChannels(), refreshApprovals(), refreshReceipts()]);
    }, "Channel remove failed");
  });

  must<HTMLFormElement>("channel-bind-telegram-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const identity = channelTelegramIdentityEl.value.trim();
      if (!identity) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_channel_bind_telegram",
        { profileId, identity },
        "channel.bind_telegram",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Telegram identity bound", result);
      await Promise.all([refreshChannels(), refreshApprovals(), refreshReceipts()]);
    }, "Bind telegram failed");
  });

  must<HTMLButtonElement>("cron-refresh").addEventListener("click", () =>
    runSafely(refreshCron, "Cron list refresh failed"),
  );

  must<HTMLFormElement>("cron-add-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const expression = cronExpressionEl.value.trim();
      const command = cronCommandEl.value.trim();
      if (!expression || !command) return;
      const tz = cronTimezoneEl.value.trim() || undefined;
      const result = await invokeGuarded<OperationResult>(
        "operations_cron_add",
        { profileId, expression, command, tz },
        "cron.add",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Cron job added", result);
      await Promise.all([refreshCron(), refreshApprovals(), refreshReceipts()]);
    }, "Cron add failed");
  });

  must<HTMLButtonElement>("cron-remove").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const id = cronJobIdEl.value.trim();
      if (!id) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_cron_remove",
        { profileId, id },
        "cron.remove",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Cron job removed", result);
      await Promise.all([refreshCron(), refreshApprovals(), refreshReceipts()]);
    }, "Cron remove failed"),
  );

  must<HTMLButtonElement>("cron-pause").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const id = cronJobIdEl.value.trim();
      if (!id) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_cron_pause",
        { profileId, id },
        "cron.pause",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Cron job paused", result);
      await Promise.all([refreshCron(), refreshApprovals(), refreshReceipts()]);
    }, "Cron pause failed"),
  );

  must<HTMLButtonElement>("cron-resume").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const id = cronJobIdEl.value.trim();
      if (!id) return;
      const result = await invokeGuarded<OperationResult>(
        "operations_cron_resume",
        { profileId, id },
        "cron.resume",
      );
      operationsOutputEl.textContent = JSON.stringify(result, null, 2);
      appendActivity("Cron job resumed", result);
      await Promise.all([refreshCron(), refreshApprovals(), refreshReceipts()]);
    }, "Cron resume failed"),
  );

  must<HTMLFormElement>("integration-install-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const integrationId = integrationIdEl.value.trim();
      if (!integrationId) return;
      const contract: PermissionContract = {
        integration_id: integrationId,
        can_access: parseCsv(integrationAccessEl.value),
        can_do: parseCsv(integrationActionsEl.value),
        data_destinations: parseCsv(integrationDestinationsEl.value),
      };
      const record = await invokeGuarded<IntegrationRecord>("integration_install", {
        profileId,
        contract,
      }, "integration.install");
      appendActivity("Installed integration", record);
      await Promise.all([refreshIntegrations(), refreshApprovals(), refreshReceipts()]);
    }, "Integration install failed");
  });

  must<HTMLFormElement>("skills-install-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: SkillInstallRequest = {
        skill_id: skillIdEl.value.trim(),
        display_name: skillNameEl.value.trim(),
        source: skillSourceEl.value.trim(),
        version: skillVersionEl.value.trim(),
        manifest_markdown: skillManifestEl.value.trim() || undefined,
        contract: {
          integration_id: `skill:${skillIdEl.value.trim()}`,
          can_access: parseCsv(skillAccessEl.value),
          can_do: parseCsv(skillActionsEl.value),
          data_destinations: parseCsv(skillDestinationsEl.value),
        },
      };
      const record = await invokeGuarded<SkillRecord>("skills_install", { profileId, request }, "skills.install");
      appendActivity("Installed skill", record);
      await Promise.all([refreshSkills(), refreshApprovals(), refreshReceipts()]);
    }, "Skill install failed");
  });

  must<HTMLFormElement>("mcp-install-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const request: McpConnectorInstallRequest = {
        connector_id: mcpIdEl.value.trim(),
        display_name: mcpNameEl.value.trim(),
        config: {
          transport: mcpTransportEl.value.trim(),
          endpoint: mcpEndpointEl.value.trim() || undefined,
          command: mcpCommandEl.value.trim() || undefined,
          args: parseCsv(mcpArgsEl.value),
          env_secret_ids: parseCsv(mcpEnvSecretIdsEl.value),
          timeout_secs: Number.isFinite(Number(mcpTimeoutEl.value))
            ? Number(mcpTimeoutEl.value)
            : undefined,
        },
        contract: {
          integration_id: `mcp:${mcpIdEl.value.trim()}`,
          can_access: parseCsv(mcpAccessEl.value),
          can_do: parseCsv(mcpActionsEl.value),
          data_destinations: parseCsv(mcpDestinationsEl.value),
        },
      };
      const record = await invokeGuarded<McpConnectorRecord>("mcp_install", { profileId, request }, "mcp.install");
      appendActivity("Installed MCP connector", record);
      await Promise.all([refreshMcpConnectors(), refreshApprovals(), refreshReceipts()]);
    }, "MCP connector install failed");
  });

  must<HTMLFormElement>("mcp-update-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const connectorId = mcpUpdateIdEl.value.trim();
      if (!connectorId) {
        return;
      }
      const config: McpConnectorConfig = {
        transport: mcpTransportEl.value.trim(),
        endpoint: mcpEndpointEl.value.trim() || undefined,
        command: mcpCommandEl.value.trim() || undefined,
        args: parseCsv(mcpArgsEl.value),
        env_secret_ids: parseCsv(mcpEnvSecretIdsEl.value),
        timeout_secs: Number.isFinite(Number(mcpTimeoutEl.value))
          ? Number(mcpTimeoutEl.value)
          : undefined,
      };
      const updated = await invokeGuarded<McpConnectorRecord>("mcp_update_config", {
        profileId,
        connectorId,
        config,
      }, "mcp.update_config");
      appendActivity("Updated MCP connector config", updated);
      await Promise.all([refreshMcpConnectors(), refreshApprovals(), refreshReceipts()]);
    }, "MCP connector update failed");
  });

  must<HTMLFormElement>("pairing-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const bundle = await invokeCommand<PairingBundle>("pairing_create_bundle", {
        profileId: ensureProfileId(),
        transport: pairingTransportEl.value.trim(),
        endpoint: pairingEndpointEl.value.trim() || undefined,
        expiresInMinutes: Number(pairingExpiresEl.value),
      });
      pairingOutputEl.textContent = JSON.stringify(bundle, null, 2);
      appendActivity("Created pairing bundle");
    }, "Pairing bundle failed");
  });

  must<HTMLButtonElement>("approvals-refresh").addEventListener("click", () =>
    runSafely(refreshApprovals, "Approvals refresh failed"),
  );

  must<HTMLButtonElement>("receipts-refresh").addEventListener("click", () =>
    runSafely(refreshReceipts, "Receipts refresh failed"),
  );

  must<HTMLButtonElement>("receipts-export").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const path = await invokeCommand<string>("receipts_export", { profileId });
      appendActivity(`Receipts exported: ${path}`);
      await refreshReceipts();
    }, "Receipts export failed"),
  );

  must<HTMLFormElement>("approval-resolve-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const approvalId = approvalIdEl.value.trim();
      if (!approvalId) {
        return;
      }
      const approved = approvalApprovedEl.value === "true";
      const resolved = await invokeCommand<ApprovalRequest>("approvals_resolve", {
        profileId,
        approvalId,
        approverRole: approvalRoleEl.value,
        approved,
        reason: approvalReasonEl.value.trim() || undefined,
      });
      appendActivity("Approval resolved", resolved);
      approvalReasonEl.value = "";
      await Promise.all([refreshApprovals(), refreshReceipts()]);
    }, "Approval resolve failed");
  });

  must<HTMLFormElement>("retention-form").addEventListener("submit", (event) => {
    event.preventDefault();
    runSafely(async () => {
      const profileId = ensureProfileId();
      const retention = await invokeCommand<RetentionPolicy>("retention_set", {
        profileId,
        receiptsDays: Number(retentionReceiptsDaysEl.value),
        approvalsDays: Number(retentionApprovalsDaysEl.value),
      });
      appendActivity("Retention updated", retention);
      await refreshAccessAndControlPlane();
    }, "Retention update failed");
  });

  must<HTMLButtonElement>("retention-purge").addEventListener("click", () =>
    runSafely(async () => {
      const profileId = ensureProfileId();
      const result = await invokeCommand<PurgeSummary>("retention_purge", { profileId });
      appendActivity("Retention purge complete", result);
      await Promise.all([refreshAccessAndControlPlane(), refreshApprovals(), refreshReceipts()]);
    }, "Retention purge failed"),
  );

  must<HTMLButtonElement>("logs-refresh").addEventListener("click", () =>
    runSafely(refreshLogs, "Log refresh failed"),
  );

  must<HTMLButtonElement>("logs-export").addEventListener("click", () =>
    runSafely(async () => {
      const path = await invokeGuarded<string>("logs_export_diagnostics", {}, "logs.export");
      appendActivity(`Diagnostics exported: ${path}`);
      await Promise.all([refreshLogs(), refreshApprovals(), refreshReceipts()]);
    }, "Diagnostics export failed"),
  );

  missionWorkspacesListEl.addEventListener("click", (event) => {
    runSafely(async () => {
      const target = event.target as HTMLElement | null;
      if (!target || target.tagName !== "BUTTON") {
        return;
      }
      const button = target as HTMLButtonElement;
      if (button.dataset.kind !== "workspace-switch") {
        return;
      }
      const profileId = button.dataset.id?.trim();
      if (!profileId) {
        return;
      }
      await invokeCommand("profiles_switch", { profileId });
      await refreshProfiles();
      profileSelectEl.value = profileId;
      await Promise.all([
        refreshSetupState(),
        refreshDeploymentCapabilities(),
        refreshHostConnection(),
        refreshRuntimeState(),
        refreshIntegrations(),
        refreshSkills(),
        refreshMcpConnectors(),
        refreshChannels(),
        refreshCron(),
        refreshLogs(),
        refreshAccessAndControlPlane(),
        refreshApprovals(),
        refreshReceipts(),
        refreshRolloutState(),
        refreshPolicyProfile(),
        refreshRbacRegistry(),
        refreshAuditVerification(),
        refreshAuditRemoteState(),
        refreshOutcomes(),
        refreshBillingState(),
        refreshWorkflowBoard(),
        refreshComplianceState(),
        refreshMissionControl(),
        loadIntegrationCatalog(),
        refreshMissionWorkspaces(),
      ]);
      setActiveRoute("mission");
      appendActivity("Workspace switched", { profileId });
    }, "Workspace switch failed");
  });

  integrationListEl.addEventListener("click", (event) => {
    runSafely(() => handleCardAction(event as MouseEvent), "Integration action failed");
  });
  skillsListEl.addEventListener("click", (event) => {
    runSafely(() => handleCardAction(event as MouseEvent), "Skills action failed");
  });
  mcpListEl.addEventListener("click", (event) => {
    runSafely(() => handleCardAction(event as MouseEvent), "MCP action failed");
  });
}

async function runSafely(fn: () => Promise<void>, message: string) {
  try {
    await fn();
  } catch (error) {
    appendActivity(`${message}: ${String(error)}`);
  }
}

async function bindRuntimeEventListeners() {
  if (!isTauriRuntime) {
    return;
  }

  const { listen } = await import("@tauri-apps/api/event");
  await listen("runtime-event", (event) => {
    appendActivity("runtime-event", event.payload);
    void refreshRuntimeState();
  });
  await listen("runtime-event-error", (event) => {
    appendActivity("runtime-event-error", event.payload);
  });
}

type MockState = {
  profiles: ProfilesIndex;
  runtime_state: string;
  integrations: Record<string, IntegrationRegistry>;
  skills: Record<string, SkillsRegistry>;
  mcp: Record<string, McpConnectorRegistry>;
  channels: Record<string, ChannelSummary[]>;
  cron: Record<string, CronJobSummary[]>;
  setup: Record<string, ProfileSetupState>;
  secrets: Record<string, Record<string, string>>;
  logs: Record<string, LogLine[]>;
  control: Record<string, ControlPlaneState>;
  host_connection: Record<string, HostConnectionState>;
  rbac: Record<string, RbacRegistry>;
  rollout: Record<string, RolloutState>;
  audit: Record<string, AuditEvent[]>;
  audit_remote: Record<string, AuditRemoteSinkState>;
  outcomes: Record<string, OutcomeRecord[]>;
  billing: Record<string, BillingState>;
  policy_profile: Record<string, PolicyProfileState | null>;
  workflow_board: Record<string, WorkflowBoardState>;
  compliance_profile: Record<string, ComplianceProfileState | null>;
};

function createBrowserMock() {
  const STORAGE_KEY = "zeroclaw-app-browser-mock-v4";

  const defaultAccess = (): AccessState => {
    return {
      plan: "org",
      active_view: "org",
      trial_started_at: null,
      trial_expires_at: null,
      updated_at: nowIso(),
    };
  };

  const defaultControl = (): ControlPlaneState => ({
    version: 1,
    access_state: defaultAccess(),
    policy_rules: [],
    retention: {
      receipts_days: 30,
      approvals_days: 90,
    },
    receipts: [],
    approvals: [],
  });

  const defaultSetup = (): ProfileSetupState => ({
    user_display_name: "Operator",
    agent_name: "Right Hand",
    workspace_mode: "workspace",
    deployment_mode: "host",
    workspace_role: "admin",
    subscription_tier: "professional",
    orchestrator_mode: "single_orchestrator",
    provider: "openrouter",
    model: "anthropic/claude-sonnet-4",
    api_url: undefined,
    default_temperature: 0.7,
    memory_backend: "sqlite",
    runtime_reasoning_enabled: undefined,
    agent_compact_context: false,
    agent_parallel_tools: false,
    agent_max_tool_iterations: 10,
    agent_max_history_messages: 50,
    agent_tool_dispatcher: "auto",
    skills_prompt_injection_mode: "full",
    skills_open_enabled: false,
    skills_open_dir: undefined,
    enable_tool_connectors: false,
    delegate_agents: {
      marketing: {
        provider: "openrouter",
        model: "anthropic/claude-sonnet-4",
        system_prompt: "You handle campaign strategy and messaging.",
        temperature: 0.4,
        max_depth: 2,
        agentic: false,
        allowed_tools: [],
        max_iterations: 10,
      },
      coding: {
        provider: "openrouter",
        model: "anthropic/claude-sonnet-4",
        system_prompt: "You implement and review production-grade code.",
        temperature: 0.2,
        max_depth: 3,
        agentic: true,
        allowed_tools: ["file_read", "file_write", "shell"],
        max_iterations: 15,
      },
    },
    has_provider_key: false,
    provider_key_id: "provider.openrouter.api_key",
    updated_at: nowIso(),
  });

  const defaultWorkflowBoard = (): WorkflowBoardState => ({
    version: 1,
    tasks: [],
    updated_at: nowIso(),
  });

  const complianceTemplates = (): ComplianceProfileTemplate[] => [
    {
      template_id: "general_baseline",
      display_name: "General Baseline",
      description: "General 2026-ready governance baseline for most organizations.",
      industry: "general",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0"],
      recommended_policy_template: "general",
      minimum_tier: "professional",
      require_signed_release: true,
      require_remote_audit: false,
      require_billing_verification: false,
      require_pairing: true,
    },
    {
      template_id: "ai_act_nist_strict",
      display_name: "AI Act + NIST Strict",
      description: "Strict baseline for signed release and auditable AI operations.",
      industry: "cross_industry",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0", "NIST SP 800-53 Rev.5"],
      recommended_policy_template: "general",
      minimum_tier: "enterprise",
      require_signed_release: true,
      require_remote_audit: true,
      require_billing_verification: true,
      require_pairing: true,
    },
    {
      template_id: "finance_fintech",
      display_name: "Finance / Fintech",
      description: "Financial-sector controls with strict transport and provider constraints.",
      industry: "finance",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0", "ISO/IEC 27001:2022", "DORA"],
      recommended_policy_template: "finance_strict",
      minimum_tier: "enterprise",
      require_signed_release: true,
      require_remote_audit: true,
      require_billing_verification: true,
      require_pairing: true,
    },
    {
      template_id: "healthcare_pharma",
      display_name: "Healthcare / Pharma",
      description: "Healthcare controls for private transport, traceability, and signed releases.",
      industry: "healthcare",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0", "ISO/IEC 42001:2023", "HIPAA"],
      recommended_policy_template: "healthcare_strict",
      minimum_tier: "enterprise",
      require_signed_release: true,
      require_remote_audit: true,
      require_billing_verification: true,
      require_pairing: true,
    },
    {
      template_id: "tech_cloud_web3_ai",
      display_name: "Tech / Cloud / Web3 / AI",
      description: "Technology profile with strong supply-chain and governance controls.",
      industry: "tech",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0", "ISO/IEC 42001:2023", "SOC 2"],
      recommended_policy_template: "general",
      minimum_tier: "professional",
      require_signed_release: true,
      require_remote_audit: false,
      require_billing_verification: true,
      require_pairing: true,
    },
    {
      template_id: "government_us_eu",
      display_name: "Government (US/EU)",
      description: "Government posture with zero-public ingress and immutable evidence.",
      industry: "government",
      standards: ["EU AI Act", "NIST AI RMF 1.0", "NIST CSF 2.0", "NIST SP 800-53 Rev.5"],
      recommended_policy_template: "gov_zero_public",
      minimum_tier: "enterprise",
      require_signed_release: true,
      require_remote_audit: true,
      require_billing_verification: true,
      require_pairing: true,
    },
  ];

  const workflowSummary = (tasks: WorkflowTaskRecord[]): WorkflowBoardSummary => {
    const total = tasks.length;
    const pending = tasks.filter((item) => item.status === "pending").length;
    const inProgress = tasks.filter((item) => item.status === "in_progress").length;
    const done = tasks.filter((item) => item.status === "done").length;
    const failed = tasks.filter((item) => item.status === "failed").length;
    const blocked = tasks.filter((item) => item.status === "blocked").length;
    const highRiskOpen = tasks.filter(
      (item) =>
        ["pending", "in_progress", "blocked"].includes(item.status) && Number(item.risk_score) >= 70,
    ).length;
    return {
      total,
      pending,
      in_progress: inProgress,
      done,
      failed,
      blocked,
      high_risk_open: highRiskOpen,
    };
  };

  const compliancePosture = (profileId: string): CompliancePosture => {
    const compliance = state.compliance_profile[profileId];
    const workflow = state.workflow_board[profileId];
    const checks: ComplianceControlCheck[] = [
      {
        control_id: "governance.rbac_separation",
        label: "RBAC role separation",
        framework: "NIST AI RMF / EU AI Act",
        required: true,
        satisfied:
          state.rbac[profileId].users.some((item) => item.role === "admin" && item.active) &&
          state.rbac[profileId].users.some((item) => item.role === "observer" && item.active),
        evidence: `rbac_users=${state.rbac[profileId].users.length}`,
        recommendation: "Keep active admin and observer roles assigned.",
      },
      {
        control_id: "assurance.signed_rollout",
        label: "Signed rollout promotion",
        framework: "NIST CSF",
        required: Boolean(compliance?.require_signed_release),
        satisfied:
          state.rollout[profileId].signature_required &&
          state.rollout[profileId].trusted_signers.length > 0,
        evidence: `signature_required=${state.rollout[profileId].signature_required}`,
        recommendation: "Enable signature requirement and trusted signers.",
      },
      {
        control_id: "audit.remote_append_only",
        label: "Remote append-only audit",
        framework: "NIST CSF / SOC2",
        required: Boolean(compliance?.require_remote_audit),
        satisfied: Boolean(state.audit_remote[profileId].enabled && state.audit_remote[profileId].endpoint),
        evidence: `remote_enabled=${state.audit_remote[profileId].enabled}`,
        recommendation: "Configure SIEM/object-lock endpoint and keep sync active.",
      },
      {
        control_id: "operations.workflow_tracking",
        label: "Workflow tracking coverage",
        framework: "NIST AI RMF",
        required: true,
        satisfied: workflow.tasks.length > 0,
        evidence: `tasks=${workflow.tasks.length}`,
        recommendation: "Track runtime and agent tasks in the workflow board.",
      },
      {
        control_id: "operations.outcome_measurement",
        label: "Outcome measurement",
        framework: "NIST AI RMF",
        required: true,
        satisfied: state.outcomes[profileId].length > 0,
        evidence: `outcomes=${state.outcomes[profileId].length}`,
        recommendation: "Record solved/partial/unsolved outcomes for value evidence.",
      },
    ];
    const missingControls = checks.filter((item) => item.required && !item.satisfied).map((item) => item.control_id);
    return {
      template_id: compliance?.template_id || null,
      standards: compliance?.standards || [],
      compliant: missingControls.length === 0,
      generated_at: nowIso(),
      checks,
      missing_controls: missingControls,
    };
  };

  const providerCatalog = (): ProviderDescriptor[] => {
    const fallbackProfile = state.profiles.profiles[0]?.id || "default";
    const profileId = state.profiles.active_profile || fallbackProfile;
    const activeProvider = (state.setup[profileId]?.provider || "openrouter").toLowerCase();
    return DEFAULT_PROVIDER_CATALOG.map((provider) => ({
      ...provider,
      active: provider.name === activeProvider,
    }));
  };

  const load = (): MockState => {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (raw) {
      try {
        return JSON.parse(raw) as MockState;
      } catch {
        // ignore parse errors and recreate
      }
    }
    const id = "default";
    return {
      profiles: {
        version: 1,
        active_profile: id,
        profiles: [
          {
            id,
            display_name: "default",
            workspace_dir: "/mock/workspace/default",
            created_at: nowIso(),
            updated_at: nowIso(),
          },
        ],
      },
      runtime_state: "stopped",
      integrations: { [id]: { records: [] } },
      skills: { [id]: { records: [] } },
      mcp: { [id]: { records: [] } },
      channels: {
        [id]: [
          { channel_type: "cli", configured: true },
          { channel_type: "telegram", configured: false },
          { channel_type: "discord", configured: false },
          { channel_type: "slack", configured: false },
          { channel_type: "webhook", configured: false },
        ],
      },
      cron: { [id]: [] },
      setup: { [id]: defaultSetup() },
      secrets: { [id]: {} },
      logs: { [id]: [] },
      control: { [id]: defaultControl() },
      host_connection: {
        [id]: {
          connected: false,
          endpoint: null,
          transport: null,
          pairing_token_hint: null,
          connected_at: null,
          updated_at: nowIso(),
          last_error: null,
        },
      },
      rbac: {
        [id]: {
          version: 1,
          users: [
            {
              user_id: "local-admin",
              display_name: "Local Admin",
              role: "admin",
              active: true,
              created_at: nowIso(),
              updated_at: nowIso(),
            },
          ],
          updated_at: nowIso(),
        },
      },
      rollout: {
        [id]: {
          version: 1,
          current_release: null,
          previous_release: null,
          staged_release: null,
          signature_required: false,
          trusted_signers: [],
          last_verified_signer: null,
          last_promoted_at: null,
          last_verification_error: null,
          updated_at: nowIso(),
        },
      },
      audit: { [id]: [] },
      audit_remote: {
        [id]: {
          version: 1,
          enabled: false,
          endpoint: null,
          sink_kind: "siem",
          auth_secret_id: null,
          verify_tls: true,
          batch_size: 200,
          last_synced_hash: null,
          last_synced_at: null,
          last_error: null,
          updated_at: nowIso(),
        },
      },
      outcomes: { [id]: [] },
      billing: {
        [id]: {
          version: 1,
          backend_url: null,
          auth_secret_id: null,
          enforce_verification: false,
          entitlement: {
            tier: "professional",
            status: "unverified",
            verified: false,
            source: "setup",
            account_id: null,
            entitlement_id: null,
            receipt_id: null,
            expires_at: null,
            last_verified_at: null,
            last_error: null,
          },
          updated_at: nowIso(),
        },
      },
      policy_profile: { [id]: null },
      workflow_board: { [id]: defaultWorkflowBoard() },
      compliance_profile: { [id]: null },
    };
  };

  let state = load();
  state.host_connection ||= {};
  state.rbac ||= {};
  state.rollout ||= {};
  state.audit ||= {};
  state.audit_remote ||= {};
  state.outcomes ||= {};
  state.billing ||= {};
  state.policy_profile ||= {};
  state.workflow_board ||= {};
  state.compliance_profile ||= {};
  const persist = () => window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));

  const activeProfile = (): string => state.profiles.active_profile || state.profiles.profiles[0].id;

  const ensureProfileStores = (profileId: string) => {
    state.integrations[profileId] ||= { records: [] };
    state.skills[profileId] ||= { records: [] };
    state.mcp[profileId] ||= { records: [] };
    state.channels[profileId] ||= [
      { channel_type: "cli", configured: true },
      { channel_type: "telegram", configured: false },
      { channel_type: "discord", configured: false },
      { channel_type: "slack", configured: false },
      { channel_type: "webhook", configured: false },
    ];
    state.cron[profileId] ||= [];
    const existingSetup = state.setup[profileId];
    if (!existingSetup) {
      state.setup[profileId] = defaultSetup();
    } else {
      const defaults = defaultSetup();
      state.setup[profileId] = {
        ...defaults,
        ...existingSetup,
        delegate_agents: existingSetup.delegate_agents || defaults.delegate_agents,
      };
    }
    state.setup[profileId].workspace_mode = "workspace";
    if (!["host", "client"].includes(state.setup[profileId].deployment_mode)) {
      state.setup[profileId].deployment_mode = "host";
    }
    const roleMap: Record<string, WorkspaceRole> = {
      owner: "admin",
      admin: "admin",
      manager: "manager",
      operator: "user",
      user: "user",
      viewer: "observer",
      observer: "observer",
    };
    state.setup[profileId].workspace_role = roleMap[state.setup[profileId].workspace_role] || "admin";
    if (!["basic", "professional", "enterprise"].includes(state.setup[profileId].subscription_tier)) {
      state.setup[profileId].subscription_tier = "professional";
    }
    state.secrets[profileId] ||= {};
    state.logs[profileId] ||= [];
    state.control[profileId] ||= defaultControl();
    state.host_connection[profileId] ||= {
      connected: false,
      endpoint: null,
      transport: null,
      pairing_token_hint: null,
      connected_at: null,
      updated_at: nowIso(),
      last_error: null,
    };
    state.rbac[profileId] ||= {
      version: 1,
      users: [
        {
          user_id: "local-admin",
          display_name: "Local Admin",
          role: "admin",
          active: true,
          created_at: nowIso(),
          updated_at: nowIso(),
        },
      ],
      updated_at: nowIso(),
    };
    state.rollout[profileId] ||= {
      version: 1,
      current_release: null,
      previous_release: null,
      staged_release: null,
      signature_required: false,
      trusted_signers: [],
      last_verified_signer: null,
      last_promoted_at: null,
      last_verification_error: null,
      updated_at: nowIso(),
    };
    state.audit[profileId] ||= [];
    state.audit_remote[profileId] ||= {
      version: 1,
      enabled: false,
      endpoint: null,
      sink_kind: "siem",
      auth_secret_id: null,
      verify_tls: true,
      batch_size: 200,
      last_synced_hash: null,
      last_synced_at: null,
      last_error: null,
      updated_at: nowIso(),
    };
    state.outcomes[profileId] ||= [];
    state.workflow_board[profileId] ||= defaultWorkflowBoard();
    state.billing[profileId] ||= {
      version: 1,
      backend_url: null,
      auth_secret_id: null,
      enforce_verification: false,
      entitlement: {
        tier: state.setup[profileId].subscription_tier,
        status: "unverified",
        verified: false,
        source: "setup",
        account_id: null,
        entitlement_id: null,
        receipt_id: null,
        expires_at: null,
        last_verified_at: null,
        last_error: null,
      },
      updated_at: nowIso(),
    };
    if (!state.billing[profileId].entitlement.verified) {
      state.billing[profileId].entitlement.tier = state.setup[profileId].subscription_tier;
    }
    state.policy_profile[profileId] ||= null;
    state.compliance_profile[profileId] ||= null;
    state.control[profileId].access_state.plan = "org";
    state.control[profileId].access_state.active_view = "org";
    state.control[profileId].access_state.trial_started_at = null;
    state.control[profileId].access_state.trial_expires_at = null;
  };

  const ensureToolConnectorsEnabled = (profileId: string) => {
    ensureProfileStores(profileId);
    if (!state.setup[profileId].enable_tool_connectors) {
      throw new Error("tool connectors are disabled in setup; enable 'Tool Connectors (MCP)' first");
    }
  };

  const writeLog = (profileId: string, level: string, component: string, message: string) => {
    ensureProfileStores(profileId);
    state.logs[profileId].unshift({
      timestamp: nowIso(),
      level,
      component,
      message,
    });
    state.logs[profileId] = state.logs[profileId].slice(0, 200);
  };

  const pushReceipt = (
    profileId: string,
    request: ActionPolicyRequest,
    result: ReceiptResult,
    reason: string,
  ): ActionReceipt => {
    const receipt: ActionReceipt = {
      id: `receipt-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
      timestamp: nowIso(),
      actor_id: request.actor_id,
      actor_role: request.actor_role,
      action: request.action,
      resource: request.resource,
      destination: request.destination,
      result,
      reason,
      context: (request.context || {}) as Record<string, unknown>,
    };
    state.control[profileId].receipts.unshift(receipt);
    state.control[profileId].receipts = state.control[profileId].receipts.slice(0, 10000);
    return receipt;
  };

  const evaluatePolicy = (profileId: string, request: ActionPolicyRequest): ActionPolicyDecision => {
    const control = state.control[profileId];

    const governed = new Set([
      "integration.install",
      "integration.enable",
      "integration.disable",
      "skills.install",
      "skills.enable",
      "skills.disable",
      "skills.remove",
      "mcp.install",
      "mcp.enable",
      "mcp.disable",
      "mcp.update_config",
      "mcp.remove",
    ]);

    if (request.actor_role === "observer") {
      const readOnly = new Set(["logs.read", "receipts.read", "profiles.read"]);
      if (!readOnly.has(request.action)) {
        const receipt = pushReceipt(profileId, request, "denied", "observer role is read-only");
        return {
          allowed: false,
          requires_approval: false,
          reason: "observer role is read-only",
          approval_id: null,
          receipt_id: receipt.id,
        };
      }
    }

    if (["user", "manager"].includes(request.actor_role) && governed.has(request.action)) {
      if (request.approval_id) {
        const existing = control.approvals.find((item) => item.id === request.approval_id);
        if (!existing) {
          const receipt = pushReceipt(profileId, request, "denied", "approval not found");
          return {
            allowed: false,
            requires_approval: false,
            reason: "approval not found",
            approval_id: request.approval_id,
            receipt_id: receipt.id,
          };
        }
        if (existing.status === "approved") {
          const receipt = pushReceipt(profileId, request, "allowed", "approved action");
          return {
            allowed: true,
            requires_approval: false,
            reason: "approved action",
            approval_id: existing.id,
            receipt_id: receipt.id,
          };
        }
        if (existing.status === "rejected") {
          const receipt = pushReceipt(profileId, request, "denied", "approval rejected");
          return {
            allowed: false,
            requires_approval: false,
            reason: "approval rejected",
            approval_id: existing.id,
            receipt_id: receipt.id,
          };
        }
        const receipt = pushReceipt(profileId, request, "pending_approval", "approval is still pending");
        return {
          allowed: false,
          requires_approval: true,
          reason: "approval is still pending",
          approval_id: existing.id,
          receipt_id: receipt.id,
        };
      }

      const approval: ApprovalRequest = {
        id: `approval-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
        created_at: nowIso(),
        actor_id: request.actor_id,
        actor_role: request.actor_role,
        action: request.action,
        resource: request.resource,
        destination: request.destination,
        status: "pending",
        decided_by: null,
        decided_at: null,
        reason: null,
        context: (request.context || {}) as Record<string, unknown>,
      };
      control.approvals.push(approval);
      const receipt = pushReceipt(profileId, request, "pending_approval", "action requires approval");
      return {
        allowed: false,
        requires_approval: true,
        reason: "action requires approval",
        approval_id: approval.id,
        receipt_id: receipt.id,
      };
    }

    const receipt = pushReceipt(profileId, request, "allowed", "policy allowed");
    return {
      allowed: true,
      requires_approval: false,
      reason: "policy allowed",
      approval_id: null,
      receipt_id: receipt.id,
    };
  };

  const ensureAllowed = (
    profileId: string,
    args: Record<string, unknown> | undefined,
    action: string,
    resource: string,
    destination: string,
  ) => {
    const request: ActionPolicyRequest = {
      actor_id: String(args?.actorId || "local-user"),
      actor_role: String(args?.actorRole || "admin"),
      action,
      resource,
      destination,
      approval_id: args?.approvalId ? String(args.approvalId) : undefined,
      occurred_at: nowIso(),
      context: {},
    };
    const decision = evaluatePolicy(profileId, request);
    const previous = state.audit[profileId].length
      ? state.audit[profileId][state.audit[profileId].length - 1].hash
      : "genesis";
    const result = decision.allowed
      ? "allowed"
      : decision.requires_approval
        ? "pending_approval"
        : "denied";
    const auditEvent: AuditEvent = {
      id: `audit-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
      timestamp: nowIso(),
      actor_id: request.actor_id,
      actor_role: request.actor_role,
      action: request.action,
      resource: request.resource,
      destination: request.destination,
      result,
      reason: decision.reason,
      receipt_id: decision.receipt_id,
      approval_id: decision.approval_id,
      prev_hash: previous,
      hash: `${previous}:${decision.receipt_id}:${Date.now().toString(36)}`,
    };
    state.audit[profileId].push(auditEvent);
    state.audit[profileId] = state.audit[profileId].slice(-5000);
    persist();
    if (decision.requires_approval) {
      throw new Error(
        `action requires approval (approval_id: ${decision.approval_id || ""}, receipt_id: ${decision.receipt_id})`,
      );
    }
    if (!decision.allowed) {
      throw new Error(`action denied by policy: ${decision.reason} (receipt_id: ${decision.receipt_id})`);
    }
  };

  return {
    async invoke(command: string, args?: Record<string, unknown>) {
      const profileId = (args?.profileId as string | undefined) || activeProfile();
      ensureProfileStores(profileId);

      if (command === "protocol_handshake") {
        return {
          core_protocol_version: "1.0.0-web-preview",
          event_schema_version: 1,
          config_schema_version: 1,
        } as ProtocolHandshake;
      }
      if (command === "profiles_list") {
        return state.profiles;
      }
      if (command === "profiles_create") {
        const displayName = String(args?.displayName || "profile");
        const id = `${displayName.toLowerCase().replace(/[^a-z0-9_-]/g, "-")}-${Date.now().toString(36)}`;
        const record: ProfileRecord = {
          id,
          display_name: displayName,
          workspace_dir: `/mock/workspace/${id}`,
          created_at: nowIso(),
          updated_at: nowIso(),
        };
        state.profiles.profiles.push(record);
        state.profiles.active_profile = record.id;
        ensureProfileStores(record.id);
        persist();
        return record;
      }
      if (command === "profiles_switch") {
        const id = String(args?.profileId || "");
        const profile = state.profiles.profiles.find((item) => item.id === id);
        if (!profile) {
          throw new Error(`profile '${id}' not found`);
        }
        profile.updated_at = nowIso();
        state.profiles.active_profile = id;
        persist();
        return profile;
      }
      if (command === "profile_setup_get") {
        return state.setup[profileId] || defaultSetup();
      }
      if (command === "profile_setup_save") {
        ensureAllowed(profileId, args, "profile.setup", `profile:${profileId}`, "local");
        const payload = (args?.payload || {}) as ProfileSetupPayload;
        const policy = state.policy_profile[profileId];
        if (
          policy &&
          policy.allowed_providers.length > 0 &&
          !policy.allowed_providers.includes(String(payload.provider || "openrouter"))
        ) {
          throw new Error(
            `provider '${payload.provider}' is not allowed by policy profile '${policy.template_id}'`,
          );
        }
        const setup: ProfileSetupState = {
          user_display_name: payload.user_display_name || "Operator",
          agent_name: payload.agent_name || "Right Hand",
          workspace_mode: payload.workspace_mode || "workspace",
          deployment_mode: payload.deployment_mode || "host",
          workspace_role: payload.workspace_role || "admin",
          subscription_tier: payload.subscription_tier || "professional",
          orchestrator_mode: payload.orchestrator_mode || "single_orchestrator",
          provider: payload.provider || "openrouter",
          model: payload.model || "anthropic/claude-sonnet-4",
          api_url: payload.api_url || undefined,
          default_temperature:
            Number.isFinite(payload.default_temperature) && payload.default_temperature >= 0
              ? payload.default_temperature
              : 0.7,
          memory_backend: payload.memory_backend || "sqlite",
          runtime_reasoning_enabled:
            payload.runtime_reasoning_enabled === true || payload.runtime_reasoning_enabled === false
              ? payload.runtime_reasoning_enabled
              : undefined,
          agent_compact_context: Boolean(payload.agent_compact_context),
          agent_parallel_tools: Boolean(payload.agent_parallel_tools),
          agent_max_tool_iterations: Math.max(1, Number(payload.agent_max_tool_iterations || 10)),
          agent_max_history_messages: Math.max(1, Number(payload.agent_max_history_messages || 50)),
          agent_tool_dispatcher: payload.agent_tool_dispatcher || "auto",
          skills_prompt_injection_mode:
            payload.skills_prompt_injection_mode === "compact" ? "compact" : "full",
          skills_open_enabled: Boolean(payload.skills_open_enabled),
          skills_open_dir: payload.skills_open_dir || undefined,
          enable_tool_connectors: Boolean(payload.enable_tool_connectors),
          delegate_agents: payload.delegate_agents || {},
          provider_key_id: `provider.${payload.provider || "openrouter"}.api_key`,
          has_provider_key: false,
          updated_at: nowIso(),
        };
        if (payload.api_key && payload.api_key.trim().length > 0) {
          state.secrets[profileId][setup.provider_key_id] = payload.api_key;
        }
        setup.has_provider_key = Boolean(state.secrets[profileId][setup.provider_key_id]);
        state.setup[profileId] = setup;
        if (!state.billing[profileId].entitlement.verified) {
          state.billing[profileId].entitlement.tier = setup.subscription_tier;
          state.billing[profileId].entitlement.status = "unverified";
          state.billing[profileId].entitlement.source = "setup";
          state.billing[profileId].updated_at = nowIso();
        }
        persist();
        return setup;
      }
      if (command === "control_plane_state") {
        return state.control[profileId];
      }
      if (command === "access_state") {
        return state.control[profileId].access_state;
      }
      if (command === "access_start_trial") {
        const access = defaultAccess();
        state.control[profileId].access_state = access;
        persist();
        return access;
      }
      if (command === "access_set_plan") {
        const plan = "org" as AccessPlan;
        const access = state.control[profileId].access_state;
        access.plan = plan;
        access.active_view = "org";
        access.updated_at = nowIso();
        access.trial_started_at = null;
        access.trial_expires_at = null;
        persist();
        return access;
      }
      if (command === "access_set_view") {
        const view = "org" as WorkspaceView;
        const access = state.control[profileId].access_state;
        access.active_view = view;
        access.updated_at = nowIso();
        persist();
        return access;
      }
      if (command === "host_connection_get") {
        return state.host_connection[profileId];
      }
      if (command === "policy_profiles_list") {
        return [
          {
            template_id: "general",
            display_name: "General",
            description: "Balanced defaults for most organizations",
            allowed_providers: [],
            allowed_transports: ["lan", "tailscale", "cloudflare", "ngrok"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "finance_strict",
            display_name: "Finance Strict",
            description: "No public tunnels, strict provider allowlists",
            allowed_providers: ["openai", "anthropic"],
            allowed_transports: ["lan", "tailscale"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "healthcare_strict",
            display_name: "Healthcare Strict",
            description: "Private transport + explicit pairing",
            allowed_providers: ["openai", "anthropic"],
            allowed_transports: ["lan", "tailscale"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "gov_zero_public",
            display_name: "Gov Zero Public",
            description: "LAN-only, no public bind",
            allowed_providers: ["openai"],
            allowed_transports: ["lan"],
            allow_public_bind: false,
            require_pairing: true,
          },
        ] as PolicyProfileTemplate[];
      }
      if (command === "policy_profile_get") {
        return state.policy_profile[profileId];
      }
      if (command === "policy_profile_apply") {
        ensureAllowed(profileId, args, "policy.apply", `profile:${profileId}`, "workspace");
        const templates: PolicyProfileTemplate[] = [
          {
            template_id: "general",
            display_name: "General",
            description: "Balanced defaults for most organizations",
            allowed_providers: [],
            allowed_transports: ["lan", "tailscale", "cloudflare", "ngrok"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "finance_strict",
            display_name: "Finance Strict",
            description: "No public tunnels, strict provider allowlists",
            allowed_providers: ["openai", "anthropic"],
            allowed_transports: ["lan", "tailscale"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "healthcare_strict",
            display_name: "Healthcare Strict",
            description: "Private transport + explicit pairing",
            allowed_providers: ["openai", "anthropic"],
            allowed_transports: ["lan", "tailscale"],
            allow_public_bind: false,
            require_pairing: true,
          },
          {
            template_id: "gov_zero_public",
            display_name: "Gov Zero Public",
            description: "LAN-only, no public bind",
            allowed_providers: ["openai"],
            allowed_transports: ["lan"],
            allow_public_bind: false,
            require_pairing: true,
          },
        ];
        const templateId = String(args?.templateId || "general");
        const template = templates.find((item) => item.template_id === templateId);
        if (!template) {
          throw new Error(`unknown policy template '${templateId}'`);
        }
        const applied: PolicyProfileState = {
          template_id: template.template_id,
          applied_at: nowIso(),
          allowed_providers: template.allowed_providers,
          allowed_transports: template.allowed_transports,
          allow_public_bind: template.allow_public_bind,
          require_pairing: template.require_pairing,
        };
        state.policy_profile[profileId] = applied;
        persist();
        return applied;
      }
      if (command === "compliance_profiles_list") {
        return complianceTemplates();
      }
      if (command === "compliance_profile_get") {
        return state.compliance_profile[profileId];
      }
      if (command === "compliance_posture_get") {
        return compliancePosture(profileId);
      }
      if (command === "compliance_profile_apply") {
        ensureAllowed(profileId, args, "compliance.apply", `profile:${profileId}`, "workspace");
        const templateId = String(args?.templateId || "general_baseline");
        const template = complianceTemplates().find((item) => item.template_id === templateId);
        if (!template) {
          throw new Error(`unknown compliance template '${templateId}'`);
        }
        const tierRank: Record<SubscriptionTier, number> = {
          basic: 1,
          professional: 2,
          enterprise: 3,
        };
        const effectiveTier = state.billing[profileId].entitlement.verified
          ? state.billing[profileId].entitlement.tier
          : state.setup[profileId].subscription_tier;
        if (tierRank[effectiveTier] < tierRank[template.minimum_tier]) {
          throw new Error(
            `compliance template '${templateId}' requires '${template.minimum_tier}' tier (current: '${effectiveTier}')`,
          );
        }
        const profile: ComplianceProfileState = {
          template_id: template.template_id,
          applied_at: nowIso(),
          industry: template.industry,
          standards: template.standards,
          recommended_policy_template: template.recommended_policy_template,
          minimum_tier: template.minimum_tier,
          require_signed_release: template.require_signed_release,
          require_remote_audit: template.require_remote_audit,
          require_billing_verification: template.require_billing_verification,
          require_pairing: template.require_pairing,
        };
        state.compliance_profile[profileId] = profile;

        if (template.recommended_policy_template) {
          const policy: PolicyProfileTemplate[] = [
            {
              template_id: "general",
              display_name: "General",
              description: "Balanced defaults for most organizations",
              allowed_providers: [],
              allowed_transports: ["lan", "tailscale", "cloudflare", "ngrok"],
              allow_public_bind: false,
              require_pairing: true,
            },
            {
              template_id: "finance_strict",
              display_name: "Finance Strict",
              description: "No public tunnels, strict provider allowlists",
              allowed_providers: ["openai", "anthropic"],
              allowed_transports: ["lan", "tailscale"],
              allow_public_bind: false,
              require_pairing: true,
            },
            {
              template_id: "healthcare_strict",
              display_name: "Healthcare Strict",
              description: "Private transport + explicit pairing",
              allowed_providers: ["openai", "anthropic"],
              allowed_transports: ["lan", "tailscale"],
              allow_public_bind: false,
              require_pairing: true,
            },
            {
              template_id: "gov_zero_public",
              display_name: "Gov Zero Public",
              description: "LAN-only, no public bind",
              allowed_providers: ["openai"],
              allowed_transports: ["lan"],
              allow_public_bind: false,
              require_pairing: true,
            },
          ];
          const selected = policy.find((item) => item.template_id === template.recommended_policy_template);
          if (selected) {
            state.policy_profile[profileId] = {
              template_id: selected.template_id,
              applied_at: nowIso(),
              allowed_providers: selected.allowed_providers,
              allowed_transports: selected.allowed_transports,
              allow_public_bind: selected.allow_public_bind,
              require_pairing: selected.require_pairing,
            };
          }
        }

        if (template.require_signed_release) {
          state.rollout[profileId].signature_required = true;
          if (!state.rollout[profileId].trusted_signers.length) {
            state.rollout[profileId].last_verification_error =
              "compliance profile requires signed rollout; configure trusted_signers";
          }
        }
        if (template.require_billing_verification) {
          state.billing[profileId].enforce_verification = true;
          state.billing[profileId].updated_at = nowIso();
        }
        if (template.require_remote_audit && !state.audit_remote[profileId].endpoint) {
          state.audit_remote[profileId].last_error =
            "compliance profile requires remote audit sink endpoint";
          state.audit_remote[profileId].updated_at = nowIso();
        }
        persist();
        return profile;
      }
      if (command === "client_connect_host") {
        ensureAllowed(profileId, args, "host.connect", `profile:${profileId}`, "network");
        const payload = (args?.payload || {}) as HostConnectPayload;
        const parsed = JSON.parse(payload.invite_payload || "{}") as PairingBundle;
        state.host_connection[profileId] = {
          connected: true,
          endpoint: parsed.endpoint || null,
          transport: String(parsed.transport || "lan"),
          pairing_token_hint: parsed.access_token ? `${parsed.access_token.slice(0, 10)}...` : null,
          connected_at: nowIso(),
          updated_at: nowIso(),
          last_error: null,
        };
        persist();
        return state.host_connection[profileId];
      }
      if (command === "rbac_users_list") {
        return state.rbac[profileId];
      }
      if (command === "rbac_user_upsert") {
        ensureAllowed(profileId, args, "rbac.manage", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as {
          user_id: string;
          display_name: string;
          role: WorkspaceRole;
          active: boolean;
        };
        const registry = state.rbac[profileId];
        const existing = registry.users.find((item) => item.user_id === request.user_id);
        if (existing) {
          existing.display_name = request.display_name;
          existing.role = request.role;
          existing.active = Boolean(request.active);
          existing.updated_at = nowIso();
        } else {
          registry.users.push({
            user_id: request.user_id,
            display_name: request.display_name,
            role: request.role,
            active: Boolean(request.active),
            created_at: nowIso(),
            updated_at: nowIso(),
          });
        }
        registry.updated_at = nowIso();
        persist();
        return registry;
      }
      if (command === "rollout_state_get") {
        return state.rollout[profileId];
      }
      if (command === "rollout_set_signing_policy") {
        ensureAllowed(profileId, args, "release.signing_policy", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as RolloutSigningPolicyRequest;
        state.rollout[profileId].signature_required = Boolean(request.signature_required);
        state.rollout[profileId].trusted_signers = (request.trusted_signers || [])
          .map((item) => item.trim())
          .filter((item) => item.length > 0);
        state.rollout[profileId].last_verification_error = null;
        state.rollout[profileId].updated_at = nowIso();
        persist();
        return state.rollout[profileId];
      }
      if (command === "rollout_stage_release") {
        ensureAllowed(profileId, args, "release.stage", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as RolloutStageRequest;
        state.rollout[profileId].staged_release = {
          release_id: request.release_id,
          version: request.version,
          checksum_sha256: request.checksum_sha256,
          signature: request.signature || null,
          sbom_checksum_sha256: request.sbom_checksum_sha256 || null,
          ring: request.ring,
          staged_at: nowIso(),
        };
        state.rollout[profileId].updated_at = nowIso();
        persist();
        return state.rollout[profileId];
      }
      if (command === "rollout_promote") {
        ensureAllowed(profileId, args, "release.promote", `profile:${profileId}`, "workspace");
        const rollout = state.rollout[profileId];
        if (rollout.staged_release) {
          if (rollout.signature_required) {
            if (!rollout.trusted_signers.length) {
              rollout.last_verification_error = "signature_required=true but no trusted signers configured";
              persist();
              throw new Error("signature_required=true but no trusted signers configured");
            }
            const signature = rollout.staged_release.signature || "";
            if (!signature.trim()) {
              rollout.last_verification_error = "release signature is required but missing";
              persist();
              throw new Error("release signature is required but missing");
            }
            const hint = signature.includes(":") ? signature.split(":")[0].trim() : "";
            if (hint) {
              const trusted = rollout.trusted_signers.some((entry) => entry.startsWith(`${hint}:`));
              if (!trusted) {
                rollout.last_verification_error = `signature key hint '${hint}' is not trusted`;
                persist();
                throw new Error(`signature key hint '${hint}' is not trusted`);
              }
              rollout.last_verified_signer = hint;
            } else {
              const firstSigner = rollout.trusted_signers[0] || "signer-1";
              rollout.last_verified_signer = firstSigner.includes(":")
                ? firstSigner.split(":")[0]
                : firstSigner;
            }
            rollout.last_verification_error = null;
          }
          rollout.previous_release = rollout.current_release;
          rollout.current_release = rollout.staged_release;
          rollout.staged_release = null;
        } else if (rollout.current_release) {
          const ringOrder: RolloutRing[] = ["pilot", "group", "all"];
          const currentIdx = ringOrder.indexOf(rollout.current_release.ring);
          rollout.current_release.ring = ringOrder[Math.min(currentIdx + 1, ringOrder.length - 1)];
        } else {
          throw new Error("no staged or current release available to promote");
        }
        rollout.last_promoted_at = nowIso();
        rollout.updated_at = nowIso();
        persist();
        return rollout;
      }
      if (command === "rollout_rollback") {
        ensureAllowed(profileId, args, "release.rollback", `profile:${profileId}`, "workspace");
        const rollout = state.rollout[profileId];
        if (!rollout.previous_release) {
          throw new Error("no previous release found for rollback");
        }
        rollout.staged_release = rollout.current_release;
        rollout.current_release = rollout.previous_release;
        rollout.updated_at = nowIso();
        persist();
        return rollout;
      }
      if (command === "audit_log_list") {
        const limit = Number(args?.limit || 300);
        return state.audit[profileId].slice(-limit);
      }
      if (command === "audit_log_verify") {
        const events = state.audit[profileId];
        let prev = "genesis";
        for (const event of events) {
          if (event.prev_hash !== prev) {
            return {
              valid: false,
              entries: events.length,
              last_hash: prev,
              error: `chain mismatch at ${event.id}`,
            } as AuditLogVerification;
          }
          prev = event.hash;
        }
        return {
          valid: true,
          entries: events.length,
          last_hash: events.length ? events[events.length - 1].hash : null,
          error: null,
        } as AuditLogVerification;
      }
      if (command === "audit_log_export") {
        return `/mock/audit-log-${Date.now()}.json`;
      }
      if (command === "audit_remote_get") {
        return state.audit_remote[profileId];
      }
      if (command === "audit_remote_configure") {
        ensureAllowed(profileId, args, "audit.remote.configure", `profile:${profileId}`, "network");
        const request = (args?.request || {}) as AuditRemoteConfigureRequest;
        const remote = state.audit_remote[profileId];
        remote.enabled = Boolean(request.enabled);
        remote.endpoint = request.endpoint?.trim() || null;
        remote.sink_kind = request.sink_kind || "siem";
        remote.auth_secret_id = request.auth_secret_id?.trim() || null;
        remote.verify_tls = request.verify_tls ?? true;
        remote.batch_size = Math.max(1, Math.min(5000, Number(request.batch_size || 200)));
        remote.updated_at = nowIso();
        persist();
        return remote;
      }
      if (command === "audit_remote_sync") {
        ensureAllowed(profileId, args, "audit.remote.sync", `profile:${profileId}`, "network");
        const remote = state.audit_remote[profileId];
        if (!remote.enabled) {
          throw new Error("remote audit sink is disabled");
        }
        if (!remote.endpoint) {
          throw new Error("remote audit sink endpoint is missing");
        }
        const events = state.audit[profileId];
        const startIndex = remote.last_synced_hash
          ? Math.max(0, events.findIndex((item) => item.hash === remote.last_synced_hash) + 1)
          : 0;
        const limit = Math.max(1, Number(args?.limit || remote.batch_size || 200));
        const pending = events.slice(startIndex, startIndex + limit);
        const syncedAt = nowIso();
        if (pending.length > 0) {
          remote.last_synced_hash = pending[pending.length - 1].hash;
          remote.last_synced_at = syncedAt;
          remote.last_error = null;
          remote.updated_at = syncedAt;
          persist();
        }
        return {
          endpoint: remote.endpoint,
          sink_kind: remote.sink_kind,
          events_sent: pending.length,
          first_hash: pending.length ? pending[0].hash : null,
          last_hash: pending.length ? pending[pending.length - 1].hash : remote.last_synced_hash,
          synced_at: syncedAt,
        } as AuditRemoteSyncResult;
      }
      if (command === "billing_state_get") {
        return state.billing[profileId];
      }
      if (command === "billing_config_set") {
        ensureAllowed(profileId, args, "billing.configure", `profile:${profileId}`, "network");
        const request = (args?.request || {}) as BillingConfigRequest;
        const billing = state.billing[profileId];
        billing.backend_url = request.backend_url?.trim() || null;
        billing.auth_secret_id = request.auth_secret_id?.trim() || null;
        billing.enforce_verification = Boolean(request.enforce_verification);
        billing.updated_at = nowIso();
        persist();
        return billing;
      }
      if (command === "billing_verify_receipt") {
        ensureAllowed(profileId, args, "billing.verify", `profile:${profileId}`, "network");
        const request = (args?.request || {}) as BillingReceiptVerifyRequest;
        if (!request.receipt_payload?.trim()) {
          throw new Error("receipt_payload is required");
        }
        const billing = state.billing[profileId];
        let parsed: Record<string, unknown> = {};
        try {
          parsed = JSON.parse(request.receipt_payload);
        } catch {
          parsed = {};
        }
        const valid = parsed.valid !== false;
        const tier =
          parsed.tier === "basic" || parsed.tier === "professional" || parsed.tier === "enterprise"
            ? (parsed.tier as SubscriptionTier)
            : state.setup[profileId].subscription_tier;
        billing.entitlement.tier = tier;
        billing.entitlement.status =
          (parsed.status as BillingEntitlementStatus | undefined) || (valid ? "active" : "unverified");
        billing.entitlement.verified = valid;
        billing.entitlement.source = "backend";
        billing.entitlement.account_id = typeof parsed.account_id === "string" ? parsed.account_id : null;
        billing.entitlement.entitlement_id =
          typeof parsed.entitlement_id === "string" ? parsed.entitlement_id : null;
        billing.entitlement.receipt_id = typeof parsed.receipt_id === "string" ? parsed.receipt_id : null;
        billing.entitlement.expires_at = typeof parsed.expires_at === "string" ? parsed.expires_at : null;
        billing.entitlement.last_verified_at = nowIso();
        billing.entitlement.last_error = valid ? null : String(parsed.reason || "receipt invalid");
        billing.updated_at = nowIso();
        persist();
        return billing;
      }
      if (command === "workflow_board_get") {
        const limit = Number(args?.limit || 400);
        const tasks = state.workflow_board[profileId].tasks.slice(0, limit);
        return {
          summary: workflowSummary(tasks),
          tasks,
        } as WorkflowBoardView;
      }
      if (command === "workflow_task_upsert") {
        ensureAllowed(profileId, args, "workflow.task_upsert", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as WorkflowTaskUpsertRequest;
        if (!request.title?.trim()) {
          throw new Error("workflow task title is required");
        }
        const board = state.workflow_board[profileId];
        const now = nowIso();
        if (request.id) {
          const existing = board.tasks.find((item) => item.id === request.id);
          if (!existing) {
            throw new Error(`workflow task '${request.id}' not found`);
          }
          existing.title = request.title.trim();
          existing.description = request.description?.trim() || null;
          if (request.status) {
            existing.status = request.status;
            if (request.status === "in_progress" && !existing.started_at) {
              existing.started_at = now;
            }
            if (request.status === "done" || request.status === "failed") {
              existing.completed_at = now;
            } else {
              existing.completed_at = null;
            }
          }
          if (request.priority) {
            existing.priority = request.priority;
          }
          existing.owner = request.owner?.trim() || null;
          existing.runtime_task_id = request.runtime_task_id?.trim() || null;
          existing.agent_id = request.agent_id?.trim() || null;
          existing.skill_id = request.skill_id?.trim() || null;
          existing.tool_id = request.tool_id?.trim() || null;
          existing.tags = (request.tags || []).map((item) => item.trim()).filter((item) => item.length > 0);
          existing.risk_score = Math.max(0, Math.min(100, Number(request.risk_score ?? existing.risk_score)));
          existing.related_receipt_id = request.related_receipt_id?.trim() || null;
          existing.updated_at = now;
          board.updated_at = now;
          persist();
          return existing;
        }
        const status: WorkflowTaskStatus = request.status || "pending";
        const record: WorkflowTaskRecord = {
          id: `task-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
          title: request.title.trim(),
          description: request.description?.trim() || null,
          status,
          priority: request.priority || "medium",
          owner: request.owner?.trim() || null,
          workspace_scope: profileId,
          runtime_task_id: request.runtime_task_id?.trim() || null,
          agent_id: request.agent_id?.trim() || null,
          skill_id: request.skill_id?.trim() || null,
          tool_id: request.tool_id?.trim() || null,
          tags: (request.tags || []).map((item) => item.trim()).filter((item) => item.length > 0),
          risk_score: Math.max(0, Math.min(100, Number(request.risk_score ?? 50))),
          related_receipt_id: request.related_receipt_id?.trim() || null,
          created_at: now,
          updated_at: now,
          started_at: status === "in_progress" ? now : null,
          completed_at: status === "done" || status === "failed" ? now : null,
        };
        board.tasks.unshift(record);
        board.tasks = board.tasks.slice(0, 4000);
        board.updated_at = now;
        persist();
        return record;
      }
      if (command === "workflow_task_move") {
        ensureAllowed(profileId, args, "workflow.task_move", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as WorkflowTaskMoveRequest;
        const task = state.workflow_board[profileId].tasks.find((item) => item.id === request.task_id);
        if (!task) {
          throw new Error(`workflow task '${request.task_id}' not found`);
        }
        const now = nowIso();
        task.status = request.status;
        task.updated_at = now;
        if (request.status === "in_progress" && !task.started_at) {
          task.started_at = now;
        }
        if (request.status === "done" || request.status === "failed") {
          task.completed_at = now;
        } else {
          task.completed_at = null;
        }
        state.workflow_board[profileId].updated_at = now;
        persist();
        return task;
      }
      if (command === "outcomes_record") {
        ensureAllowed(profileId, args, "outcomes.record", `profile:${profileId}`, "workspace");
        const request = (args?.request || {}) as OutcomeUpsertRequest;
        const record: OutcomeRecord = {
          id: `outcome-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`,
          timestamp: nowIso(),
          title: request.title,
          status: request.status,
          impact_score: Math.max(0, Math.min(100, Number(request.impact_score || 0))),
          owner: request.owner || null,
          related_receipt_id: request.related_receipt_id || null,
          notes: request.notes || null,
        };
        state.outcomes[profileId].unshift(record);
        state.outcomes[profileId] = state.outcomes[profileId].slice(0, 5000);
        persist();
        return record;
      }
      if (command === "outcomes_list") {
        const limit = Number(args?.limit || 200);
        return state.outcomes[profileId].slice(0, limit);
      }
      if (command === "outcomes_summary") {
        const outcomes = state.outcomes[profileId];
        const total = outcomes.length;
        const solved = outcomes.filter((item) => item.status === "solved").length;
        const partial = outcomes.filter((item) => item.status === "partial").length;
        const unsolved = outcomes.filter((item) => item.status === "unsolved").length;
        const solvedRate = total ? solved / total : 0;
        const avgImpactScore = total
          ? outcomes.reduce((acc, item) => acc + item.impact_score, 0) / total
          : 0;
        return {
          total,
          solved,
          partial,
          unsolved,
          solved_rate: solvedRate,
          avg_impact_score: avgImpactScore,
        } as OutcomeSummary;
      }
      if (command === "mission_control_summary") {
        const outcomes = state.outcomes[profileId];
        const total = outcomes.length;
        const solved = outcomes.filter((item) => item.status === "solved").length;
        const partial = outcomes.filter((item) => item.status === "partial").length;
        const unsolved = outcomes.filter((item) => item.status === "unsolved").length;
        const solvedRate = total ? solved / total : 0;
        const avgImpactScore = total
          ? outcomes.reduce((acc, item) => acc + item.impact_score, 0) / total
          : 0;
        const auditEvents = state.audit[profileId];
        return {
          deployment: {
            platform: "web-preview",
            supports_host: true,
            supports_client: true,
            configured_mode: state.setup[profileId].deployment_mode || "host",
            effective_mode: state.setup[profileId].deployment_mode || "host",
            workspace_mode: "workspace",
            workspace_role: state.setup[profileId].workspace_role || "admin",
            subscription_tier: state.setup[profileId].subscription_tier || "professional",
            note: "browser preview mission control mock",
          },
          rollout: state.rollout[profileId],
          rbac_users: state.rbac[profileId].users.length,
          audit: {
            valid: true,
            entries: auditEvents.length,
            last_hash: auditEvents.length ? auditEvents[auditEvents.length - 1].hash : null,
            error: null,
          },
          audit_remote: state.audit_remote[profileId],
          billing: state.billing[profileId],
          workflow: workflowSummary(state.workflow_board[profileId].tasks),
          compliance: compliancePosture(profileId),
          outcomes: {
            total,
            solved,
            partial,
            unsolved,
            solved_rate: solvedRate,
            avg_impact_score: avgImpactScore,
          },
          approvals_pending: state.control[profileId].approvals.filter((item) => item.status === "pending").length,
          receipts_total: state.control[profileId].receipts.length,
        } as MissionControlSummary;
      }
      if (command === "evidence_export") {
        return {
          output_dir: `/mock/evidence-${Date.now()}`,
          files: [
            "audit-log.json",
            "audit-verify.json",
            "rollout-state.json",
            "rbac-users.json",
            "outcomes.json",
            "audit-remote-state.json",
            "billing-state.json",
            "workflow-board.json",
            "compliance-profile.json",
            "compliance-posture.json",
            "mission-summary.json",
            "version-manifest.json",
            "sbom-manifest.json",
            "incident-playbook.md",
          ],
        } as EvidenceExportSummary;
      }
      if (command === "policy_evaluate") {
        const request = args?.request as ActionPolicyRequest;
        const decision = evaluatePolicy(profileId, request);
        persist();
        return decision;
      }
      if (command === "approvals_list") {
        const pendingOnly = Boolean(args?.pendingOnly);
        if (!pendingOnly) {
          return state.control[profileId].approvals;
        }
        return state.control[profileId].approvals.filter((item) => item.status === "pending");
      }
      if (command === "approvals_resolve") {
        const approvalId = String(args?.approvalId || "");
        const approverRole = String(args?.approverRole || "");
        if (!["admin", "manager"].includes(approverRole)) {
          throw new Error("only admin/manager can resolve approvals");
        }
        const approval = state.control[profileId].approvals.find((item) => item.id === approvalId);
        if (!approval) {
          throw new Error(`approval '${approvalId}' not found`);
        }
        const approved = Boolean(args?.approved);
        approval.status = approved ? "approved" : "rejected";
        approval.decided_by = approverRole;
        approval.decided_at = nowIso();
        approval.reason = (args?.reason as string | undefined) || null;
        persist();
        return approval;
      }
      if (command === "receipts_list") {
        const limit = Number(args?.limit || 200);
        return state.control[profileId].receipts.slice(0, limit);
      }
      if (command === "retention_set") {
        state.control[profileId].retention = {
          receipts_days: Math.max(1, Number(args?.receiptsDays || 30)),
          approvals_days: Math.max(1, Number(args?.approvalsDays || 90)),
        };
        persist();
        return state.control[profileId].retention;
      }
      if (command === "retention_purge") {
        const retention = state.control[profileId].retention;
        const now = Date.now();
        const receiptsBefore = state.control[profileId].receipts.length;
        const approvalsBefore = state.control[profileId].approvals.length;
        state.control[profileId].receipts = state.control[profileId].receipts.filter((item) => {
          return now - new Date(item.timestamp).getTime() <= retention.receipts_days * 24 * 60 * 60 * 1000;
        });
        state.control[profileId].approvals = state.control[profileId].approvals.filter((item) => {
          return now - new Date(item.created_at).getTime() <= retention.approvals_days * 24 * 60 * 60 * 1000;
        });
        persist();
        return {
          removed_receipts: receiptsBefore - state.control[profileId].receipts.length,
          removed_approvals: approvalsBefore - state.control[profileId].approvals.length,
        } as PurgeSummary;
      }
      if (command === "receipts_export") {
        return `/mock/receipts-${Date.now()}.json`;
      }
      if (command === "runtime_start") {
        ensureAllowed(profileId, args, "runtime.start", `profile:${profileId}`, "local");
        const setup = state.setup[profileId];
        if (setup.deployment_mode === "client") {
          throw new Error("runtime_start is disabled for deployment_mode=client");
        }
        state.runtime_state = "running";
        writeLog(profileId, "info", "runtime", "runtime started (mock)");
        persist();
        return;
      }
      if (command === "runtime_stop") {
        ensureAllowed(profileId, args, "runtime.stop", `profile:${profileId}`, "local");
        state.runtime_state = "stopped";
        writeLog(profileId, "info", "runtime", "runtime stopped (mock)");
        persist();
        return;
      }
      if (command === "runtime_state") {
        return state.runtime_state;
      }
      if (command === "secret_set") {
        const key = String(args?.key || "");
        state.secrets[profileId][key] = String(args?.value || "");
        persist();
        return;
      }
      if (command === "secret_get") {
        const key = String(args?.key || "");
        return state.secrets[profileId][key] ?? null;
      }
      if (command === "secret_exists") {
        const key = String(args?.key || "");
        return Boolean(state.secrets[profileId][key]);
      }
      if (command === "secret_delete") {
        const key = String(args?.key || "");
        delete state.secrets[profileId][key];
        persist();
        return;
      }
      if (command === "secret_backend") {
        return "browser-local-storage";
      }
      if (command === "operations_status") {
        return {
          config_path: `/mock/workspace/${profileId}/config.toml`,
          workspace_dir: `/mock/workspace/${profileId}`,
          default_provider: state.setup[profileId].provider,
          default_model: state.setup[profileId].model,
          temperature: 0.7,
          gateway_host: "127.0.0.1",
          gateway_port: 3000,
          channels: state.channels[profileId].reduce<Record<string, boolean>>((acc, item) => {
            acc[item.channel_type] = item.configured;
            return acc;
          }, {}),
          peripherals_enabled: false,
          peripheral_boards: 0,
        } as StatusReport;
      }
      if (command === "operations_auth_profiles") {
        const setup = state.setup[profileId];
        return [
          {
            id: `${setup.provider}:default`,
            provider: setup.provider,
            profile_name: "default",
            kind: "token",
            active: true,
            account_id: null,
            workspace_id: null,
            expires_at: null,
            updated_at: nowIso(),
          },
        ];
      }
      if (command === "operations_integrations_catalog") {
        return [
          {
            name: "Telegram",
            description: "Bot API long polling channel",
            category: "Chat Providers",
            status: state.channels[profileId].some((item) => item.channel_type === "telegram" && item.configured)
              ? "active"
              : "available",
            setup_hint: "Create token in BotFather, then add telegram channel config.",
          },
          {
            name: "Discord",
            description: "Servers, channels, and DMs",
            category: "Chat Providers",
            status: state.channels[profileId].some((item) => item.channel_type === "discord" && item.configured)
              ? "active"
              : "available",
            setup_hint: "Create bot token + message intent, then add discord channel config.",
          },
          {
            name: "Slack",
            description: "Workspace app integration",
            category: "Chat Providers",
            status: state.channels[profileId].some((item) => item.channel_type === "slack" && item.configured)
              ? "active"
              : "available",
            setup_hint: "Create Slack app token/signing secret, then add slack channel config.",
          },
          {
            name: "Webhooks",
            description: "HTTP trigger endpoint",
            category: "Platforms",
            status: "available",
            setup_hint: "Configure webhook secret and gateway route policy.",
          },
        ] as IntegrationCatalogEntry[];
      }
      if (command === "operations_memory_list") {
        ensureAllowed(profileId, args, "memory.list", `profile:${profileId}`, "workspace");
        return state.logs[profileId].slice(0, Number(args?.limit || 100)).map((line, index) => ({
          id: `memory-${index + 1}`,
          key: `${line.component}-${index + 1}`,
          category: "conversation",
          timestamp: line.timestamp,
          session_id: null,
          score: null,
          content_preview: line.message,
        }));
      }
      if (command === "operations_migrate_openclaw") {
        ensureAllowed(profileId, args, "migrate.openclaw", `profile:${profileId}`, "workspace");
        return {
          operation: "migrate_openclaw",
          ok: true,
          detail: args?.dryRun ? "OpenClaw migration dry-run completed (mock)" : "OpenClaw migration completed (mock)",
        } as OperationResult;
      }
      if (command === "operations_generate_shell_completions") {
        const shell = String(args?.shell || "bash");
        return `# mock completion for ${shell}\n# generated by wrapper preview`;
      }
      if (command === "operations_config_schema") {
        return {
          title: "ZeroClaw Config Schema",
          type: "object",
          note: "mock preview",
        };
      }
      if (command === "operations_command_surface") {
        return [
          { family: "auth", supported: true, coverage: "core + ui", note: "profile visibility exposed" },
          { family: "config", supported: true, coverage: "core + ui", note: "schema export exposed" },
          { family: "memory", supported: true, coverage: "core + ui", note: "memory listing exposed" },
          { family: "migrate", supported: true, coverage: "core + ui", note: "OpenClaw migration exposed in operations panel" },
          { family: "integration_catalog", supported: true, coverage: "core + ui", note: "integration catalog with setup hints exposed" },
          { family: "tool_connectors", supported: true, coverage: "wrapper + ui", note: "MCP connectors with explicit setup opt-in" },
          { family: "rollout", supported: true, coverage: "wrapper + ui", note: "ring rollout controls exposed" },
          { family: "rbac", supported: true, coverage: "wrapper + ui", note: "central role assignments exposed" },
          { family: "audit", supported: true, coverage: "wrapper + ui", note: "tamper-evident log verification exposed" },
          { family: "audit_remote", supported: true, coverage: "wrapper + ui", note: "remote append-only audit sync exposed" },
          { family: "outcomes", supported: true, coverage: "wrapper + ui", note: "success and impact tracking exposed" },
          { family: "workflow", supported: true, coverage: "wrapper + ui", note: "kanban-style workspace task tracking exposed" },
          { family: "compliance", supported: true, coverage: "wrapper + ui", note: "AI Act/NIST/industry posture controls exposed" },
          { family: "billing", supported: true, coverage: "wrapper + ui", note: "tier entitlement verification exposed" },
          { family: "evidence", supported: true, coverage: "wrapper + ui", note: "one-click evidence pack export exposed" },
          { family: "completions", supported: true, coverage: "wrapper + ui", note: "shell completion generation exposed in operations panel" },
        ];
      }
      if (command === "deployment_capabilities") {
        const setup = state.setup[profileId];
        const configured = setup.deployment_mode || "host";
        const capabilities: DeploymentCapabilities = {
          platform: "web-preview",
          supports_host: true,
          supports_client: true,
          configured_mode: configured,
          effective_mode: configured,
          workspace_mode: setup.workspace_mode || "workspace",
          workspace_role: setup.workspace_role || "admin",
          subscription_tier: setup.subscription_tier || "professional",
          note: "browser preview mock supports both deployment modes for UX validation",
        };
        return capabilities;
      }
      if (command === "operations_cost_summary") {
        const setup = state.setup[profileId];
        const requestCount = state.logs[profileId].filter((line) => line.component === "agent").length;
        const totalTokens = requestCount * 512;
        const totalCostUsd = Number((totalTokens * 0.000003).toFixed(6));
        return {
          enabled: true,
          total_cost_usd: totalCostUsd,
          daily_cost_usd: totalCostUsd,
          monthly_cost_usd: totalCostUsd,
          total_tokens: totalTokens,
          request_count: requestCount,
          by_model: [
            {
              model: setup.model,
              request_count: requestCount,
              total_tokens: totalTokens,
              total_cost_usd: totalCostUsd,
            },
          ],
        } as CostSummaryReport;
      }
      if (command === "operations_response_cache_stats") {
        const entries = Math.min(200, state.logs[profileId].length);
        return {
          enabled: true,
          ttl_minutes: 60,
          max_entries: 1000,
          entries,
          hits: Math.floor(entries / 3),
          tokens_saved: entries * 128,
        } as ResponseCacheStatsReport;
      }
      if (command === "operations_doctor") {
        ensureAllowed(profileId, args, "doctor.run", `profile:${profileId}`, "local");
        writeLog(profileId, "info", "doctor", "doctor run complete (mock)");
        persist();
        return {
          operation: "doctor",
          ok: true,
          detail: "doctor completed (mock)",
        } as OperationResult;
      }
      if (command === "operations_channel_doctor") {
        ensureAllowed(profileId, args, "channel.doctor", `profile:${profileId}`, "local");
        writeLog(profileId, "info", "channels", "channel doctor complete (mock)");
        persist();
        return {
          operation: "channel_doctor",
          ok: true,
          detail: "channel doctor completed (mock)",
        } as OperationResult;
      }
      if (command === "operations_channels_list") {
        return state.channels[profileId];
      }
      if (command === "operations_channel_add") {
        const channelType = String(args?.channelType || "");
        ensureAllowed(profileId, args, "channel.add", `channel:${channelType}`, "integration");
        const existing = state.channels[profileId].find((item) => item.channel_type === channelType);
        if (existing) {
          existing.configured = true;
        } else {
          state.channels[profileId].push({ channel_type: channelType, configured: true });
        }
        persist();
        return {
          operation: "channel_add",
          ok: true,
          detail: `channel '${channelType}' configured (mock)`,
        } as OperationResult;
      }
      if (command === "operations_channel_remove") {
        const name = String(args?.name || "");
        ensureAllowed(profileId, args, "channel.remove", `channel:${name}`, "integration");
        state.channels[profileId] = state.channels[profileId].map((item) =>
          item.channel_type === name ? { ...item, configured: false } : item,
        );
        persist();
        return {
          operation: "channel_remove",
          ok: true,
          detail: `channel '${name}' removed (mock)`,
        } as OperationResult;
      }
      if (command === "operations_channel_bind_telegram") {
        const identity = String(args?.identity || "");
        ensureAllowed(
          profileId,
          args,
          "channel.bind_telegram",
          `channel:telegram:${identity}`,
          "integration",
        );
        persist();
        return {
          operation: "channel_bind_telegram",
          ok: true,
          detail: `telegram identity '${identity}' bound (mock)`,
        } as OperationResult;
      }
      if (command === "operations_cron_list") {
        return state.cron[profileId];
      }
      if (command === "operations_cron_add") {
        ensureAllowed(profileId, args, "cron.add", `profile:${profileId}`, "workspace");
        const expression = String(args?.expression || "");
        const commandValue = String(args?.command || "");
        const entry: CronJobSummary = {
          id: `cron-${Date.now().toString(36)}`,
          schedule: `cron:${expression}`,
          command: commandValue,
          enabled: true,
          next_run: new Date(Date.now() + 60_000).toISOString(),
          last_run: null,
          last_status: null,
        };
        state.cron[profileId].push(entry);
        persist();
        return {
          operation: "cron_add",
          ok: true,
          detail: `cron job added for '${expression}' (mock)`,
        } as OperationResult;
      }
      if (command === "operations_cron_remove") {
        ensureAllowed(profileId, args, "cron.remove", `cron:${String(args?.id || "")}`, "workspace");
        const id = String(args?.id || "");
        state.cron[profileId] = state.cron[profileId].filter((item) => item.id !== id);
        persist();
        return {
          operation: "cron_remove",
          ok: true,
          detail: `cron job '${id}' removed (mock)`,
        } as OperationResult;
      }
      if (command === "operations_cron_pause") {
        ensureAllowed(profileId, args, "cron.pause", `cron:${String(args?.id || "")}`, "workspace");
        const id = String(args?.id || "");
        state.cron[profileId] = state.cron[profileId].map((item) =>
          item.id === id ? { ...item, enabled: false } : item,
        );
        persist();
        return {
          operation: "cron_pause",
          ok: true,
          detail: `cron job '${id}' paused (mock)`,
        } as OperationResult;
      }
      if (command === "operations_cron_resume") {
        ensureAllowed(profileId, args, "cron.resume", `cron:${String(args?.id || "")}`, "workspace");
        const id = String(args?.id || "");
        state.cron[profileId] = state.cron[profileId].map((item) =>
          item.id === id ? { ...item, enabled: true } : item,
        );
        persist();
        return {
          operation: "cron_resume",
          ok: true,
          detail: `cron job '${id}' resumed (mock)`,
        } as OperationResult;
      }
      if (command === "operations_providers") {
        return providerCatalog();
      }
      if (command === "operations_models_refresh") {
        const provider = String(args?.provider || "default");
        ensureAllowed(profileId, args, "models.refresh", `provider:${provider}`, "provider");
        writeLog(profileId, "info", "models", `model catalog refreshed for ${provider} (mock)`);
        persist();
        return {
          operation: "models_refresh",
          ok: true,
          detail: `model catalog refreshed for provider '${provider}' (mock)`,
        } as OperationResult;
      }
      if (command === "operations_service") {
        const action = String(args?.action || "status") as ServiceLifecycleAction;
        const serviceActionMap: Record<ServiceLifecycleAction, string> = {
          install: "service.install",
          start: "service.start",
          stop: "service.stop",
          status: "service.status",
          uninstall: "service.uninstall",
        };
        const policyAction = serviceActionMap[action];
        if (!policyAction) {
          throw new Error(`unsupported service action '${action}'`);
        }
        if (action !== "status") {
          ensureAllowed(profileId, args, policyAction, `profile:${profileId}`, "local");
        }
        writeLog(profileId, "info", "service", `service action '${action}' complete (mock)`);
        persist();
        return {
          operation: "service_lifecycle",
          ok: true,
          detail: `service action '${action}' completed (mock)`,
        } as OperationResult;
      }
      if (command === "runtime_send_message") {
        ensureAllowed(profileId, args, "runtime.send_message", `profile:${profileId}`, "provider");
        const message = String(args?.message || "");
        writeLog(profileId, "info", "agent", `message processed: ${message}`);
        persist();
        return `Mock response for: ${message}`;
      }
      if (command === "logs_tail") {
        return state.logs[profileId] || [];
      }
      if (command === "logs_export_diagnostics") {
        ensureAllowed(profileId, args, "logs.export", `profile:${profileId}`, "workspace");
        return `/mock/diagnostics-${Date.now()}.jsonl`;
      }
      if (command === "integration_list") {
        return state.integrations[profileId];
      }
      if (command === "integration_install") {
        const contract = args?.contract as PermissionContract;
        ensureAllowed(
          profileId,
          args,
          "integration.install",
          `integration:${contract.integration_id}`,
          contract.data_destinations[0] || "integration",
        );
        const existing = state.integrations[profileId].records.find(
          (item) => item.integration_id === contract.integration_id,
        );
        if (existing) {
          existing.contract = contract;
          persist();
          return existing;
        }
        const record: IntegrationRecord = {
          integration_id: contract.integration_id,
          installed_at: nowIso(),
          enabled: false,
          enabled_at: null,
          contract,
        };
        state.integrations[profileId].records.push(record);
        persist();
        return record;
      }
      if (command === "integration_enable") {
        const integrationId = String(args?.integrationId || "");
        ensureAllowed(profileId, args, "integration.enable", `integration:${integrationId}`, "integration");
        const record = state.integrations[profileId].records.find((item) => item.integration_id === integrationId);
        if (!record) throw new Error(`integration '${integrationId}' not found`);
        record.enabled = true;
        record.enabled_at = nowIso();
        persist();
        return record;
      }
      if (command === "integration_disable") {
        const integrationId = String(args?.integrationId || "");
        ensureAllowed(profileId, args, "integration.disable", `integration:${integrationId}`, "integration");
        const record = state.integrations[profileId].records.find((item) => item.integration_id === integrationId);
        if (!record) throw new Error(`integration '${integrationId}' not found`);
        record.enabled = false;
        persist();
        return record;
      }
      if (command === "integration_remove") {
        const integrationId = String(args?.integrationId || "");
        ensureAllowed(profileId, args, "integration.remove", `integration:${integrationId}`, "integration");
        state.integrations[profileId].records = state.integrations[profileId].records.filter(
          (item) => item.integration_id !== integrationId,
        );
        persist();
        return;
      }
      if (command === "skills_list") {
        return state.skills[profileId];
      }
      if (command === "skills_install") {
        const request = args?.request as SkillInstallRequest;
        ensureAllowed(
          profileId,
          args,
          "skills.install",
          `skill:${request.skill_id}`,
          request.contract.data_destinations[0] || "integration",
        );
        const existing = state.skills[profileId].records.find((item) => item.skill_id === request.skill_id);
        if (existing) {
          existing.display_name = request.display_name;
          existing.source = request.source;
          existing.version = request.version;
          existing.contract = request.contract;
          persist();
          return existing;
        }
        const record: SkillRecord = {
          skill_id: request.skill_id,
          display_name: request.display_name,
          source: request.source,
          version: request.version,
          installed_at: nowIso(),
          enabled: false,
          enabled_at: null,
          skill_dir: `/mock/skills/${request.skill_id}`,
          contract: request.contract,
        };
        state.skills[profileId].records.push(record);
        persist();
        return record;
      }
      if (command === "skills_enable") {
        const skillId = String(args?.skillId || "");
        ensureAllowed(profileId, args, "skills.enable", `skill:${skillId}`, "integration");
        const record = state.skills[profileId].records.find((item) => item.skill_id === skillId);
        if (!record) throw new Error(`skill '${skillId}' not found`);
        record.enabled = true;
        record.enabled_at = nowIso();
        persist();
        return record;
      }
      if (command === "skills_disable") {
        const skillId = String(args?.skillId || "");
        ensureAllowed(profileId, args, "skills.disable", `skill:${skillId}`, "integration");
        const record = state.skills[profileId].records.find((item) => item.skill_id === skillId);
        if (!record) throw new Error(`skill '${skillId}' not found`);
        record.enabled = false;
        persist();
        return record;
      }
      if (command === "skills_remove") {
        const skillId = String(args?.skillId || "");
        ensureAllowed(profileId, args, "skills.remove", `skill:${skillId}`, "integration");
        state.skills[profileId].records = state.skills[profileId].records.filter((item) => item.skill_id !== skillId);
        persist();
        return;
      }
      if (command === "mcp_list") {
        return state.mcp[profileId];
      }
      if (command === "mcp_install") {
        const request = args?.request as McpConnectorInstallRequest;
        ensureToolConnectorsEnabled(profileId);
        ensureAllowed(
          profileId,
          args,
          "mcp.install",
          `mcp:${request.connector_id}`,
          request.contract.data_destinations[0] || "integration",
        );
        const existing = state.mcp[profileId].records.find((item) => item.connector_id === request.connector_id);
        if (existing) {
          existing.display_name = request.display_name;
          existing.config = request.config;
          existing.contract = request.contract;
          existing.updated_at = nowIso();
          persist();
          return existing;
        }
        const record: McpConnectorRecord = {
          connector_id: request.connector_id,
          display_name: request.display_name,
          installed_at: nowIso(),
          updated_at: nowIso(),
          enabled: false,
          enabled_at: null,
          config: request.config,
          contract: request.contract,
        };
        state.mcp[profileId].records.push(record);
        persist();
        return record;
      }
      if (command === "mcp_update_config") {
        const connectorId = String(args?.connectorId || "");
        ensureToolConnectorsEnabled(profileId);
        ensureAllowed(profileId, args, "mcp.update_config", `mcp:${connectorId}`, "integration");
        const config = args?.config as McpConnectorConfig;
        const record = state.mcp[profileId].records.find((item) => item.connector_id === connectorId);
        if (!record) throw new Error(`mcp connector '${connectorId}' not found`);
        record.config = config;
        record.updated_at = nowIso();
        persist();
        return record;
      }
      if (command === "mcp_enable") {
        const connectorId = String(args?.connectorId || "");
        ensureToolConnectorsEnabled(profileId);
        ensureAllowed(profileId, args, "mcp.enable", `mcp:${connectorId}`, "integration");
        const record = state.mcp[profileId].records.find((item) => item.connector_id === connectorId);
        if (!record) throw new Error(`mcp connector '${connectorId}' not found`);
        record.enabled = true;
        record.enabled_at = nowIso();
        record.updated_at = nowIso();
        persist();
        return record;
      }
      if (command === "mcp_disable") {
        const connectorId = String(args?.connectorId || "");
        ensureToolConnectorsEnabled(profileId);
        ensureAllowed(profileId, args, "mcp.disable", `mcp:${connectorId}`, "integration");
        const record = state.mcp[profileId].records.find((item) => item.connector_id === connectorId);
        if (!record) throw new Error(`mcp connector '${connectorId}' not found`);
        record.enabled = false;
        record.updated_at = nowIso();
        persist();
        return record;
      }
      if (command === "mcp_remove") {
        const connectorId = String(args?.connectorId || "");
        ensureToolConnectorsEnabled(profileId);
        ensureAllowed(profileId, args, "mcp.remove", `mcp:${connectorId}`, "integration");
        state.mcp[profileId].records = state.mcp[profileId].records.filter(
          (item) => item.connector_id !== connectorId,
        );
        persist();
        return;
      }
      if (command === "pairing_create_bundle") {
        const policy = state.policy_profile[profileId];
        const requestedTransport = String(args?.transport || "lan");
        if (
          policy &&
          policy.allowed_transports.length > 0 &&
          !policy.allowed_transports.includes(requestedTransport)
        ) {
          throw new Error(
            `transport '${requestedTransport}' is blocked by policy profile '${policy.template_id}'`,
          );
        }
        const token = `mock-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
        const pairingId = `pairing-${Date.now().toString(36)}`;
        return {
          pairing_id: pairingId,
          hub_device: "mock-host",
          qr_payload: JSON.stringify({
            pairing_id: pairingId,
            endpoint: String(args?.endpoint || "http://127.0.0.1:8080"),
            transport: requestedTransport,
            access_token: token,
          }),
          endpoint: String(args?.endpoint || "http://127.0.0.1:8080"),
          transport: requestedTransport,
          access_token: token,
          created_at: nowIso(),
          expires_at: new Date(Date.now() + 15 * 60 * 1000).toISOString(),
          snapshot_sync_mode: "placeholder_encrypted_snapshot",
          notes: "web preview pairing bundle mock",
        } as PairingBundle;
      }
      if (command === "background_capabilities") {
        return {
          supports_always_on: false,
          requires_ongoing_notification: false,
          best_effort_only: true,
        };
      }
      if (command === "background_enable") {
        ensureAllowed(profileId, args, "background.enable", `profile:${profileId}`, "local");
        persist();
        return;
      }
      if (command === "background_disable") {
        ensureAllowed(profileId, args, "background.disable", `profile:${profileId}`, "local");
        persist();
        return;
      }

      throw new Error(`Unsupported command in web preview: ${command}`);
    },
  };
}

async function main() {
  const storedTheme = window.localStorage.getItem(THEME_STORAGE_KEY);
  applyTheme(storedTheme === "dark" ? "dark" : "light");
  const storedWorkflowView = window.localStorage.getItem(WORKFLOW_VIEW_MODE_STORAGE_KEY);
  setWorkflowViewMode(storedWorkflowView === "list" ? "list" : "board");
  onboardingCompletionByProfile = loadOnboardingCompletionState();
  completionBinaryPathEl.value = window.localStorage.getItem(COMPLETION_BINARY_PATH_STORAGE_KEY) || "";
  setupEditorExpanded = false;
  applyProviderCatalogPresets(DEFAULT_PROVIDER_CATALOG);
  renderProviderGuidance(setupProviderEl.value);
  syncSecretKeyEditingMode();
  bindUiHandlers();
  setActiveRoute("profile");
  await bindRuntimeEventListeners();
  await refreshAll();
  refreshProfileIdentity();
  refreshSetupGateUi();
  const readyForMission = !isOnboardingLocked();
  setActiveRoute(readyForMission ? "mission" : "profile");
}

void main().catch((error) => {
  appendActivity(`bootstrap failed: ${String(error)}`);
});
