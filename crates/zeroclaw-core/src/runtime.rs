use crate::events::{EventBus, RuntimeEvent, RuntimeEventKind};
use crate::lifecycle::{AgentState, LifecycleController};
use crate::logs::{LogLine, LogSink};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, oneshot, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStartConfig {
    pub profile_id: String,
    pub config_path: PathBuf,
    pub workspace_dir: PathBuf,
}

#[async_trait]
pub trait AgentRuntime: Send + Sync {
    async fn start(&self, config: RuntimeStartConfig) -> Result<()>;
    async fn stop(&self, reason: &str) -> Result<()>;
    async fn send_user_message(&self, message: &str) -> Result<String>;
    fn subscribe_events(&self) -> broadcast::Receiver<RuntimeEvent>;
    fn state(&self) -> AgentState;
}

#[async_trait]
pub trait AgentSession: Send + Sync {
    async fn run_message(&mut self, message: &str) -> Result<String>;
}

pub trait AgentSessionFactory: Send + Sync {
    fn create_session(&self, config: &zeroclaw::Config) -> Result<Box<dyn AgentSession>>;
}

pub struct ZeroclawAgentSession {
    inner: zeroclaw::agent::Agent,
}

#[async_trait]
impl AgentSession for ZeroclawAgentSession {
    async fn run_message(&mut self, message: &str) -> Result<String> {
        self.inner.run_single(message).await
    }
}

pub struct ZeroclawAgentSessionFactory;

impl AgentSessionFactory for ZeroclawAgentSessionFactory {
    fn create_session(&self, config: &zeroclaw::Config) -> Result<Box<dyn AgentSession>> {
        let agent = zeroclaw::agent::Agent::from_config(config)
            .context("failed to create zeroclaw agent session")?;
        Ok(Box::new(ZeroclawAgentSession { inner: agent }))
    }
}

struct RuntimeInner {
    profile_id: Option<String>,
    session: Option<Box<dyn AgentSession>>,
    health_shutdown: Option<oneshot::Sender<()>>,
    health_task: Option<tokio::task::JoinHandle<()>>,
}

impl RuntimeInner {
    fn new() -> Self {
        Self {
            profile_id: None,
            session: None,
            health_shutdown: None,
            health_task: None,
        }
    }
}

pub struct LocalAgentRuntime {
    event_bus: EventBus,
    lifecycle: Arc<LifecycleController>,
    log_sink: Arc<dyn LogSink>,
    factory: Arc<dyn AgentSessionFactory>,
    inner: Mutex<RuntimeInner>,
}

impl LocalAgentRuntime {
    pub fn new(log_sink: Arc<dyn LogSink>) -> Self {
        Self::with_factory(log_sink, Arc::new(ZeroclawAgentSessionFactory))
    }

    pub fn with_factory(log_sink: Arc<dyn LogSink>, factory: Arc<dyn AgentSessionFactory>) -> Self {
        Self {
            event_bus: EventBus::default(),
            lifecycle: Arc::new(LifecycleController::default()),
            log_sink,
            factory,
            inner: Mutex::new(RuntimeInner::new()),
        }
    }

    fn publish(&self, event: RuntimeEvent) {
        self.event_bus.publish(event);
    }

    fn write_log(&self, profile_id: &str, level: &str, component: &str, message: &str) {
        let mut line = LogLine::new(level, component, message);
        line.fields.insert(
            "profile_id".into(),
            serde_json::Value::String(profile_id.to_string()),
        );
        if let Err(error) = self.log_sink.write(&line) {
            tracing::warn!("failed to write runtime log: {error}");
        }

        self.publish(RuntimeEvent::new(
            profile_id,
            RuntimeEventKind::LogLine {
                level: level.to_string(),
                component: component.to_string(),
                message: message.to_string(),
            },
        ));
    }

