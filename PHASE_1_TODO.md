# Phase 1: Proof of Concept — Implementation Plan

**Goal:** Prove the Extensible Architecture works end-to-end. A string of data must travel from the Native Core, through the IPC Bridge, be processed by a Feature Plugin (backend Wasm), and finally be rendered on the canvas by a Display Plugin (frontend Wasm/JS).

---

## Workspace Layout (Target)

```
ensembly/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── ensembly-types/         # Shared serde types (Item, IPC payloads)
│   ├── ensembly-core/          # Native Core (Rust binary: file I/O, Turso, Wasmtime, IPC)
│   └── ensembly-shell/         # Dioxus App Shell (Rust binary: main window, sidebar)
└── plugins/
    ├── hello-feature/          # PoC Feature Plugin → compiles to .wasm (wasm32-wasip2)
    └── hello-display/          # PoC Display Plugin → compiles to .wasm (wasm32-unknown-unknown) + JS glue
```

---

## Step 1 — Rust Workspace Bootstrap

- [x] Create `Cargo.toml` at the repo root declaring a Cargo workspace with members:
  `crates/*` and `plugins/*`
- [x] Run `cargo new --lib crates/ensembly-types`
- [x] Run `cargo new --bin crates/ensembly-core`
- [x] Run `cargo new --bin crates/ensembly-shell`
- [x] Run `cargo new --lib plugins/hello-feature`
- [x] Run `cargo new --lib plugins/hello-display`
- [x] Add `.cargo/config.toml` specifying `wasm32-wasip2` as the default target for the
  `plugins/hello-feature` crate and `wasm32-unknown-unknown` for `plugins/hello-display`
- [x] Verify `cargo build` compiles the workspace without errors

---

## Step 2 — Shared Types Crate (`ensembly-types`)

*Purpose: Both the Core and all plugins depend on this crate to ensure identical serialization shapes across the Wasm boundary.*

- [x] Add dependencies: `serde` (with `derive` feature), `serde_json`
- [x] Define the **Base Item Schema** structs:
  ```
  EnsemblyItem { system: SystemMeta, core: CoreData, attributes: HashMap<String, Value>, relations: HashMap<String, Vec<Relation>> }
  SystemMeta { id, collection_id, schema_version, created_at, updated_at }
  CoreData { title, primary_image: Option<String>, tags: Vec<String>, description }
  Relation { relation_type, target_id, target_collection }
  ```
- [x] Define the **IPC Bridge** message structs:
  ```
  IpcRequest { message_id, r#type: "REQUEST", action, plugin_id, payload: Value }
  IpcResponse { message_id, r#type: "RESPONSE", status: ResponseStatus, payload: Value }
  IpcEvent { message_id, r#type: "EVENT", action, payload: Value }
  ResponseStatus { Success, Error }
  ```
- [ ] Publish crate internally so `ensembly-core`, `ensembly-shell`, and both plugins can depend on it with `path = "../../crates/ensembly-types"`
- [x] Write unit tests confirming round-trip JSON serialization of each struct

---

## Step 3 — Native Core: Project Skeleton & Turso Init

- [x] Add dependencies to `ensembly-core`: `turso` (or `libsql`), `tokio` (async runtime),
  `serde_json`, `ensembly-types`
- [x] On startup, determine the app data directory (e.g., `~/.local/share/ensembly/` on Linux,
  `~/Library/Application Support/Ensembly/` on macOS via the `dirs` crate)
- [x] Create the directory structure on first run:
  ```
  <data_dir>/
  ├── collections/     # Source-of-truth JSON files
  └── ensembly.db      # Turso index
  ```
- [x] Initialize Turso and run a bootstrap migration to create the `items` index table:
  ```sql
  CREATE TABLE IF NOT EXISTS items (
      id TEXT PRIMARY KEY,
      collection_id TEXT NOT NULL,
      title TEXT NOT NULL,
      tags TEXT,           -- JSON array stored as text
      file_path TEXT NOT NULL,
      last_modified INTEGER NOT NULL
  );
  ```
