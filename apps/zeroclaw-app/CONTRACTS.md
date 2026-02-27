# Right Hand Tauri Shell Contracts

This contract maps app-shell commands to `zeroclaw-core` interfaces.

## Protocol and Compatibility
- `core_protocol_version`: semantic version string from core.
- `event_schema_version`: integer event schema version.
- `config_schema_version`: integer config compatibility version.

Wrapper policy:
- Shell validates versions at startup.
- Shell refuses unsupported versions with a clear upgrade message.

## Command Surface
- `protocol_handshake() -> ProtocolHandshake`
- `profiles_list() -> ProfilesIndex`
- `profiles_create(display_name) -> ProfileRecord`
- `profiles_switch(profile_id) -> ProfileRecord`
- `profile_setup_get(profile_id) -> ProfileSetupState`
- `profile_setup_save(profile_id, payload, actor_id?, actor_role?, approval_id?) -> ProfileSetupState`
- `deployment_capabilities(profile_id?) -> DeploymentCapabilities`
- `policy_profiles_list() -> Vec<PolicyProfileTemplate>`
- `policy_profile_get(profile_id) -> Option<PolicyProfileState>`
- `policy_profile_apply(profile_id, template_id, actor_id?, actor_role?, approval_id?) -> PolicyProfileState`
- `compliance_profiles_list() -> Vec<ComplianceProfileTemplate>`
- `compliance_profile_get(profile_id) -> Option<ComplianceProfileState>`
- `compliance_profile_apply(profile_id, template_id, actor_id?, actor_role?, approval_id?) -> ComplianceProfileState`
- `compliance_posture_get(profile_id) -> CompliancePosture`
- `host_connection_get(profile_id) -> HostConnectionState`
- `client_connect_host(profile_id, payload, actor_id?, actor_role?, approval_id?) -> HostConnectionState`
- `rbac_users_list(profile_id) -> RbacRegistry`
- `rbac_user_upsert(profile_id, request, actor_id?, actor_role?, approval_id?) -> RbacRegistry`
- `rollout_state_get(profile_id) -> RolloutState`
- `rollout_stage_release(profile_id, request, actor_id?, actor_role?, approval_id?) -> RolloutState`
- `rollout_set_signing_policy(profile_id, request, actor_id?, actor_role?, approval_id?) -> RolloutState`
- `rollout_promote(profile_id, actor_id?, actor_role?, approval_id?) -> RolloutState`
- `rollout_rollback(profile_id, actor_id?, actor_role?, approval_id?) -> RolloutState`
- `audit_log_list(profile_id, limit?) -> Vec<AuditEvent>`
- `audit_log_verify(profile_id) -> AuditLogVerification`
- `audit_log_export(profile_id, output_path?) -> PathBuf`
- `audit_remote_get(profile_id) -> AuditRemoteSinkState`
- `audit_remote_configure(profile_id, request, actor_id?, actor_role?, approval_id?) -> AuditRemoteSinkState`
- `audit_remote_sync(profile_id, limit?, actor_id?, actor_role?, approval_id?) -> AuditRemoteSyncResult`
- `billing_state_get(profile_id) -> BillingState`
- `billing_config_set(profile_id, request, actor_id?, actor_role?, approval_id?) -> BillingState`
- `billing_verify_receipt(profile_id, request, actor_id?, actor_role?, approval_id?) -> BillingState`
- `workflow_board_get(profile_id, limit?) -> WorkflowBoardView`
- `workflow_task_upsert(profile_id, request, actor_id?, actor_role?, approval_id?) -> WorkflowTaskRecord`
- `workflow_task_move(profile_id, request, actor_id?, actor_role?, approval_id?) -> WorkflowTaskRecord`
- `outcomes_record(profile_id, request, actor_id?, actor_role?, approval_id?) -> OutcomeRecord`
- `outcomes_list(profile_id, limit?) -> Vec<OutcomeRecord>`
- `outcomes_summary(profile_id) -> OutcomeSummary`
- `mission_control_summary(profile_id) -> MissionControlSummary`
- `evidence_export(profile_id, output_dir?) -> EvidenceExportSummary`
- `control_plane_state(profile_id) -> ControlPlaneState`
- `access_state(profile_id) -> AccessState`
- `access_start_trial(profile_id) -> AccessState`
- `access_set_plan(profile_id, plan) -> AccessState`
- `access_set_view(profile_id, view) -> AccessState`
- `policy_evaluate(profile_id, request) -> ActionPolicyDecision`
- `approvals_list(profile_id, pending_only?) -> Vec<ApprovalRequest>`
- `approvals_resolve(profile_id, approval_id, approver_role, approved, reason?) -> ApprovalRequest`
- `receipts_list(profile_id, limit?) -> Vec<ActionReceipt>`
- `receipts_export(profile_id, output_path?) -> PathBuf`
- `retention_set(profile_id, receipts_days, approvals_days) -> RetentionPolicy`
- `retention_purge(profile_id) -> PurgeSummary`
- `runtime_start(profile_id, actor_id?, actor_role?, approval_id?) -> ()`
- `runtime_stop(actor_id?, actor_role?, approval_id?, reason?) -> ()`
- `runtime_send_message(message, actor_id?, actor_role?, approval_id?) -> String`
- `runtime_state() -> String`
- `logs_tail(limit?) -> Vec<LogLine>`
- `logs_export_diagnostics(output_path?, actor_id?, actor_role?, approval_id?) -> PathBuf`
- `secret_set(profile_id, key, value) -> ()`
- `secret_get(profile_id, key) -> Option<String>`
- `secret_exists(profile_id, key) -> bool`
- `secret_delete(profile_id, key) -> ()`
- `secret_backend() -> String`
- `integration_install(profile_id, contract, actor_id?, actor_role?, approval_id?) -> IntegrationRecord`
- `integration_enable(profile_id, integration_id, approved, actor_id?, actor_role?, approval_id?) -> IntegrationRecord`
- `integration_disable(profile_id, integration_id, actor_id?, actor_role?, approval_id?) -> IntegrationRecord`
- `integration_remove(profile_id, integration_id, actor_id?, actor_role?, approval_id?) -> ()`
- `integration_list(profile_id) -> IntegrationRegistry`
- `skills_install(profile_id, request, actor_id?, actor_role?, approval_id?) -> SkillRecord`
- `skills_enable(profile_id, skill_id, approved, actor_id?, actor_role?, approval_id?) -> SkillRecord`
- `skills_disable(profile_id, skill_id, actor_id?, actor_role?, approval_id?) -> SkillRecord`
- `skills_remove(profile_id, skill_id, actor_id?, actor_role?, approval_id?) -> ()`
- `skills_list(profile_id) -> SkillsRegistry`
- `mcp_install(profile_id, request, actor_id?, actor_role?, approval_id?) -> McpConnectorRecord`
- `mcp_update_config(profile_id, connector_id, config, actor_id?, actor_role?, approval_id?) -> McpConnectorRecord`
- `mcp_enable(profile_id, connector_id, approved, actor_id?, actor_role?, approval_id?) -> McpConnectorRecord`
- `mcp_disable(profile_id, connector_id, actor_id?, actor_role?, approval_id?) -> McpConnectorRecord`
- `mcp_remove(profile_id, connector_id, actor_id?, actor_role?, approval_id?) -> ()`
- `mcp_list(profile_id) -> McpConnectorRegistry`
- `pairing_create_bundle(profile_id, transport, endpoint?, expires_in_minutes?) -> PairingBundle`
- `pairing_snapshot_sync_placeholder() -> String`
- `operations_status(profile_id) -> StatusReport`
- `operations_doctor(profile_id, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_channel_doctor(profile_id, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_channels_list(profile_id) -> Vec<ChannelSummary>`
- `operations_channel_add(profile_id, channel_type, config_json, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_channel_remove(profile_id, name, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_channel_bind_telegram(profile_id, identity, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_integrations_catalog(profile_id) -> Vec<IntegrationCatalogEntry>`
- `operations_providers(profile_id) -> Vec<ProviderDescriptor>`
- `operations_models_refresh(profile_id, provider?, force?, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_cron_list(profile_id) -> Vec<CronJobSummary>`
- `operations_cron_add(profile_id, expression, command, tz?, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_cron_remove(profile_id, id, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_cron_pause(profile_id, id, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_cron_resume(profile_id, id, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_service(profile_id, action, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_config_schema() -> JsonValue`
- `operations_auth_profiles(profile_id) -> Vec<AuthProfileSummary>`
- `operations_memory_list(profile_id, category?, session_id?, limit?, actor_id?, actor_role?, approval_id?) -> Vec<MemoryEntrySummary>`
- `operations_migrate_openclaw(profile_id, source?, dry_run, actor_id?, actor_role?, approval_id?) -> OperationResult`
- `operations_command_surface() -> Vec<CommandSurfaceCapability>`
- `operations_cost_summary(profile_id) -> CostSummaryReport`
- `operations_response_cache_stats(profile_id) -> ResponseCacheStatsReport`
- `operations_generate_shell_completions(shell, binary_path?) -> String`
- `background_capabilities() -> BackgroundCapabilities`
- `background_enable(profile_id?, actor_id?, actor_role?, approval_id?) -> ()`
- `background_disable(profile_id?, actor_id?, actor_role?, approval_id?) -> ()`

