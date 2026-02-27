# Right Hand Wrapper Build Plan (ZeroClaw Runtime, 2026)

## 1) Product objective
Ship one app product that embeds ZeroClaw runtime and supports:
- `host` deployment for enterprise runtime nodes
- `client` deployment for secure team endpoints
- strict role-based governance for high-risk sectors (finance, healthcare, tech, government)

The setup must stay simple for one operator and scalable for regulated organizations.

## 2) Runtime modes and mock clarification
Runtime paths:
- `npm run dev`: frontend-only preview with a browser mock bridge (no native Tauri/Rust runtime)
- `npm run tauri dev`: real native app bridge + real Rust command execution

Why mock exists:
- fast UI iteration
- safer frontend-only development without native/mobile toolchain dependencies

Real backend validation path (recommended):
1. `cd apps/zeroclaw-app`
2. `npm run tauri dev` (desktop host/client runtime path)
3. `npm run android:studio` (Android native runtime via Android Studio; keep terminal process alive)
4. use mission control actions in-app (not browser-only preview) to validate real command execution and persisted workspace state.

## 3) Platform matrix
| Platform | Host | Client |
|---|---:|---:|
| Linux | Yes | Yes |
| macOS | Yes | Yes |
| Windows | Yes | Yes |
| Android | No | Yes |
| iOS | No | Yes |

Policy in wrapper bridge:
- `runtime_start` is host-only on desktop targets
- mobile defaults to `client`
- unsupported deployment mode is rejected on setup save

## 4) Workspace, role, and subscription model
Workspace model:
- single organization-first workspace mode (`workspace`)
- profile-per-domain isolation boundary for trust and memory separation

Roles:
- `admin`
- `manager`
- `user`
- `observer`

Subscription tiers:
- `basic`
- `professional`
- `enterprise`

## 5) Current mission-control status (implemented)
Implemented and wired end-to-end in wrapper backend + UI:
- host/client onboarding via pairing invite payload (`client_connect_host`, `host_connection_get`)
- setup/config parity extensions for behavior/personality/runtime:
  - `api_url`, `default_temperature`
  - runtime reasoning toggle
  - agent loop controls (`compact_context`, `parallel_tools`, `max_tool_iterations`, `max_history_messages`, `tool_dispatcher`)
  - skills prompt injection/open-skills settings
  - delegate-agent advanced controls (`agentic`, `allowed_tools`, `max_iterations`)
- policy templates and enforcement (`policy_profiles_list`, `policy_profile_get`, `policy_profile_apply`)
- compliance templates and posture evaluation (`compliance_profiles_list`, `compliance_profile_get`, `compliance_profile_apply`, `compliance_posture_get`)
- RBAC registry (`rbac_users_list`, `rbac_user_upsert`)
- rollout controls (`rollout_stage_release`, `rollout_promote`, `rollout_rollback`)
- rollout signer policy (`rollout_set_signing_policy`)
- tamper-evident local audit chain (`audit_log_list`, `audit_log_verify`, `audit_log_export`)
- remote append-only audit sink (`audit_remote_get`, `audit_remote_configure`, `audit_remote_sync`)
- billing entitlement state and verification (`billing_state_get`, `billing_config_set`, `billing_verify_receipt`)
- tool connectors (MCP) are now setup-gated and disabled by default unless explicitly enabled in profile setup
- outcome tracking (`outcomes_record`, `outcomes_list`, `outcomes_summary`)
- kanban workflow tracking by workspace/task/agent/skill/tool (`workflow_board_get`, `workflow_task_upsert`, `workflow_task_move`)
- evidence export bundle (`evidence_export`)

## 6) Three production-hardening controls (implemented)
### 6.1 Cryptographic signer attestation on rollout
- staged release promotion now verifies Ed25519 signatures against trusted signer keys
- signer policy is configurable per profile/workspace
- promotion fails closed when `signature_required=true` and verification fails

