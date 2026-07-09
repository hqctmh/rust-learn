use serde_json::{Value, json};
use uuid::Uuid;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgentEvent {
    pub event: String,
    pub data: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PersistenceAction {
    AppendText(String),
    InsertRaw {
        event_type: String,
        response: String,
    },
}

impl AgentEvent {
    pub fn from_sse(event: &str, data: &str) -> Self {
        Self {
            event: normalized_event_name(event, data),
            data: data.to_string(),
        }
    }

    pub fn turn_created(conversation_id: Uuid, turn_id: Uuid) -> Self {
        Self {
            event: "turn_created".to_string(),
            data: json!({
                "type": "turn_created",
                "conversation_id": conversation_id,
                "turn_id": turn_id,
            })
            .to_string(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            event: "error".to_string(),
            data: json!({
                "type": "error",
                "message": message.into(),
            })
            .to_string(),
        }
    }

    pub fn persistence_action(&self) -> PersistenceAction {
        if self.event == "text"
            && let Some(content) = text_content(&self.data)
        {
            return PersistenceAction::AppendText(content);
        }

        PersistenceAction::InsertRaw {
            event_type: self.event.clone(),
            response: self.data.clone(),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self.event.as_str(), "run_completed" | "error")
    }
}

fn normalized_event_name(event: &str, data: &str) -> String {
    if !event.is_empty() && event != "message" {
        return event.to_string();
    }

    serde_json::from_str::<Value>(data)
        .ok()
        .and_then(|value| value.get("type")?.as_str().map(str::to_string))
        .unwrap_or_else(|| {
            if event.is_empty() {
                "message".to_string()
            } else {
                event.to_string()
            }
        })
}

fn text_content(data: &str) -> Option<String> {
    serde_json::from_str::<Value>(data)
        .ok()?
        .get("content")?
        .as_str()
        .map(str::to_string)
}
