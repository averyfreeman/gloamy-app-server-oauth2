# Quickstart: Tauri 2 Desktop POC

## Prerequisites

- Rust toolchain at repository MSRV or newer and Node/pnpm versions declared under `web/`.
- macOS: Xcode Command Line Tools for the Apple Silicon app.
- Linux: native runner for the target architecture plus Tauri WebKitGTK 4.1 build dependencies.

## Local development

From `web/`:

```bash
pnpm install --frozen-lockfile
pnpm tauri dev
```

The Tauri dev command starts Vite and builds/stages a native `gloamy` sidecar. On first run, the app starts the daemon if no healthy local Gloamy daemon is available, then opens the existing configuration route. On an unauthenticated install, choose **Request local pairing code**, copy or enter the code, and complete the existing pairing flow.

## Build the native target

From `web/`:

```bash
pnpm tauri build
```

The host build stages the current machine’s sidecar and produces a local Tauri bundle. Linux CI uses native x64 and ARM64 runners and produces `.deb` artifacts for compatibility validation.

## End-to-end checks

1. With no daemon running, launch the app and verify one managed daemon becomes ready.
2. Pair, load the configuration screen, edit a harmless value, save, and reload it.
3. Set keep-running Off, close the app, and confirm the managed daemon stops.
4. Set keep-running On, close and reopen the app, and confirm the same managed daemon remains reachable.
5. With a pre-existing unmanaged Gloamy daemon, launch and close the app; verify it remains running.
6. With existing paired clients, request a fresh code, pair another UI, and verify old tokens remain valid.

## Validation commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cd web && pnpm lint && pnpm typecheck && pnpm test:unit && pnpm build
```
