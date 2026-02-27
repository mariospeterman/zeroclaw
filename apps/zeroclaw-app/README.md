# right-hand-app (Tauri v2 Shell)

Tauri v2 desktop/mobile shell wired to `zeroclaw-core`.

## Product model
- Single-package consumer app (Right Hand branding).
- No separate ZeroClaw CLI installation required.
- CLI remains optional for diagnostics only.
- Single default workspace model (organization-first; no personal/org mode split).

Deployment matrix:
- `host` mode: linux / macOS / windows (runs local runtime node)
- `client` mode: macOS / windows / Android / iOS (approvals, alerts, status, chat, lightweight actions)

Subscription tiers:
- `basic` (monthly)
- `professional` (monthly)
- `enterprise` (monthly)

## Runtime behavior clarity
- `npm run dev` is web preview mode with mock bridge data.
- `npm run tauri dev` runs the real local native runtime bridge.
- `npm run tauri -- android dev` runs the native Android app on emulator/device and uses a local Vite dev server as the WebView content source.
- Production app builds are expected to run local runtime + local profile workspace.

## Implemented foundation
- Core protocol handshake command
- Profile creation/switch/runtime wiring
- Guided setup wizard (profile identity, deployment mode, role, provider/model/memory defaults, temperature/API URL, runtime reasoning, agent loop controls, skills injection mode, key vault write)
- Runtime event forwarding to frontend
- Structured log tail/export commands
- Operations command bridge for:
  - status summary
  - doctor and channel-doctor
  - provider catalog
  - model refresh
  - service lifecycle actions
  - channel add/remove/bind + channel list
  - cron list/add/remove/pause/resume
- Adaptive secret vault commands
  - keychain/keystore via keyring backend when available
  - encrypted-file fallback when keyring backend is unavailable
- Integration permission contract commands (Install != Enable)
- Skill install/manage commands with explicit enable consent
- Tool Connector (MCP) install/config/enable commands with explicit enable consent and setup-level opt-in gate
- Control-plane commands for:
  - single workspace governance with policy gating
  - per-action policy checks
  - approval queue resolution
  - action receipts + retention + export
- Deployment capability introspection command (`deployment_capabilities`) for platform/mode checks
- Policy profile templates and apply/get commands (`policy_profiles_list`, `policy_profile_apply`, `policy_profile_get`)
- Compliance profile templates and posture commands (`compliance_profiles_list`, `compliance_profile_apply`, `compliance_profile_get`, `compliance_posture_get`)
- Optional hub/client pairing bundle command (QR payload + token)
- Explicit client onboarding command for host attach (`client_connect_host`) + host connection state (`host_connection_get`)
- Background mode capability and enable/disable command hooks
- Mission Control controls:
  - rollout rings (`rollout_stage_release`, `rollout_promote`, `rollout_rollback`)
  - rollout signing policy and signer attestation (`rollout_set_signing_policy`)
  - RBAC registry (`rbac_users_list`, `rbac_user_upsert`)
  - tamper-evident audit log chain (`audit_log_verify`, `audit_log_export`)
  - remote append-only audit sink (`audit_remote_get`, `audit_remote_configure`, `audit_remote_sync`)
  - billing entitlement verification (`billing_state_get`, `billing_config_set`, `billing_verify_receipt`)
  - workflow kanban operations (`workflow_board_get`, `workflow_task_upsert`, `workflow_task_move`)
  - outcomes tracking (`outcomes_record`, `outcomes_summary`)
  - evidence export pack (`evidence_export`)
- Browser-safe command bridge for `npm run dev` UI preview mode

## App-permission rule
Every integration is treated as a permission contract:
- Install != Enable
- Enable requires explicit approval
- Approval must show access, actions, and data destinations

## Workspace and roles
- Workspace model is always `workspace` (organization-first default).
- Roles used by wrapper UI and setup:
  - `admin`
  - `manager`
  - `user`
  - `observer`
- Legacy role names from older setup data are automatically mapped.
- Guarded actions emit policy receipts for audit/replay.

## Enterprise hardening controls
- Signed rollout promotion now verifies Ed25519 signatures against trusted signer keys.
- Local hash-chain audit remains source-of-truth, with optional remote append-only sync for SIEM/object-lock style ingestion.
- Subscription tiers can be bound to backend/store receipt verification and enforcement (`enforce_verification` mode).
- Compliance posture packs (AI Act/NIST + industry variants) can enforce stricter rollout/audit/billing controls and report missing controls.

