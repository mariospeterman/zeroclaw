use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundCapabilities {
    pub supports_always_on: bool,
    pub requires_ongoing_notification: bool,
    pub best_effort_only: bool,
}

pub trait PlatformBackground: Send + Sync {
    fn platform_name(&self) -> &'static str;
    fn capabilities(&self) -> BackgroundCapabilities;
    fn enable_background_mode(&self) -> Result<()>;
    fn disable_background_mode(&self) -> Result<()>;
    fn schedule_wakeup(&self, reason: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct DesktopBackgroundAdapter {
    config_path: PathBuf,
    workspace_dir: PathBuf,
}

impl DesktopBackgroundAdapter {
    pub fn new(config_path: PathBuf, workspace_dir: PathBuf) -> Self {
        Self {
            config_path,
            workspace_dir,
        }
    }

    fn load_config(&self) -> Result<zeroclaw::Config> {
        load_config(&self.config_path, &self.workspace_dir)
    }
}

impl PlatformBackground for DesktopBackgroundAdapter {
    fn platform_name(&self) -> &'static str {
        "desktop"
    }

    fn capabilities(&self) -> BackgroundCapabilities {
        BackgroundCapabilities {
            supports_always_on: true,
            requires_ongoing_notification: false,
            best_effort_only: false,
        }
    }

    fn enable_background_mode(&self) -> Result<()> {
        let cfg = self.load_config()?;
        zeroclaw::service::handle_command(&zeroclaw::ServiceCommands::Install, &cfg)
            .context("failed to install desktop service")?;
        zeroclaw::service::handle_command(&zeroclaw::ServiceCommands::Start, &cfg)
            .context("failed to start desktop service")?;
        Ok(())
    }

    fn disable_background_mode(&self) -> Result<()> {
        let cfg = self.load_config()?;
        zeroclaw::service::handle_command(&zeroclaw::ServiceCommands::Stop, &cfg)
            .context("failed to stop desktop service")
    }

    fn schedule_wakeup(&self, _reason: &str) -> Result<()> {
        // Desktop service can run continuously; explicit wake scheduling is not required.
        Ok(())
    }
}

pub struct AndroidBackgroundAdapter;

impl PlatformBackground for AndroidBackgroundAdapter {
    fn platform_name(&self) -> &'static str {
        "android"
    }

    fn capabilities(&self) -> BackgroundCapabilities {
        BackgroundCapabilities {
            supports_always_on: false,
            requires_ongoing_notification: true,
            best_effort_only: false,
        }
    }

    fn enable_background_mode(&self) -> Result<()> {
        anyhow::bail!(
            "android background mode requires a native foreground-service bridge and explicit user action"
        )
    }

    fn disable_background_mode(&self) -> Result<()> {
        anyhow::bail!("android background mode is controlled by the mobile shell bridge")
    }

    fn schedule_wakeup(&self, _reason: &str) -> Result<()> {
        anyhow::bail!("android wake scheduling should be implemented via WorkManager bridge")
    }
}

pub struct IosBackgroundAdapter;

impl PlatformBackground for IosBackgroundAdapter {
    fn platform_name(&self) -> &'static str {
        "ios"
    }

    fn capabilities(&self) -> BackgroundCapabilities {
        BackgroundCapabilities {
            supports_always_on: false,
            requires_ongoing_notification: false,
            best_effort_only: true,
        }
    }

    fn enable_background_mode(&self) -> Result<()> {
        anyhow::bail!("ios background execution is best-effort and requires BGTaskScheduler bridge")
    }

    fn disable_background_mode(&self) -> Result<()> {
        Ok(())
    }

    fn schedule_wakeup(&self, _reason: &str) -> Result<()> {
        anyhow::bail!("ios wake scheduling should be implemented via BGTaskScheduler in app shell")
    }
}

fn load_config(config_path: &Path, workspace_dir: &Path) -> Result<zeroclaw::Config> {
    if config_path.exists() {
        let data = std::fs::read_to_string(config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        let mut cfg: zeroclaw::Config =
            toml::from_str(&data).context("failed to parse profile config")?;
        cfg.config_path = config_path.to_path_buf();
        cfg.workspace_dir = workspace_dir.to_path_buf();
        cfg.apply_env_overrides();
        return Ok(cfg);
    }

    let mut cfg = zeroclaw::Config::default();
    cfg.config_path = config_path.to_path_buf();
    cfg.workspace_dir = workspace_dir.to_path_buf();
    cfg.save().context("failed to initialize profile config")?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_adapters_expose_expected_capabilities() {
        let android = AndroidBackgroundAdapter;
        let ios = IosBackgroundAdapter;

        assert!(android.capabilities().requires_ongoing_notification);
        assert!(ios.capabilities().best_effort_only);
    }
}
