use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use dioxus::prelude::*;
use ensembly_core::ipc::{self, CoreBridgeHalf, IpcBridge};
use ensembly_types::{IpcRequest, IpcRequestType};

// Set once in main(), read from App component.
// Arc<Mutex<>> makes IpcBridge shareable across async closures.
static BRIDGE: OnceLock<Arc<tokio::sync::Mutex<IpcBridge>>> = OnceLock::new();
static CORE_HALF: OnceLock<std::sync::Mutex<Option<CoreBridgeHalf>>> = OnceLock::new();

fn main() {
    let wasm_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/wasm32-wasip2/debug");

    let (bridge, core_half) = ipc::create(wasm_dir);

    BRIDGE.set(Arc::new(tokio::sync::Mutex::new(bridge))).ok();
    CORE_HALF
        .set(std::sync::Mutex::new(Some(core_half)))
        .ok();

    dioxus::launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    // Spawn the IPC dispatch loop once on first render.
    use_hook(|| {
        if let Some(ch) = CORE_HALF.get().unwrap().lock().unwrap().take() {
            spawn(ch.run());
        }
    });

    let mut result: Signal<Option<serde_json::Value>> = use_signal(|| None);

    rsx! {
        div {
            style: "display: flex; height: 100vh; font-family: system-ui, sans-serif; background: #F6F4F0; margin: 0; box-sizing: border-box;",

            // Sidebar
            div {
                style: "width: 200px; padding: 24px 16px; border-right: 1px solid rgba(23,23,23,0.2); flex-shrink: 0;",
                h3 {
                    style: "margin: 0 0 16px; font-size: 11px; color: rgba(23,23,23,0.5); text-transform: uppercase; letter-spacing: 0.08em;",
                    "Rooms"
                }
                p { style: "margin: 8px 0; font-size: 15px; color: #171717; cursor: pointer;", "Library" }
                p { style: "margin: 8px 0; font-size: 15px; color: #171717; cursor: pointer;", "Records" }
            }

            // Main canvas area
            div {
                style: "flex: 1; padding: 24px; display: flex; flex-direction: column; min-width: 0;",

                // Toolbar row
                div {
                    style: "display: flex; justify-content: flex-end; margin-bottom: 16px;",
                    button {
                        style: "background: #C84B31; color: white; border: none; padding: 8px 16px; border-radius: 6px; font-size: 14px; cursor: pointer; font-family: inherit;",
                        onclick: move |_| {
                            let bridge = BRIDGE.get().unwrap().clone();
                            spawn(async move {
                                let mut b = bridge.lock().await;
                                if let Some(resp) = b.send_request(IpcRequest {
                                    message_id: "msg-poc-001".into(),
                                    message_type: IpcRequestType::Request,
                                    action: "RUN_FEATURE_PLUGIN".into(),
                                    plugin_id: Some("hello-feature".into()),
                                    payload: serde_json::json!(null),
                                }).await {
                                    result.set(Some(resp.payload));
                                }
                            });
                        },
                        "Run PoC Test"
                    }
                }

                // Plugin canvas
                div {
                    id: "plugin-canvas",
                    style: "flex: 1; background: rgba(23,23,23,0.04); border-radius: 8px; padding: 24px;",
                    match result() {
                        Some(data) => rsx! {
                            p {
                                style: "font-size: 15px; color: #171717;",
                                "{data}"
                            }
                        },
                        None => rsx! {
                            p {
                                style: "font-size: 15px; color: rgba(23,23,23,0.4);",
                                "Plugin canvas — click Run PoC Test to load"
                            }
                        },
                    }
                }
            }
        }
    }
}
