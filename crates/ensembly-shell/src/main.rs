use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use dioxus::desktop::use_asset_handler;
use dioxus::prelude::*;
use ensembly_core::ipc::{self, CoreBridgeHalf, IpcBridge};
use ensembly_types::{IpcRequest, IpcRequestType};

// Set once in main(), read from App component.
static BRIDGE: OnceLock<Arc<tokio::sync::Mutex<IpcBridge>>> = OnceLock::new();
static CORE_HALF: OnceLock<std::sync::Mutex<Option<CoreBridgeHalf>>> = OnceLock::new();

/// Directory containing compiled Display Plugin files (hello_display.js + hello_display_bg.wasm).
static PLUGIN_PKG_DIR: OnceLock<PathBuf> = OnceLock::new();

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let wasm_dir = manifest_dir.join("../../target/wasm32-wasip2/debug");
    let (bridge, core_half) = ipc::create(wasm_dir);

    BRIDGE.set(Arc::new(tokio::sync::Mutex::new(bridge))).ok();
    CORE_HALF
        .set(std::sync::Mutex::new(Some(core_half)))
        .ok();
    PLUGIN_PKG_DIR
        .set(manifest_dir.join("../../plugins/hello-display/pkg"))
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

    // Serve Display Plugin files under dioxus://localhost/plugins/<filename>
    // This keeps them same-origin with the shell page, avoiding CORS issues.
    use_asset_handler("plugins", |request, responder| {
        // URI path is like /plugins/hello_display.js
        let filename = request
            .uri()
            .path()
            .trim_start_matches("/plugins/")
            .to_string();

        let pkg_dir = PLUGIN_PKG_DIR.get().unwrap().clone();
        let file_path = pkg_dir.join(&filename);

        spawn(async move {
            match tokio::fs::read(&file_path).await {
                Ok(bytes) => {
                    let content_type = if filename.ends_with(".wasm") {
                        "application/wasm"
                    } else if filename.ends_with(".js") {
                        "application/javascript"
                    } else {
                        "application/octet-stream"
                    };
                    responder.respond(
                        http::Response::builder()
                            .header("Content-Type", content_type)
                            .body(bytes)
                            .unwrap(),
                    );
                }
                Err(e) => {
                    eprintln!("[shell] asset not found: {} — {e}", file_path.display());
                    responder.respond(
                        http::Response::builder()
                            .status(404)
                            .body(vec![])
                            .unwrap(),
                    );
                }
            }
        });
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
                                    if resp.status == ensembly_types::ResponseStatus::Success {
                                        // Trigger the Display Plugin in the WebView.
                                        // Embed payload as a double-encoded JSON string so it
                                        // is safe to interpolate verbatim into JS source.
                                        let payload_js_str = serde_json::to_string(
                                            &resp.payload.to_string()
                                        ).unwrap_or_default();

                                        let js = format!(r#"
                                            (async () => {{
                                                try {{
                                                    const {{ default: init, render }} =
                                                        await import('/plugins/hello_display.js');
                                                    await init();
                                                    render(JSON.parse({payload_js_str}));
                                                }} catch (e) {{
                                                    console.error('[ensembly] display plugin error:', e);
                                                }}
                                            }})();
                                        "#);
                                        let _ = document::eval(&js);
                                        result.set(Some(resp.payload));
                                    }
                                }
                            });
                        },
                        "Run PoC Test"
                    }
                }

                // Plugin canvas — Dioxus owns this div; hello-display will append into it.
                // We deliberately render nothing inside once a plugin has been loaded so
                // Dioxus doesn't clobber the plugin's DOM on re-render.
                div {
                    id: "plugin-canvas",
                    style: "flex: 1; background: rgba(23,23,23,0.04); border-radius: 8px; padding: 24px;",
                    if result().is_none() {
                        p {
                            style: "font-size: 15px; color: rgba(23,23,23,0.4);",
                            "Plugin canvas — click Run PoC Test to load"
                        }
                    }
                    // After first load, Dioxus renders nothing here; the Display Plugin owns
                    // the subtree and Dioxus will not re-render over it.
                }
            }
        }
    }
}
