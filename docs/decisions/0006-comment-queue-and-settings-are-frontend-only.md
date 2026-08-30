# Comment Queue and Settings are frontend-only state, not backend-persisted

Neither the [[Comment Queue]] nor app Settings gets a Rust command or disk-backed storage right now. The Comment Queue lives in frontend session state and clears when the app closes — consistent with ADR 0001's one-sitting "review now, hand off to a CLI agent" workflow, so there's no established need yet for it to survive a restart. Settings persist via the Tauri WebView's own `localStorage`, which already survives app restarts without any Rust involvement. This keeps the early command surface scoped to the git/filesystem reads described in ADR 0005, rather than adding IPC and a storage format for state that doesn't need one yet.

## Considered Options

- **Backend-persisted Comment Queue** (a Rust command + on-disk file): rejected for now — adds IPC surface and a storage format decision ahead of an actual need; revisit if reviewers want a queue that survives closing the app mid-review.
- **Backend-persisted Settings** (e.g. `~/.config/ziff/settings.json`): rejected for now — `localStorage` already covers cross-restart persistence for free; revisit only if settings need to be read outside the webview (e.g. by a CLI companion).
