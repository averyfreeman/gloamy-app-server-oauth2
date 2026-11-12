# Codex OAuth integration implementation

Implement the supplied Integrations-page design on the existing shared model catalog. Connect the `openai-codex` provider to the user's ChatGPT account without changing global selection until explicitly requested.

- [x] Verify official account-aware Codex model discovery and dated catalog fallback metadata.
- [x] Add a bounded, expiring, owner-bound device-code service using existing OAuth and encrypted AuthService persistence; test transitions, cancellation, redaction and save failures.
- [x] Add stable integration metadata and strictly bearer-authenticated start/inspect/cancel, model-list and explicit model-application routes. Serialize config writes and preserve unrelated settings.
- [x] Add an accessible setup dialog and integration-ID registry. Only implemented cards are interactive; backend owns OAuth polling and the browser polls safe status.
- [x] Update runtime documentation, including catalog limitations and rollback. No localized runtime-reference equivalents exist in this checkout; this gap is recorded in the validation report.
- [x] Run mocked Rust tests, existing frontend typecheck/build/lint, manual browser verification and a requirement-by-requirement review. Results and baseline failures are recorded in the [validation report](../../codex-oauth-validation.md); commit directly to main as requested.

OAuth secrets and pending device codes never enter browser responses. Credentials are encrypted even when the general config secret setting is disabled. Connection status comes from auth profiles, not the selected global provider. Account model discovery must use the same connected account; an unrelated local Codex login and OpenAI's public API model endpoint are not acceptable substitutes. Catalog fallback and custom IDs remain available when account discovery is unavailable.

Rollback: revert the integration change; existing auth profiles and config values remain in their existing stores. Cancel/expiry stop pending polling; successful credential persistence has a clear commit boundary before reporting success.