## Browser Mock Meaning
- The browser mock bridge is only for `npm run dev` frontend preview.
- It simulates command results for UI development and local demos.
- Real governance, runtime execution, and secure operations must be validated with `npm run tauri dev` or target mobile/desktop builds.

## Local development
```bash
cd apps/zeroclaw-app
npm install
npm run dev        # browser preview mode (mock command bridge)
npm run dev:tauri  # fixed-port dev server for tauri (reuses existing server)
npm run tauri dev
npm run android:dev # native Android dev run (emulator/device)
npm run android:studio # recommended Android Studio flow (opens Studio + keeps dev bridge alive)
```

Why `android dev` can look like "web":
- Tauri mobile dev always needs a frontend dev server (`build.devUrl`) during development.
- The Android app is still native; it embeds that dev URL in the app WebView.
- If you only open `http://localhost:1420` in desktop browser, you are only seeing the frontend preview, not the full Android runtime path.

Official docs alignment (Tauri v2):
- prerequisites: `https://v2.tauri.app/start/prerequisites/#configure-for-mobile-targets`
- development workflow: `https://v2.tauri.app/develop/`

Recommended Android Studio workflow (Tauri v2):
```bash
cd apps/zeroclaw-app
npm run android:studio
```
- Keep that terminal running while you build/run from Android Studio.
- In Android Studio, select `x86_64Debug` for x86_64 emulator and `arm64Debug` for physical arm64 devices.

## Mobile init status
- `tauri android init`: completed and regenerated after bundle identifier rename (`com.righthand.app`):
  - `ANDROID_HOME` / `ANDROID_SDK_ROOT`
  - `JAVA_HOME`
  - `NDK_HOME`
- `tauri ios init`: requires modern Xcode/xcodegen support; on macOS 12 this fails because Xcode 15.3 cannot be installed.

Example Android init command:
```bash
export JAVA_HOME="$HOME/.local/jdk/jdk-21/Contents/Home"
export ANDROID_HOME="$HOME/Library/Android/sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export NDK_HOME="$ANDROID_HOME/ndk/27.3.13750724"
npm run tauri -- android init --ci --skip-targets-install
```

Recommended stable emulator baseline for this project:
```bash
"$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager" "system-images;android-35;google_apis_playstore;x86_64"
"$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/avdmanager" create avd -n RightHand_API_35 -k "system-images;android-35;google_apis_playstore;x86_64" -d medium_phone --force
"$ANDROID_SDK_ROOT/emulator/emulator" -avd RightHand_API_35 -wipe-data
```

If Android init fails after changing app identifier/package name:
```bash
rm -rf src-tauri/gen/android
npm run tauri -- android init --ci --skip-targets-install
```

## Troubleshooting
- `TypeError ... invoke` in `npm run dev`:
  - fixed by browser-safe bridge; `npm run dev` now runs in preview mode with mock data and a mock command bridge.
  - use `npm run tauri dev` for real native command execution.
- `Port 1420 is already in use`:
  - Tauri mobile/desktop dev expects fixed `devUrl` port `1420`.
  - if another process owns `1420`, stop it first:
    - `lsof -nP -iTCP:1420 -sTCP:LISTEN`
    - `kill <pid>`
  - `npm run tauri dev` and `npm run tauri -- android dev` both rely on `npm run dev:tauri` using strict fixed port.
- Cargo auto-clean warning during `npm run tauri dev`:
  - `tauri` script now sets `CARGO_AUTO_CLEAN_FREQUENCY=never` to avoid noisy global cache cleanup attempts.
  - `tauri` script also pins `RUSTUP_TOOLCHAIN=1.92.0` so Cargo supports `edition2024` dependencies.
  - if you want to permanently fix ownership, re-own your cargo registry entries created by root:
    - `sudo chown -R "$(id -u)":"$(id -g)" ~/.cargo/registry`
