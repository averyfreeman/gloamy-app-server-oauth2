# Feature Data Model

## Integration catalog entry

| Field | Meaning | Constraint |
|---|---|---|
| `id` | Stable machine identity for the integration | Unique, lowercase, and independent of its display name |
| `implementation_type` | One of the four backend implementation families | `ai_provider`, `communication_channel`, `remote_service`, or `local_runtime`; must match the documented role |
| `setup_capability` | Setup operation exposed to UI clients | Optional; only present when a matching executable handler is registered |
| `name`, `description`, `category`, `status` | Existing display and readiness fields | Preserve current names and meanings |
| `connected` | Optional account connection state | Present for OpenAI Codex; independent of the global default provider |

## Setup handler registration

A registration binds an integration ID and setup capability to an implementation family and an executable handler. The current registration is `openai-codex` + `oauth_device_code` + `ai_provider`. Catalog metadata is derived only from this registration. An unknown ID, mismatched capability, or missing handler resolves as unavailable.

The Codex handler dispatches `start`, `inspect`, and `cancel` to the existing `CodexOAuthSetup` lifecycle service. It does not own token persistence; successful credentials continue through `AuthService` and the encrypted auth-profile store.

## Public authorization progress

`FlowView` exposes only the opaque flow ID, safe status (`pending`, `success`, `error`, `expired`, or `cancelled`), verification URI, user code, expiration time, and sanitized error text. Device codes, access tokens, and refresh tokens remain server-side.

## Accessibility audit finding

An audit record identifies one WCAG success criterion, route, UI state, viewport or zoom setting, verification method, disposition (`observed-no-gap`, `gap`, `unverified`, or `conditional`), reproducible evidence, priority, and next action. “Observed-no-gap” describes only the recorded test conditions and is not a conformance claim.