## Event Stream
- stream source: `LocalAgentRuntime::subscribe_events()`
- forwarded to frontend as Tauri event: `runtime-event`
- event envelope includes `schema_version`
- event kinds:
  - `task_started`
  - `task_finished`
  - `error`
  - `shutdown`
  - `health_tick`
  - `log_line`
  - `state_changed`

## Integration Permission Contract
Treat every integration as an app-permission contract.

- Install does not imply enable.
- Enable requires explicit approval.
- Approval screen must show:
  - what it can access (`can_access`)
  - what it can do (`can_do`)
  - where data goes (`data_destinations`)

The same contract applies to:
- skill installs (`skills_*` commands)
- tool connectors (`mcp_*` commands, MCP transport), which are disabled by default until explicitly enabled in profile setup

## Governance Enforcement
- Per-action policy checks are enforced in app commands (`who`, `what`, `where`, `when`).
- Risky operator actions generate approval requests and action receipts.
- Approved actions can be replayed by submitting the related `approval_id`.

## Workspace and Role Model
- Wrapper setup uses a single workspace mode: `workspace`.
- Deployment mode is explicit in setup:
  - `host`: linux/macos/windows runtime node
  - `client`: macos/windows/android/ios companion endpoint
- Subscription tier is explicit in setup:
  - `basic`
  - `professional`
  - `enterprise`
