use anyhow::{Context, Result};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub trait SecretVault: Send + Sync {
    fn backend_name(&self) -> &str;
    fn set_secret(&self, profile_id: &str, key: &str, value: &str) -> Result<()>;
    fn get_secret(&self, profile_id: &str, key: &str) -> Result<Option<String>>;
    fn delete_secret(&self, profile_id: &str, key: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct KeyringSecretVault {
    service_name: String,
}

impl KeyringSecretVault {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    fn entry(&self, profile_id: &str, key: &str) -> Result<keyring::Entry> {
        let username = format!("{profile_id}:{key}");
        keyring::Entry::new(&self.service_name, &username).context("failed to open keyring entry")
    }
}

impl SecretVault for KeyringSecretVault {
    fn backend_name(&self) -> &str {
        "keyring"
    }

    fn set_secret(&self, profile_id: &str, key: &str, value: &str) -> Result<()> {
        self.entry(profile_id, key)?
            .set_password(value)
            .context("failed to write keyring secret")
    }

    fn get_secret(&self, profile_id: &str, key: &str) -> Result<Option<String>> {
        match self.entry(profile_id, key)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(anyhow::Error::new(error).context("failed to read keyring secret")),
        }
    }

    fn delete_secret(&self, profile_id: &str, key: &str) -> Result<()> {
        match self.entry(profile_id, key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(anyhow::Error::new(error).context("failed to delete keyring secret")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FileSecretMap {
    values: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct EncryptedFileSecretVault {
    data_path: PathBuf,
    store: zeroclaw::security::SecretStore,
    lock: std::sync::Arc<Mutex<()>>,
}

impl EncryptedFileSecretVault {
    pub fn new(root_dir: impl AsRef<Path>, encrypt: bool) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        fs::create_dir_all(&root_dir)
            .with_context(|| format!("failed to create secret root {}", root_dir.display()))?;

        let data_path = root_dir.join("vault.json");
        let store = zeroclaw::security::SecretStore::new(&root_dir, encrypt);
        Ok(Self {
            data_path,
            store,
            lock: std::sync::Arc::new(Mutex::new(())),
        })
    }

    fn entry_key(profile_id: &str, key: &str) -> String {
        format!("{profile_id}::{key}")
    }

    fn load_map(&self) -> Result<FileSecretMap> {
        if !self.data_path.exists() {
            return Ok(FileSecretMap::default());
        }

        let body = fs::read_to_string(&self.data_path)
            .with_context(|| format!("failed to read {}", self.data_path.display()))?;
        serde_json::from_str(&body).context("failed to parse encrypted vault file")
    }

    fn save_map(&self, map: &FileSecretMap) -> Result<()> {
        let body = serde_json::to_string_pretty(map).context("failed to serialize vault map")?;
        let tmp = self.data_path.with_extension("json.tmp");
        fs::write(&tmp, body).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.data_path)
            .with_context(|| format!("failed to replace {}", self.data_path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.data_path, fs::Permissions::from_mode(0o600));
        }

        Ok(())
    }
}

impl SecretVault for EncryptedFileSecretVault {
    fn backend_name(&self) -> &str {
        "encrypted-file"
    }

    fn set_secret(&self, profile_id: &str, key: &str, value: &str) -> Result<()> {
        let _guard = self.lock.lock();
        let mut map = self.load_map()?;
        let encrypted = self
            .store
            .encrypt(value)
            .with_context(|| format!("failed to encrypt secret {key}"))?;
        map.values
            .insert(Self::entry_key(profile_id, key), encrypted);
        self.save_map(&map)
    }

    fn get_secret(&self, profile_id: &str, key: &str) -> Result<Option<String>> {
        let _guard = self.lock.lock();
        let map = self.load_map()?;
        let Some(raw) = map.values.get(&Self::entry_key(profile_id, key)) else {
            return Ok(None);
        };

        let value = self
            .store
            .decrypt(raw)
            .with_context(|| format!("failed to decrypt secret {key}"))?;
        Ok(Some(value))
    }

    fn delete_secret(&self, profile_id: &str, key: &str) -> Result<()> {
        let _guard = self.lock.lock();
        let mut map = self.load_map()?;
        map.values.remove(&Self::entry_key(profile_id, key));
        self.save_map(&map)
    }
}

#[derive(Debug, Clone)]
pub struct AdaptiveSecretVault {
    keyring: KeyringSecretVault,
    fallback: EncryptedFileSecretVault,
}

impl AdaptiveSecretVault {
    pub fn new(app_root: impl AsRef<Path>) -> Result<Self> {
        let app_root = app_root.as_ref().to_path_buf();
        let keyring = KeyringSecretVault::new("zeroclaw.app");
        let fallback = EncryptedFileSecretVault::new(app_root.join("secrets"), true)?;
        Ok(Self { keyring, fallback })
    }
}

impl SecretVault for AdaptiveSecretVault {
    fn backend_name(&self) -> &str {
        "adaptive"
    }

    fn set_secret(&self, profile_id: &str, key: &str, value: &str) -> Result<()> {
        match self.keyring.set_secret(profile_id, key, value) {
            Ok(()) => Ok(()),
            Err(error) => {
                tracing::warn!("keyring set failed, falling back to encrypted file: {error}");
                self.fallback.set_secret(profile_id, key, value)
            }
        }
    }

    fn get_secret(&self, profile_id: &str, key: &str) -> Result<Option<String>> {
        match self.keyring.get_secret(profile_id, key) {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.fallback.get_secret(profile_id, key),
            Err(error) => {
                tracing::warn!("keyring get failed, falling back to encrypted file: {error}");
                self.fallback.get_secret(profile_id, key)
            }
        }
    }

    fn delete_secret(&self, profile_id: &str, key: &str) -> Result<()> {
        let keyring_res = self.keyring.delete_secret(profile_id, key);
        let file_res = self.fallback.delete_secret(profile_id, key);

        if let Err(error) = keyring_res {
            tracing::warn!("keyring delete failed: {error}");
        }

        file_res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn encrypted_file_vault_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let vault = EncryptedFileSecretVault::new(tmp.path(), true).unwrap();

        vault
            .set_secret("profile-a", "openai_api_key", "sk-test-value")
            .unwrap();
        let value = vault
            .get_secret("profile-a", "openai_api_key")
            .unwrap()
            .unwrap();

        assert_eq!(value, "sk-test-value");

        vault.delete_secret("profile-a", "openai_api_key").unwrap();
        assert!(vault
            .get_secret("profile-a", "openai_api_key")
            .unwrap()
            .is_none());
    }
}
