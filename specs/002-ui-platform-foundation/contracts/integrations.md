# Integration API and Setup Contract

## Catalog

`GET /api/integrations` retains the existing `{"integrations": [...]}` wrapper and legacy entry fields. Additive fields are `id`, `implementation_type`, optional `setup_capability`, and Codex's optional `connected` state. Existing clients may ignore new fields. Setup UI is offered only when an executable server handler is registered for that entry and capability.

Allowed `implementation_type` values are `ai_provider`, `communication_channel`, `remote_service`, and `local_runtime`. Display `category` remains a separate field.

## Codex OAuth operations

All operations require a valid paired bearer token. These existing routes and methods remain stable:

| Method and path | Operation | Success response |
|---|---|---|
| `POST /api/integrations/openai-codex/oauth` | Start authorization | Safe `FlowView` with verification URI/code and expiration |
| `GET /api/integrations/openai-codex/oauth/{id}` | Inspect owned flow | Current safe `FlowView` |
| `DELETE /api/integrations/openai-codex/oauth/{id}` | Cancel owned flow | Current safe `FlowView` |

An invalid or missing bearer token returns `401`. Unknown, expired/cleaned-up, or differently-owned flows return the existing not-found response without revealing ownership. A provider failure is represented by a sanitized error status/message; no device secret or token is returned. A successful flow persists credentials only through the existing encrypted auth-profile store.

The endpoint registration remains Codex-specific and separate from the OpenAI API-key provider. No public models endpoint for ChatGPT OAuth is introduced by this contract.
