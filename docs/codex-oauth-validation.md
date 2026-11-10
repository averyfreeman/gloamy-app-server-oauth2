# Codex OAuth integration validation

Validation date: 2026-11-10 (UTC). Scope: the Integrations-page ChatGPT OAuth setup for the existing `openai-codex` provider.

## Interface and preservation checks

- The existing integrations response wrapper and legacy fields remain. Codex adds stable ID `openai-codex`, `setup_capability: oauth_device_code`, and an auth-profile-derived `connected` flag. OpenAI API-key setup remains distinct (`openai-api`).
- Five paired-bearer-only operations start, inspect, cancel, list model choices, and explicitly apply a model. See the [gateway contract](gateway-api-reference.md#codex-oauth-integration-setup).
- Pending authorization data and tokens stay in the daemon. Public state contains only verification information and safe progress/errors. Owner isolation, expiry, bounded retention, concurrent-start rejection, cancellation, and credential-save failure have mocked coverage.
- Successful credentials use the existing AuthService and encrypted profile store. Subsequent writes with general encryption disabled still preserve Codex OAuth encryption. Debug formatting and HTTP errors redact token/device payloads.
- OAuth connection preserves provider/model selection. Applying a model requires an explicit default-provider choice when switching providers; already-default Codex changes only `default_model`. Tests compare unrelated configuration, routes, fallbacks, personalization, secrets, and auth profiles. Failed config saves leave the runtime selection unchanged.
- Account discovery uses the connected profile and fixed Codex backend. Mocked HTTP coverage checks version/query and authentication headers, visibility filtering, and the current three-step device authorization exchange. Shared catalog fallback, empty fallback, and custom IDs have deterministic coverage.

## Automated validation

- `cargo check --locked --all-targets`: passed.
- `cargo test --locked --no-fail-fast`: passed across library, binary, integration, and documentation targets; no failures. The library and binary suites ran 3,332 and 3,357 passing tests respectively.
- After the final authentication-error representation change, `cargo test --locked --lib codex -- --test-threads=1`: 49 passed. An intervening parallel focused run exposed two intermittent existing App Server subprocess assertions (`invalid_app_server_output_fails_without_echoing_payload` and `app_server_process_exit_is_reported_with_status`); both passed in the full run and serial rerun. Those subprocess tests were not changed.
- Changed Rust files: `rustfmt --edition 2024 --config skip_children=true --check` passed. `git diff --check` passed.
- `cargo fmt --all -- --check`: blocked by pre-existing formatting drift in unrelated files; no changed Rust file needs formatting.
- `cargo clippy --locked --all-targets -- -D warnings`: 11 repository baseline failures remain in unrelated expressions (sorting, iterator conversion, format references, return bindings, chunk iteration, loops, and match structure). The new large-error lint finding was corrected and is absent from the final run.
- Frontend `pnpm run typecheck`, `pnpm run build`, and `pnpm exec eslint src/components/integrations`: passed. The build retains existing font-import ordering and dependency metadata/deprecation warnings.
- Full frontend `pnpm run lint`: existing repository-wide failures remain (the initial run reported 477 errors and 227 warnings). New dialog/registry/type files have no lint findings; the new import-order finding on the existing Integrations page was corrected and a page-specific rerun confirmed its removal.
- Affected documentation markdown lint: passed (seven files, zero errors).

## Manual browser verification

Used the real Vite-rendered application in an isolated headless Chrome profile with a local mocked gateway and synthetic bearer/code/model values. No real ChatGPT login or account credentials were used.

- Codex is a keyboard-operable button opening a named modal. The OpenAI API-key, unimplemented service, and skill cards remain visibly noninteractive.
- Explicit Connect displays the verification link, user code, and expiry. Browser polling contacts gateway status routes only.
- Pending, success, error, expiry, and cancellation states render. A failed cancellation remains pending with an unconfirmed-cancellation message; a successful retry shows cancellation.
- Connection success refreshes card status and loads backend model labels without applying a global default. A custom ID cannot switch providers until the explicit default checkbox is selected; applying sends the exact custom ID and choice.
- Tab and Shift+Tab remain within the dialog, including after asynchronous state changes. Escape closes it and restores focus to the card.
- Closing while a delayed start request is in flight cancels the eventual pending flow. The mock recorded one cancellation and no orphaned status polling.

## Limitations, documentation, and rollback

The [official-source investigation](codex-oauth-sources.md) documents the first-party Codex model endpoint and device protocol. This backend has no published third-party compatibility guarantee; discovery is best-effort with a pinned compatibility version. The single verified shared fallback is `gpt-6.1-sol`; it has no inferred ChatGPT token pricing and does not prove account entitlement. Explicit custom IDs remain available. No public OpenAI `/v1/models` OAuth request or unrelated local CLI account is used.

Live account sign-in, account revocation, and production backend compatibility were not exercised. Connected means usable stored credentials or refresh capability, rather than a live entitlement check. Pending flows are in memory and disappear on daemon restart. Cancellation cannot undo persistence already in progress. Switching defaults with a global API URL override requires resolving that override explicitly in Configuration.

The gateway, provider, configuration, and troubleshooting references are updated. This checkout contains only `docs/i18n/README.md`, with no localized runtime-reference equivalents to update; translation coverage remains a repository documentation gap. Navigation was not changed.

Rollback by reverting the integration commit. Existing credentials remain in the existing auth store and can still be managed through existing auth commands; unrelated profiles and configuration retain their existing formats.
