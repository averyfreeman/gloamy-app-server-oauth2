# Research: Tauri 2 Desktop POC

## Decision: Reuse the Vue client inside Tauri 2

**Rationale**: The current frontend is already a Vue/Vite SPA and includes the configuration, authentication, and settings routes. Tauri provides a native macOS/Linux window around the system webview, avoiding a second UI implementation and aligning with the Rust host runtime.

**Alternatives considered**:
- Electron: reuses Vue but adds a Node/Chromium desktop runtime and duplicates process integration.
- SwiftUI: native macOS UI, but requires recreating the existing configuration/auth flows and does not cover Linux targets.
- Rust-native GUI: fits Rust but requires a new UI instead of using the existing SPA.

Sources: [Tauri architecture](https://v2.tauri.app/concept/architecture/), [webview versions](https://v2.tauri.app/reference/webview-versions/).

## Decision: Keep Tauri in a separate nested Cargo workspace and bundle the existing `gloamy` binary as a resource

**Rationale**: The root crate is a published Rust CLI/runtime with explicit binary-size and dependency constraints. A nested Tauri workspace prevents GUI dependencies from entering the CLI dependency graph. The build stages one host-matched daemon resource per native build. The Rust host resolves that resource and launches it with fixed arguments.

**Alternatives considered**: Add Tauri to the root workspace; this broadens the published crate’s dependency surface and build graph. Reimplement or link daemon internals in the Tauri host; this couples the desktop app to crate-private runtime modules and complicates shutdown ownership. Use Tauri’s shell plugin to launch the resource; the plugin cleans up child processes when the app exits, defeating the keep-running preference.

Source: [Tauri resource directory](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html).

## Decision: Use a same-user Unix socket for status, recovery, and shutdown

**Rationale**: The current gateway permits wildcard CORS, and the existing `/pair` route is network-facing. Adding unauthenticated code generation there would enlarge the remote attack surface. A user-owned socket with restrictive permissions keeps the privileged operation local and lets the daemon expose a narrow status/control contract.

**Alternatives considered**: Add a localhost HTTP route (still subject to local web-origin and CORS complications); use a shared secret file (introduces new secret persistence and rotation); use only process signals (cannot request a fresh pairing code or query current bind readiness).

## Decision: Keep a single pairing guard shared between daemon control and gateway

**Rationale**: The gateway already owns pairing and persists only token hashes after the existing `/pair` flow. The desktop recovery operation should mutate that same in-memory pending code and preserve the established persistence path. A shared guard survives gateway supervisor restarts while the daemon process remains alive.

**Compatibility**: Existing bearer token validation, pairing rate limits, config masking, and paired token hashes remain unchanged. A fresh code is not written to the configuration or log.

## Decision: Build Linux targets natively

**Rationale**: Tauri Linux builds link to host GUI libraries, including WebKitGTK 4.1. Native x64 and ARM64 runners avoid unsupported cross-linker assumptions. The ARM64 runner label is documented by GitHub.

Sources: [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/), [GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

## Risks and mitigations

- **Legacy daemon detection**: Use a hidden, non-secret `desktop-info` command to read resolved gateway bind settings; probe `/health` before launching. Treat successful legacy daemons as unmanaged.
- **Socket path security**: Validate directory owner, file type, and mode; do not follow symlinks; remove only a verified stale socket.
- **Exit timing**: Prevent app exit until an owned daemon has acknowledged shutdown. Bound the wait and report failure without signaling an unmanaged PID.
- **Linux GUI dependencies**: CI installs the documented WebKitGTK 4.1 development packages and builds natively for each architecture.
