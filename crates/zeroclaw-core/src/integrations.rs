use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationPermissionContract {
    pub integration_id: String,
    pub can_access: Vec<String>,
    pub can_do: Vec<String>,
    pub data_destinations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationRecord {
    pub integration_id: String,
    pub installed_at: String,
    pub enabled: bool,
    pub enabled_at: Option<String>,
    pub contract: IntegrationPermissionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntegrationRegistry {
    pub records: Vec<IntegrationRecord>,
}

#[derive(Debug, Clone)]
pub struct IntegrationRegistryStore {
    path: PathBuf,
}

impl IntegrationRegistryStore {
    pub fn for_workspace(workspace_dir: &Path) -> Self {
        Self {
            path: workspace_dir.join("integrations.json"),
        }
    }

    pub fn load(&self) -> Result<IntegrationRegistry> {
        if !self.path.exists() {
            return Ok(IntegrationRegistry::default());
        }

        let body = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        serde_json::from_str(&body).context("failed to parse integration registry")
    }

    pub fn save(&self, registry: &IntegrationRegistry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let body = serde_json::to_string_pretty(registry)
            .context("failed to serialize integration registry")?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, body).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("failed to replace {}", self.path.display()))?;
        Ok(())
    }

    pub fn install(&self, contract: IntegrationPermissionContract) -> Result<IntegrationRecord> {
        let mut registry = self.load()?;
        let now = Utc::now().to_rfc3339();

        if let Some(existing_idx) = registry
            .records
            .iter()
            .position(|record| record.integration_id == contract.integration_id)
        {
            registry.records[existing_idx].contract = contract.clone();
            let existing = registry.records[existing_idx].clone();
            self.save(&registry)?;
            return Ok(existing);
        }

        let record = IntegrationRecord {
            integration_id: contract.integration_id.clone(),
            installed_at: now,
            enabled: false,
            enabled_at: None,
            contract,
        };

        registry.records.push(record.clone());
        self.save(&registry)?;
        Ok(record)
    }

    pub fn enable(&self, integration_id: &str, approved: bool) -> Result<IntegrationRecord> {
        if !approved {
            anyhow::bail!(
                "integration enable denied: explicit consent is required (Install != Enable)"
            );
        }

        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.integration_id == integration_id)
        else {
            anyhow::bail!("integration '{}' is not installed", integration_id);
        };

        record.enabled = true;
        record.enabled_at = Some(Utc::now().to_rfc3339());

        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn disable(&self, integration_id: &str) -> Result<IntegrationRecord> {
        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.integration_id == integration_id)
        else {
            anyhow::bail!("integration '{}' is not installed", integration_id);
        };

        record.enabled = false;
        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_then_enable_requires_explicit_approval() {
        let tmp = TempDir::new().unwrap();
        let store = IntegrationRegistryStore::for_workspace(tmp.path());

        store
            .install(IntegrationPermissionContract {
                integration_id: "slack".into(),
                can_access: vec!["messages.read".into()],
                can_do: vec!["messages.send".into()],
                data_destinations: vec!["api.slack.com".into()],
            })
            .unwrap();

        assert!(store.enable("slack", false).is_err());

        let enabled = store.enable("slack", true).unwrap();
        assert!(enabled.enabled);
    }
}
