# Quickstart: Remaining Feature Validation

## Prerequisites

- Checkout `feat/ui-platform-foundation` and initialize the pinned `web` submodule.
- Use the existing Rust and pnpm toolchains. Do not add a frontend test framework.
- For manual OAuth checks, use the existing mocked Rust backend; do not place real tokens in fixtures or logs.

## Rust catalog and dispatch checks

```sh
mkdir -p "$HOME/.cargo/tmp"
TMPDIR="$HOME/.cargo/tmp" CARGO_BUILD_JOBS=1 cargo test --lib codex -- --test-threads=1
```

Verify that every serialized implementation family is one of the four supported values; Email and the iOS/Android client entries use their documented roles; only registered handlers advertise capabilities; unauthorized start/inspect/cancel calls fail; mocked flow transitions stay sanitized; and token persistence continues through the encrypted auth store. If the configured temporary or target directory is quota-limited, choose an existing writable location for `TMPDIR`/`CARGO_TARGET_DIR` rather than deleting build artifacts.

## UI checks

```sh
cd web
pnpm lint
pnpm typecheck
pnpm build
pnpm test:smoke
```

With an authenticated browser session, inspect `/`, `/cowork`, `/tools`, `/scheduled`, `/integrations`, `/memory`, `/configuration`, `/cost-tracking`, `/logs`, `/doctor`, `/authentication`, and `/settings`. At 320, 375, 768, 1024, and desktop CSS-pixel widths, check page overflow, readable content, and operable controls. Also test 200% zoom, WCAG text-spacing overrides, keyboard-only route and dialog flows, focus visibility/order, and the browser accessibility tree. Repeat applicable checks for light/dark theme and loading, empty, error, and connected states.

For every observed issue, record the criterion, exact route/state, viewport or zoom, verification steps, priority, and follow-up action in `web/docs/accessibility/wcag-2.2-aa-roadmap.md`. Mark conditions that could not be verified as `unverified`; do not infer site-wide conformance from automated checks or a subset of routes.

## Validation record (2026-11-12)

- UI PRs #1 and #2 passed GitHub CI, including lint, typecheck, unit tests,
  production build, and Playwright browser smoke. Local `pnpm lint`,
  `pnpm typecheck`, `pnpm build`, and `pnpm test:unit` also passed (12 tests).
- The local `pnpm test:smoke` command could not launch because the Playwright
  Chromium binary is not installed. Manual Chrome verification covered the
  route matrix, text-spacing override, mobile drawer focus/escape behavior, and
  the mocked Codex pending/cancel dialog states. The success/error/expiry
  dialog states were also inspected with mocked responses in the browser
  session.
- `TMPDIR="$HOME/.cargo/tmp" CARGO_BUILD_JOBS=1 cargo test --lib codex --
  --test-threads=1` passed: 50 tests. Coverage includes OAuth success, cancel,
  expiry, redaction, encrypted persistence, failed saves, bearer enforcement,
  model catalog fallback, custom IDs, and config preservation.
- A full Rust library run with the default `/tmp` location reported 2,916
  passes and 420 failures; the focused Codex failures reproduced `EDQUOT` while
  creating temporary test files and encryption keys. The corrected focused run
  passed with `TMPDIR` under the home filesystem. The later full `cargo test`
  attempt was interrupted by the workstation crash and has no result. The
  updated `~/.cargo/config.toml` points Cargo child processes at
  `$HOME/.cargo/tmp`; the destination is writable on the disk-backed filesystem,
  and `TMPDIR="$HOME/.cargo/tmp" cargo metadata --no-deps` parsed the project
  configuration without compiling.
- `rustfmt --check` passed for the four changed Rust files. The repository-wide
  `cargo fmt --all -- --check` reports extensive formatting drift in untouched
  files; those unrelated files were not reformatted.
- Parent CI run `43178454819` passed on implementation commit `407c0d8`: Rust
  tests on Ubuntu and macOS, the Rust 1.96 feature check, release builds for
  x86_64 Linux, aarch64 Linux, and aarch64 macOS, and Tauri bundles for all
  three supported desktop targets. A local Clippy run found warnings in
  untouched modules; no Clippy changes were made.
- Browser zoom at 200%/400%, screen-reader use, contrast measurements, and a
  complete route-by-route visual overlap review remain unverified. See the UI
  accessibility roadmap for route evidence and prioritized follow-up.
- The validation run above completed before this final evidence-only update; the
  PR workflow reruns on the updated head before merge.

## Documentation checks

The docs quality gate passed with only existing markdown issues outside changed lines. The offline link-integrity tool `lychee` is unavailable locally and was not installed. Confirm the six moved files remain under `_deprecated/`, root `GLOSSARY.md` contains the copied glossary, and maintained docs no longer link to their old locations; inspect the final staged diff for unrelated changes or secrets.
