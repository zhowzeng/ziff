# Rust owns all repo filesystem/git reads — no tauri-plugin-fs

A [[Repo]] is an arbitrary directory the reviewer adds at runtime via a folder picker, and `tauri-plugin-fs`'s permission model requires paths or patterns declared statically in `capabilities/*.json`, which can't enumerate directories chosen at runtime. So every read of repo content — file tree, diffs, full file content, branches — goes through custom `#[tauri::command]`s backed by `std::fs`/`gix` (ADR 0003), and access control, confirming a requested path falls under a registered Repo, lives in that Rust code rather than in static capability config.

## Considered Options

- **`tauri-plugin-fs` with runtime-granted scope**: rejected — the plugin's scope model targets paths known at build time or matched by patterns, not directories a user adds one at a time after install, which makes runtime-safe scoping awkward for this use case.
- **A frontend-requested scope grant per added Repo**: rejected — a permission-request round trip for no benefit, since Rust already needs to own these reads for git access (ADR 0003) regardless of how filesystem reads are gated.
