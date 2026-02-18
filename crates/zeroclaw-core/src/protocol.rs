use serde::{Deserialize, Serialize};

pub const CORE_PROTOCOL_VERSION: &str = "1.0.0";
pub const EVENT_SCHEMA_VERSION: u32 = 1;
pub const CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolHandshake {
    pub core_protocol_version: String,
    pub event_schema_version: u32,
    pub config_schema_version: u32,
}

pub fn protocol_handshake() -> ProtocolHandshake {
    ProtocolHandshake {
        core_protocol_version: CORE_PROTOCOL_VERSION.to_string(),
        event_schema_version: EVENT_SCHEMA_VERSION,
        config_schema_version: CONFIG_SCHEMA_VERSION,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_has_expected_versions() {
        let handshake = protocol_handshake();
        assert_eq!(handshake.core_protocol_version, "1.0.0");
        assert_eq!(handshake.event_schema_version, 1);
        assert_eq!(handshake.config_schema_version, 1);
    }
}
