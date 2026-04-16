#![allow(unsafe_op_in_unsafe_fn)] // suppresses warnings from wit-bindgen generated code

wit_bindgen::generate!({
    world: "feature-plugin",
    path: "../../wit",
});

use crate::ensembly::plugin::host;

struct Plugin;

impl Guest for Plugin {
    fn run() -> String {
        host::log("INFO", "hello-feature plugin started");
        serde_json::json!({
            "greeting": "Hello from the Feature Plugin!"
        })
        .to_string()
    }
}

export!(Plugin);
