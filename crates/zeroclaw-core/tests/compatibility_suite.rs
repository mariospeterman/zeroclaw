use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use zeroclaw_core::{
    protocol_handshake, RuntimeEvent, RuntimeEventKind, CONFIG_SCHEMA_VERSION, EVENT_SCHEMA_VERSION,
};

#[derive(Debug, Deserialize)]
struct CorePinFile {
    core: CorePin,
}

#[derive(Debug, Deserialize)]
struct CorePin {
    core_protocol_version: String,
    event_schema_version: u32,
    config_schema_version: u32,
}

#[test]
fn protocol_versions_match_wrapper_pin() {
    let workspace_root = workspace_root();
    let pin_path = workspace_root.join("apps/zeroclaw-app/core-pin.toml");
    let raw = fs::read_to_string(&pin_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", pin_path.display()));
    let parsed: CorePinFile = toml::from_str(&raw)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", pin_path.display()));

    let handshake = protocol_handshake();
    assert_eq!(
        handshake.core_protocol_version,
        parsed.core.core_protocol_version
    );
    assert_eq!(EVENT_SCHEMA_VERSION, parsed.core.event_schema_version);
    assert_eq!(CONFIG_SCHEMA_VERSION, parsed.core.config_schema_version);
}

#[test]
fn event_schema_matches_golden_fixture() {
    let event = RuntimeEvent {
        id: "evt-compat-001".into(),
        schema_version: EVENT_SCHEMA_VERSION,
        profile_id: "compat-profile".into(),
        timestamp: "2026-01-01T00:00:00Z".into(),
        kind: RuntimeEventKind::TaskStarted {
            task_id: "task-001".into(),
            message: "compat-check".into(),
        },
    };

    let actual = serde_json::to_string_pretty(&event).expect("failed to serialize event");
    let fixture = fs::read_to_string(
        workspace_root().join("crates/zeroclaw-core/tests/fixtures/runtime_event_golden.json"),
    )
    .expect("failed to read runtime_event_golden fixture");

    assert_eq!(actual.trim(), fixture.trim());
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("failed to resolve workspace root")
        .to_path_buf()
}
