# Implementation Plan: Tauri 2 Desktop POC

**Branch**: `codex/tauri-desktop-poc` | **Date**: 2026-11-11 | **Spec**: [spec.md](spec.md)

## Summary

Wrap the existing Vue/Vite client in Tauri 2 and bundle the matching `gloamy` daemon binary. Add a same-user Unix control socket for daemon readiness, pairing recovery, and graceful shutdown. The POC ends when an Apple Silicon Mac can open the app, pair, and load/save the existing configuration page. Native Linux builds validate the two other repository targets.

## Brownfield Findings

- **Current behavior and architecture**: `src/main.rs` routes `gloamy daemon` into `src/daemon/mod.rs`; `src/gateway/mod.rs` creates a `PairingGuard`, serves `/health`, `/pair`, and authenticated `/api/config`; `src/security/pairing.rs` creates a startup code only when no tokens exist and consumes it once. `web/` is a Vue 3/Vite app with an existing auth page, configuration editor, and Settings page. `web/src/stores/auth.ts` persists bearer token and API URL, and the router sends unauthenticated users through `/authentication` before `/configuration`.
- **Security condition**: the gateway currently accepts wildcard CORS, so pairing-code generation stays off HTTP. The control socket is same-user only and creates an additive pairing token through the existing `/pair` path.
- **Naming and governance**: use Rust `snake_case`, Vue/TypeScript project conventions, no config-schema changes, preserve the three supported targets, and keep high-risk runtime/security behavior localized and tested per `AGENTS.md`.
- **Files to create/modify**: create `web/src-tauri/` as an independent Cargo workspace; create `src/daemon/control.rs`; modify `src/daemon/mod.rs`, `src/gateway/mod.rs`, `src/security/pairing.rs`, and `src/main.rs`; add a Vue Tauri bridge and desktop controls in the existing auth/settings views; update the web locale JSON files, CI, `.gitignore`, and operational/security/troubleshooting docs. No existing user configuration or paired-token data is removed.
- **Preserve**: retain the existing browser SPA, `/pair` HTTP contract, bearer-token checks, config masking/persistence behavior, provider/channel runtime, and existing Cargo workspace. Do not add desktop dependencies to the root CLI crate.

## Technical Context

**Language / version**: Rust 2024, workspace MSRV 1.96; Vue 3, TypeScript, Vite, pnpm 12.18.

**Primary dependencies**: Tauri 2 in `web/src-tauri`; existing Vue frontend; existing `serde`, `serde_json`, Tokio Unix sockets, and `libc` in the daemon.

**Storage**: desktop-only `keep_daemon_running` boolean in Tauri app preferences, default false; no new Gloamy config key. Pairing code remains in memory. Paired bearer-token hashes continue to be persisted by the existing pairing handler.

**Testing and quality gates**: focused Rust unit tests for pairing, socket permissions/protocol, and lifecycle decisions; Vue unit tests for desktop bridge/auth/settings behavior; `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `pnpm lint`, `pnpm typecheck`, `pnpm test:unit`, and `pnpm build`. Build Tauri natively for each requested target.

**Supported host targets**:

| User label | Rust target | POC runner |
| --- | --- | --- |
| `linux-amd64` | `x86_64-unknown-linux-gnu` | `ubuntu-24.04` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` |
| `aarch64-darwin` | `aarch64-apple-darwin` | `macos-15` |

