use std::path::PathBuf;

use ensembly_types::{IpcRequest, IpcResponse, IpcResponseType, ResponseStatus};
use serde_json::json;
use tokio::sync::mpsc;

use crate::plugin_runtime::PluginRuntime;

// ---------------------------------------------------------------------------
// IpcBridge — channel pair connecting Shell (sender) and Core (receiver).
// ---------------------------------------------------------------------------

pub struct IpcBridge {
    /// Shell writes requests here; Core reads from the other end.
    pub request_tx: mpsc::Sender<IpcRequest>,
    /// Core writes responses here; Shell reads from the other end.
    pub response_rx: mpsc::Receiver<IpcResponse>,
}

/// Returned to the Core so it can run the dispatch loop.
pub struct CoreBridgeHalf {
    request_rx: mpsc::Receiver<IpcRequest>,
    response_tx: mpsc::Sender<IpcResponse>,
    wasm_dir: PathBuf,
}

impl IpcBridge {
    /// Send a request and wait for the matching response.
    /// Returns `None` if the core has shut down.
    pub async fn send_request(&mut self, req: IpcRequest) -> Option<IpcResponse> {
        self.request_tx.send(req).await.ok()?;
        self.response_rx.recv().await
    }
}

/// Create a linked (IpcBridge, CoreBridgeHalf) pair.
/// `wasm_dir` is where compiled plugin `.wasm` files live.
pub fn create(wasm_dir: PathBuf) -> (IpcBridge, CoreBridgeHalf) {
    let (request_tx, request_rx) = mpsc::channel(32);
    let (response_tx, response_rx) = mpsc::channel(32);

    let shell_half = IpcBridge {
        request_tx,
        response_rx,
    };
    let core_half = CoreBridgeHalf {
        request_rx,
        response_tx,
        wasm_dir,
    };

    (shell_half, core_half)
}

// ---------------------------------------------------------------------------
// Dispatch loop — runs as a tokio task for the app lifetime.
// ---------------------------------------------------------------------------

impl CoreBridgeHalf {
    pub async fn run(mut self) {
        let runtime = match PluginRuntime::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[core] failed to initialise PluginRuntime: {e}");
                return;
            }
        };

        while let Some(request) = self.request_rx.recv().await {
            let response = dispatch(&runtime, &self.wasm_dir, request).await;
            if self.response_tx.send(response).await.is_err() {
                // Shell dropped its receiver — time to shut down.
                break;
            }
        }
    }
}

async fn dispatch(runtime: &PluginRuntime, wasm_dir: &PathBuf, req: IpcRequest) -> IpcResponse {
    let payload = match req.action.as_str() {
        "PING" => {
            ok_response(&req.message_id, json!({ "message": "pong from core" }))
        }

        "RUN_FEATURE_PLUGIN" => {
            let wasm_path = wasm_dir.join("hello_feature.wasm");
            match runtime.load_feature_plugin(&wasm_path) {
                Ok(mut plugin) => match plugin.call_run() {
                    Ok(json_str) => match serde_json::from_str(&json_str) {
                        Ok(payload) => ok_response(&req.message_id, payload),
                        Err(e) => err_response(&req.message_id, &format!("INVALID_JSON: {e}")),
                    },
                    Err(e) => err_response(&req.message_id, &format!("PLUGIN_ERROR: {e}")),
                },
                Err(e) => err_response(&req.message_id, &format!("LOAD_ERROR: {e}")),
            }
        }

        unknown => err_response(
            &req.message_id,
            &format!("UNKNOWN_ACTION: {unknown}"),
        ),
    };

    payload
}

fn ok_response(message_id: &str, payload: serde_json::Value) -> IpcResponse {
    IpcResponse {
        message_id: message_id.to_string(),
        message_type: IpcResponseType::Response,
        status: ResponseStatus::Success,
        payload,
    }
}

fn err_response(message_id: &str, error_code: &str) -> IpcResponse {
    IpcResponse {
        message_id: message_id.to_string(),
        message_type: IpcResponseType::Response,
        status: ResponseStatus::Error,
        payload: json!({ "error_code": error_code }),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ensembly_types::{IpcRequestType, ResponseStatus};
    use serde_json::json;

    fn make_request(action: &str) -> IpcRequest {
        IpcRequest {
            message_id: "test-msg-001".into(),
            message_type: IpcRequestType::Request,
            action: action.into(),
            plugin_id: None,
            payload: json!(null),
        }
    }

    #[tokio::test]
    async fn ping_returns_pong() {
        let runtime = PluginRuntime::new().unwrap();
        let wasm_dir = PathBuf::from("/nonexistent"); // not needed for PING
        let resp = dispatch(&runtime, &wasm_dir, make_request("PING")).await;
        assert_eq!(resp.status, ResponseStatus::Success);
        assert_eq!(resp.payload["message"], "pong from core");
        assert_eq!(resp.message_id, "test-msg-001");
    }

    #[tokio::test]
    async fn unknown_action_returns_error() {
        let runtime = PluginRuntime::new().unwrap();
        let wasm_dir = PathBuf::from("/nonexistent");
        let resp = dispatch(&runtime, &wasm_dir, make_request("DOES_NOT_EXIST")).await;
        assert_eq!(resp.status, ResponseStatus::Error);
        assert!(resp.payload["error_code"]
            .as_str()
            .unwrap()
            .contains("UNKNOWN_ACTION"));
    }

    #[tokio::test]
    async fn run_feature_plugin_with_built_wasm() {
        // Only runs if the wasm binary has been built.
        let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/wasm32-wasip2/debug/hello_feature.wasm");
        if !wasm_path.exists() {
            println!("Skipping: hello_feature.wasm not built yet");
            return;
        }
        let wasm_dir = wasm_path.parent().unwrap().to_path_buf();
        let runtime = PluginRuntime::new().unwrap();
        let resp = dispatch(&runtime, &wasm_dir, make_request("RUN_FEATURE_PLUGIN")).await;
        assert_eq!(resp.status, ResponseStatus::Success);
        assert_eq!(resp.payload["greeting"], "Hello from the Feature Plugin!");
    }
}
