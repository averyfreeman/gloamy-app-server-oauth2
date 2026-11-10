# Codex OAuth model catalog verification

Verified against official Codex documentation and the `openai/codex` source
repository on 2026-11-10.

## Finding

The reused device-code helper is aligned with the current first-party
[device authorization implementation](https://github.com/openai/codex/blob/main/codex-rs/login/src/device_code_auth.rs):
JSON POST to `/api/accounts/deviceauth/usercode`, JSON polling of
`/api/accounts/deviceauth/token` (403/404 mean pending), then an OAuth
authorization-code exchange using the returned verifier and
`https://auth.openai.com/deviceauth/callback`. The verification page is
`https://auth.openai.com/codex/device`; the authorization ID and verifier stay
server-side. This replaces the helper's previous assumed RFC device-token
grant while retaining its public helper interface and existing AuthService
storage/refresh path. The mocked HTTP test covers the complete exchange.

The official Codex client fetches its account-aware catalog from the Codex
backend with an HTTP request, not from OpenAI Platform's public `/v1/models`
endpoint. In current source, the default ChatGPT Codex base URL is
`https://chatgpt.com/backend-api/codex`; `OpenAiModelsEndpoint` uses this route
when it has ChatGPT auth and no provider URL override. `ModelsClient` sends
`GET /models?client_version=<major.minor.patch>` and decodes a `ModelsResponse`
whose entries use `slug`, `display_name`, `visibility`, and other Codex model
metadata. The implementation comments explicitly distinguish this from
public `/v1/models`. See [Codex model-provider endpoint](https://github.com/openai/codex/blob/main/codex-rs/model-provider/src/models_endpoint.rs), [Codex API models client](https://github.com/openai/codex/blob/main/codex-rs/codex-api/src/endpoint/models.rs), and [shared `ModelInfo` response type](https://github.com/openai/codex/blob/main/codex-rs/protocol/src/openai_models.rs).

This is the best account-aware source for Gloamy's existing `openai-codex`
OAuth account. Use the selected profile's access token and ChatGPT account ID
on the server for the fixed Codex origin. The first-party auth provider sends
`Authorization: Bearer <access-token>` and `ChatGPT-Account-Id: <account-id>`.
It also sets an `originator` and a Codex `User-Agent`; first-party default
values are `codex_cli_rs` and a platform-specific string. See [Codex bearer
auth headers](https://github.com/openai/codex/blob/main/codex-rs/model-provider/src/bearer_auth_provider.rs) and [Codex default HTTP headers](https://github.com/openai/codex/blob/main/codex-rs/login/src/auth/default_client.rs).

Keep these credentials server-side. Do not use the App Server's
`chatgptAuthTokens` mode as a token bridge: although the public App Server docs
describe it, the current protocol source marks it "UNSTABLE - FOR OPENAI
INTERNAL USE ONLY - DO NOT USE" and says supplied tokens stay in memory. This
restriction applies to that App Server auth mode; the Codex source separately
shows the direct authenticated model endpoint used by the first-party client.
See [Codex App Server auth modes](https://learn.chatgpt.com/docs/app-server#auth-endpoints) and [the protocol's `AuthMode` definition](https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/common.rs#L2205-L2223).

## Request details and limits

- **URL:** fixed origin `https://chatgpt.com/backend-api/codex`; `GET
  /models?client_version=...`. Do not accept a catalog origin from a browser
  request or provider config for this feature.
- **Auth:** use the server-side `openai-codex` profile's current access token
  and account ID. The official first-party auth provider emits a bearer token
  and `ChatGPT-Account-Id` header. Respect the existing AuthService refresh
  path before making the request, and never log headers or raw response bodies.
- **Version:** the Codex client compiles its own models-manager crate version
  into `client_version` with `CARGO_PKG_VERSION_MAJOR/MINOR/PATCH`; its source
  does not shell out to `codex --version`. The current latest stable official
  CLI release at verification time is `0.167.0` (released November 6, 2026;
  see the [official Codex releases](https://github.com/openai/codex/releases)).
  There is no official third-party versioning contract. For Gloamy, use a
  deliberately pinned Codex compatibility version, update it when reviewing
  this source/response format, and fall back if the backend rejects it or
  returns no usable entries. Do not read the installed CLI version: it may be
  absent or unrelated to Gloamy's provider.
- **Originator:** first-party source defaults to `codex_cli_rs` and a UA built
  from the Codex package version, OS, and architecture. Codex source/issues
  show catalog metadata can vary by `originator`. Gloamy should identify itself
  truthfully (for example `gloamy`) rather than claim to be the Codex CLI; an
  unrecognized originator may receive a different catalog. This behavior is
  not specified as a public API contract.
- **Response:** parse the Codex `models` array. `slug` is the exact model ID;
  `display_name` is the provider-supplied label; `visibility` may be `list`,
  `hide`, or `none`. Filter to picker-visible models and reject malformed or
  duplicate IDs. Account access and minimum-client-version metadata can vary;
  treat the response as a discovery hint, not an inference guarantee.
- **Failure:** this backend route is used by official Codex source, but no
  public API documentation promises stability for third-party clients. Treat
  authentication, status, decoding, empty-catalog, and timeout failures as
  discovery failures and fall back to curated `openai-codex` models plus an
  explicit custom model ID. Never fall back to the public `/v1/models` API.

The official OpenAI API model directory and `GET /models` reference describe
API Platform model discovery; they do not establish ChatGPT OAuth support or
account-aware Codex model availability there. See [OpenAI API models](https://developers.openai.com/api/docs/models) and [Models — List API reference](https://developers.openai.com/api/reference/resources/models/methods/list). Codex App Server's documented `model/list` is another account-aware picker surface for a Codex App Server account, but Gloamy's current `CodexAppServerClient` reads the separately signed-in local CLI account; it does not authenticate that request with the Gloamy `openai-codex` profile. See [Codex App Server model/list](https://learn.chatgpt.com/docs/app-server#list-models-modellist) and [Gloamy's current model-list client](../src/codex_app_server/models.rs).

## Shared-catalog fallback

Stage 1's shared catalog is present in `src/model_catalog.rs` and
`src/model_catalog.toml`. The existing `openai` entries describe OpenAI API
models. Do not copy API prices to `openai-codex`; the OAuth integration uses
ChatGPT subscription access, and no Codex subscription token price was
verified.

The narrow fallback row verified for Codex is:

| Provider key | Exact ID | Label | Release date | Price |
| --- | --- | --- | --- | --- |
| `openai-codex` | `gpt-6.1-sol` | GPT-6.1 Sol | 2026-09-29 | Unknown / omitted |

The Codex App Server `model/list` example returns this exact ID as an available
model; the official Codex guide names `gpt-6.1-sol` as a model Codex works best
with; and the OpenAI API changelog verifies its model ID and release date of
September 29, 2026. These sources verify the model and Codex eligibility as
separate facts. See [App Server model/list](https://learn.chatgpt.com/docs/app-server#list-models-modellist), [Codex model guidance](https://developers.openai.com/api/docs/guides/code-generation#use-codex), and [the September 29 release entry](https://developers.openai.com/api/docs/changelog?mw_entry=2026-09-29-962e96e45069e83d).

Do not add other OpenAI API models to `openai-codex` based only on matching
names or API release dates. A live Codex response can add account-visible
models; curated metadata supplies stable labels for known fallbacks. Keep
explicit custom IDs available even when absent from both sources. The curated
fallback cannot prove entitlement for a particular account.
