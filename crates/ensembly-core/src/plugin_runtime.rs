use std::path::Path;

use anyhow::Result;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};

// Generate host-side bindings from the shared WIT definition.
wasmtime::component::bindgen!({
    world: "feature-plugin",
    path: "../../wit",
});

// ---------------------------------------------------------------------------
// Host state — holds the WASI context and implements the host interface.
// ---------------------------------------------------------------------------

struct PluginState {
    wasi: WasiCtx,
    table: ResourceTable,
}

impl WasiView for PluginState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl ensembly::plugin::host::Host for PluginState {
    fn log(&mut self, level: String, message: String) {
        println!("[PLUGIN {level}] {message}");
    }

    fn get_item(&mut self, id: String) -> String {
        // Hardcoded PoC item — Step 5 will wire this to DbManager.
        serde_json::json!({
            "id": id,
            "title": "The Name of the Wind",
            "tags": ["fantasy", "fiction"],
            "description": "A hero's story told in his own words."
        })
        .to_string()
    }
}

// ---------------------------------------------------------------------------
// PluginRuntime — owns the Engine (expensive to create, reused across loads).
// ---------------------------------------------------------------------------

pub struct PluginRuntime {
    engine: Engine,
}

impl PluginRuntime {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    pub fn load_feature_plugin(&self, wasm_path: &Path) -> Result<FeaturePluginHandle> {
        let mut linker: Linker<PluginState> = Linker::new(&self.engine);

        // Provide all standard WASI Preview 2 interfaces.
        wasmtime_wasi::add_to_linker_sync(&mut linker)?;

        // Provide our custom `ensembly:plugin/host` interface.
        ensembly::plugin::host::add_to_linker(&mut linker, |state| state)?;

        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let state = PluginState {
            wasi,
            table: ResourceTable::new(),
        };
        let mut store = Store::new(&self.engine, state);

        let component = Component::from_file(&self.engine, wasm_path)?;
        let instance = FeaturePlugin::instantiate(&mut store, &component, &linker)?;

        Ok(FeaturePluginHandle { store, instance })
    }
}

// ---------------------------------------------------------------------------
// FeaturePluginHandle — a loaded, ready-to-call plugin instance.
// ---------------------------------------------------------------------------

pub struct FeaturePluginHandle {
    store: Store<PluginState>,
    instance: FeaturePlugin,
}

impl FeaturePluginHandle {
    pub fn call_run(&mut self) -> Result<String> {
        self.instance.call_run(&mut self.store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plugin_runtime_initialises() {
        PluginRuntime::new().expect("PluginRuntime::new should succeed");
    }

    #[test]
    fn load_nonexistent_plugin_returns_error() {
        let runtime = PluginRuntime::new().unwrap();
        let result = runtime.load_feature_plugin(Path::new("/nonexistent/plugin.wasm"));
        assert!(result.is_err(), "loading a missing file should fail");
    }

    #[test]
    fn host_log_does_not_panic() {
        // Exercise the host implementation directly without a Wasm boundary.
        let mut state = PluginState {
            wasi: wasmtime_wasi::WasiCtxBuilder::new().build(),
            table: wasmtime_wasi::ResourceTable::new(),
        };
        // Should not panic for any log level or message.
        ensembly::plugin::host::Host::log(&mut state, "INFO".into(), "test message".into());
        ensembly::plugin::host::Host::log(&mut state, "ERROR".into(), "".into());
    }

    #[test]
    fn host_get_item_returns_valid_json() {
        let mut state = PluginState {
            wasi: wasmtime_wasi::WasiCtxBuilder::new().build(),
            table: wasmtime_wasi::ResourceTable::new(),
        };
        let json = ensembly::plugin::host::Host::get_item(&mut state, "item-001".into());
        let parsed: serde_json::Value =
            serde_json::from_str(&json).expect("get_item should return valid JSON");
        assert_eq!(parsed["id"], "item-001");
    }
}
