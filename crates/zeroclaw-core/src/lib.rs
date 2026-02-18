#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::too_many_lines
)]

pub mod background;
pub mod control_plane;
pub mod events;
pub mod integrations;
pub mod lifecycle;
pub mod logs;
pub mod mcp;
pub mod pairing_mode;
pub mod profiles;
pub mod protocol;
pub mod runtime;
pub mod secrets;
pub mod skills;

pub use background::{
    AndroidBackgroundAdapter, BackgroundCapabilities, DesktopBackgroundAdapter,
    IosBackgroundAdapter, PlatformBackground,
};
pub use control_plane::{
    AccessPlan, AccessState, ActionPolicyDecision, ActionPolicyRequest, ActionReceipt,
    ApprovalRequest, ApprovalStatus, ControlPlaneState, ControlPlaneStore, PolicyRule,
    PurgeSummary, ReceiptResult, RetentionPolicy, WorkspaceView,
};
pub use events::{EventBus, RuntimeEvent, RuntimeEventKind};
pub use integrations::{
    IntegrationPermissionContract, IntegrationRecord, IntegrationRegistry, IntegrationRegistryStore,
};
pub use lifecycle::{AgentState, LifecycleController, LifecycleSnapshot};
pub use logs::{JsonlLogSink, LogLine, LogSink, LogSinkConfig};
pub use mcp::{
    McpConnectorConfig, McpConnectorInstallRequest, McpConnectorRecord, McpConnectorRegistry,
    McpConnectorStore,
};
pub use pairing_mode::{
    create_pairing_bundle, PairingBundle, PairingRequest, PairingTransport, SnapshotSyncMode,
};
pub use profiles::{ProfileManager, ProfileRecord, ProfileWorkspace, ProfilesIndex};
pub use protocol::{
    protocol_handshake, ProtocolHandshake, CONFIG_SCHEMA_VERSION, CORE_PROTOCOL_VERSION,
    EVENT_SCHEMA_VERSION,
};
pub use runtime::{
    AgentRuntime, AgentSession, AgentSessionFactory, LocalAgentRuntime, RuntimeStartConfig,
    ZeroclawAgentSessionFactory,
};
pub use secrets::{AdaptiveSecretVault, EncryptedFileSecretVault, KeyringSecretVault, SecretVault};
pub use skills::{SkillInstallRequest, SkillRecord, SkillsRegistry, SkillsRegistryStore};
