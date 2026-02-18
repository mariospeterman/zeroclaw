use anyhow::{Context, Result};
use chrono::Utc;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const PROFILES_INDEX_FILE: &str = "profiles.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileRecord {
    pub id: String,
    pub display_name: String,
    pub workspace_dir: PathBuf,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilesIndex {
    pub version: u32,
    pub active_profile: Option<String>,
    pub profiles: Vec<ProfileRecord>,
}

impl Default for ProfilesIndex {
    fn default() -> Self {
        Self {
            version: 1,
            active_profile: None,
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfileWorkspace {
    pub root_dir: PathBuf,
    pub config_path: PathBuf,
    pub memory_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub skills_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProfileManager {
    root_dir: PathBuf,
}

impl ProfileManager {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    pub fn default_root() -> Result<PathBuf> {
        let Some(project_dirs) = ProjectDirs::from("com", "RightHand", "right-hand") else {
            anyhow::bail!("failed to resolve platform app data directory");
        };

        Ok(project_dirs.data_local_dir().to_path_buf())
    }

    pub fn default() -> Result<Self> {
        Ok(Self::new(Self::default_root()?))
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(self.profiles_root())
            .with_context(|| format!("failed to create {}", self.profiles_root().display()))
    }

    pub fn load_index(&self) -> Result<ProfilesIndex> {
        self.ensure_layout()?;
        let path = self.index_path();
        if !path.exists() {
            return Ok(ProfilesIndex::default());
        }

        let data = fs::read_to_string(&path)
            .with_context(|| format!("failed to read profiles index {}", path.display()))?;

        let index: ProfilesIndex = serde_json::from_str(&data)
            .with_context(|| format!("failed to parse profiles index {}", path.display()))?;

        Ok(index)
    }

    pub fn save_index(&self, index: &ProfilesIndex) -> Result<()> {
        self.ensure_layout()?;
        let path = self.index_path();
        let payload =
            serde_json::to_string_pretty(index).context("failed to serialize profiles index")?;

        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, payload).with_context(|| {
            format!("failed to write temporary profiles index {}", tmp.display())
        })?;
        fs::rename(&tmp, &path)
            .with_context(|| format!("failed to replace profiles index {}", path.display()))?;
        Ok(())
    }

    pub fn create_profile(&self, display_name: &str) -> Result<ProfileRecord> {
        let mut index = self.load_index()?;
        let now = Utc::now().to_rfc3339();

        let mut slug = slugify(display_name);
        if slug.is_empty() {
            slug = "profile".into();
        }
        let id = format!(
            "{}-{}",
            slug,
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        );

        let workspace = self.profiles_root().join(&id);
        let profile = ProfileRecord {
            id: id.clone(),
            display_name: display_name.to_string(),
            workspace_dir: workspace,
            created_at: now.clone(),
            updated_at: now,
        };

        self.ensure_profile_workspace(&profile.workspace_dir)?;
        self.ensure_config_file(&profile.workspace_dir)?;

        index.profiles.push(profile.clone());
        if index.active_profile.is_none() {
            index.active_profile = Some(profile.id.clone());
        }
        self.save_index(&index)?;
        Ok(profile)
    }

    pub fn switch_active_profile(&self, profile_id: &str) -> Result<ProfileRecord> {
        let mut index = self.load_index()?;
        let profile_clone = {
            let Some(profile) = index.profiles.iter_mut().find(|p| p.id == profile_id) else {
                anyhow::bail!("profile '{}' not found", profile_id);
            };
            profile.updated_at = Utc::now().to_rfc3339();
            profile.clone()
        };
        index.active_profile = Some(profile_id.to_string());
        self.save_index(&index)?;
        Ok(profile_clone)
    }

    pub fn get_active_profile(&self) -> Result<Option<ProfileRecord>> {
        let index = self.load_index()?;
        let Some(active_id) = index.active_profile else {
            return Ok(None);
        };
        Ok(index.profiles.into_iter().find(|p| p.id == active_id))
    }

    pub fn workspace_for_profile(&self, profile_id: &str) -> Result<ProfileWorkspace> {
        let index = self.load_index()?;
        let Some(profile) = index.profiles.into_iter().find(|p| p.id == profile_id) else {
            anyhow::bail!("profile '{}' not found", profile_id);
        };

        Ok(ProfileWorkspace {
            root_dir: profile.workspace_dir.clone(),
            config_path: profile.workspace_dir.join("config.toml"),
            memory_dir: profile.workspace_dir.join("memory"),
            logs_dir: profile.workspace_dir.join("logs"),
            skills_dir: profile.workspace_dir.join("skills"),
        })
    }

    pub fn index_path(&self) -> PathBuf {
        self.root_dir.join(PROFILES_INDEX_FILE)
    }

    fn profiles_root(&self) -> PathBuf {
        self.root_dir.join("profiles")
    }

    fn ensure_profile_workspace(&self, workspace_dir: &Path) -> Result<()> {
        fs::create_dir_all(workspace_dir.join("memory")).with_context(|| {
            format!(
                "failed to create {}",
                workspace_dir.join("memory").display()
            )
        })?;
        fs::create_dir_all(workspace_dir.join("logs")).with_context(|| {
            format!("failed to create {}", workspace_dir.join("logs").display())
        })?;
        fs::create_dir_all(workspace_dir.join("skills")).with_context(|| {
            format!(
                "failed to create {}",
                workspace_dir.join("skills").display()
            )
        })?;
        Ok(())
    }

    fn ensure_config_file(&self, workspace_dir: &Path) -> Result<()> {
        let config_path = workspace_dir.join("config.toml");
        if config_path.exists() {
            return Ok(());
        }

        let mut cfg = zeroclaw::Config::default();
        cfg.config_path = config_path;
        cfg.workspace_dir = workspace_dir.to_path_buf();
        cfg.save().context("failed to create profile config.toml")
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_hyphen = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }

    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_profile_initializes_isolated_workspace() {
        let tmp = TempDir::new().unwrap();
        let manager = ProfileManager::new(tmp.path().to_path_buf());

        let profile = manager.create_profile("Primary User").unwrap();
        let workspace = manager.workspace_for_profile(&profile.id).unwrap();

        assert!(workspace.config_path.exists());
        assert!(workspace.memory_dir.exists());
        assert!(workspace.logs_dir.exists());
        assert!(workspace.skills_dir.exists());
    }

    #[test]
    fn switching_profiles_updates_active_profile() {
        let tmp = TempDir::new().unwrap();
        let manager = ProfileManager::new(tmp.path().to_path_buf());

        let a = manager.create_profile("A").unwrap();
        let b = manager.create_profile("B").unwrap();

        manager.switch_active_profile(&b.id).unwrap();
        let active = manager.get_active_profile().unwrap().unwrap();

        assert_eq!(active.id, b.id);
        assert_ne!(active.id, a.id);
    }
}