### 6.2 Fleet-oriented remote append-only audit sink
- local hash-chain audit stays authoritative
- optional remote sync pushes append-only batches to SIEM/object-lock endpoint
- supports auth secret reference and last-synced hash continuity

### 6.3 Tier entitlement binding to billing receipts
- setup tier is persisted locally
- optional backend receipt verification updates entitlement state
- optional strict enforcement mode gates premium control-plane capabilities

## 7) Industry policy profiles and enforcement flow
Profiles:
- `general`
- `finance_strict`
- `healthcare_strict`
- `gov_zero_public`

Apply/get/list flow:
- list templates -> select template -> apply to profile
- applied policy is persisted and reflected in setup/runtime controls

Enforcement currently active in wrapper paths:
- provider allowlist enforcement on setup save
- transport allowlist enforcement on pairing/client connect
- gateway bind/pairing defaults applied to profile config

This preserves ZeroClaw trait-driven modularity: providers/channels/tools remain swappable; policy constraints are injected at control-plane boundary.

## 8) Mission-control process tracking coverage
Current per-workspace process visibility includes:
- approvals queue (`pending`)
- receipts (`allowed`/`denied`/`pending_approval`)
- kanban workflow lanes (`pending`/`in_progress`/`done`/`failed`/`blocked`)
- per-task operational links (`runtime_task_id`, `agent_id`, `skill_id`, `tool_id`, `risk_score`)
- outcomes (`solved`/`partial`/`unsolved`)
- rollout ring stage progression (`pilot` -> `group` -> `all`) and rollback history
- audit chain integrity and remote sync status

## 9) Compliance logic posture (AI Act / NIST-oriented control intent)
The wrapper now enforces and reports controls aligned to high-risk governance patterns:
- policy-gated privileged actions with approval/receipt evidence
- immutable-style local audit chain with verification API
- release promotion control with signer verification
- least-privilege role separation via RBAC
- profile-scoped workspace isolation
- compliance profile packs with mission posture checks for:
  - AI Act + NIST strict baseline
  - finance/fintech
  - healthcare/pharma
  - tech/cloud/web3/ai
  - government (US/EU oriented)

This is implementation alignment, not legal certification by itself.

## 10) Evidence export contents
Evidence pack now includes:
- `audit-log.json`
- `audit-verify.json`
- `rollout-state.json`
- `rbac-users.json`
- `outcomes.json`
- `audit-remote-state.json`
- `billing-state.json`
- `workflow-board.json`
- `compliance-profile.json`
- `compliance-posture.json`
- `mission-summary.json`
- `version-manifest.json`
- `sbom-manifest.json`
- `incident-playbook.md`

## 11) Validation checklist
For each release candidate:
1. Host mode only on desktop targets.
2. Client mode cannot start runtime.
3. Connect-to-host flow succeeds from invite payload.
4. Rollout stage/promote/rollback succeeds under signer policy.
5. RBAC changes persist and appear in mission summary.
6. Audit chain verifies after governed operations.
7. Remote audit sync succeeds to configured endpoint.
8. Billing receipt verification updates entitlement state.
9. Workflow board updates and lane moves persist correctly.
10. Compliance profile apply updates posture checks and required controls.
11. Evidence export generates full artifact set.

## 12) Next production step
1. Replace mock/demo signing key inputs with managed enterprise key lifecycle (rotation + revocation service).
2. Add authenticated remote-audit ingestion acknowledgement verification (server-side hash anchoring).
3. Connect billing verification endpoint to real store-receipt validation and subscription management backend.
4. Add fleet-level multi-host mission aggregation once control plane spans multiple host nodes.
5. Add policy-pack versioning and signed policy updates for regulated change control.

## 13) zeroclawlabs.ai/docs parity check (Usage / Configuration / Advanced)
Mapped against current public docs sections:
- Docs endpoint note:
  - `https://zeroclawlabs.ai/docs` is SPA-delivered; parity verification is anchored to repository docs + command-surface checks in CI.