- [x] Write a `DbManager` struct wrapping the Turso connection with `async fn get_item`,
  `async fn upsert_item`, `async fn query_items`
- [x] Smoke-test: insert a hardcoded PoC item row on startup and assert it reads back correctly

---

## Step 4 — Native Core: Wasmtime Runtime & Feature Plugin Host

- [x] Add dependencies: `wasmtime`, `wasmtime-wasi`
- [x] Create a `PluginRuntime` struct that:
  - Holds a `wasmtime::Engine` (shared, reused across plugin loads)
  - Has `fn load_feature_plugin(wasm_path: &Path) -> Result<FeaturePlugin>`
- [x] Define the **Host Functions** that plugins can call (under the `ensembly_host` namespace).
  For PoC, implement only the subset needed for the Hello World test:
  - `host_log` and `host_get_item` defined via WIT in `wit/feature-plugin.wit`
  - Implemented via `ensembly::plugin::host::Host` trait on `PluginState`
- [x] Wire host functions into the Wasmtime `Linker` before instantiating any plugin module
- [x] Create a `FeaturePluginHandle` struct with `fn call_run(&mut self) -> Result<String>` that
  invokes the plugin's exported `run` function and returns its string result
- [x] Note on memory: Component Model handles string passing natively via WIT — no manual
  `(ptr, len)` helpers needed

---

## Step 5 — Native Core: IPC Bridge (Backend Side)

- [ ] Choose IPC mechanism for Dioxus ↔ Core communication.
  **Decision:** Use Dioxus's built-in `use_coroutine` + `tokio::sync::mpsc` channels:
  the Shell and Core run in the same process for the PoC; the Bridge is an async channel pair.
- [ ] Create an `IpcBridge` struct with:
  - `tx: mpsc::Sender<IpcRequest>` — Shell sends requests into Core
  - `rx: mpsc::Receiver<IpcResponse>` — Core sends responses back to Shell
- [ ] Implement a `async fn dispatch(request: IpcRequest) -> IpcResponse` handler in the Core:
  - `"PING"` action → responds with `{ status: Success, payload: { message: "pong from core" } }`
  - `"RUN_FEATURE_PLUGIN"` action → loads `hello-feature.wasm`, calls `run()`, returns result
  - Unknown actions → `{ status: Error, payload: { error_code: "UNKNOWN_ACTION" } }`
- [ ] The Core's async `dispatch` loop runs as a `tokio::spawn`-ed task for the lifetime of the app

---

## Step 6 — Hello Feature Plugin (`plugins/hello-feature`)

*A headless backend Wasm plugin. Its sole job: call `host_log`, then return a greeting string.*

- [ ] Set `crate-type = ["cdylib"]` in `Cargo.toml`
- [ ] Add dependencies: `ensembly-types`, `serde_json`
- [ ] Declare the external host functions the plugin imports:
  ```rust
  extern "C" {
      fn host_log(level_ptr: *const u8, level_len: usize, msg_ptr: *const u8, msg_len: usize);
  }
  ```
- [ ] Implement and export a `pub extern "C" fn run() -> *const u8` (or `(ptr, len)` pair) that:
  1. Calls `host_log("INFO", "hello-feature plugin started")`
  2. Returns the JSON string `{ "greeting": "Hello from the Feature Plugin!" }`
- [ ] Build target: `wasm32-wasip2`
- [ ] Confirm `cargo build --target wasm32-wasip2` produces `hello_feature.wasm`

---

## Step 7 — Dioxus App Shell (`ensembly-shell`)

*The main window. For PoC, it just needs to exist and communicate with the Core.*

