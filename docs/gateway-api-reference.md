# Gateway API Reference

HTTP API endpoints exposed by the Gloamy gateway server.

Last verified: **November 9, 2026**.

## Base URL

Default: `http://127.0.0.1:42617`

Configurable via `[gateway]` config section or `--host`/`--port` CLI flags.

## Authentication

All endpoints require authentication via one of:

1. **Pairing token** (when `gateway.require_pairing = true`):
   - Header: `Authorization: Bearer <pairing_token>`
   - Obtained via initial pairing flow

2. **API key** (for trusted clients):
   - Header: `X-API-Key: <api_key>`

`POST /api/models/catalog/reload` is an exception: it requires a valid stored paired bearer token, even when `gateway.require_pairing = false`. `X-API-Key` does not authorize this route.

## API Endpoints

### Status & Health

#### `GET /api/status`

Runtime status summary including provider, model, version info.

**Response:**

```json
{
  "version": "0.1.0",
  "provider": "openai",
  "model": "synthetic-model-037",
  "uptime_secs": 3600
}
```

---

#### `GET /api/health`

Component health snapshot for monitoring.

**Response:**

```json
{
  "health": {
    "gateway": "healthy",
    "memory": "healthy",
    "provider": "healthy",
    "channels": "healthy"
  }
}
```

---

### Model Catalog

#### `POST /api/models/catalog/reload`

Reload `model-catalog.toml` from the active config directory in the running gateway. The catalog is local and this route does not fetch model metadata from providers.

**Authentication:** Send a valid stored paired bearer token, whether or not pairing is enabled:

```http
Authorization: Bearer <PAIRED_BEARER_TOKEN>
```

**Success (`200`):**

```json
{"reloaded": true}
```

**Errors:** `401` means the bearer token is missing or invalid. `400` means the local catalog could not be loaded or validated; the response is intentionally generic and the last valid in-memory catalog remains active. Daemon logs record the rejection without including file contents; run `gloamy models list` locally to see the loading error.

The request reads the complete `model-catalog.toml` in the active config directory. If that file is absent, reload selects the bundled catalog. This endpoint does not fetch models or pricing from the network. Validation finishes before publication: operations holding the prior immutable snapshot can finish, and later operations use the newly published snapshot. A `400` leaves the last valid snapshot active.

Example:

```bash
curl -X POST http://127.0.0.1:42617/api/models/catalog/reload \
  -H "Authorization: Bearer $GLOAMY_PAIRED_TOKEN"
```

---

### Configuration

#### `GET /api/config`

Retrieve current configuration (secrets masked).

**Response:** `config.toml` as JSON with sensitive values replaced by `***`.

---

#### `POST /api/config`

Update configuration fields.

**Request:** Partial config object with changes.

**Notes:**
- Only specified fields are modified
- Secrets are preserved when masked values submitted
- Runtime validates changes before applying

---

### Tools

#### `GET /api/tools`

List all registered tools with their schemas.

**Response:**

```json
{
  "tools": [
    {
      "name": "shell",
      "description": "Execute shell commands",
      "parameters": { ... }
    }
  ]
}
```

---

#### `GET /api/cli-tools`

Discover CLI tools available on the system.

**Response:**

```json
{
  "cli_tools": [
    {
      "name": "git",
      "path": "/usr/bin/git",
      "version": "2.42.0",
      "category": "version_control"
    },
    {
      "name": "docker",
      "path": "/usr/local/bin/docker",
      "version": "24.0.7",
      "category": "container"
    }
  ]
}
```

**Discovered tools include:** `git`, `python`, `python3`, `node`, `npm`, `pip`, `pip3`, `docker`, `cargo`, `make`, `kubectl`, `rustc`

---

### Integrations

#### `GET /api/integrations`

List the integration catalog and current status. Each record has a stable `id`,
an `implementation_type`, `name`, `description`, `category`, and `status`.
`implementation_type` is one of `ai_provider`, `communication_channel`,
`remote_service`, or `local_runtime`; it is separate from the display category.
`setup_capability` is omitted unless a matching setup handler is registered.

**Response:**

