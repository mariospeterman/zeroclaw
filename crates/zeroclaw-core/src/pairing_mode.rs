use anyhow::Result;
use base64::Engine;
use chrono::{Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PairingTransport {
    Lan,
    Tailscale,
    CloudflareTunnel,
    NgrokTunnel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSyncMode {
    Disabled,
    PlaceholderEncryptedSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingBundle {
    pub pairing_id: String,
    pub hub_device: String,
    pub endpoint: String,
    pub transport: PairingTransport,
    pub access_token: String,
    pub created_at: String,
    pub expires_at: String,
    pub qr_payload: String,
    pub snapshot_sync_mode: SnapshotSyncMode,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingRequest {
    pub hub_device: String,
    pub endpoint: String,
    pub transport: PairingTransport,
    pub expires_in_minutes: u32,
}

pub fn create_pairing_bundle(req: PairingRequest) -> Result<PairingBundle> {
    let now = Utc::now();
    let expires = now + Duration::minutes(i64::from(req.expires_in_minutes.max(1)));

    let mut token_bytes = [0_u8; 32];
    let mut rng = rand::rng();
    rng.fill_bytes(&mut token_bytes);
    let access_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token_bytes);
    let pairing_id = uuid::Uuid::new_v4().to_string();

    let qr_json = serde_json::json!({
        "pairing_id": pairing_id,
        "hub_device": req.hub_device,
        "endpoint": req.endpoint,
        "transport": req.transport,
        "access_token": access_token,
        "expires_at": expires.to_rfc3339(),
        "snapshot_sync_mode": SnapshotSyncMode::PlaceholderEncryptedSnapshot,
    });

    Ok(PairingBundle {
        pairing_id,
        hub_device: req.hub_device,
        endpoint: req.endpoint,
        transport: req.transport,
        access_token,
        created_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        qr_payload: qr_json.to_string(),
        snapshot_sync_mode: SnapshotSyncMode::PlaceholderEncryptedSnapshot,
        notes: "Android can act as remote client; Mac hub executes and returns logs/results. Encrypted snapshot sync is placeholder-only for later implementation.".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_bundle_contains_qr_payload() {
        let bundle = create_pairing_bundle(PairingRequest {
            hub_device: "mac_hub".into(),
            endpoint: "https://example.tailnet.ts.net".into(),
            transport: PairingTransport::Tailscale,
            expires_in_minutes: 15,
        })
        .unwrap();

        assert!(!bundle.access_token.is_empty());
        assert!(bundle.qr_payload.contains("access_token"));
        assert!(matches!(
            bundle.snapshot_sync_mode,
            SnapshotSyncMode::PlaceholderEncryptedSnapshot
        ));
    }
}
