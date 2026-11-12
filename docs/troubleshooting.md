# Gloamy Troubleshooting

This guide focuses on common setup/runtime failures and fast resolution paths.

Last verified: **November 9, 2026**.

## Installation / Bootstrap

### `cargo` not found

Symptom:

- bootstrap exits with `cargo is not installed`

Fix:

```bash
./bootstrap.sh --install-rust
```

Or install from <https://rustup.rs/>.

### Missing system build dependencies

Symptom:

- build fails due to compiler or `pkg-config` issues

Fix:

```bash
./bootstrap.sh --install-system-deps
```

### Build fails on low-RAM / low-disk hosts

Symptoms:

- `cargo build --release` is killed (`signal: 9`, OOM killer, or `cannot allocate memory`)
- Build crashes after adding swap because disk space runs out

Why this happens:

- Runtime memory (<5MB for common operations) is not the same as compile-time memory.
- Full source build can require **2 GB RAM + swap** and **6+ GB free disk**.
- Enabling swap on a tiny disk can avoid RAM OOM but still fail due to disk exhaustion.

Preferred path for constrained machines:

```bash
./bootstrap.sh --prefer-prebuilt
```

Binary-only mode (no source fallback):

```bash
./bootstrap.sh --prebuilt-only
```

If you must compile from source on constrained hosts:

1. Add swap only if you also have enough free disk for both swap + build output.
1. Limit cargo parallelism:

```bash
CARGO_BUILD_JOBS=1 cargo build --release --locked
```

1. Cross-compile on a stronger machine and copy the binary to the target host.

### Build is very slow or appears stuck

Symptoms:

- `cargo check` / `cargo build` appears stuck at `Checking gloamy` for a long time
- repeated `Blocking waiting for file lock on package cache` or `build directory`

Why this happens in Gloamy:

- Matrix E2EE stack (`matrix-sdk`, `ruma`, `vodozemac`) is large and expensive to type-check.
- TLS + crypto native build scripts (`aws-lc-sys`, `ring`) add noticeable compile time.
- `rusqlite` with bundled SQLite compiles C code locally.
- Running multiple cargo jobs/worktrees in parallel causes lock contention.

Fast checks:

```bash
cargo check --timings
cargo tree -d
```

The timing report is written to `target/cargo-timings/cargo-timing.html`.

Faster local iteration (when Matrix channel is not needed):

```bash
cargo check
```

This uses the lean default feature set and can significantly reduce compile time.

To build with Matrix support explicitly enabled:

```bash
cargo check --features channel-matrix
```

To build with Matrix + Lark support:

```bash
cargo check --features channel-matrix,channel-lark
```

Lock-contention mitigation:

```bash
pgrep -af "cargo (check|build|test)|cargo check|cargo build|cargo test"
```

Stop unrelated cargo jobs before running your own build.

### `gloamy` command not found after install

Symptom:

- install succeeds but shell cannot find `gloamy`