```json
{
  "integrations": [
    {
      "id": "openai-codex",
      "implementation_type": "ai_provider",
      "name": "OpenAI Codex (ChatGPT OAuth)",
      "description": "Connect your ChatGPT account for Codex; separate from OpenAI API keys",
      "category": "AiModel",
      "status": "Available",
      "setup_capability": "oauth_device_code",
      "connected": false
    },
    {
      "id": "openai-api",
      "implementation_type": "ai_provider",
      "name": "OpenAI",
      "description": "OpenAI API language models using an API key (separate API billing)",
      "category": "AiModel",
      "status": "Available"
    }
  ]
}
```

---

### Skills

#### `GET /api/skills`

List installed skills and their metadata.

**Response:**

```json
{
  "skills": [
    {
      "name": "docx",
      "version": "1.0.0",
      "tools": ["docx_create", "docx_edit"]
    }
  ]
}
```

---

### Cron / Scheduling

#### `GET /api/cron`

List scheduled tasks.

**Response:** Array of scheduled jobs with expression, command, and status.

---

### Memory

#### `GET /api/memory`

Query memory store.

**Query Parameters:**
- `q`: Search query
- `category`: Filter by category (`core`, `daily`, `episode`, etc.)
- `limit`: Maximum results (default: 10)

---

### Cost Tracking

#### `GET /api/cost`

Current cost tracking summary.

**Response:**

```json
{
  "enabled": true,
  "daily_cost_usd": 2.45,
  "monthly_cost_usd": 42.10,
  "total_tokens": 1500000,
  "request_count": 250,
  "by_model": {
    "synthetic-model-037": { "requests": 200, "cost_usd": 1.50 },
    "synthetic-model-013": { "requests": 50, "cost_usd": 0.95 }
  }
}
```

---

#### `GET /api/cost/timeline`

Daily token usage trend (up to 3660 days).

**Response:**

```json
{
  "timeline": [
    { "date": "2026-04-01", "tokens": 50000, "cost_usd": 1.20 },
    { "date": "2026-04-02", "tokens": 45000, "cost_usd": 1.08 }
  ]
}
```

---

### Logs

#### `GET /api/logs`

Retrieve daemon logs (requires pairing token).

**Query Parameters:**
- `tail`: Number of lines (default: 100, max: 1000)
- `level`: Filter by level (`error`, `warn`, `info`, `debug`)

**Response:**

```json
{
  "logs": [
    { "timestamp": "2026-04-08T10:30:00Z", "level": "INFO", "message": "Gateway started" },
    { "timestamp": "2026-04-08T10:30:05Z", "level": "WARN", "message": "High memory usage" }
  ]
}
```

**Log sources:** `daemon.stdout.log`, `daemon.stderr.log` from config directory.

---

### Diagnostics

#### `GET /api/doctor`

Run diagnostics and return health report.

**Response:**

```json
{
  "status": "healthy",
  "checks": [
    { "name": "config", "status": "pass" },
    { "name": "provider", "status": "pass" },
    { "name": "memory", "status": "pass" }
  ]
}
```

---

## WebSocket Endpoints

Real-time event streaming is available via WebSocket at:

```
ws://127.0.0.1:42617/ws
```

**Authentication:** Same as HTTP (token in `Authorization` header during handshake).

**Events:**
- `message` - Incoming channel messages
- `tool_call` - Tool execution events
- `cost_update` - Real-time cost tracking updates

---

## Webhook Endpoints

Channel webhooks receive events at:

| Channel | Endpoint |
|---------|----------|
| Telegram | `POST /telegram` |
| Discord | `POST /discord` |
| Slack | `POST /slack` |
| WhatsApp | `POST /whatsapp` |
| Linq | `POST /linq` |
| Nextcloud Talk | `POST /nextcloud-talk` |
| WATI | `POST /wati` |

---

## Error Responses

All endpoints return consistent error format:

```json
{
  "error": "Description of what went wrong",
  "code": "ERROR_CODE"
}
```

**HTTP Status Codes:**

| Code | Meaning |
|------|---------|
| `200` | Success |
| `400` | Bad request (invalid parameters) |
| `401` | Unauthorized (missing/invalid auth) |
| `403` | Forbidden (insufficient permissions) |
| `404` | Endpoint not found |
| `500` | Internal server error |

