# Ensembly: Project Roadmap

## Phase 0: Documentation & Blueprinting
*The goal: Establish the strict contracts and design rules before writing any code.*
* **The Source of Truth Schema:** Define the exact JSON/Markdown structure for the Base Item Schema.
* **The IPC Bridge Protocol:** Document the specific JSON payloads for asynchronous message-passing between the Rust Native Core and the Wasm plugins.
* **The Plugin API Spec:** Define the exact functions a Wasm plugin (Feature or Display) is allowed to call.
* **The Lean Design System:** Define the CSS variables, typography, and component primitives for the Dioxus App Shell.

## Phase 1: Proof of Concept (PoC) Implementation
*The goal: Prove the "Extensible Architecture" actually works end-to-end.*
* **Bare Metal Init:** Initialize the Rust Native Core and the Turso database.
* **The Shell:** Spin up a basic Dioxus window with a sidebar and the empty `<div id="plugin-canvas">`.
* **The Wasm Swarm Test:** Build a "Hello World" Feature Plugin (backend) and a simple Display Plugin (frontend) to successfully pass a string of data through the IPC bridge and render it on the canvas.

## Phase 2: Core Engine Perfection
*The goal: Stabilize the host application so it is ready to support complex plugins.*
* **Smart Boot & Sync:** Implement the logic to compare local file timestamps and update the Turso Index.
* **Plugin Manager:** Build the internal systems to install, enable, disable, and sandbox the `.wasm` files securely.
* **CRUD Operations:** Ensure the engine can safely create, read, update, and delete the physical JSON files and keep the database in sync.

## Phase 3: Core Plugins Development
*The goal: Build the first-party modules that give Ensembly its baseline utility.*
* **Archivist Plugin:** Build the spreadsheet-style grid UI for heavy data editing.
* **Gallery Plugin:** Build the responsive, visual browsing UI.
* **Custom Schema Builder:** Build the module that lets users attach new, arbitrary fields to the baseline items.
* **Global Search Plugin:** Build the backend Wasm feature that queries the Turso Index.

## Phase 4: Domain Specialization (Collection Plugins)
*The goal: Build the highly specialized "Snap-On" data silos.*
* **The Book Collection Plugin:** Build the specialized silo, including features like the ISBN API lookup and AI auto-synopsis.
* **Future Plugins:** Plan and implement modules for Vinyl, Action Figures, etc.