- Android `adb: device offline` / `adb reverse tcp:1420` fails:
  - this is the real failure when Gradle reports `:app:rustBuildX86_64Debug FAILED`.
  - run these recovery commands, then retry:
    - `"$ANDROID_HOME/platform-tools/adb" kill-server`
    - `"$ANDROID_HOME/platform-tools/adb" start-server`
    - `"$ANDROID_HOME/platform-tools/adb" devices`
  - in Android Studio: Device Manager -> emulator -> `Cold Boot Now` (or restart the device).
  - `npm run tauri android dev ...` includes ADB recovery (`reconnect`, `kill-server`, `start-server`, retry loop, emulator boot wait).
  - optional targeting:
    - `export RIGHT_HAND_ANDROID_AVD="RightHand_API_35"`
    - `export RIGHT_HAND_ANDROID_SERIAL="emulator-5554"`
  - `android-studio-script` no longer runs ADB preflight, so plain Android Studio `assemble*` builds are not blocked by emulator state.
  - if no ready device is detected, it auto-starts an emulator AVD and waits for boot completion (`sys.boot_completed=1`) before invoking Tauri Android commands.
  - optional: pin which AVD to use:
    - `export RIGHT_HAND_ANDROID_AVD="RightHand_API_35"`
- Android Studio error `Execution failed for task ':app:rustBuildArmDebug'` with `Cannot run program "npm"`:
  - cause: Android Studio often runs Gradle with a restricted PATH that does not include Node/npm.
  - fix in repo: Gradle rust task now resolves npm from known install paths (`/usr/local/bin`, `/opt/homebrew/bin`, `~/.nvm/...`) and augments PATH so `node` is found when npm runs scripts.
  - fix in repo: Gradle rust task also falls back to shell execution (`zsh -lc`) when direct execution fails.
  - optional explicit override:
    - `export RIGHT_HAND_NPM_BIN="/opt/homebrew/bin/npm"` (or your actual npm path)
    - `export NPM_BIN="/opt/homebrew/bin/npm"`
- Android Studio error `Inconsistent JVM Target Compatibility Between Java and Kotlin Tasks`:
  - this repo now aligns app Java/Kotlin targets to JVM 11 in:
    - `src-tauri/gen/android/app/build.gradle.kts`
  - and sets Kotlin toolchain to 11 to prevent reoccurrence on Gradle sync.
  - if Android Studio still shows stale target values, run:
    - `./gradlew --stop && ./gradlew clean`
    - then Sync Project with Gradle Files.
- Kotlin compile daemon error `Could not connect to Kotlin compile daemon`:
  - fix in repo: `src-tauri/gen/android/gradle.properties` sets:
    - `kotlin.compiler.execution.strategy=in-process`
  - this disables Kotlin daemon usage for this project and avoids transient Studio daemon IPC failures.
  - then run:
    - `cd apps/zeroclaw-app && npm run android:reset-lock`
    - in Android Studio: Sync Project with Gradle Files, then Rebuild Project.
- Android Studio/Upgrade Assistant plugin crash or cast errors after AGP 9 migration:
  - do not run AGP Upgrade Assistant on this generated Tauri Android project.
  - keep this stack aligned:
    - AGP `8.11.0`
    - Gradle `8.14.3`
    - Kotlin Android plugin `1.9.25`
  - if accidentally upgraded, regenerate Android project files:
    - `rm -rf src-tauri/gen/android`
    - `npm run tauri -- android init --ci --skip-targets-install`
- Gradle failure on `:app:checkArmDebugClasspath` with `compileVersionMap ... cannot be serialized`:
  - this is an AGP classpath-check path issue on some generated Tauri Android setups.
  - fix in repo:
    - `src-tauri/gen/android/gradle.properties` uses:
      - `android.dependency.excludeLibraryComponentsFromConstraints=true`
      - `android.enableClasspathCheckTasks=false`
  - then run:
    - `./gradlew --stop`
    - `rm -rf .gradle build buildSrc/.gradle buildSrc/build`
    - `./gradlew :app:assembleArmDebug --no-daemon --refresh-dependencies`
- Gradle lock timeout (`buildLogic.lock` in use by another PID):
  - cause: Android Studio sync/build and CLI build running concurrently.
  - use one command to reset lock state:
    - `cd apps/zeroclaw-app && npm run android:reset-lock`
  - reset also terminates stale Kotlin compile daemon processes.
  - then run only one build path at a time (Studio **or** terminal), not both in parallel.
