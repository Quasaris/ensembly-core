# Ensembly: Lean Design System

## 1. Visual Theme & Atmosphere
Ensembly builds upon a warm, analog foundation but introduces rich archival accents to support diverse visual media. It utilizes a "Two-Faced" visual strategy:
* **Gallery Mode:** Feels like a high-end museum placard. Lavish whitespace, elegant serif typography for titles, and focus on the artwork.
* **Archivist Mode:** Feels like a pristine ledger. Dense, border-driven containment, sans-serif utility fonts, and highly legible data grids.

Instead of arbitrary gray hex codes, Ensembly uses an **Opacity-Driven Neutral Scale** derived from our base Ink Black, ensuring perfect tonal harmony across the app.

## 2. Color Palette & Roles

### Foundation (The Lovable Inheritance)
* **Archive Cream (`#F6F4F0`):** The universal page and surface background. Warm, tactile, and reduces eye strain.
* **Ink Black (`#171717`):** The primary text and strong UI element color. 

### The Neutral Scale (Opacity-Based)
* **Ink 100% (`#171717`):** Primary headings, icons, dark button surfaces.
* **Ink 80% (`rgba(23,23,23,0.8)`):** Body copy, secondary headers.
* **Ink 40% (`rgba(23,23,23,0.4)`):** Passive borders, dividers, unselected states.
* **Ink 5% (`rgba(23,23,23,0.05)`):** Hover states, table row striping, subtle interactive fills.

### The "Curator" Accents (Breaking the Monochrome)
To inject personality without overpowering the user's uploaded images, we use deep, classic pigment colors for interactive states and primary actions.
* **Primary Accent — Terracotta (`#C84B31`):** Used for primary CTA buttons, active tab underlines, and key focus states. Evokes classic brick and leather bindings.
* **Secondary Accent — Slate Blue (`#3B5B7E`):** Used for text links, informational tags, and secondary highlights.
* **Success — Sage Green (`#6B8E6B`):** Used for positive UI feedback (e.g., "Item Synced" toasts).

## 3. Typography: The Dual-Font System

Unlike single-font systems, Ensembly uses a pairing to separate the "Museum" from the "Ledger."

### Primary Display (Serif)
* **Font:** `Lora`, `Playfair Display`, or `ui-serif`
* **Role:** Gallery Mode item titles, Room headers, and "Museum Placard" details.
* **Treatment:** Used at large sizes (24px+) with standard tracking. Brings an elegant, curated warmth to the browsing experience.

### Utility & Data (Sans-Serif)
* **Font:** `Inter`, `Geist Sans`, or `system-ui`
* **Role:** Archivist Mode data grids, tags, button labels, UI navigation, and long-form descriptions.
* **Treatment:** Highly legible, clean, and modern. 

## 4. Components & Depth

Ensembly's depth model is shallow and bordered to maintain the tactile "analog" feel, using shadows strictly for interaction feedback rather than structural layout.

### Buttons
* **Primary Action:** Terracotta background (`#C84B31`), Archive Cream text. 
    * *Shadow:* We retain the tactile inset technique: `rgba(255,255,255,0.2) 0px 1px 0px inset, rgba(0,0,0,0.15) 0px 2px 4px`.
* **Secondary Action (Ghost):** Transparent background, `1px solid rgba(23,23,23,0.4)` border, Ink Black text. 
    * *Hover:* Fills with Ink 5% (`rgba(23,23,23,0.05)`).
* **Radius:** `6px` for standard data-entry buttons; `9999px` (full pill) for Gallery Mode action toggles.

### Cards, Silos & Containment
* **Borders over Shadows:** Cards and images are contained by a subtle `1px solid rgba(23,23,23,0.2)` border on top of the Archive Cream background. No default drop shadows.
* **Corner Radius:** `8px` for data blocks/tables; `12px` for media/image cards.
* **The "Gallery Lift":** When hovering over an item in Gallery Mode, the border transitions to Terracotta and a soft, warm shadow appears (`rgba(200,75,49,0.15) 0px 8px 24px`) to indicate interactivity.

### Tags & Metadata Pills
* **Style:** Archive Cream background, Ink 40% border, Ink 80% text.
* **Size:** Small (`12px` text, `4px 8px` padding).
* **Radius:** `4px` (slightly sharper to distinguish from action buttons).

## 5. UI Spacing & Grid (Base 8 System)
* **Micro:** 4px, 8px (Inner button padding, tag spacing).
* **Component:** 16px, 24px (Card padding, form field gaps).
* **Sectional:** 48px, 64px (Spacing between distinct UI sections).
* **Gallery Rhythm:** Gallery Mode utilizes a responsive masonry or fluid grid layout, allowing item cards to dictate their own height based on the media's aspect ratio (e.g., vertical books next to square vinyl).