    fn transition_state(
        &self,
        profile_id: &str,
        target: AgentState,
        reason: Option<String>,
    ) -> Result<()> {
        let from = self.lifecycle.snapshot().state;
        self.lifecycle.transition(target, reason.clone())?;
        self.publish(RuntimeEvent::new(
            profile_id,
            RuntimeEventKind::StateChanged {
                from: from.as_str().to_string(),
                to: target.as_str().to_string(),
            },
        ));
        Ok(())
    }
}

#[async_trait]
impl AgentRuntime for LocalAgentRuntime {
    async fn start(&self, config: RuntimeStartConfig) -> Result<()> {
        if self.lifecycle.snapshot().state != AgentState::Stopped {
            anyhow::bail!("runtime is already active");
        }

        self.transition_state(&config.profile_id, AgentState::Starting, None)?;
        self.write_log(
            &config.profile_id,
            "info",
            "runtime",
            "starting runtime session",
        );

        let loaded = load_profile_config(&config.config_path, &config.workspace_dir)?;
        let session = match self.factory.create_session(&loaded) {
            Ok(session) => session,
            Err(error) => {
                let message = error.to_string();
                let _ = self.transition_state(
                    &config.profile_id,
                    AgentState::Degraded,
                    Some(message.clone()),
                );
                self.publish(RuntimeEvent::new(
                    &config.profile_id,
                    RuntimeEventKind::Error {
                        component: "runtime_start".into(),
                        message: message.clone(),
                    },
                ));
                self.write_log(&config.profile_id, "error", "runtime", &message);
                return Err(error);
            }
        };

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
        let profile_id = config.profile_id.clone();
        let bus = self.event_bus.clone();
        let lifecycle = Arc::clone(&self.lifecycle);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(Duration::from_secs(30));
            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let state = lifecycle.snapshot().state.as_str().to_string();
                        bus.publish(RuntimeEvent::new(
                            &profile_id,
                            RuntimeEventKind::HealthTick { state },
                        ));
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        let mut inner = self.inner.lock().await;
        inner.profile_id = Some(config.profile_id.clone());
        inner.session = Some(session);
        inner.health_shutdown = Some(shutdown_tx);
        inner.health_task = Some(handle);
        drop(inner);

        self.transition_state(&config.profile_id, AgentState::Running, None)?;
        self.write_log(&config.profile_id, "info", "runtime", "runtime is running");

        Ok(())
    }

    async fn stop(&self, reason: &str) -> Result<()> {
        let profile_id = {
            let guard = self.inner.lock().await;
            guard
                .profile_id
                .clone()
                .unwrap_or_else(|| "unknown-profile".to_string())
        };

        let current = self.lifecycle.snapshot().state;
        if current == AgentState::Stopped {
            return Ok(());
        }

        self.transition_state(&profile_id, AgentState::Stopping, Some(reason.to_string()))?;

        let (shutdown, handle) = {
            let mut guard = self.inner.lock().await;
            guard.session = None;
            guard.profile_id = None;
            (guard.health_shutdown.take(), guard.health_task.take())
        };

        if let Some(tx) = shutdown {
            let _ = tx.send(());
        }
        if let Some(task) = handle {
            let _ = task.await;
        }

        self.publish(RuntimeEvent::new(
            &profile_id,
            RuntimeEventKind::Shutdown {
                reason: reason.to_string(),
            },
        ));
        self.transition_state(&profile_id, AgentState::Stopped, Some(reason.to_string()))?;
        self.write_log(&profile_id, "info", "runtime", "runtime stopped");

        Ok(())
    }

