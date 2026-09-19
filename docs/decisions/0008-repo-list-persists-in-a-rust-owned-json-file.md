# The Repo list persists in a Rust-owned JSON file, not in the frontend

The reviewer's [[Repo]] list lives in `repos.json` under Tauri's app config directory, written and read by Rust (`src-tauri/src/store.rs`). ADR 0006 leaves the Comment Queue and Settings to the frontend, but it doesn't cover the Repo list, and the Repo list isn't like them: ADR 0005 makes Rust the authority on which directories may be read, and it can only check "does this path belong to a registered Repo" if it holds the registry itself. A list kept in the webview would have to be re-supplied to Rust on every call — which is the same as letting the frontend name any path it likes.

A Repo is identified by its canonical filesystem path, so adding the same folder twice keeps one entry and no id has to be minted or kept unique across restarts.

## Considered Options

- **`localStorage`, like Settings (ADR 0006)**: rejected — it puts the registry outside Rust, which contradicts ADR 0005's access control, and settings don't gate filesystem reads the way this list does.
- **`tauri-plugin-store`**: rejected — a plugin, its permissions, and a frontend-facing key/value API for one small file that Rust has to read anyway; `serde_json` plus `std::fs` is less moving parts.
- **A database (SQLite)**: rejected — the list is a handful of paths a reviewer edits by hand-picking folders, with no querying, migration, or concurrency needs.
