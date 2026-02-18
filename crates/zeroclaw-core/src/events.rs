use crate::protocol::EVENT_SCHEMA_VERSION;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuntimeEventKind {
    TaskStarted {
        task_id: String,
        message: String,
    },
    TaskFinished {
        task_id: String,
        success: bool,
    },
    Error {
        component: String,
        message: String,
    },
    Shutdown {
        reason: String,
    },
    HealthTick {
        state: String,
    },
    LogLine {
        level: String,
        component: String,
        message: String,
    },
    StateChanged {
        from: String,
        to: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEvent {
    pub id: String,
    pub schema_version: u32,
    pub profile_id: String,
    pub timestamp: String,
    pub kind: RuntimeEventKind,
}

impl RuntimeEvent {
    pub fn new(profile_id: impl Into<String>, kind: RuntimeEventKind) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            schema_version: EVENT_SCHEMA_VERSION,
            profile_id: profile_id.into(),
            timestamp: Utc::now().to_rfc3339(),
            kind,
        }
    }
}

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<RuntimeEvent>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let capacity = buffer.max(16);
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: RuntimeEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.tx.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn event_bus_delivers_published_events() {
        let bus = EventBus::new(16);
        let mut sub = bus.subscribe();

        bus.publish(RuntimeEvent::new(
            "profile-a",
            RuntimeEventKind::HealthTick {
                state: "running".into(),
            },
        ));

        let event = sub.recv().await.unwrap();
        assert_eq!(event.profile_id, "profile-a");
        assert_eq!(event.schema_version, EVENT_SCHEMA_VERSION);
        assert!(matches!(event.kind, RuntimeEventKind::HealthTick { .. }));
    }
}