- Android Studio run error: `armDebug uses split APKs ... none compatible with x86_64, arm64-v8a`:
  - cause: wrong build variant selected for the emulator ABI.
  - use `x86_64Debug` for Android Studio emulator runs (or `universalDebug`).
  - use `arm64Debug` for physical arm64 devices.
  - in this repo, Rust Android flavors now default to 64-bit only (`arm64`, `x86_64`) to avoid incompatible `arm`/`x86` defaults.
  - install directly to emulator from terminal:
    - `cd src-tauri/gen/android`
    - `./gradlew :app:installX86_64Debug --no-daemon`
- Install failure `cmd: Can't find service: package` or `Failure calling service package: Broken pipe`:
  - this is an unhealthy emulator runtime/image (package manager service not stable), not app code.
  - recovery order:
    - Device Manager -> emulator menu -> `Cold Boot Now`
    - if still failing: `Wipe Data`
    - if still failing: recreate AVD with **Google APIs / Google Play API 35 x86_64**
  - if you see `NullPointerException ... StorageManager.getVolumes()` during install, treat it as the same emulator-image issue.
  - quick verify before install:
    - `adb -s emulator-5554 shell getprop sys.boot_completed` should be `1`
    - `adb -s emulator-5554 shell pm list packages | head` should return package lines
    - `adb -s emulator-5554 shell cmd package list packages | head` should also return package lines (no `Can't find service: package`)
  - then run:
    - `./gradlew :app:installX86_64Debug --no-daemon`
  - stable baseline recommendation for this repo:
    - avoid preview API `36.1` for daily dev runs.
    - use API `35` x86_64 Google APIs/Play image for emulator development.
- `android-studio-script` panic with `failed to build WebSocket client ... Connection refused`:
  - this occurs when Android Studio Gradle tasks run without a live Tauri Android dev session.
  - recommended run flow for emulator:
    - `cd apps/zeroclaw-app`
    - start emulator first in Android Studio
    - run `npm run android:studio`
  - keep that terminal running while using hot reload.
- Panic `failed to read missing addr file ... <identifier>-server-addr` during `rustBuild*`:
  - cause: `tauri android android-studio-script` expects a temp server-addr file in some debug flows.
  - fix in repo: `scripts/tauri-cli.mjs` now auto-creates that file from `tauri.conf.json` (`build.devUrl`) with fallback `127.0.0.1:1420`.
  - optional override:
    - `export RIGHT_HAND_DEV_SERVER_ADDR="127.0.0.1:1420"`
- `persist.device_config...enable_xr_simulated_env` setprop failure:
  - this is not required for Tauri Android development.
  - ignore/remove that step unless you are explicitly developing Android XR features on a supported image.
- Android Studio sync error `tauri.settings.gradle ... does not exist`:
  - cause: those files are generated by Tauri and are gitignored by default.
  - fix path in this repo:
    - `src-tauri/gen/android/settings.gradle` now guards `apply from: 'tauri.settings.gradle'` behind an existence check.
    - `src-tauri/gen/android/app/build.gradle.kts` now guards `apply(from = "tauri.build.gradle.kts")` behind an existence check.
    - safe placeholders are included at:
      - `src-tauri/gen/android/tauri.settings.gradle`
      - `src-tauri/gen/android/app/tauri.build.gradle.kts`
  - regenerate/autowire when needed:
    - `cd apps/zeroclaw-app && npm run tauri android init -- --ci --skip-targets-install`
    - `cd apps/zeroclaw-app && npm run tauri android build -- --debug --apk -t aarch64`
- Gradle parse error `Unexpected input ... <ydef tauriSettings = file(...)`:
  - this is a malformed `src-tauri/gen/android/settings.gradle` line.
  - `npm run tauri ...` now auto-repairs it before Android commands run.
  - manual fallback:
    - edit `src-tauri/gen/android/settings.gradle` line 6 to:
      - `def tauriSettings = file('tauri.settings.gradle')`

## Platform update policy
- Desktop: Tauri updater path.
- Android/iOS: store-driven updates only.

## Current scope limits
- Mobile native plugin plumbing (Foreground Service/BGTaskScheduler/APNs/FCM) is still pending implementation.
- Encrypted snapshot sync is currently placeholder-only by design.
- Runtime safety is policy-guarded and local-first, but strict OS-level sandbox confinement is not yet fully implemented across all targets.
- Shell completions remain CLI-only by design (not a runtime UI operation).