Linux runners install Tauri’s WebKitGTK 4.1 development dependencies. Builds are native, avoiding cross-linking host GUI libraries. Tauri uses WKWebView on macOS and WebKitGTK on Linux ([webview versions](https://v2.tauri.app/reference/webview-versions/), [Linux prerequisites](https://v2.tauri.app/start/prerequisites/)); sidecar names are target-suffixed as required by [Tauri sidecar packaging](https://v2.tauri.app/develop/sidecar/).

**Project type**: Rust CLI/runtime plus Vue SPA with an added Tauri desktop host.

**Constraints**: Sidecar spawning uses only a fixed executable and arguments from the Rust host. Frontend capabilities expose no generic shell permission. The control socket is a local owner-only channel under a private per-user temporary directory; the gateway gets no code-generation route. No signing, notarization, updater, tray, auto-start, Windows, or Intel macOS work.

## Constitution Check

- [x] Least privilege and explicit errors preserved.
- [x] Module boundaries and narrow traits preserved.
- [x] Change is minimal and reversible.
- [x] Validation follows repository risk tier.
- [x] Build, release, and docs agree with the three supported targets.

## Design and Implementation

### Daemon control and pairing

- Add `src/daemon/control.rs` with a one-request-per-connection newline-delimited JSON protocol over a per-user Unix socket. Operations: `status`, `request_pairing_code`, and `shutdown`. The server returns the actual gateway bind address, readiness, and the durable `desktop_managed` marker.
- Create the socket under `std::env::temp_dir()/gloamy-<effective-uid>/control.sock`; enforce directory mode `0700`, socket mode `0600`, owner checks, stale-socket validation, and no symlink replacement. Reject unsafe paths and occupied sockets.
- Add `PairingGuard::request_pairing_code()` to create a cryptographically random replacement code only when pairing is enabled. It is returned only over the local socket, consumed by the existing `/pair` handler, and never logged or persisted. Existing paired hashes remain intact.
- Share one `Arc<PairingGuard>` between the daemon control server and gateway supervisor so gateway restarts do not invalidate the pending code. Keep standalone `gloamy gateway` pairing behavior unchanged.
- Add a hidden `--desktop-managed` daemon argument and hidden `desktop-info` command. The former makes ownership survive app restarts and suppresses console pairing-code output for app-managed daemons; the latter emits only configured bind host/port for legacy-daemon health probing. Neither is added to normal CLI help/docs.
- Run the daemon until Ctrl+C or socket shutdown; then stop supervisors and remove only the socket created by that daemon. Control status remains unavailable until the gateway publishes its actual bound address.

### Tauri host and frontend

- Create a separate `[workspace]` at `web/src-tauri/Cargo.toml`, with `tauri.conf.json`, `build.rs`, `src/main.rs`, and a narrowly permissioned `capabilities/default.json`. Reuse `web/dist`; set the initial hash route to `/configuration`.
- Add `scripts/build-tauri-sidecar.sh` to build the root `gloamy` binary for the native host and stage it as `web/src-tauri/resources/gloamy`. Bundle it as a Tauri resource and launch it from the Rust host with `std::process::Command` and fixed arguments. Do not use the shell plugin: its child cleanup on app exit conflicts with the keep-running preference. Never add Tauri crates to root `Cargo.toml`.
- In the Rust host, query the socket first. If a valid daemon exists, attach and respect its managed marker. If no socket exists, query `desktop-info`, probe the configured address’s `/health`, and attach a valid legacy Gloamy daemon as unmanaged. If no daemon is healthy, launch the bundled binary with fixed `daemon --desktop-managed` arguments, then poll the socket for readiness. If the port is occupied by another service or startup fails, report an explicit error and do not retry by starting duplicates.
- Use the socket’s actual listener address to supply `api_base_url` to the Vue auth store before token validation; convert wildcard listeners to loopback. If a specific non-loopback listener cannot be safely reached by the packaged webview, show a clear unsupported-bind error.
- Store the keep-running preference in the Tauri app config directory with restrictive permissions. On exit, request socket shutdown only if the daemon status says desktop-managed and the preference is Off. With the preference On, detach and leave it available to browser UI. Never issue shutdown to an unmanaged daemon.
- Add `web/src/lib/desktop.ts` to expose only typed IPC for status, pairing code, and preference read/write. Do not add shell/process plugins or frontend process permissions. Apply Tauri capabilities and CSP only to the app bundle and loopback daemon API ([capability model](https://v2.tauri.app/security/capabilities/)).
- On the auth page, show a desktop-only “Request local pairing code” action, display/copy the one-time code, and use the existing pairing input and `/pair` flow. Replace the destructive “clear paired tokens” recovery advice with instructions to use desktop recovery. In Settings, add desktop-only daemon status, the default-Off keep-running toggle, and a second pairing-code recovery action. Translate new strings in `en`, `fr`, and `tr`, the existing web locales.

### Builds, docs, and CI

- Add a web script for Tauri dev/build. The host build runs the frontend build and sidecar staging as prerequisites.
- Add Tauri build jobs to `.github/workflows/ci.yml`: `macos-15` for Apple Silicon, `ubuntu-24.04` for Linux x64, and `ubuntu-24.04-arm` for Linux ARM64. Install WebKitGTK 4.1 and bundler dependencies on Linux; produce a local `.app` / CI `.deb` build artifact, not a public release.
- Add generated Tauri build directories and staged target sidecars to `.gitignore`.
- Update English and localized operational, security, and troubleshooting references for the local control socket, managed-daemon preference, and pairing recovery. Keep README navigation unchanged.

### Project Structure

```text
src/daemon/control.rs                 # Unix socket protocol and permissions
src/daemon/mod.rs                     # daemon lifetime, control server, shared pairing state
src/gateway/mod.rs                   # shared pairing guard and bound-address publication
src/security/pairing.rs              # one-time recovery code issuance
web/src-tauri/                       # isolated Tauri app, capabilities, native host
web/src/lib/desktop.ts               # typed Tauri IPC bridge
scripts/build-tauri-sidecar.sh        # per-host daemon staging
specs/002-tauri-desktop-poc/         # feature docs and contracts
```

**Structure Decision**: The Tauri host is a separate nested Cargo workspace and bundles the existing Rust CLI as a target-specific sidecar. The daemon’s Unix control protocol stays inside `src/daemon`; pairing state remains owned by `src/security/pairing` and shared with the gateway.

## Complexity Tracking

| Violation | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| Local control socket plus hidden daemon metadata command | Allows same-user recovery and safe start/attach/shutdown without opening a new HTTP security boundary; metadata supports legacy daemons | An unauthenticated HTTP recovery endpoint conflicts with wildcard CORS and remote gateway exposure; guessing the configured port would fail custom configs |
