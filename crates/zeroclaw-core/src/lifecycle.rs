use anyhow::Result;
use chrono::Utc;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Stopped,
    Starting,
    Running,
    Degraded,
    Stopping,
}

impl AgentState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Degraded => "degraded",
            Self::Stopping => "stopping",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSnapshot {
    pub state: AgentState,
    pub reason: Option<String>,
    pub updated_at: String,
}

pub struct LifecycleController {
    state: Mutex<LifecycleSnapshot>,
}

impl LifecycleController {
    pub fn new(initial: AgentState) -> Self {
        Self {
            state: Mutex::new(LifecycleSnapshot {
                state: initial,
                reason: None,
                updated_at: Utc::now().to_rfc3339(),
            }),
        }
    }

    pub fn snapshot(&self) -> LifecycleSnapshot {
        self.state.lock().clone()
    }

    pub fn transition(
        &self,
        target: AgentState,
        reason: Option<String>,
    ) -> Result<LifecycleSnapshot> {
        let mut guard = self.state.lock();
        let from = guard.state;

        if from == target {
            guard.reason = reason;
            guard.updated_at = Utc::now().to_rfc3339();
            return Ok(guard.clone());
        }

        if !is_valid_transition(from, target) {
            anyhow::bail!(
                "invalid lifecycle transition: {} -> {}",
                from.as_str(),
                target.as_str()
            );
        }

        guard.state = target;
        guard.reason = reason;
        guard.updated_at = Utc::now().to_rfc3339();
        Ok(guard.clone())
    }
}

fn is_valid_transition(from: AgentState, to: AgentState) -> bool {
    matches!(
        (from, to),
        (AgentState::Stopped, AgentState::Starting)
            | (AgentState::Starting, AgentState::Running)
            | (AgentState::Starting, AgentState::Degraded)
            | (AgentState::Starting, AgentState::Stopping)
            | (AgentState::Running, AgentState::Degraded)
            | (AgentState::Running, AgentState::Stopping)
            | (AgentState::Degraded, AgentState::Running)
            | (AgentState::Degraded, AgentState::Stopping)
            | (AgentState::Stopping, AgentState::Stopped)
    )
}

impl Default for LifecycleController {
    fn default() -> Self {
        Self::new(AgentState::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transitions_are_accepted() {
        let lifecycle = LifecycleController::default();

        lifecycle.transition(AgentState::Starting, None).unwrap();
        lifecycle.transition(AgentState::Running, None).unwrap();
        lifecycle.transition(AgentState::Stopping, None).unwrap();
        lifecycle.transition(AgentState::Stopped, None).unwrap();

        assert_eq!(lifecycle.snapshot().state, AgentState::Stopped);
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let lifecycle = LifecycleController::default();
        let err = lifecycle.transition(AgentState::Running, None).unwrap_err();

        assert!(err.to_string().contains("invalid lifecycle transition"));
    }
}