- [ ] Add dependencies: `dioxus`, `dioxus-desktop`, `tokio`, `serde_json`, `ensembly-types`
- [ ] Bootstrap a `dioxus::launch(App)` entry point
- [ ] Build the **App Shell layout** using Dioxus RSX:
  ```
  ┌──────────┬──────────────────────────────┐
  │ Sidebar  │  <div id="plugin-canvas">    │
  │          │  (empty grey placeholder)    │
  │ [rooms]  │                              │
  └──────────┴──────────────────────────────┘
  ```
  - Sidebar: static list of placeholder room names ("Library", "Records")
  - Canvas: a styled `div` that will host Display Plugin output
- [ ] Apply PoC inline styles matching the Design System:
  - Background: Archive Cream `#F6F4F0`
  - Sidebar border: `1px solid rgba(23,23,23,0.2)`
  - Font: `system-ui` (sans-serif fallback until fonts are bundled)
- [ ] Wire up the `IpcBridge` channels (passed in via Dioxus context or global state)
- [ ] Add a **"Run PoC Test" button** (Terracotta, top-right of canvas) that:
  1. Sends an `IpcRequest` with `action: "RUN_FEATURE_PLUGIN"` over the bridge
  2. Awaits the `IpcResponse`
  3. Stores the response payload in a `use_signal` reactive state

---

## Step 7 (addendum) — Shell: Asset Serving for Display Plugins

*The Shell must be able to load a foreign `.wasm` binary + its JS glue into the WebView. This is
the first feasibility checkpoint: Dioxus Desktop must allow serving local binary assets and
executing them inside its WebView.*

- [ ] Investigate Dioxus Desktop's asset protocol. Two candidate approaches:
  - **Option A — Dioxus asset system:** Place compiled plugin files under `assets/` and reference
    them via Dioxus's built-in `asset!()` macro / custom protocol handler.
  - **Option B — Custom `wry` protocol:** Register a custom URI scheme (e.g., `plugin://`) on the
    underlying `wry::WebView` to serve `.wasm` and `.js` files from disk at runtime without
    bundling them at compile time. This is the preferred long-term approach as plugins are
    installed dynamically.
- [ ] Implement whichever option proves viable; document why the other was rejected in
  `docs/PHASE_1_RESULTS.md`
- [ ] Confirm the WebView can fetch and instantiate a `.wasm` module via
  `WebAssembly.instantiateStreaming()` from the chosen protocol
- [ ] Add a `<script>` bootstrap tag in the Dioxus HTML template that registers the plugin loader
  function in the global JS scope so the Shell can trigger it via `use_eval`

---

## Step 8 — Hello Display Plugin (`plugins/hello-display`)

*The highest-risk PoC component. A real `.wasm` binary — compiled from Rust, using `wasm-bindgen`
and `web-sys` — must be served into the Dioxus WebView and directly manipulate the DOM inside
`#plugin-canvas`. This is the core architectural feasibility test.*

### 8a — Build the Wasm Binary
- [ ] Set `crate-type = ["cdylib"]` in `Cargo.toml`
- [ ] Add dependencies: `wasm-bindgen`, `web-sys` (features: `Window`, `Document`, `Element`,
  `HtmlElement`, `Node`), `serde-wasm-bindgen`, `ensembly-types`
- [ ] Build target: `wasm32-unknown-unknown`
- [ ] Implement a `#[wasm_bindgen]`-exported `render(data: JsValue)` function that:
  1. Calls `web_sys::window()?.document()?.get_element_by_id("plugin-canvas")`
  2. Creates a child `div` with a greeting card layout (greeting text, timestamp, plugin label)
  3. Applies inline styles: Terracotta border `#C84B31`, Archive Cream background `#F6F4F0`
  4. Appends it to the canvas element
- [ ] Run `wasm-bindgen --target web` to generate the JS glue file (`hello_display.js`) alongside
  the `.wasm` binary. Add this as a build step in the workspace `Makefile` or `build.rs`.

### 8b — Load the Plugin in the WebView
- [ ] Place (or serve via custom protocol) both `hello_display_bg.wasm` and `hello_display.js`
  where the WebView can reach them
