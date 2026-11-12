# Tauri IPC Contract

Only the bundled local frontend invokes these commands; no shell or arbitrary process API is granted to JavaScript.

- `get_daemon_status() -> { ready, api_base_url, desktop_managed }`
- `request_pairing_code() -> string`
- `get_keep_daemon_running() -> boolean`
- `set_keep_daemon_running(value: boolean) -> void`

The Rust host stores preferences under the Tauri app config directory. The `set` command writes atomically with owner-only file permissions. The frontend hides these controls outside Tauri.
