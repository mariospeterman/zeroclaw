use crate::integrations::IntegrationPermissionContract;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConnectorConfig {
    pub transport: String,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env_secret_ids: Vec<String>,
    #[serde(default)]
    pub timeout_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConnectorInstallRequest {
    pub connector_id: String,
    pub display_name: String,
    pub config: McpConnectorConfig,
    pub contract: IntegrationPermissionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpConnectorRecord {
    pub connector_id: String,
    pub display_name: String,
    pub installed_at: String,
    pub updated_at: String,
    pub enabled: bool,
    pub enabled_at: Option<String>,
    pub config: McpConnectorConfig,
    pub contract: IntegrationPermissionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConnectorRegistry {
    pub records: Vec<McpConnectorRecord>,
}

#[derive(Debug, Clone)]
pub struct McpConnectorStore {
    path: PathBuf,
}

impl McpConnectorStore {
    pub fn for_workspace(workspace_dir: &Path) -> Self {
        Self {
            path: workspace_dir.join("mcp_connectors.json"),
        }
    }

    pub fn load(&self) -> Result<McpConnectorRegistry> {
        if !self.path.exists() {
            return Ok(McpConnectorRegistry::default());
        }

        let body = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        serde_json::from_str(&body).context("failed to parse mcp connector registry")
    }

    fn save(&self, registry: &McpConnectorRegistry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let body = serde_json::to_string_pretty(registry)
            .context("failed to serialize mcp connector registry")?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, body).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("failed to replace {}", self.path.display()))?;
        Ok(())
    }

    pub fn install(&self, request: McpConnectorInstallRequest) -> Result<McpConnectorRecord> {
        validate_identifier(&request.connector_id)?;
        if request.display_name.trim().is_empty() {
            anyhow::bail!("display_name must not be empty");
        }
        validate_config(&request.config)?;

        let mut registry = self.load()?;
        let now = Utc::now().to_rfc3339();

        if let Some(existing_idx) = registry
            .records
            .iter()
            .position(|record| record.connector_id == request.connector_id)
        {
            let existing = &mut registry.records[existing_idx];
            existing.display_name = request.display_name;
            existing.config = request.config;
            existing.contract = request.contract;
            existing.updated_at = now;
            let out = existing.clone();
            self.save(&registry)?;
            return Ok(out);
        }

        let record = McpConnectorRecord {
            connector_id: request.connector_id,
            display_name: request.display_name,
            installed_at: now.clone(),
            updated_at: now,
            enabled: false,
            enabled_at: None,
            config: request.config,
            contract: request.contract,
        };

        registry.records.push(record.clone());
        self.save(&registry)?;
        Ok(record)
    }

    pub fn update_config(
        &self,
        connector_id: &str,
        config: McpConnectorConfig,
    ) -> Result<McpConnectorRecord> {
        validate_config(&config)?;
        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.connector_id == connector_id)
        else {
            anyhow::bail!("mcp connector '{}' is not installed", connector_id);
        };

        record.config = config;
        record.updated_at = Utc::now().to_rfc3339();
        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn enable(&self, connector_id: &str, approved: bool) -> Result<McpConnectorRecord> {
        if !approved {
            anyhow::bail!(
                "mcp connector enable denied: explicit consent is required (Install != Enable)"
            );
        }

        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.connector_id == connector_id)
        else {
            anyhow::bail!("mcp connector '{}' is not installed", connector_id);
        };

        record.enabled = true;
        record.enabled_at = Some(Utc::now().to_rfc3339());
        record.updated_at = Utc::now().to_rfc3339();
        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn disable(&self, connector_id: &str) -> Result<McpConnectorRecord> {
        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.connector_id == connector_id)
        else {
            anyhow::bail!("mcp connector '{}' is not installed", connector_id);
        };

        record.enabled = false;
        record.updated_at = Utc::now().to_rfc3339();
        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn remove(&self, connector_id: &str) -> Result<()> {
        let mut registry = self.load()?;
        let before = registry.records.len();
        registry
            .records
            .retain(|record| record.connector_id != connector_id);
        if registry.records.len() == before {
            anyhow::bail!("mcp connector '{}' is not installed", connector_id);
        }
        self.save(&registry)?;
        Ok(())
    }
}

fn validate_identifier(id: &str) -> Result<()> {
    if id.trim().is_empty() {
        anyhow::bail!("identifier must not be empty");
    }
    if id
        .chars()
        .any(|ch| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
    {
        anyhow::bail!("identifier contains invalid characters");
    }
    Ok(())
}

fn validate_config(config: &McpConnectorConfig) -> Result<()> {
    let transport = config.transport.trim().to_ascii_lowercase();
    match transport.as_str() {
        "stdio" => {
            if config
                .command
                .as_ref()
                .is_none_or(|command| command.trim().is_empty())
            {
                anyhow::bail!("stdio transport requires command");
            }
        }
        "sse" | "http" | "https" | "websocket" | "ws" | "wss" => {
            if config
                .endpoint
                .as_ref()
                .is_none_or(|endpoint| endpoint.trim().is_empty())
            {
                anyhow::bail!("network transport requires endpoint");
            }
        }
        _ => anyhow::bail!("unsupported transport '{}'", config.transport),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_enable_disable_remove_flow() {
        let tmp = TempDir::new().unwrap();
        let store = McpConnectorStore::for_workspace(tmp.path());

        let request = McpConnectorInstallRequest {
            connector_id: "linear".into(),
            display_name: "Linear MCP".into(),
            config: McpConnectorConfig {
                transport: "sse".into(),
                endpoint: Some("https://mcp.linear.app/sse".into()),
                command: None,
                args: vec![],
                env_secret_ids: vec!["linear_api_key".into()],
                timeout_secs: Some(30),
            },
            contract: IntegrationPermissionContract {
                integration_id: "mcp:linear".into(),
                can_access: vec!["issues.read".into()],
                can_do: vec!["issues.update".into()],
                data_destinations: vec!["mcp.linear.app".into()],
            },
        };

        let installed = store.install(request).unwrap();
        assert_eq!(installed.connector_id, "linear");
        assert!(!installed.enabled);

        assert!(store.enable("linear", false).is_err());
        assert!(store.enable("linear", true).unwrap().enabled);

        let updated = store
            .update_config(
                "linear",
                McpConnectorConfig {
                    transport: "stdio".into(),
                    endpoint: None,
                    command: Some("npx".into()),
                    args: vec!["-y".into(), "@modelcontextprotocol/server-linear".into()],
                    env_secret_ids: vec!["linear_api_key".into()],
                    timeout_secs: Some(60),
                },
            )
            .unwrap();
        assert_eq!(updated.config.transport, "stdio");

        let disabled = store.disable("linear").unwrap();
        assert!(!disabled.enabled);

        store.remove("linear").unwrap();
        assert_eq!(store.load().unwrap().records.len(), 0);
    }
}
