use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BridgeRequest {
    pub id: String,
    pub cmd: String,
    #[serde(flatten)]
    pub payload: serde_json::Map<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BridgeResponse {
    pub id: Option<String>,
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub data: Option<Value>,
    pub event: Option<String>,
    pub phase: Option<String>,
    pub message: Option<String>,
    pub current: Option<usize>,
    pub total: Option<usize>,
}

impl BridgeRequest {
    pub fn new(id: impl Into<String>, cmd: impl Into<String>, payload: Value) -> Self {
        let mut map = serde_json::Map::new();
        if let Value::Object(obj) = payload {
            map = obj;
        }
        Self {
            id: id.into(),
            cmd: cmd.into(),
            payload: map,
        }
    }
}

impl BridgeResponse {
    pub fn is_progress_event(&self) -> bool {
        self.event.as_deref() == Some("progress")
    }
}
