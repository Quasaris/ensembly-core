# Ensembly: Technical Specifications

## Overview
This document defines the strict contracts and schemas required for the Ensembly application architecture. It governs the physical data structure, the communication protocol between the native host and the UI, and the API boundaries for the WebAssembly plugin ecosystem.

---

## 1. The Source of Truth Schema

The primary storage format is cleanly structured JSON. Every item in the database conforms to this Base Item Schema. To prevent data corruption and schema collisions between different plugins, all non-core data **must** be strictly namespaced using the respective `plugin_id`.

**File Naming Convention:** `[item_id].json`

### Example JSON Structure
```json
{
  "system": {
    "id": "item_01HQ7P8XYZ2ABCDEF987654",
    "collection_id": "books_main", 
    "schema_version": "1.1",
    "created_at": "2026-04-11T12:00:00Z",
    "updated_at": "2026-04-11T12:15:00Z"
  },
  "core": {
    "title": "Dune",
    "primary_image": "images/cover_01HQ7P8.jpg",
    "tags": ["sci-fi", "unread", "classic"],
    "description": "A 1965 epic science fiction novel by Frank Herbert."
  },
  "attributes": {
    "plugin_schema_builder": {
      "personal_rating": 5,
      "storage_location": "Living Room > Bookshelf A"
    },
    "plugin_book_tools": {
      "isbn_13": "978-0441172719",
      "author": "Frank Herbert",
      "format": "Hardcover"
    }
  },
  "relations": {
    "plugin_cross_linker": [
      {
        "relation_type": "adaptation",
        "target_id": "item_01HQ8X9ABCDEF987654XYZ2", 
        "target_collection": "dvd_main"
      }
    ]
  }
}
```

---

## 2. The IPC Bridge Protocol

This protocol defines the asynchronous message-passing standard between the **Frontend (Dioxus App Shell / Display Plugins)** and the **Backend (Rust Native Core)**. All communication happens via serialized JSON payloads following a standard Request/Response/Event architecture.

### A. The Request Payload (Frontend -> Backend)
Triggered when the UI needs data or wants to execute an action. State mutations must target a specific plugin namespace.

```json
{
  "message_id": "req_88291a",
  "type": "REQUEST",
  "action": "UPDATE_ATTRIBUTES",
  "plugin_id": "plugin_book_tools",
  "payload": {
    "item_id": "item_01HQ7P8XYZ2ABCDEF987654",
    "attributes": {
      "isbn_13": "978-0441172719"
    }
  }
}
```

### B. The Response Payload (Backend -> Frontend)
The Native Core's reply to a specific `message_id`.

```json
{
  "message_id": "req_88291a",
  "type": "RESPONSE",
  "status": "SUCCESS", 
  "payload": {
    "items": [
      { /* item data matching the Source of Truth schema */ }
    ],
    "total_count": 1
  }
}
```
*(Note: If `status` is `ERROR`, the payload contains `{"error_code": "...", "message": "..."}`)*

### C. The Event Payload (Backend -> Frontend / Fire-and-Forget)
Used when the Native Core proactively alerts the UI of state changes (e.g., background sync completion, index updates).

```json
{
  "message_id": "evt_991b2c",
  "type": "EVENT",
  "action": "INDEX_UPDATED",
  "payload": {
    "collection_id": "books_main",
    "modified_items": ["item_01HQ7P8XYZ2ABCDEF987654"]
  }
}
```

---

## 3. The Plugin API Spec (Wasm Host Functions)

WebAssembly plugins operate in a strict sandbox without direct access to the filesystem or network. They interact with the system by calling Host Functions exposed by the Rust Native Core under the `ensembly_host` namespace.

### Type Safety & The Wasm Boundary
To maintain compile-time type safety while passing serialized JSON across the Wasm memory boundary, Ensembly uses a shared library crate (`ensembly-types`). Both the Core Engine and all Rust-based Wasm plugins must include this crate as a dependency. It contains the strict `serde` structs for Items, Requests, and Responses, ensuring both sides serialize and deserialize identical shapes.

### Core Host Functions

#### Data Access & Manipulation
* `host_get_item(item_id: String) -> String`
    * Returns the JSON string of the requested item.
* `host_query_index(query_json: String) -> String`
    * Passes a JSON query to the Turso index (including vector searches) and returns an array of matching item IDs or partial data.
* `host_upsert_item(item_json: String) -> Result<String, Error>`
    * Sends a modified or new JSON item to the Core. The Core handles writing it to the local file system and updating the Turso index.

#### External Communication
* `host_fetch_api(url: String, method: String, headers: String, body: Option<String>) -> String`
    * Allows a Feature Plugin to make external network requests securely through the Rust Core, restricting network access at the app-permission level and avoiding browser CORS limitations.

#### System & UI Interaction
* `host_log(level: String, message: String)`
    * Writes to the central Ensembly application log.
* `host_trigger_notification(title: String, message: String, type: String)`
    * Instructs the Dioxus App Shell to render a native toast notification.
* `host_prompt_user(prompt_config_json: String) -> String`
    * Pauses plugin execution, asks the Core Engine to render an input dialog (e.g., "Enter API Key"), and returns the user's input.