---

## CORS Support

The gateway supports CORS for browser clients:

- `Access-Control-Allow-Origin: *` (configurable)
- Preflight (`OPTIONS`) requests handled automatically
- Credentials supported when origin explicitly configured

---

## Rate Limiting

Default rate limits per IP:

- API endpoints: 100 requests/minute
- Webhook endpoints: 60 requests/minute per channel

Configurable via `[gateway.rate_limiting]` config section.

---

## Codex OAuth Integration Setup

`GET /api/integrations` keeps its existing wrapper and legacy entry fields. All
78 records now add stable `id` and `implementation_type` fields. The four
implementation families are AI providers, communication channels, remote
services, and local runtimes. `setup_capability` is optional and only appears
when a concrete handler is registered through the integration setup
dispatcher. The Codex entry adds `setup_capability: "oauth_device_code"` and
`connected`. Its display name distinguishes ChatGPT OAuth from the separate
OpenAI API-key integration (`id: "openai-api"`). Connected state reflects an
OAuth profile with usable or refreshable credentials, independently of the
global default provider; it is not an entitlement or live-revocation check.

All setup operations below require a valid stored gateway bearer token, even when general pairing enforcement is disabled. Flow IDs are opaque and bound to the bearer identity that started them; another paired caller cannot inspect or cancel them.

| Method | Path | Behavior |
| --- | --- | --- |
| POST | `/api/integrations/openai-codex/oauth` | Start device authorization; no credential input required |
| GET | `/api/integrations/openai-codex/oauth/{id}` | Inspect safe server-owned progress |
| DELETE | `/api/integrations/openai-codex/oauth/{id}` | Cancel pending authorization |
| GET | `/api/integrations/openai-codex/models` | Account-aware model discovery, eligible curated fallback, or custom-only selection |
| POST | `/api/integrations/openai-codex/model` | Explicitly apply a model, optionally choosing Codex as default |

A flow response contains `id`, `status` (`pending`, `success`, `error`, `expired`, or `cancelled`), optional `verification_uri`, `user_code`, `expires_at`, and a safe optional `error`. It never contains the private device code, access token, refresh token, ID token, or raw provider error payload. The daemon polls OpenAI; browser polling only inspects this safe state. Pending flows expire at the provider deadline or 15 minutes, whichever is sooner. At most eight authorizations can be pending, with one per bearer owner; concurrent start requests are rejected immediately. The service retains at most 32 flows in total, evicting older terminal results when needed. Terminal results otherwise expire after five minutes. Cancel cannot undo a credential write that has already crossed its persistence commit boundary.

The model response contains `models: [{id, label}]`, `source` (`account`, `catalog`, or `custom`), `warning`, `selected_model`, `is_default`, and `connected`. Account discovery uses the same Gloamy OAuth profile against the Codex backend, never public `/v1/models` or an unrelated local Codex CLI login. See [verified mechanism and limitations](codex-oauth-sources.md). Custom IDs remain allowed regardless of either catalog.

Apply a model with `{"model":"CUSTOM_MODEL_ID","use_as_default":false}`. If Codex is already default, this changes only `default_model`. If another provider is default, `use_as_default: true` is required and only the provider/model selection changes. Connection alone never edits config. A conflicting global API URL override must be removed explicitly in Configuration before switching providers; it is not silently erased or reused for Codex.

Successful application returns `{"applied":true,"is_default":true,"selected_model":"CUSTOM_MODEL_ID"}`. Missing/invalid bearer tokens return `401`; unknown or other-owner flow IDs return `404`. Start failures can return a safe error flow or `502` when no flow can be reserved. Model-application conflicts return `409`, invalid request/config returns `400`, and persistence failure returns `500` without changing the in-memory selection. Credential-save failure never reports OAuth success.

## Related Documentation

- [commands-reference.md](commands-reference.md) - CLI commands
- [config-reference.md](config-reference.md) - Gateway configuration
- [channels-reference.md](channels-reference.md) - Channel webhook setup
- [network-deployment.md](network-deployment.md) - Production deployment
