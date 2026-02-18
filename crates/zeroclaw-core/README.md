# zeroclaw-core

Core contracts and runtime primitives for ZeroClaw desktop/mobile app shells.

## Intent
- provide stable runtime APIs for wrapper UI shells
- enforce compatibility via protocol/version handshake
- avoid CLI subprocess parsing as the main runtime interface

## Modules
- `protocol`: compatibility/version handshake and schema constants
- `runtime`: `AgentRuntime` contract + local runtime implementation
- `profiles`: profile index and per-profile workspace provisioning
- `logs`: structured JSONL logging, rotation, diagnostics export
- `events`: runtime event bus and event types
- `lifecycle`: deterministic runtime state machine
- `background`: desktop/mobile background capability adapters
- `secrets`: adaptive keychain/keystore-first vault with encrypted-file fallback
- `integrations`: permission-contract registry (`Install != Enable`)
- `skills`: skill install/enable/disable/remove registry under permission contract
- `mcp`: MCP connector install/config/enable registry under permission contract
- `pairing_mode`: optional hub/client pairing bundle generation with QR payload

## Upstream strategy
- consume from a minimal core fork pinned by tag/commit in wrapper repos
- keep wrapper-specific UI/business logic outside core
- validate upgrades through compatibility suite before version bumps
