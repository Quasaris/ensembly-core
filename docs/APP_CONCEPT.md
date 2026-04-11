# Ensembly: Personal Collections & Catalog App - Architecture & Product Concept

## Core Philosophy
A "once-and-for-all" personal cataloging solution combining the structural power of a database, the data ownership of an offline-first file system, and the visual joy of a curated digital museum. It follows an **Extensible Software Architecture** (similar to Obsidian), where the app is a lightweight engine and all major features are plugins.

## 1. System Architecture (The Plugin Model)
The application is built on a strict hierarchy to ensure modularity, speed, and future-proofing.

### The Core Engine (The Host App)
The absolute base application. It does almost nothing on its own except manage the data silos, expose an API for plugins, handle the Plugin Manager, and enforce the **Base Item Schema** across all items:
* **Item Title** (String)
* **Primary Image** (Media, Optional)
* **User-Defined Tags** (Array, universal across all silos)
* **General Notes/Description** (Rich Text)
* **System Metadata** (Created At, Updated At)

### Core Plugins (First-Party Extensions)
Bundled features that provide standard cataloging functionality without polluting the Core Engine. 
* **Custom Schema Plugin:** Allows users to attach custom fields (Numbers, Dates, Checkboxes, Dropdowns) to the Base Item Schema within a specific silo.
* **Gallery View:** A UI module rendering data visually (posters, masonry layouts).
* **Archivist View:** A UI module rendering data in a dense, editable spreadsheet grid.
* **Global Search:** The engine querying across tags, titles, and notes.
* **Cloud Sync:** The background engine for S3-style remote syncing.

### Collection Plugins (Domain-Specific Extensions)
Highly specialized "Snap-Ons" that govern specific, isolated data silos.
* **Book Collection Plugin:** Manages the "Library" silo. Adds ISBN scanning, API lookups, and auto-synopsis features.
* **Vinyl Collection Plugin:** Manages the "Records" silo. Adds specific fields (RPM, Pressing Year) and Discogs API integration.

## 2. Storage Philosophy (Indexed Filesystem)
The app uses a **Persisted Hybrid Storage Model** to balance future-proof data ownership with blazing-fast relational database performance.

* **The Source of Truth (Local Files):** Every item is saved as a discrete, human-readable file (e.g., `.json` or `.md` with YAML frontmatter) alongside its local media assets. This guarantees total data ownership and longevity.
* **The Query Engine (Persisted Local Database):** A hidden, persistent database (like SQLite) acts purely as a high-speed index for the UI to run complex sorting and filtering.
* **Smart Startup (Differential Sync):** On boot, the app compares file `last_modified` timestamps against the database. It only updates the database rows for files that have been changed, resulting in instant startup times regardless of collection size.
* **The Escape Hatch:** A "Rebuild Index" button allows the user to completely wipe the local database and reconstruct it from the source-of-truth files to instantly resolve any sync conflicts.

## 3. Interface & UX Concepts

### The "Two-Faced" UI
Users can toggle between two distinct modes based on their current task:
* **Archivist Mode:** The dense, utilitarian grid view for bulk editing, schema definition, and tag management.
* **Gallery Mode:** A highly visual, responsive browsing experience focused on cover art and clean layouts, completely hiding the database complexity.

### Separate Digital "Rooms" (Data Silos)
Collections governed by different Collection Plugins are kept in strictly isolated environments to prevent unrelated items from muddying the browsing experience. Gallery aesthetics adapt to the active module (e.g., vertical posters for books, square grids for vinyl).

### The AI-Assisted "Inbox" Workflow
Item addition balances AI automation with strict manual user control to prevent database pollution.
* **Auto-First Ingestion:** Users scan a barcode or snap a photo. Multimodal LLMs and APIs draft metadata, write synopses, and suggest tags.
* **The Approval Inbox:** All auto-generated data enters a "Drafts" holding area. The user reviews, tweaks, and approves the metadata before it officially commits to the local files.

### Bi-Directional "Wormholes"
While data silos remain structurally isolated, related items can be connected via discrete cross-links (e.g., linking a physical book in the Library silo to a movie adaptation in the DVD silo).

### Natural Language Curation
Advanced search capabilities powered by LLMs allow for conversational filtering (e.g., *"Show me highly rated sci-fi books I haven't read yet that are stored in the attic."*).