- [ ] When the Shell receives a successful `IpcResponse`, use `dioxus_desktop::use_eval` to
  inject and execute the following JS sequence in the WebView:
  ```js
  import init, { render } from '<plugin_url>/hello_display.js';
  await init();
  render(JSON.parse('<response_payload_json>'));
  ```
- [ ] Confirm there are no Content Security Policy (CSP) blocks on the Dioxus WebView that
  prevent dynamic module loading; disable or relax CSP for PoC if needed and document it

### 8c — Feasibility Checkpoints
These must all pass before Step 8 is considered complete:
- [ ] The `.wasm` binary loads without errors in the WebView's JS console
- [ ] `web-sys` DOM calls execute successfully (no "document is undefined" or similar errors
  that would indicate the Wasm context lacks access to the live DOM)
- [ ] The greeting card visually appears inside `#plugin-canvas` — not adjacent to it, not in a
  shadow DOM, but as a direct child of the element owned by Dioxus
- [ ] Dioxus's own Virtual DOM is not corrupted or re-rendered after the plugin mutates the DOM
  (the Shell's sidebar and button must still be functional after the plugin renders)

---

## Step 9 — End-to-End Integration & PoC Validation

*Wire everything together and confirm the full data path works.*

- [ ] Ensure startup sequence:
  1. `ensembly-core` initializes Turso, confirms DB connection
  2. `ensembly-core` spawns the IPC dispatch loop
  3. `ensembly-shell` launches the Dioxus window with the IPC bridge channels in context
- [ ] **Manual test — full data path:**
  - [ ] Click "Run PoC Test" button in the shell
  - [ ] Core receives `RUN_FEATURE_PLUGIN` request
  - [ ] Core loads `hello_feature.wasm` via Wasmtime
  - [ ] Feature Plugin calls `host_log` → log appears in terminal
  - [ ] Feature Plugin returns greeting JSON
  - [ ] Core wraps it in an `IpcResponse` and sends it back
  - [ ] Shell receives response, updates reactive state
  - [ ] Shell triggers the Display Plugin load sequence in the WebView
  - [ ] `hello-display` Wasm binary initialises, `render()` is called with the payload
  - [ ] Greeting card appears inside `#plugin-canvas`; Shell UI remains functional
- [ ] **Manual test — PING:**
  - [ ] Add a second "Ping Core" button; confirm `pong from core` response renders via the same
    Display Plugin path
- [ ] All channels confirmed: Core init ✓, Feature Plugin (Wasmtime) ✓, IPC Bridge ✓,
  Display Plugin (wasm-bindgen in WebView) ✓, DOM isolation ✓

---

## Step 10 — Housekeeping & Phase 1 Wrap-Up

- [ ] Add a top-level `README.md` with build instructions:
  - Prerequisites: Rust toolchain, `wasm32-wasip2` target, `wasm32-unknown-unknown` target,
    `wasm-bindgen-cli` (`cargo install wasm-bindgen-cli`)
  - `cargo build --target wasm32-wasip2 -p hello-feature`
  - `cargo build --target wasm32-unknown-unknown -p hello-display && wasm-bindgen ...`
  - `cargo run -p ensembly-shell` to launch the app
- [ ] Write `docs/PHASE_1_RESULTS.md` documenting:
  - Which asset-serving approach was chosen (Option A vs B) and why
  - Whether DOM isolation held up (Dioxus VDOM vs plugin-mutated DOM)
  - Any CSP or WebView restrictions encountered and how they were resolved
  - Known limitations to address in Phase 2
- [ ] Tag the git commit as `poc-phase-1-complete`

---

## Success Criteria

Phase 1 is complete when a developer can run a single command, click one button in the window,
and see a greeting card — originating from a compiled `.wasm` Feature Plugin (via Wasmtime),
routed through the IPC bridge, and painted onto the canvas by a real `wasm-bindgen` Display
Plugin running inside the Dioxus WebView — appear on screen without errors, while the surrounding
Shell UI remains fully interactive.