    async fn send_user_message(&self, message: &str) -> Result<String> {
        let state = self.lifecycle.snapshot().state;
        if !matches!(state, AgentState::Running | AgentState::Degraded) {
            anyhow::bail!("runtime is not running");
        }

        let task_id = uuid::Uuid::new_v4().to_string();

        let (profile_id, response) = {
            let mut guard = self.inner.lock().await;
            let profile_id = guard
                .profile_id
                .clone()
                .unwrap_or_else(|| "unknown-profile".into());
            let Some(session) = guard.session.as_mut() else {
                anyhow::bail!("runtime session not initialized");
            };

            self.publish(RuntimeEvent::new(
                &profile_id,
                RuntimeEventKind::TaskStarted {
                    task_id: task_id.clone(),
                    message: message.to_string(),
                },
            ));
            self.write_log(&profile_id, "info", "agent", "task started");

            let response = session.run_message(message).await;
            (profile_id, response)
        };

        match response {
            Ok(output) => {
                self.publish(RuntimeEvent::new(
                    &profile_id,
                    RuntimeEventKind::TaskFinished {
                        task_id,
                        success: true,
                    },
                ));
                self.write_log(&profile_id, "info", "agent", "task finished");
                Ok(output)
            }
            Err(error) => {
                let message = error.to_string();
                self.publish(RuntimeEvent::new(
                    &profile_id,
                    RuntimeEventKind::Error {
                        component: "agent".into(),
                        message: message.clone(),
                    },
                ));
                self.write_log(&profile_id, "error", "agent", &message);
                let _ =
                    self.transition_state(&profile_id, AgentState::Degraded, Some(message.clone()));
                Err(error)
            }
        }
    }

    fn subscribe_events(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.event_bus.subscribe()
    }

    fn state(&self) -> AgentState {
        self.lifecycle.snapshot().state
    }
}

fn load_profile_config(config_path: &Path, workspace_dir: &Path) -> Result<zeroclaw::Config> {
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
    use crate::logs::{JsonlLogSink, LogSinkConfig};
    use tempfile::TempDir;

    struct MockSession {
        fail: bool,
    }

    #[async_trait]
    impl AgentSession for MockSession {
        async fn run_message(&mut self, message: &str) -> Result<String> {
            if self.fail {
                anyhow::bail!("simulated session failure");
            }
            Ok(format!("echo:{message}"))
        }
    }

    struct MockFactory {
        fail: bool,
    }

    impl AgentSessionFactory for MockFactory {
        fn create_session(&self, _config: &zeroclaw::Config) -> Result<Box<dyn AgentSession>> {
            Ok(Box::new(MockSession { fail: self.fail }))
        }
    }

    fn runtime_with_factory(tmp: &TempDir, fail: bool) -> LocalAgentRuntime {
        let sink =
            Arc::new(JsonlLogSink::new(LogSinkConfig::new(tmp.path().join("logs"))).unwrap());
        LocalAgentRuntime::with_factory(sink, Arc::new(MockFactory { fail }))
    }

    fn start_config(tmp: &TempDir) -> RuntimeStartConfig {
        RuntimeStartConfig {
            profile_id: "profile-a".into(),
            config_path: tmp.path().join("workspace").join("config.toml"),
            workspace_dir: tmp.path().join("workspace"),
        }
    }

    #[tokio::test]
    async fn start_send_and_stop_runtime() {
        let tmp = TempDir::new().unwrap();
        let runtime = runtime_with_factory(&tmp, false);

        runtime.start(start_config(&tmp)).await.unwrap();
        let result = runtime.send_user_message("hi").await.unwrap();
        runtime.stop("test complete").await.unwrap();

        assert_eq!(result, "echo:hi");
        assert_eq!(runtime.state(), AgentState::Stopped);
    }

    #[tokio::test]
    async fn runtime_moves_to_degraded_on_task_error() {
        let tmp = TempDir::new().unwrap();
        let runtime = runtime_with_factory(&tmp, true);

        runtime.start(start_config(&tmp)).await.unwrap();
        let err = runtime.send_user_message("hi").await.unwrap_err();

        assert!(err.to_string().contains("simulated session failure"));
        assert_eq!(runtime.state(), AgentState::Degraded);
    }
}
