# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Full desktop app (Tauri + Vue) — primary development mode
npm run tauri:dev

# Frontend only (Vite dev server at localhost:3000, no Tauri)
npm run dev

# Type-check without emitting
npm run type-check

# Lint
npm run lint

# Production build (Tauri bundles frontend automatically)
npm run tauri build
```

There are no tests. `npm run dev` starts the Vite frontend in isolation, but most functionality requires the Tauri runtime — `invoke()` calls will fail without it.

## Architecture

This is a **Tauri 2 desktop app**: a Rust backend and a Vue 3 frontend that communicate exclusively via Tauri commands (`invoke()`). The two halves are in `src/` (frontend) and `src-tauri/src/` (backend).

### Rust backend (`src-tauri/src/`)

The backend is organized by domain, each with a `mod.rs` (business logic) and `api.rs` (Tauri command handlers):

```
app/      — initialization, photo search, refresh
photos/   — photo metadata mutations
people/   — person and category management
places/   — locations, layers, shapes
tags/     — tag graph, validation
settings/ — theme, color palette
```

**Global state** is held in `lazy_static` Tokio `Mutex`es defined at the top of each domain's `mod.rs`. The most important ones are in `app/mod.rs`:
- `DB` — the SQLite connection (set during `initialize`, must exist before any query)
- `OPEN_FOLDER`, `THUMBNAIL_DIR` — the active folder and thumbnail cache path

The database is SQLite managed by Diesel. Migrations live in `src-tauri/migrations/`. Multi-valued fields (tags, people) are stored as comma-separated strings and converted with `row_to_vec` / `vec_to_row`. Any new Tauri command must be registered in `main.rs`'s `invoke_handler`.

Thumbnails for RAW and video files are generated during `initialize` and stored in `app_data_dir`. Regular image files serve their original path via Tauri's asset protocol.

### Frontend (`src/`)

**Auto-imports** — don't add explicit imports for Vue composition API, vue-router composables (`useRoute`, `useRouter`), or `storeToRefs`/`defineStore` from Pinia. They are auto-imported by `unplugin-auto-import`. All components under `src/components/` are auto-imported by their kebab-case name.

**Routing** is file-based via `vue-router/auto`. Files in `src/pages/` become routes automatically. The layout system (`vite-plugin-vue-layouts-next`) wraps pages; the nav rail in `App.vue` is hidden on `/` (the folder-picker landing page).

**API layer** (`src/api/`) contains thin `invoke()` wrappers, one file per domain. Most calls are fire-and-forget `async` functions. Calls that need structured error handling return an `APIResult`:

```ts
// APIResult builder pattern — call ok(), err(), then send()
new APIResult(async () => invoke('get_tags')).ok(tags => ...).err(msg => ...).send();
```

**Domain classes** (`src/classes/`) are rich objects that own their own mutations. A `Photo` instance calls the backend directly via its methods (`photo.setTags(...)`, `photo.setRating(...)`), rather than going through a store action. The same pattern applies to `Tag`, `Place`, `Person`, etc.

**State management** — `src/stores/fileStore.ts` is the only Pinia store. It holds cross-page state: the search query, sort mode, items-per-row, active theme, and a `globalError` string. Errors surface through a global snackbar in `App.vue`.

### Design system

`PRODUCT.md` and `DESIGN.md` at the project root define the design spec. The active theme is "The Observatory" — a dark, near-monochrome system with a single slate blue accent.

- **Canonical color tokens**: `src/styles/tokens.css` — OKLCH values, ease curves, durations. Use these CSS custom properties (e.g. `var(--color-primary)`, `var(--ease-standard)`) for any styling not handled by Vuetify.
- **Vuetify theme**: `src/plugins/vuetify.ts` — hex approximations of the OKLCH tokens for Vuetify's theme engine. When updating colors, change both files and keep them in sync.
- **Vuetify SASS overrides**: `src/styles/settings.scss` — currently unused but wired up as the Vuetify config file for component-level SASS variable overrides.
- Tag graph SVG colors reference `var(--color-*)` tokens directly rather than `--v-theme-*` vars.