- Wrapper role model:
  - `admin`
  - `manager`
  - `user`
  - `observer`
- Legacy role names from older profiles are accepted and normalized in the wrapper bridge.
- `access_start_trial`, `access_set_plan`, and `access_set_view` remain for compatibility but are pinned to the org workspace path in wrapper UX.
- Setup payload/state also carries advanced runtime parity fields:
  - `api_url`, `default_temperature`
  - `runtime_reasoning_enabled`
  - `agent_compact_context`, `agent_parallel_tools`, `agent_max_tool_iterations`, `agent_max_history_messages`, `agent_tool_dispatcher`
  - `skills_prompt_injection_mode`, `skills_open_enabled`, `skills_open_dir`
  - `enable_tool_connectors`
  - delegate-agent advanced fields: `agentic`, `allowed_tools`, `max_iterations`

## Mission Control Contract
- Rollout control is ring-based (`pilot` -> `group` -> `all`) with rollback support.
- Rollout promotion supports signer attestation: staged release signatures can be required and verified against trusted signer keys before promotion.
- Audit logs are append-only with hash-chain verification endpoints.
- Remote append-only audit export supports SIEM/object-lock style ingestion while keeping local hash-chain logs as primary evidence.
- RBAC is centrally editable from wrapper control-plane commands.
- Billing entitlement verification can be linked to backend/store receipts and optionally enforced for paid control-plane features.
- Workflow board provides kanban-style operational tracking (`pending`, `in_progress`, `done`, `failed`, `blocked`) with agent/skill/tool references.
- Compliance profiles provide AI Act/NIST/industry posture checks and expose missing-control evidence in mission summaries.
- Outcomes are first-class records (`solved`/`partial`/`unsolved` + impact score) and summarized for business KPI visibility.
- Evidence export outputs a compliance-ready artifact set including audit/rollout/RBAC/workflow/compliance/outcomes/remote-audit/billing/version/SBOM-checksum files.

## Industry Policy Profiles
- Wrapper supports swappable policy profiles (industry-agnostic contract):
  - `general`
  - `finance_strict`
  - `healthcare_strict`
  - `gov_zero_public`
- Policy profiles constrain provider allowlists and pairing transport allowlists without changing provider/channel/tool trait implementations.

## Optional Pairing Mode
- Host devices can operate as runtime + memory host.
- Client devices can pair via QR payload and token.
- Transport options: LAN / Tailscale / Cloudflare tunnel / ngrok.
- Encrypted snapshot sync remains an explicit placeholder for later implementation.

## Orchestration Setup Contract
- One orchestrator agent per workspace is the recommended mode.
- Specialized sub-agents are configured through `profile_setup_save.payload.delegate_agents`.
- For strict isolation (memory/logs/config), users should use multiple profiles/workspaces instead of sharing one memory state.

## Command-Surface Parity Note
- Core command families now include `Auth`, `Config`, `Memory`, `Completions`, and `Integration Catalog`.
- Wrapper exposes `Auth`, `Config`, `Memory`, `Completions`, and `Integration Catalog` operations directly in UI.
- Completions generation resolves the `zeroclaw` binary in this order: explicit UI path, `ZEROCLAW_BIN`, packaged resources sidecar, then `PATH`.

## Platform UX Requirements
- Android label: "Background Mode (persistent notification required)"
- iOS label: "Background Mode (best effort; OS may pause execution)"
- Desktop update label: "Auto-update available"
- Mobile update label: "Update via App Store / Play Store"
