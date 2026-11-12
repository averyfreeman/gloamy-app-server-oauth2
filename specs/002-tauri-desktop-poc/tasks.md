# Tasks: Tauri 2 Desktop POC

**Input**: Design documents from `specs/002-tauri-desktop-poc/`

**Prerequisites**: `plan.md` and `spec.md`; include `research.md`, `data-model.md`, `quickstart.md`, and contracts when the plan creates them.

**Tests**: Explicitly requested by the feature acceptance criteria and repository risk policy.

## Phase 1: Brownfield Inventory

- [x] T001 Record current worktree and branch state in the feature handoff.
- [x] T002 Inventory daemon, gateway, pairing, Vue auth/config/settings, docs, CI, and target paths in `specs/002-tauri-desktop-poc/plan.md`.
- [x] T003 Identify existing Rust and Vue checks and inspect current CI in `specs/002-tauri-desktop-poc/plan.md`.
- [x] T004 Record config/token compatibility and preservation boundaries in `specs/002-tauri-desktop-poc/spec.md`.

## Phase 2: Foundational Daemon Control

- [x] T005 [P] Add `PairingGuard` unit tests for code regeneration, one-use consumption, disabled pairing, and preserving existing tokens in `src/security/pairing.rs`.
- [x] T006 Implement local one-time code issuance in `src/security/pairing.rs`.
- [x] T007 [P] Add control protocol, private directory/socket permissions, stale-socket, and operation tests in `src/daemon/control.rs`.
- [x] T008 Implement the Unix-domain control server and safe socket lifecycle in `src/daemon/control.rs`.
- [x] T009 Share pairing state, publish actual gateway address, and stop daemon supervisors on control shutdown in `src/daemon/mod.rs` and `src/gateway/mod.rs`.
- [x] T010 Add hidden `desktop-info` and `--desktop-managed` interfaces in `src/main.rs`.

## Phase 3: User Story 1 - Desktop Window and Configuration

- [x] T011 Add the nested Tauri workspace, app configuration, capabilities, and target sidecar resource build script in `web/src-tauri/` and `scripts/build-tauri-sidecar.sh`.
- [x] T012 Add daemon start/attach/readiness and exit ownership tests in `web/src-tauri/src/manager.rs`.
- [x] T013 Implement the Tauri daemon manager and preference persistence in `web/src-tauri/src/manager.rs` and `web/src-tauri/src/main.rs`.
- [x] T014 Add the typed Tauri bridge and load its actual API URL before auth validation in `web/src/lib/desktop.ts` and `web/src/stores/auth.ts`.
- [x] T015 Add frontend tests for desktop runtime initialization and browser-mode compatibility in `web/tests/unit/desktop.spec.ts`.
- [x] T016 Verify the existing configuration page loads and saves through the Tauri-provided API URL in `web/src/pages/configuration.vue` and its existing tests.

## Phase 4: User Story 2 - Pairing Recovery

- [x] T017 Add authentication recovery UI and browser guidance in `web/src/pages/authentication.vue` and use the existing `/pair` API flow.
- [x] T018 Add desktop settings recovery control and keep-running toggle in `web/src/pages/settings.vue`.
- [x] T019 Add the new UI strings to `web/src/i18n/locales/en.json`, `web/src/i18n/locales/fr.json`, and `web/src/i18n/locales/tr.json`.

## Phase 5: User Story 3 - Managed Daemon Lifetime

- [x] T020 Ensure keep-running defaults Off, persists in native app preferences, and is applied on window exit/reopen in `web/src-tauri/src/manager.rs`.
- [x] T021 Verify and test that shutdown requests are sent only to daemons marked desktop-managed in `web/src-tauri/src/manager.rs`.

## Phase 6: User Story 4 - Three-Target Builds

- [x] T022 Add native Tauri build jobs for macOS ARM64, Linux x64, and Linux ARM64 with required WebKitGTK dependencies in `.github/workflows/ci.yml`.
- [x] T023 Add Tauri generated outputs and staged sidecars to `.gitignore`.
- [x] T024 Validate host-specific sidecar staging and native build outputs in `scripts/build-tauri-sidecar.sh` and CI.

## Phase 7: Documentation and Security Review

- [x] T025 Document daemon control and pairing recovery in `docs/operations-runbook.md`, `docs/security/`, and `docs/troubleshooting.md`.
- [x] T026 Add desktop guidance for all six supported documentation locales and update shared navigation/indexes.
- [x] T027 Review Tauri capabilities, CSP, socket ownership, code logging, and daemon lifecycle boundaries in `web/src-tauri/capabilities/` and the complete diff.

## Phase 8: Validation and Delivery

- [x] T028 Run authorized Rust formatting, clippy, and test checks and record results.
- [x] T029 Run web lint, typecheck, unit tests, and production build.
- [x] T030 Build and launch the macOS ARM64 app; verify pairing and configuration load/save.
- [x] T031 Build Linux x86_64 and ARM64 app bundles on native runners.
- [x] T032 Review locale parity, lockfiles, generated files, and sensitive data in the final diff.

## Validation Results

- `cargo test --locked`: passed the complete root Rust suite (3,360 library tests plus integration and documentation tests).
- `cargo test --manifest-path web/src-tauri/Cargo.toml --locked`: passed (5 tests).
- `cargo clippy --manifest-path web/src-tauri/Cargo.toml --all-targets --locked -- -D warnings`: passed.
- Tauri host formatting and targeted formatting for changed root Rust files: passed.
- `pnpm lint`, `pnpm typecheck`, `pnpm test:unit`, and `pnpm build`: passed (12 frontend unit tests).
- `pnpm exec tauri build --bundles app --ci`: passed natively on `aarch64-apple-darwin`; the app opened, requested a one-time recovery code, paired, loaded the existing configuration view, and saved a configuration edit.
- Lifecycle smoke checks passed for keep-running Off and On across app exit/restart. The Off path stopped only the desktop-managed daemon; the On path left it running, and a restarted app reattached. The test config, pairing token, app preference, daemon, and socket were cleaned up after verification.
- `git diff --check` and local Markdown link validation: passed. The touched operational runbook has seven pre-existing Markdown ordered-list warnings (`MD029`) under the repository's Markdown lint configuration.
- Repository-wide `cargo fmt --all -- --check` reports existing formatting drift across untouched root files with the current Rust 1.99 formatter. Repository-wide `cargo clippy --all-targets --locked -- -D warnings` reports 11 pre-existing lints in untouched modules; no changed Rust files were among those lint locations. These two repository baseline failures are separate from the passing focused checks above.
- Native GitHub CI builds passed for all three targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu`. The Linux Tauri jobs produced successful `.deb` bundles on the matching x64 and ARM64 runners.

## Dependencies and Execution Order

- T005 precedes T006; T007 precedes T008; daemon/gateway integration follows both control and pairing changes.
- T011 and T013 establish the native host before frontend bridge/UI integration.
- Pairing UI and daemon settings depend on the bridge and manager commands.
- Linux CI and documentation depend on the native control contract but can be completed after the first local macOS bundle works.
- Validation is last; stop after macOS window/configuration proof and Linux target build checks.
