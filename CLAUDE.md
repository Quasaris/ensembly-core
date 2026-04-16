# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

This is a **pre-code repository**. All documentation is in `docs/`. Implementation starts at Phase 1 — see `PHASE_1_TODO.md` for the step-by-step plan.

## What This Is

Ensembly is a local-first personal cataloging app (books, vinyl, collectibles, etc.) built on a plugin architecture. The host app is a lightweight engine; all major features are isolated WebAssembly plugins.

## Planned Tech Stack

- **Native Core:** Rust binary (`crates/ensembly-core`) — file I/O, Turso DB, Wasmtime runtime, IPC dispatch
- **UI Shell:** Rust binary (`crates/ensembly-shell`) — Dioxus + native OS WebView
- **Shared Types:** Library crate (`crates/ensembly-types`) — serde structs shared between host and plugins
- **Feature Plugins:** Rust → `wasm32-wasip2` — headless data processors running in Wasmtime (no UI)
- **Display Plugins:** Rust → `wasm32-unknown-unknown` + `wasm-bindgen` — full-canvas UIs running in the WebView

## Architecture Invariants

These decisions are load-bearing — don't change them without revisiting the full architecture:

**Two storage layers, one source of truth.** Every item is a `[item_id].json` file on disk (owned by the user). Turso is a pure query index rebuilt from those files — it is never the source of truth. Smart boot compares `last_modified` timestamps and only re-indexes changed files.

**All plugin state is namespaced.** In item JSON, `attributes` and `relations` are keyed by `plugin_id`. In IPC messages, `plugin_id` is required on all state mutations. This prevents schema collisions between plugins.

**Plugins have no direct system access.** Feature Plugins call host functions exposed under the `ensembly_host` namespace (see `docs/TECHNICAL_SPECIFICATION.md §3`). They cannot touch the filesystem or network directly.

**Display Plugins own their canvas.** The Dioxus shell renders `<div id="plugin-canvas">` and then hands it off. A Display Plugin manipulates that subtree via `web-sys` DOM calls. Dioxus must not re-render over plugin-owned DOM — this interaction is a key Phase 1 feasibility question.

## IPC Bridge Pattern

All Shell ↔ Core communication uses typed JSON messages (defined in `ensembly-types`):
- `IpcRequest` (Frontend → Backend): `{ message_id, type: "REQUEST", action, plugin_id, payload }`
- `IpcResponse` (Backend → Frontend): `{ message_id, type: "RESPONSE", status, payload }`
- `IpcEvent` (Backend → Frontend, fire-and-forget): `{ message_id, type: "EVENT", action, payload }`

For Phase 1 PoC, the bridge is in-process `tokio::mpsc` channels. The Shell and Core are the same binary.

## Key Documents

| File | Purpose |
|------|---------|
| `docs/APP_ARCHITECTURE.md` | Tech stack, layer definitions, plugin zones, data flow |
| `docs/TECHNICAL_SPECIFICATION.md` | Item JSON schema, IPC message shapes, Wasm host function signatures |
| `docs/DESIGN_SYSTEM.md` | Color palette, typography, component styles, spacing system |
| `docs/PROJECT_ROADMAP.md` | Phase 0–4 goals |
| `PHASE_1_TODO.md` | Detailed step-by-step PoC implementation checklist |

## Build Commands (once code exists)

```bash
# Native crates
cargo build

# Feature plugin (backend Wasm)
cargo build --target wasm32-wasip2 -p hello-feature

# Display plugin (frontend Wasm) — requires wasm-bindgen-cli
cargo build --target wasm32-unknown-unknown -p hello-display
wasm-bindgen --target web target/wasm32-unknown-unknown/debug/hello_display.wasm --out-dir plugins/hello-display/pkg

# Run the app
cargo run -p ensembly-shell
```

## Design System Quick Reference

- **Backgrounds:** Archive Cream `#F6F4F0`
- **Text / Icons:** Ink Black `#171717` (100%), body copy at 80% opacity, borders at 40%
- **Primary action:** Terracotta `#C84B31` (buttons, active states)
- **Secondary / links:** Slate Blue `#3B5B7E`
- **Success:** Sage Green `#6B8E6B`
- **Spacing:** Base-8 system (4, 8, 16, 24, 48, 64 px)
- **Fonts:** Lora/Playfair Display (Gallery/serif display) · Inter/Geist (Archivist/UI/data)
- **Border radius:** 6px buttons · 8px data blocks · 12px media cards
