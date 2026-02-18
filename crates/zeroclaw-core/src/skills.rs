use crate::integrations::IntegrationPermissionContract;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillInstallRequest {
    pub skill_id: String,
    pub display_name: String,
    pub source: String,
    pub version: String,
    #[serde(default)]
    pub manifest_markdown: Option<String>,
    pub contract: IntegrationPermissionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRecord {
    pub skill_id: String,
    pub display_name: String,
    pub source: String,
    pub version: String,
    pub installed_at: String,
    pub enabled: bool,
    pub enabled_at: Option<String>,
    pub skill_dir: PathBuf,
    pub contract: IntegrationPermissionContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillsRegistry {
    pub records: Vec<SkillRecord>,
}

#[derive(Debug, Clone)]
pub struct SkillsRegistryStore {
    path: PathBuf,
    skills_dir: PathBuf,
}

impl SkillsRegistryStore {
    pub fn for_workspace(workspace_dir: &Path) -> Self {
        Self {
            path: workspace_dir.join("skills_registry.json"),
            skills_dir: workspace_dir.join("skills"),
        }
    }

    pub fn load(&self) -> Result<SkillsRegistry> {
        if !self.path.exists() {
            return Ok(SkillsRegistry::default());
        }

        let body = fs::read_to_string(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        serde_json::from_str(&body).context("failed to parse skills registry")
    }

    fn save(&self, registry: &SkillsRegistry) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let body = serde_json::to_string_pretty(registry)
            .context("failed to serialize skills registry")?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, body).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.path)
            .with_context(|| format!("failed to replace {}", self.path.display()))?;
        Ok(())
    }

    pub fn install(&self, request: SkillInstallRequest) -> Result<SkillRecord> {
        validate_identifier(&request.skill_id)?;
        if request.display_name.trim().is_empty() {
            anyhow::bail!("display_name must not be empty");
        }

        fs::create_dir_all(&self.skills_dir)
            .with_context(|| format!("failed to create {}", self.skills_dir.display()))?;

        let mut registry = self.load()?;
        let now = Utc::now().to_rfc3339();
        let skill_dir = self.skills_dir.join(&request.skill_id);
        fs::create_dir_all(&skill_dir)
            .with_context(|| format!("failed to create {}", skill_dir.display()))?;
        write_skill_manifest(&skill_dir, &request)?;

        if let Some(existing_idx) = registry
            .records
            .iter()
            .position(|record| record.skill_id == request.skill_id)
        {
            let existing = &mut registry.records[existing_idx];
            existing.display_name = request.display_name;
            existing.source = request.source;
            existing.version = request.version;
            existing.contract = request.contract;
            existing.skill_dir = skill_dir.clone();
            let out = existing.clone();
            self.save(&registry)?;
            return Ok(out);
        }

        let record = SkillRecord {
            skill_id: request.skill_id,
            display_name: request.display_name,
            source: request.source,
            version: request.version,
            installed_at: now,
            enabled: false,
            enabled_at: None,
            skill_dir,
            contract: request.contract,
        };

        registry.records.push(record.clone());
        self.save(&registry)?;
        Ok(record)
    }

    pub fn enable(&self, skill_id: &str, approved: bool) -> Result<SkillRecord> {
        if !approved {
            anyhow::bail!("skill enable denied: explicit consent is required (Install != Enable)");
        }

        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.skill_id == skill_id)
        else {
            anyhow::bail!("skill '{}' is not installed", skill_id);
        };

        record.enabled = true;
        record.enabled_at = Some(Utc::now().to_rfc3339());

        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn disable(&self, skill_id: &str) -> Result<SkillRecord> {
        let mut registry = self.load()?;
        let Some(record) = registry
            .records
            .iter_mut()
            .find(|record| record.skill_id == skill_id)
        else {
            anyhow::bail!("skill '{}' is not installed", skill_id);
        };

        record.enabled = false;
        let out = record.clone();
        self.save(&registry)?;
        Ok(out)
    }

    pub fn remove(&self, skill_id: &str) -> Result<()> {
        let mut registry = self.load()?;
        let before = registry.records.len();
        registry
            .records
            .retain(|record| record.skill_id != skill_id);
        if registry.records.len() == before {
            anyhow::bail!("skill '{}' is not installed", skill_id);
        }

        let skill_dir = self.skills_dir.join(skill_id);
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)
                .with_context(|| format!("failed to remove {}", skill_dir.display()))?;
        }

        self.save(&registry)?;
        Ok(())
    }
}

fn write_skill_manifest(skill_dir: &Path, request: &SkillInstallRequest) -> Result<()> {
    let manifest = request
        .manifest_markdown
        .clone()
        .unwrap_or_else(|| default_skill_manifest(&request.skill_id, &request.display_name));
    fs::write(skill_dir.join("SKILL.md"), manifest)
        .with_context(|| format!("failed to write {}/SKILL.md", skill_dir.display()))?;
    Ok(())
}

fn default_skill_manifest(skill_id: &str, display_name: &str) -> String {
    format!("# {display_name}\n\nid: {skill_id}\n\nInstalled via ZeroClaw app.\n",)
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_enable_disable_remove_flow() {
        let tmp = TempDir::new().unwrap();
        let store = SkillsRegistryStore::for_workspace(tmp.path());

        let request = SkillInstallRequest {
            skill_id: "markdown_summarizer".into(),
            display_name: "Markdown Summarizer".into(),
            source: "catalog".into(),
            version: "1.0.0".into(),
            manifest_markdown: None,
            contract: IntegrationPermissionContract {
                integration_id: "skill:markdown_summarizer".into(),
                can_access: vec!["workspace/files".into()],
                can_do: vec!["read markdown".into()],
                data_destinations: vec!["local-only".into()],
            },
        };

        let installed = store.install(request).unwrap();
        assert_eq!(installed.skill_id, "markdown_summarizer");
        assert!(!installed.enabled);
        assert!(installed.skill_dir.join("SKILL.md").exists());

        assert!(store.enable("markdown_summarizer", false).is_err());
        assert!(store.enable("markdown_summarizer", true).unwrap().enabled);

        let disabled = store.disable("markdown_summarizer").unwrap();
        assert!(!disabled.enabled);

        store.remove("markdown_summarizer").unwrap();
        assert_eq!(store.load().unwrap().records.len(), 0);
    }
}
