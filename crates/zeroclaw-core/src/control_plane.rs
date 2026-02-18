use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONTROL_PLANE_FILE: &str = "control_plane.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceView {
    Personal,
    Org,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccessPlan {
    Trial,
    Personal,
    Org,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessState {
    pub plan: AccessPlan,
    pub active_view: WorkspaceView,
    pub trial_started_at: Option<String>,
    pub trial_expires_at: Option<String>,
    pub updated_at: String,
}

impl Default for AccessState {
    fn default() -> Self {
        Self {
            plan: AccessPlan::Trial,
            active_view: WorkspaceView::Personal,
            trial_started_at: None,
            trial_expires_at: None,
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl AccessState {
    pub fn start_trial(&mut self) {
        let now = Utc::now();
        self.plan = AccessPlan::Trial;
        self.active_view = WorkspaceView::Personal;
        self.trial_started_at = Some(now.to_rfc3339());
        self.trial_expires_at = Some((now + Duration::hours(24)).to_rfc3339());
        self.updated_at = now.to_rfc3339();
    }

    pub fn set_paid_plan(&mut self, plan: AccessPlan) -> Result<()> {
        match plan {
            AccessPlan::Trial => {
                self.start_trial();
            }
            AccessPlan::Personal => {
                self.plan = AccessPlan::Personal;
                self.active_view = WorkspaceView::Personal;
                self.updated_at = Utc::now().to_rfc3339();
            }
            AccessPlan::Org => {
                self.plan = AccessPlan::Org;
                self.active_view = WorkspaceView::Org;
                self.updated_at = Utc::now().to_rfc3339();
            }
        }
        Ok(())
    }

    pub fn set_active_view(&mut self, view: WorkspaceView) -> Result<()> {
        if !self.can_access_view(&view) {
            anyhow::bail!("current plan does not allow '{}' view", view.as_str());
        }
        self.active_view = view;
        self.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn can_access_view(&self, view: &WorkspaceView) -> bool {
        match self.plan {
            AccessPlan::Trial => self.is_trial_active(),
            AccessPlan::Personal => matches!(view, WorkspaceView::Personal),
            AccessPlan::Org => matches!(view, WorkspaceView::Org),
        }
    }

    pub fn is_trial_active(&self) -> bool {
        matches!(self.plan, AccessPlan::Trial)
            && self
                .trial_expires_at
                .as_deref()
                .and_then(parse_rfc3339)
                .is_some_and(|expires| expires > Utc::now())
    }
}

impl WorkspaceView {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkspaceView::Personal => "personal",
            WorkspaceView::Org => "org",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionPolicy {
    pub receipts_days: u32,
    pub approvals_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            receipts_days: 30,
            approvals_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyRule {
    pub id: String,
    pub actor_roles: Vec<String>,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
    pub destinations: Vec<String>,
    pub require_approval: bool,
    pub enabled: bool,
}

impl PolicyRule {
    fn matches(&self, request: &ActionPolicyRequest) -> bool {
        self.enabled
            && matches_filter(&self.actor_roles, &request.actor_role)
            && matches_filter(&self.actions, &request.action)
            && matches_filter(&self.resources, &request.resource)
            && matches_filter(&self.destinations, &request.destination)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionPolicyRequest {
    pub actor_id: String,
    pub actor_role: String,
    pub action: String,
    pub resource: String,
    pub destination: String,
    #[serde(default)]
    pub approval_id: Option<String>,
    #[serde(default)]
    pub occurred_at: Option<String>,
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionPolicyDecision {
    pub allowed: bool,
    pub requires_approval: bool,
    pub reason: String,
    pub approval_id: Option<String>,
    pub receipt_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptResult {
    Allowed,
    Denied,
    PendingApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionReceipt {
    pub id: String,
    pub timestamp: String,
    pub actor_id: String,
    pub actor_role: String,
    pub action: String,
    pub resource: String,
    pub destination: String,
    pub result: ReceiptResult,
    pub reason: String,
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApprovalRequest {
    pub id: String,
    pub created_at: String,
    pub actor_id: String,
    pub actor_role: String,
    pub action: String,
    pub resource: String,
    pub destination: String,
    pub status: ApprovalStatus,
    pub decided_by: Option<String>,
    pub decided_at: Option<String>,
    pub reason: Option<String>,
    #[serde(default)]
    pub context: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PurgeSummary {
    pub removed_receipts: usize,
    pub removed_approvals: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlPlaneState {
    pub version: u32,
    pub access_state: AccessState,
    pub policy_rules: Vec<PolicyRule>,
    pub retention: RetentionPolicy,
    pub receipts: Vec<ActionReceipt>,
    pub approvals: Vec<ApprovalRequest>,
}

impl Default for ControlPlaneState {
    fn default() -> Self {
        Self {
            version: 1,
            access_state: AccessState::default(),
            policy_rules: default_policy_rules(),
            retention: RetentionPolicy::default(),
            receipts: Vec::new(),
            approvals: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ControlPlaneStore {
    path: PathBuf,
}

impl ControlPlaneStore {
    pub fn for_workspace(workspace_dir: &Path) -> Self {
        Self {
            path: workspace_dir.join(CONTROL_PLANE_FILE),
        }
    }

    pub fn load(&self) -> Result<ControlPlaneState> {
        if !self.path.exists() {
            let mut state = ControlPlaneState::default();
            state.access_state.start_trial();
            self.save(&state)?;
            return Ok(state);
        }

        let body = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        let mut state: ControlPlaneState =
            serde_json::from_str(&body).context("failed to parse control plane state")?;
        self.normalize(&mut state);
        Ok(state)
    }

    pub fn save(&self, state: &ControlPlaneState) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let body = serde_json::to_string_pretty(state)
            .context("failed to serialize control plane state")?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, body).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("failed to replace {}", self.path.display()))?;
        Ok(())
    }

    pub fn get_state(&self) -> Result<ControlPlaneState> {
        self.load()
    }

    pub fn start_trial(&self) -> Result<AccessState> {
        let mut state = self.load()?;
        state.access_state.start_trial();
        self.save(&state)?;
        Ok(state.access_state)
    }

    pub fn set_paid_plan(&self, plan: AccessPlan) -> Result<AccessState> {
        let mut state = self.load()?;
        state.access_state.set_paid_plan(plan)?;
        self.save(&state)?;
        Ok(state.access_state)
    }

    pub fn set_active_view(&self, view: WorkspaceView) -> Result<AccessState> {
        let mut state = self.load()?;
        state.access_state.set_active_view(view)?;
        self.save(&state)?;
        Ok(state.access_state)
    }

    pub fn evaluate_action(&self, request: ActionPolicyRequest) -> Result<ActionPolicyDecision> {
        let mut state = self.load()?;
        let now = request
            .occurred_at
            .as_deref()
            .and_then(parse_rfc3339)
            .unwrap_or_else(Utc::now);

        let decision = if !state
            .access_state
            .can_access_view(&state.access_state.active_view)
        {
            let receipt = push_receipt(
                &mut state,
                &request,
                ReceiptResult::Denied,
                "access plan does not permit the current workspace view",
            );
            ActionPolicyDecision {
                allowed: false,
                requires_approval: false,
                reason: "access plan does not permit the current workspace view".into(),
                approval_id: None,
                receipt_id: receipt,
            }
        } else if let Some(rule) = state
            .policy_rules
            .iter()
            .find(|rule| rule.matches(&request))
        {
            if rule.require_approval {
                if let Some(existing_approval_id) = request.approval_id.as_deref() {
                    if let Some(approval) = state
                        .approvals
                        .iter()
                        .find(|approval| approval.id == existing_approval_id)
                    {
                        let matches_request = approval.actor_id == request.actor_id
                            && approval.actor_role == request.actor_role
                            && approval.action == request.action
                            && approval.resource == request.resource
                            && approval.destination == request.destination;

                        if !matches_request {
                            let receipt = push_receipt(
                                &mut state,
                                &request,
                                ReceiptResult::Denied,
                                "approval does not match action request",
                            );
                            self.save(&state)?;
                            return Ok(ActionPolicyDecision {
                                allowed: false,
                                requires_approval: false,
                                reason: "approval does not match action request".into(),
                                approval_id: Some(existing_approval_id.to_string()),
                                receipt_id: receipt,
                            });
                        }

                        match approval.status {
                            ApprovalStatus::Approved => {
                                let receipt = push_receipt(
                                    &mut state,
                                    &request,
                                    ReceiptResult::Allowed,
                                    "approved action",
                                );
                                ActionPolicyDecision {
                                    allowed: true,
                                    requires_approval: false,
                                    reason: "approved action".into(),
                                    approval_id: Some(existing_approval_id.to_string()),
                                    receipt_id: receipt,
                                }
                            }
                            ApprovalStatus::Rejected => {
                                let receipt = push_receipt(
                                    &mut state,
                                    &request,
                                    ReceiptResult::Denied,
                                    "approval rejected",
                                );
                                ActionPolicyDecision {
                                    allowed: false,
                                    requires_approval: false,
                                    reason: "approval rejected".into(),
                                    approval_id: Some(existing_approval_id.to_string()),
                                    receipt_id: receipt,
                                }
                            }
                            ApprovalStatus::Pending => {
                                let receipt = push_receipt(
                                    &mut state,
                                    &request,
                                    ReceiptResult::PendingApproval,
                                    "approval is still pending",
                                );
                                ActionPolicyDecision {
                                    allowed: false,
                                    requires_approval: true,
                                    reason: "approval is still pending".into(),
                                    approval_id: Some(existing_approval_id.to_string()),
                                    receipt_id: receipt,
                                }
                            }
                        }
                    } else {
                        let receipt = push_receipt(
                            &mut state,
                            &request,
                            ReceiptResult::Denied,
                            "approval not found",
                        );
                        ActionPolicyDecision {
                            allowed: false,
                            requires_approval: false,
                            reason: "approval not found".into(),
                            approval_id: Some(existing_approval_id.to_string()),
                            receipt_id: receipt,
                        }
                    }
                } else {
                    let approval_id = uuid::Uuid::new_v4().to_string();
                    state.approvals.push(ApprovalRequest {
                        id: approval_id.clone(),
                        created_at: now.to_rfc3339(),
                        actor_id: request.actor_id.clone(),
                        actor_role: request.actor_role.clone(),
                        action: request.action.clone(),
                        resource: request.resource.clone(),
                        destination: request.destination.clone(),
                        status: ApprovalStatus::Pending,
                        decided_by: None,
                        decided_at: None,
                        reason: None,
                        context: request.context.clone(),
                    });
                    let receipt = push_receipt(
                        &mut state,
                        &request,
                        ReceiptResult::PendingApproval,
                        "action requires approval",
                    );
                    ActionPolicyDecision {
                        allowed: false,
                        requires_approval: true,
                        reason: "action requires approval".into(),
                        approval_id: Some(approval_id),
                        receipt_id: receipt,
                    }
                }
            } else {
                let receipt = push_receipt(
                    &mut state,
                    &request,
                    ReceiptResult::Allowed,
                    "policy allowed",
                );
                ActionPolicyDecision {
                    allowed: true,
                    requires_approval: false,
                    reason: "policy allowed".into(),
                    approval_id: None,
                    receipt_id: receipt,
                }
            }
        } else {
            let receipt = push_receipt(
                &mut state,
                &request,
                ReceiptResult::Denied,
                "no matching policy rule",
            );
            ActionPolicyDecision {
                allowed: false,
                requires_approval: false,
                reason: "no matching policy rule".into(),
                approval_id: None,
                receipt_id: receipt,
            }
        };

        self.save(&state)?;
        Ok(decision)
    }

    pub fn list_receipts(&self, limit: usize) -> Result<Vec<ActionReceipt>> {
        let state = self.load()?;
        Ok(state
            .receipts
            .into_iter()
            .take(limit.clamp(1, 1000))
            .collect())
    }

    pub fn list_approvals(&self, pending_only: bool) -> Result<Vec<ApprovalRequest>> {
        let state = self.load()?;
        if pending_only {
            return Ok(state
                .approvals
                .into_iter()
                .filter(|request| matches!(request.status, ApprovalStatus::Pending))
                .collect());
        }
        Ok(state.approvals)
    }

    pub fn resolve_approval(
        &self,
        approval_id: &str,
        approver_role: &str,
        approved: bool,
        reason: Option<String>,
    ) -> Result<ApprovalRequest> {
        if !matches!(approver_role, "owner" | "admin") {
            anyhow::bail!("only owner/admin can resolve approvals");
        }

        let mut state = self.load()?;
        let Some(approval) = state
            .approvals
            .iter_mut()
            .find(|request| request.id == approval_id)
        else {
            anyhow::bail!("approval '{}' not found", approval_id);
        };

        approval.status = if approved {
            ApprovalStatus::Approved
        } else {
            ApprovalStatus::Rejected
        };
        approval.decided_by = Some(approver_role.to_string());
        approval.decided_at = Some(Utc::now().to_rfc3339());
        approval.reason = reason;

        let out = approval.clone();
        self.save(&state)?;
        Ok(out)
    }

    pub fn set_retention(
        &self,
        receipts_days: u32,
        approvals_days: u32,
    ) -> Result<RetentionPolicy> {
        let mut state = self.load()?;
        state.retention = RetentionPolicy {
            receipts_days: receipts_days.max(1),
            approvals_days: approvals_days.max(1),
        };
        let out = state.retention.clone();
        self.save(&state)?;
        Ok(out)
    }

    pub fn purge_by_retention(&self) -> Result<PurgeSummary> {
        let mut state = self.load()?;
        let now = Utc::now();

        let receipts_cutoff = now - Duration::days(i64::from(state.retention.receipts_days));
        let approvals_cutoff = now - Duration::days(i64::from(state.retention.approvals_days));

        let receipts_before = state.receipts.len();
        state.receipts.retain(|receipt| {
            parse_rfc3339(&receipt.timestamp).is_none_or(|created| created >= receipts_cutoff)
        });

        let approvals_before = state.approvals.len();
        state.approvals.retain(|request| {
            parse_rfc3339(&request.created_at).is_none_or(|created| created >= approvals_cutoff)
        });

        let out = PurgeSummary {
            removed_receipts: receipts_before.saturating_sub(state.receipts.len()),
            removed_approvals: approvals_before.saturating_sub(state.approvals.len()),
        };
        self.save(&state)?;
        Ok(out)
    }

    pub fn export_receipts(&self, output_path: &Path) -> Result<PathBuf> {
        let state = self.load()?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let payload = serde_json::to_string_pretty(&state.receipts)
            .context("failed to serialize control-plane receipts")?;
        fs::write(output_path, payload)
            .with_context(|| format!("failed to write {}", output_path.display()))?;
        Ok(output_path.to_path_buf())
    }

    fn normalize(&self, state: &mut ControlPlaneState) {
        if state.policy_rules.is_empty() {
            state.policy_rules = default_policy_rules();
        }
        if state.access_state.trial_started_at.is_none()
            && matches!(state.access_state.plan, AccessPlan::Trial)
        {
            state.access_state.start_trial();
        }
    }
}

fn push_receipt(
    state: &mut ControlPlaneState,
    request: &ActionPolicyRequest,
    result: ReceiptResult,
    reason: &str,
) -> String {
    let receipt_id = uuid::Uuid::new_v4().to_string();
    state.receipts.insert(
        0,
        ActionReceipt {
            id: receipt_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            actor_id: request.actor_id.clone(),
            actor_role: request.actor_role.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
            destination: request.destination.clone(),
            result,
            reason: reason.to_string(),
            context: request.context.clone(),
        },
    );
    if state.receipts.len() > 10_000 {
        state.receipts.truncate(10_000);
    }
    receipt_id
}

fn matches_filter(filters: &[String], value: &str) -> bool {
    filters.is_empty()
        || filters
            .iter()
            .any(|filter| filter == "*" || filter == value)
}

fn parse_rfc3339(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn default_policy_rules() -> Vec<PolicyRule> {
    vec![
        PolicyRule {
            id: "owner-full-access".into(),
            actor_roles: vec!["owner".into()],
            actions: vec!["*".into()],
            resources: vec!["*".into()],
            destinations: vec!["*".into()],
            require_approval: false,
            enabled: true,
        },
        PolicyRule {
            id: "admin-full-access".into(),
            actor_roles: vec!["admin".into()],
            actions: vec!["*".into()],
            resources: vec!["*".into()],
            destinations: vec!["*".into()],
            require_approval: false,
            enabled: true,
        },
        PolicyRule {
            id: "operator-runtime".into(),
            actor_roles: vec!["operator".into()],
            actions: vec![
                "runtime.start".into(),
                "runtime.stop".into(),
                "runtime.send_message".into(),
                "background.enable".into(),
                "background.disable".into(),
                "logs.read".into(),
                "logs.export".into(),
                "receipts.read".into(),
            ],
            resources: vec!["*".into()],
            destinations: vec!["local".into(), "provider".into(), "workspace".into()],
            require_approval: false,
            enabled: true,
        },
        PolicyRule {
            id: "operator-governed-changes".into(),
            actor_roles: vec!["operator".into()],
            actions: vec![
                "integration.install".into(),
                "integration.enable".into(),
                "integration.disable".into(),
                "skills.install".into(),
                "skills.enable".into(),
                "skills.disable".into(),
                "skills.remove".into(),
                "mcp.install".into(),
                "mcp.enable".into(),
                "mcp.disable".into(),
                "mcp.update_config".into(),
                "mcp.remove".into(),
            ],
            resources: vec!["*".into()],
            destinations: vec!["*".into()],
            require_approval: true,
            enabled: true,
        },
        PolicyRule {
            id: "viewer-readonly".into(),
            actor_roles: vec!["viewer".into()],
            actions: vec![
                "logs.read".into(),
                "receipts.read".into(),
                "profiles.read".into(),
            ],
            resources: vec!["*".into()],
            destinations: vec!["local".into(), "workspace".into()],
            require_approval: false,
            enabled: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn trial_allows_personal_and_org_views() {
        let tmp = TempDir::new().unwrap();
        let store = ControlPlaneStore::for_workspace(tmp.path());
        let mut access = store.start_trial().unwrap();

        assert!(access.can_access_view(&WorkspaceView::Personal));
        assert!(access.can_access_view(&WorkspaceView::Org));

        access.set_paid_plan(AccessPlan::Personal).unwrap();
        assert!(access.can_access_view(&WorkspaceView::Personal));
        assert!(!access.can_access_view(&WorkspaceView::Org));
    }

    #[test]
    fn operator_enable_actions_require_approval() {
        let tmp = TempDir::new().unwrap();
        let store = ControlPlaneStore::for_workspace(tmp.path());
        let _ = store.start_trial().unwrap();

        let decision = store
            .evaluate_action(ActionPolicyRequest {
                actor_id: "operator-a".into(),
                actor_role: "operator".into(),
                action: "integration.enable".into(),
                resource: "integration:slack".into(),
                destination: "api.slack.com".into(),
                approval_id: None,
                occurred_at: None,
                context: BTreeMap::new(),
            })
            .unwrap();

        assert!(!decision.allowed);
        assert!(decision.requires_approval);
        assert!(decision.approval_id.is_some());
    }

    #[test]
    fn approved_action_replay_is_allowed() {
        let tmp = TempDir::new().unwrap();
        let store = ControlPlaneStore::for_workspace(tmp.path());
        let _ = store.start_trial().unwrap();

        let initial = store
            .evaluate_action(ActionPolicyRequest {
                actor_id: "operator-a".into(),
                actor_role: "operator".into(),
                action: "integration.enable".into(),
                resource: "integration:slack".into(),
                destination: "api.slack.com".into(),
                approval_id: None,
                occurred_at: None,
                context: BTreeMap::new(),
            })
            .unwrap();

        let approval_id = initial.approval_id.clone().unwrap();
        let _ = store
            .resolve_approval(&approval_id, "admin", true, Some("approved".into()))
            .unwrap();

        let replay = store
            .evaluate_action(ActionPolicyRequest {
                actor_id: "operator-a".into(),
                actor_role: "operator".into(),
                action: "integration.enable".into(),
                resource: "integration:slack".into(),
                destination: "api.slack.com".into(),
                approval_id: Some(approval_id),
                occurred_at: None,
                context: BTreeMap::new(),
            })
            .unwrap();

        assert!(replay.allowed);
        assert!(!replay.requires_approval);
    }
}
