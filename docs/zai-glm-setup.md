# Z.AI GLM Setup

Gloamy supports Z.AI's GLM models through OpenAI-compatible endpoints.
This guide covers practical setup options that match current Gloamy provider behavior.

## Overview

Gloamy supports these Z.AI aliases and endpoints out of the box:

| Alias | Endpoint | Notes |
|-------|----------|-------|
| `zai` | `https://api.z.ai/api/coding/paas/v4` | Global endpoint |
| `zai-cn` | `https://open.bigmodel.cn/api/paas/v4` | China endpoint |

If you need a custom base URL, see `docs/custom-providers.md`.

## Setup

### Quick Start

```bash
gloamy onboard \
  --provider "zai" \
  --api-key "YOUR_ZAI_API_KEY"
```

### Manual Configuration

Edit `~/.gloamy/config.toml`:

```toml
api_key = "YOUR_ZAI_API_KEY"
default_provider = "zai"
default_model = "MODEL_ID_FROM_ZAI_DISCOVERY"
default_temperature = 0.7
```

The model value above is a placeholder. Model IDs and availability vary by account, endpoint, and region. Gloamy does not currently include a verified curated recommendation for this provider. With an API key configured, discover IDs and choose one exactly as returned:

```bash
gloamy models refresh --provider zai
gloamy models list --provider zai
gloamy models set MODEL_ID_FROM_ZAI_DISCOVERY
```

Alternatively, pass the chosen ID to `gloamy onboard --provider zai --model <MODEL_ID>`. Gloamy preserves that explicit choice even when it is outside the curated catalog.

## Verify Setup

### Test with curl

```bash
# Test OpenAI-compatible endpoint
curl -X POST "https://api.z.ai/api/coding/paas/v4/chat/completions" \
  -H "Authorization: Bearer YOUR_ZAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "MODEL_ID_FROM_ZAI_DISCOVERY",
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

Expected response:
```json
{
  "choices": [{
    "message": {
      "content": "Hello! How can I help you today?",
      "role": "assistant"
    }
  }]
}
```

### Test with Gloamy CLI

```bash
# Test agent directly
echo "Hello" | gloamy agent

# Check status
gloamy status
```

## Environment Variables

Add to your `.env` file:

```bash
# Z.AI API Key
ZAI_API_KEY=your-id.secret

# Optional generic key (used by many providers)
# API_KEY=your-id.secret
```

The key format is `id.secret` (for example: `abc123.xyz789`).

## Troubleshooting

### Rate Limiting

**Symptom:** `rate_limited` errors

**Solution:**
- Wait and retry.
- Check your Z.AI plan limits.
- If your account offers multiple model IDs, discover them from the configured endpoint and choose one that meets the latency and quota needs.

### Authentication Errors

**Symptom:** 401 or 403 errors

**Solution:**
- Verify your API key format is `id.secret`
- Check the key hasn't expired
- Ensure no extra whitespace in the key

### Model Not Found

**Symptom:** Model not available error

**Solution:**
- List model IDs exposed by the configured endpoint:
```bash
curl -s "https://api.z.ai/api/coding/paas/v4/models" \
  -H "Authorization: Bearer YOUR_ZAI_API_KEY" | jq '.data[].id'
```

For the eight-month curated eligibility rule, exact catalog file schema, and missing-rate behavior, see [Model Catalog](config-reference.md#curated-model-catalog). Catalog provider research is not exhaustive; a provider without an entry has no verified automatic default yet.

## Getting an API Key

1. Go to [Z.AI](https://z.ai)
2. Sign up for a Coding Plan
3. Generate an API key from the dashboard
4. Key format: `id.secret` (e.g., `abc123.xyz789`)

## Related Documentation

- [Gloamy README](../README.md)
- [Custom Provider Endpoints](./custom-providers.md)
- [PR Workflow](./pr-workflow.md)
