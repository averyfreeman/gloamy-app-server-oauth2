# Feature Specification: Tauri 2 Desktop POC

**Feature Branch**: `codex/tauri-desktop-poc`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User description: "Implement Tauri 2 for macOS aarch64-darwin, with Linux x86_64 and aarch64 compatibility if practical. Stop at a working desktop window and the existing configuration screen. The app can start a daemon if none is running, lets the user keep an app-managed daemon alive for browser UI use, and provides local pairing recovery."

## User Scenarios & Testing

### User Story 1 - Open and use configuration in a desktop window (Priority: P1)

As a Gloamy operator on Apple Silicon macOS, I can launch a desktop app and use the existing configuration screen so I can verify the desktop UI infrastructure without changing the established configuration experience.

**Independent Test**: Launch the packaged macOS ARM64 app against a local daemon, authenticate, load the existing configuration screen, and save a valid configuration.

**Acceptance Scenarios**:

1. **Given** the desktop app is installed and no daemon is running, **When** the app starts, **Then** it starts a local daemon and opens the existing configuration UI after authentication.
2. **Given** a reachable local Gloamy daemon is already running, **When** the app starts, **Then** it connects to that daemon without starting a duplicate or claiming ownership.
3. **Given** the user is authenticated, **When** they load and save configuration, **Then** the existing gateway API reads and persists it successfully.

### User Story 2 - Recover access and pair a desktop or browser client (Priority: P1)

As an operator whose configuration already contains paired-token hashes, I can request a fresh local pairing code so I can pair a new UI without deleting existing clients.

**Independent Test**: With existing paired-token hashes and no bearer token in the new UI, request a code through the local control channel, pair successfully, and verify existing tokens remain valid.

**Acceptance Scenarios**:

1. **Given** pairing is required and one or more tokens are already paired, **When** the user requests a code in the desktop UI, **Then** a fresh one-time code is returned without removing existing tokens.
2. **Given** a valid pending code, **When** a client pairs, **Then** the code is consumed once and its token hash is persisted through the existing pairing flow.
3. **Given** pairing recovery is requested, **When** the gateway is reachable over HTTP, **Then** it does not expose a new unauthenticated code-generation endpoint.

### User Story 3 - Control daemon lifetime from the desktop app (Priority: P1)

As an operator, I can choose whether a daemon started by the desktop app remains available after the app closes, so browser UI use is possible without the desktop window.

**Independent Test**: Start the app with no daemon, verify the default closes its managed daemon, then enable keep-running, close and reopen the app, and verify the same daemon remains available and is recognized as desktop-managed.

**Acceptance Scenarios**:

1. **Given** no daemon is running, **When** the desktop app starts, **Then** it starts one and records that it is Tauri-managed.
2. **Given** the keep-running preference is Off, **When** the app closes, **Then** it requests graceful shutdown only for a Tauri-managed daemon.
3. **Given** the keep-running preference is On, **When** the app closes, **Then** the managed daemon continues serving the browser UI.
4. **Given** the app attached to a daemon it did not start or mark as desktop-managed, **When** the app closes, **Then** it does not stop that daemon.
5. **Given** a non-Gloamy service occupies the configured local address, **When** the app starts, **Then** it reports a conflict instead of launching a duplicate daemon.

### User Story 4 - Build for the repository's supported desktop hosts (Priority: P2)

As a maintainer, I can build the same desktop UI infrastructure for all supported host triples so desktop support does not fork the existing Rust and Vue application.

**Independent Test**: Build the app and matching daemon sidecar for macOS ARM64, Linux x86_64, and Linux ARM64 on native runners.

**Acceptance Scenarios**:

1. **Given** native host dependencies are installed, **When** the macOS ARM64 build runs, **Then** it produces a launchable Tauri app bundle.
2. **Given** WebKitGTK 4.1 build dependencies are installed, **When** either Linux build runs on a matching native architecture, **Then** it packages the Tauri app and matching daemon sidecar.

### Edge Cases

- A daemon is running but does not support local desktop control: attach only if it is identified as Gloamy; treat it as unmanaged and do not stop it.
- Two desktop launches race to start a daemon: only one daemon binds the control socket; the other attaches to the valid instance or reports the conflict.
- A socket path is stale, owned by another user, or has unsafe permissions: refuse to use it and report a local control error.
- Pairing is disabled: recovery is unavailable and the existing authentication behavior remains in effect.
- The gateway is bound to a non-loopback interface: the desktop app uses a local address only when the daemon listener makes that address reachable; otherwise report a clear connection error.
- The daemon exits while the desktop window is open: show an unavailable state and allow reconnecting after the daemon is healthy again.

## Requirements

### Functional Requirements

