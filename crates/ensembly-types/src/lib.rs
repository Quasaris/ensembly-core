use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Base Item Schema
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnsemblyItem {
    pub system: SystemMeta,
    pub core: CoreData,
    #[serde(default)]
    pub attributes: HashMap<String, Value>,
    #[serde(default)]
    pub relations: HashMap<String, Vec<Relation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMeta {
    pub id: String,
    pub collection_id: String,
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoreData {
    pub title: String,
    pub primary_image: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub relation_type: String,
    pub target_id: String,
    pub target_collection: String,
}

// ---------------------------------------------------------------------------
// IPC Bridge Messages
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcRequest {
    pub message_id: String,
    #[serde(rename = "type")]
    pub message_type: IpcRequestType,
    pub action: String,
    pub plugin_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IpcRequestType {
    Request,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcResponse {
    pub message_id: String,
    #[serde(rename = "type")]
    pub message_type: IpcResponseType,
    pub status: ResponseStatus,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IpcResponseType {
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IpcEvent {
    pub message_id: String,
    #[serde(rename = "type")]
    pub message_type: IpcEventType,
    pub action: String,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum IpcEventType {
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ResponseStatus {
    Success,
    Error,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_item() -> EnsemblyItem {
        EnsemblyItem {
            system: SystemMeta {
                id: "item-001".into(),
                collection_id: "books".into(),
                schema_version: 1,
                created_at: "2026-04-16T00:00:00Z".into(),
                updated_at: "2026-04-16T00:00:00Z".into(),
            },
            core: CoreData {
                title: "The Name of the Wind".into(),
                primary_image: Some("cover.jpg".into()),
                tags: vec!["fantasy".into(), "fiction".into()],
                description: "A hero's story told in his own words.".into(),
            },
            attributes: HashMap::new(),
            relations: HashMap::new(),
        }
    }

    #[test]
    fn item_round_trip() {
        let item = sample_item();
        let json = serde_json::to_string(&item).unwrap();
        let restored: EnsemblyItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, restored);
    }

    #[test]
    fn ipc_request_round_trip() {
        let req = IpcRequest {
            message_id: "msg-001".into(),
            message_type: IpcRequestType::Request,
            action: "RUN_FEATURE_PLUGIN".into(),
            plugin_id: Some("hello-feature".into()),
            payload: json!({ "wasm": "hello_feature.wasm" }),
        };
        let json = serde_json::to_string(&req).unwrap();
        let restored: IpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req, restored);
    }

    #[test]
    fn ipc_response_round_trip() {
        let resp = IpcResponse {
            message_id: "msg-001".into(),
            message_type: IpcResponseType::Response,
            status: ResponseStatus::Success,
            payload: json!({ "greeting": "Hello from the Feature Plugin!" }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let restored: IpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, restored);
    }

    #[test]
    fn ipc_event_round_trip() {
        let event = IpcEvent {
            message_id: "evt-001".into(),
            message_type: IpcEventType::Event,
            action: "ITEM_UPDATED".into(),
            payload: json!({ "id": "item-001" }),
        };
        let json = serde_json::to_string(&event).unwrap();
        let restored: IpcEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn response_status_serialises_as_uppercase() {
        assert_eq!(
            serde_json::to_string(&ResponseStatus::Success).unwrap(),
            "\"SUCCESS\""
        );
        assert_eq!(
            serde_json::to_string(&ResponseStatus::Error).unwrap(),
            "\"ERROR\""
        );
    }

    #[test]
    fn ipc_type_field_serialises_correctly() {
        let req = IpcRequest {
            message_id: "x".into(),
            message_type: IpcRequestType::Request,
            action: "PING".into(),
            plugin_id: None,
            payload: json!(null),
        };
        let v: Value = serde_json::to_value(&req).unwrap();
        assert_eq!(v["type"], "REQUEST");
    }
}