- Usage:
  - commands/agent/gateway/channels/tools/memory/migrate/completions are represented in wrapper via `operations_*` command bridge + mission controls.
- Configuration:
  - config/providers/security are represented via setup wizard, provider/model/temperature/API URL, runtime/agent loop controls, skills injection controls, secret vault, and policy profiles.
  - environment handling remains split between wrapper setup and host runtime environment deployment.
- Advanced:
  - architecture/gateway-api/auth/development/docs-hub remain source-of-truth in core docs and repository docs.
  - wrapper exposes operational controls while preserving trait-driven swappability from core.

## 14) Decision/Risk Maps and ADKAR fit
Current recommendation for MVP:
- keep decision/risk maps lightweight and operational (workflow `risk_score`, compliance posture checks, receipts/audit evidence)
- full ADKAR engine is overengineering now.
- Use lightweight operational risk/decision tracking (workflow `risk_score` + compliance posture + receipts/audit evidence), then export ADKAR-friendly reports later from evidence artifacts.

Reason:
- this wrapper is the runtime control plane; overloading it with heavyweight change-management modeling slows core reliability goals
- for 2026 B2B MVP, operational evidence, rollout safety, and compliance posture provide higher immediate value

Planned next step (optional):
- export ADKAR-friendly reporting views from existing evidence artifacts instead of embedding a full ADKAR workflow module.

## 15) Known Gaps (intentional)
- Full ADKAR workflow engine is deferred.
- Completions generation resolves `zeroclaw` in this order: explicit UI binary path, `ZEROCLAW_BIN`, bundled sidecar (`resources/bin` or `resources/binaries`), then `PATH`.
- Hardware/peripheral flashing remains feature-gated by platform/toolchain and is not enabled by default in standard business deployments.
- Wrapper control plane is the primary operations UI; CLI remains optional for engineering automation and CI.

## 16) Runtime and Build Caveats
- `npm run dev` is mock preview only; does not execute real Tauri/Rust backend commands.
- `npm run tauri dev` is the real local backend execution path.
- Android Studio + terminal Gradle/Tauri concurrently can deadlock on `.gradle` locks; use one build driver at a time.
- If Gradle lock contention appears:
  1. stop active Gradle daemons (`./gradlew --stop`)
  2. close duplicate Android Studio sync/build jobs
  3. rerun single build path.
- `doctor` output notes to interpret correctly in wrapper context:
  - `no channels configured` is expected until channel onboarding is completed.
  - missing `daemon_state.json` means runtime daemon is not currently running.
  - missing `SOUL.md` / `AGENTS.md` is optional.

## 17) UX Redesign Status (implemented)
Implemented in wrapper UI:
1. Strict 6-page IA flow is active (`Setup -> Keys -> Safety -> Team Tools -> Runtime Ops -> Mission`) using stepper routing.
2. One-active-page mode is enforced (focus mode) to reduce overlap/confusion and keep onboarding/state transitions explicit.
3. Setup completion minimizes the editor into summary mode with explicit reopen (`Edit setup`) and mission jump (`Open Mission`).
4. Team Tools exposes lifecycle controls (`Draft -> Validate -> Publish`) plus runtime reload and integration catalog actions.
5. Integration install now uses catalog-backed dropdown options (registry categories/status/setup hints) instead of free text only.
6. Skill and Tool Connector install flows include preset dropdowns that prefill safe defaults for permissions and config.
7. Runtime Ops includes in-page runtime message controls and keeps migration/completion/service/doctor workflows fully in-app.
8. Mission includes workspace operations (create/switch/list), runtime controls, kanban workflow, rollout/RBAC/audit/outcomes/billing.
9. Section-level contextual `?` popups provide why/what/how guidance for first-time operators.
10. Footer now contains legal/privacy/compliance links; disclosure clutter removed from primary setup surface.
11. Mission workflow now supports `Board/List` toggle with persisted preference and drag/drop kanban transitions backed by `workflow_task_move`.