- **FR-001**: The desktop app MUST use the existing Vue/Vite UI and show the existing configuration screen in a Tauri 2 window.
- **FR-002**: The desktop app and its daemon MUST build for each supported target: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `aarch64-unknown-linux-gnu`.
- **FR-003**: On startup, the desktop host MUST attach to a healthy local Gloamy daemon when one is already running and MUST NOT start a duplicate.
- **FR-004**: If no healthy local Gloamy daemon is running, the desktop host MUST start its bundled daemon and wait for readiness before opening the authenticated UI.
- **FR-005**: The daemon MUST provide a local owner-only control path with only status/address, fresh pairing-code, and graceful-shutdown operations; this path MUST NOT be exposed through the network API.
- **FR-006**: The local control path MUST be restricted to the current operating-system user; invalid or foreign-owned control endpoints MUST be rejected.
- **FR-007**: The daemon MUST identify whether it was started by the desktop app so that ownership survives the app window closing and reopening.
- **FR-008**: The desktop preference “Keep daemon running after closing” MUST default to Off and MUST be stored in desktop-app preferences, not in Gloamy runtime configuration.
- **FR-009**: On app close with the preference Off, the desktop host MUST request graceful shutdown only for a daemon marked Tauri-managed. With the preference On, it MUST leave that daemon running.
- **FR-010**: The desktop app MUST NOT stop a daemon it attached to without a Tauri-managed marker.
- **FR-011**: Pairing recovery MUST issue a fresh, one-time code through the local owner-only control path even when existing clients are paired.
- **FR-012**: Pairing recovery MUST preserve existing paired clients and MUST not persist or log the plaintext code; successful pairing MUST continue to use the existing pairing flow.
- **FR-013**: The network API MUST NOT add an unauthenticated operation for generating pairing codes.
- **FR-014**: The desktop UI MUST expose only the app operations required for daemon status, pairing recovery, and the keep-running preference; it MUST NOT provide arbitrary operating-system command execution.
- **FR-015**: If a non-Gloamy service occupies the configured address, the app MUST report a conflict and MUST NOT start another daemon.
- **FR-016**: Build validation MUST cover native macOS ARM64, Linux x86_64, and Linux ARM64 targets; Linux validation MUST install Tauri’s WebKitGTK 4.1 prerequisites.
- **FR-017**: User-facing operational, security, and troubleshooting documentation affected by the feature MUST follow the repository’s supported locale policy.

### Existing Project Impact

- **User-facing surfaces**: Adds a desktop window and desktop-only daemon lifetime and pairing recovery controls. Keeps the existing browser UI, Gloamy runtime configuration schema, provider/channel behavior, and pairing contract. No general CLI behavior change is intended.
- **Brownfield paths**: Adds `web/src-tauri/` as an independent Cargo workspace; integrates with the existing `web/` Vite application and configuration/authentication views; changes daemon lifecycle in `src/daemon/mod.rs`, gateway pairing in `src/gateway/mod.rs`, and code generation in `src/security/pairing.rs`; updates relevant docs and CI workflows.
- **Compatibility and migration**: No Gloamy configuration migration. Existing paired clients remain valid. A daemon without local desktop control may be attached through its existing health API but is never treated as desktop-managed or shut down by the desktop app.
- **Preservation boundary**: Do not add a public HTTP code-generation route, change existing bearer-token validation, stop manually/service-managed daemons, broaden Tauri shell permissions, or add unsupported host architectures.

### Key Entities

- **Desktop-managed daemon**: A daemon started with the Tauri-managed marker; its status is observable over the local control channel and remains identifiable when the desktop window closes.
- **Desktop preference**: A local app preference controlling whether a Tauri-managed daemon is left running when the window closes; defaults to Off.
- **Pending pairing code**: A single-use, in-memory code that can provision an additional paired token without revoking existing clients.

## Success Criteria

- **SC-001**: On an Apple Silicon Mac, the built app opens a window, reaches the existing configuration screen, and can load and save a valid configuration after pairing.
- **SC-002**: With existing token hashes and no bearer token in the UI, an operator can pair a new client without deleting or invalidating any existing paired token.
- **SC-003**: Across startup and close scenarios, the app starts at most one daemon, leaves unmanaged daemons untouched, and follows the keep-running preference for managed daemons.
- **SC-004**: Native builds succeed for all three configured target triples, including Linux ARM64.
- **SC-005**: No plaintext pairing code is written to runtime configuration or application logs, and no HTTP code-generation route is exposed.

## Assumptions

- Linux support in this POC means native build/package compatibility on the two specified Linux architectures, not a distro release matrix or long-term runtime support promise.
- The existing Vue configuration UI remains the desktop configuration UI.
- Code signing, notarization, updater, auto-start, tray UI, and broader screen redesign are outside the POC.
- The macOS ARM64 working window and configuration flow define the completion boundary; Linux targets are build-compatibility checks.
