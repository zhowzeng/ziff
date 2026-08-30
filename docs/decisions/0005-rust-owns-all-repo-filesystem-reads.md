# Rust owns all repo filesystem/git reads — no tauri-plugin-fs

A [[Repo]] is an arbitrary directory the reviewer adds at runtime via a folder picker, so its path isn't known ahead of time. `tauri-plugin-fs`'s permission model requires paths or patterns to be declared statically in `capabilities/*.json`, which can't enumerate directories chosen at runtime. So every read of repo content — file tree, diffs, full file content, branches — goes through custom `#[tauri::command]`s backed by `std::fs`/`gix` (see ADR 0003) instead of the fs plugin. Access control (confirming a requested path falls under a registered Repo) lives in that Rust code, not in static capability config.

## Considered Options

- **`tauri-plugin-fs` with runtime-granted scope**: the plugin's scope model targets paths known at build time or matched by patterns, not directories a user adds one at a time after install — makes runtime-safe scoping awkward for this use case.
- **Frontend requests a scope grant per added Repo**: adds a permission-request round trip for no benefit, since Rust already needs to own these reads for git access (ADR 0003) regardless of how filesystem reads are gated.