Fix:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
which gloamy
```

Persist in your shell profile if needed.

## Runtime / Gateway

### Desktop app reports that the configured gateway address is occupied

The desktop app will not start a second daemon when another service is already
using the configured gateway host and port. Stop that service or change
`gateway.host` / `gateway.port` in `~/.gloamy/config.toml`, then reopen Gloamy.
The desktop webview currently connects only to `127.0.0.1` or `::1`; a specific
non-loopback bind address is not supported by this POC.

### Desktop pairing recovery is unavailable

Use Authentication or Settings inside the desktop app to request a fresh code.
Browser clients cannot request one because code generation is available only on
the same-user Unix control socket. If pairing is disabled, the daemon rejects
the request. Requesting a new code does not invalidate already paired tokens.

### Desktop app cannot reach an existing daemon

The app attaches only to a healthy daemon at the configured local address. Check
that `gloamy daemon` is running and that `gateway.host` / `gateway.port` match.
The app never stops an attached daemon unless its local control status marks it
as desktop-managed; a legacy daemon without a control socket is treated as
unmanaged.

### HostAgent perception fails on macOS

Symptoms:

- `gloamy agent` fails with `HostAgent runtime perception failed`
- interactive CLI turns stop before tool execution on macOS

Why this happens:

- the HostAgent runtime treats perception as a hard dependency
- missing Accessibility or Screen Recording permission is surfaced as a turn failure instead of silently degrading to an empty screen state

Fix:

1. Grant Accessibility permission to the app hosting the CLI session (for example Terminal, iTerm, Warp, VS Code, or Cursor)
1. Grant Screen Recording permission if screenshot capture is also blocked
1. Restart the affected app after changing macOS privacy settings
1. Re-run the same `gloamy agent` command

### HostAgent perception fails on Linux

Symptoms:

- `gloamy agent` fails with `HostAgent runtime perception failed`
- interactive CLI turns stop before tool execution

Why this happens:

- HostAgent runtime perception no longer accepts empty runtime state fallback
- screenshot capture and widget-tree runtime context must produce at least one usable signal
- missing screenshot backend binaries or blocked desktop session permissions can leave runtime signals empty

Fix:

1. On Linux, install at least one screenshot backend available in PATH (`gnome-screenshot`, `scrot`, or `import` from ImageMagick)
1. Ensure the process has desktop session access (for example X11/Wayland permissions in the active user session)
1. Re-run `gloamy agent` after confirming host desktop capture is permitted

### `mac_automation click_at` is blocked by perception policy

Symptoms:

- runtime returns `Blocked by runtime policy: call perception_capture...`

Why this happens:

- coordinate clicks now require a successful `perception_capture` preflight with `include_widget_tree=true` and `include_ocr=true`
- partial or degraded perception output no longer satisfies the click preflight

Fix:

1. Call `perception_capture` with both `include_widget_tree=true` and `include_ocr=true`
1. Confirm the tool output reports both modalities as completed
1. Retry the `mac_automation click_at` action only after the successful preflight

### OCR extraction fails during `perception_capture`

Symptoms:

- runtime returns `OCR extraction failed: ...`
- `perception_capture` diagnostics show `ocr.completed = false`

Checks:

```bash
which tesseract
tesseract --version
```

Fix:

1. Install Tesseract if it is missing from the host
1. If the runtime needs a non-default language pack or tessdata path, pass the `ocr` object in the tool call:

```json
{
  "include_widget_tree": true,
  "include_ocr": true,
  "ocr": {
    "language": "eng",
    "psm": 11,
    "oem": 1
  }
}
```

1. Set `ocr.tessdata_dir` when traineddata files live outside the system default location

### Gateway unreachable

Checks:

```bash
gloamy status
gloamy doctor
```

Verify `~/.gloamy/config.toml`:

- `[gateway].host` (default `127.0.0.1`)
- `[gateway].port` (default `42617`)
- `allow_public_bind` only when intentionally exposing LAN/public interfaces

### Gateway opens in browser but no dashboard appears

Expected behavior:

- the legacy browser dashboard has been removed
- `gloamy gateway` now serves webhook, API, and WebSocket endpoints for desktop and external clients

What to do:

- use the web app for the primary UI
- use gateway routes only for integrations, automation, health checks, and API traffic

### Pairing / auth failures on webhook

Checks:

1. Ensure pairing completed (`/pair` flow)
2. Ensure bearer token is current
3. Re-run diagnostics:

```bash
gloamy doctor
```

## Channel Issues

### Telegram conflict: `terminated by other getUpdates request`

Cause:

- multiple pollers using same bot token

Fix:

- keep only one active runtime for that token
- stop extra `gloamy daemon` / `gloamy channel start` processes

### Channel unhealthy in `channel doctor`

Checks:

```bash
gloamy channel doctor
```

Then verify channel-specific credentials + allowlist fields in config.

## Service Mode

### Service installed but not running

Checks:

```bash
gloamy service status
```

Recovery:

```bash
gloamy service stop
gloamy service start
```

Linux logs:

```bash
journalctl --user -u gloamy.service -f
```

## Legacy Installer Compatibility

Both still work:

```bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/bootstrap.sh | bash
curl -fsSL https://raw.githubusercontent.com/iBz-04/gloamy/main/scripts/install.sh | bash
```

`install.sh` is a compatibility entry and forwards/falls back to bootstrap behavior.

## Model Selection and Catalog

### No eligible default model

**Symptom:** Startup, onboarding, or a new operation says a model ID is required.

**Cause:** The provider has no recommended model within the catalog's eight-calendar-month release window, or its metadata has not been verified.

**Fix:** Discover IDs from the configured provider, inspect the results, then choose one explicitly:

```bash
gloamy models refresh --provider <PROVIDER_ID>
gloamy models list --provider <PROVIDER_ID>
gloamy models set <EXACT_MODEL_ID>
```

You can also set `default_model` in `config.toml`. Gloamy preserves configured and dynamically discovered IDs when they are not in the curated catalog.

### Invalid local catalog

**Symptom:** Config loading fails with a `model-catalog.toml` parse or validation error, or authenticated reload returns `400`.

**Fix:** Correct the complete local file or remove it if the bundled catalog is desired. A present invalid file does not fall back to the bundled file. A failed reload in a running daemon keeps its last valid in-memory snapshot; run `gloamy models list` locally to see the loading error. Daemon logs omit file contents.

### Cost is zero or lower than expected

An unknown model rate is represented as zero while input/output token counts are still recorded. This means unknown, not free, and leaves cost totals and budget calculations as lower bounds. Add a verified exact entry under `[cost.prices]` when you know the applicable rate. The catalog does not infer aliases, suffix matches, or OpenAI API prices for Codex subscription/OAuth usage.

See the [catalog and pricing reference](config-reference.md#curated-model-catalog) and [gateway reload endpoint](gateway-api-reference.md#post-apimodelscatalogreload).

## Still Stuck?

Collect and include these outputs when filing an issue:

```bash
gloamy --version
gloamy status
gloamy doctor
gloamy channel doctor
```

Also include OS, install method, and sanitized config snippets (no secrets).

## Codex OAuth Setup in Integrations

- **The OpenAI API card does not start ChatGPT sign-in:** choose **OpenAI Codex**, which uses ChatGPT OAuth. Other cards and skills without implemented setup remain noninteractive.
- **Authorization expired or was denied:** start a new flow. The daemon owns device-code polling and expiry; refreshing the page does not extend the deadline. Reconnect when credentials have been revoked.
- **Cancellation could not reach the daemon:** the dialog reports that cancellation is unconfirmed. Retry or inspect status; the server flow can still finish before its deadline. Cancellation after credential persistence starts cannot undo a successful connection.
- **OAuth succeeded but credential save failed:** the flow reports an error, not connected success. Repair local auth-directory permissions or storage availability and retry; do not move tokens into browser storage or config text.
- **Model discovery is unavailable:** curated fallback and custom IDs remain available. The Codex backend may reject a compatibility version/originator or vary by account. Public OpenAI `/v1/models` is not a ChatGPT OAuth fallback, and a separate local Codex CLI login may be a different account.
- **Applying a model leaves the provider unchanged:** connection alone preserves selection. Select **Use Codex as default** before switching from another provider. If a global API URL override is configured, remove it explicitly in Configuration before switching; unrelated values are preserved.

See [verified sources and limitations](codex-oauth-sources.md) and [setup API states](gateway-api-reference.md#codex-oauth-integration-setup).

## Related Docs

- [operations-runbook.md](operations-runbook.md)
- [one-click-bootstrap.md](one-click-bootstrap.md)
- [channels-reference.md](channels-reference.md)
- [network-deployment.md](network-deployment.md)
