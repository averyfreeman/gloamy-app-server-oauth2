# Integration Readiness Inventory

**Snapshot**: 2026-11-12

## Remaining-work analysis

- The active feature remains `specs/002-ui-platform-foundation`. UI PR #1 delivered the responsive extraction, and UI PR #2 merged the Tauri desktop shell at UI main `087c51b`. Parent PR #15 remains open on `feat/ui-platform-foundation`; its full Rust/Tauri CI run `43178454819` passed on implementation commit `407c0d8`.
- PR review confirmed that setup metadata must correspond to an executable handler. The Codex device-code handler is now explicitly registered, and authenticated start/inspect/cancel operations dispatch through it. OAuth lifecycle, safe progress views, and encrypted profile writes remain in the existing auth service.
- Registry evidence confirms Email is an IMAP/SMTP communication channel. iOS and Android catalog descriptions refer to connecting through Telegram/Discord, so their implementation family is `communication_channel` while their display category remains `platform`; they do not represent native runtime targets.
- The UI router declares 12 routes, including `/cost-tracking`; the updated accessibility roadmap records that route, five viewport widths, the text-spacing override, keyboard checks, observed gaps, and remaining unverified work.
- The user explicitly approved including the six existing `_deprecated/` moves and root `GLOSSARY.md` in the parent documentation changes. Their content is preserved and active links are repaired; these are no longer excluded worktree changes.

## Decisions and rationale

| Topic | Decision | Rationale | Alternatives considered |
|---|---|---|---|
| Capability source | Explicit executable handler registration is authoritative; catalog metadata is emitted only when the matching ID/capability resolves to that handler. | Prevents clients from offering a setup flow that has no backend operation. | Trusting the catalog field alone, which was the behavior flagged in review. |
| Setup operations | Keep the three current Codex HTTP routes and dispatch each operation through the registered Codex handler. | Retains the existing API contract and bearer checks while making the central handler executable. | Replacing routes or moving OAuth lifecycle/persistence into the integration catalog. |
| Mobile platform family | Classify iOS/Android as `communication_channel`; preserve `platform` display category. | Their documented connection is through existing chat channels, not native Gloamy runtime support. | `local_runtime`, which incorrectly implies a supported native host. |
| WCAG disposition | Record verified observations, concrete gaps, and unverified criteria separately; do not claim conformance. | The route audit is a planning and prioritization step, not a certification. | Marking a criterion complete solely because it appears in the roadmap or passes an automated scan. |
| Archive scope | Include the exact moved files and root glossary in this parent PR. | Explicit user selection overrides the earlier task note that excluded those changes. | Leaving them unstaged or restoring the old paths. |

## Scope and method

The backend registry currently returns **78 integration entries**. The Integrations page also loads a separate Skills catalog from /api/skills; Skills are not counted here. The bands below are ordered easiest to hardest. Entries within a band are approximate ties. The estimate weighs existing dedicated code, required configuration/authentication, platform/security constraints, and the amount of app-specific work still missing.

There are no Git submodules in the current checkout. **No listed integration currently requires a Git submodule.** External CLIs, OS permissions, or service credentials are separate runtime prerequisites, not reasons to vendor a source repository as a submodule.

Catalog status is not sufficient evidence of setup readiness. Active/Available status can depend on configuration, while Nextcloud Talk, Nostr, and LM Studio have implementation code despite fixed Coming Soon status. The current UI registers a setup dialog for OpenAI Codex; other cards are not setup buttons today.

| Effort band | Integration | Existing backend support and remaining work |
|---|---|---|
| 0 — Already implemented | OpenAI Codex (ChatGPT OAuth) | Dedicated openai-codex provider, device-code service, encrypted auth profiles, authenticated gateway operations, and a registered UI setup dialog already exist. No submodule. Reference implementation; no new card setup needed. |
| 1 — Lowest: named provider implementation exists | OpenRouter | Dedicated provider and provider-factory registration exist. Add the reusable credential/model setup UI; no new inference wrapper. |
| 1 — Lowest: named provider implementation exists | Anthropic | Dedicated provider and provider-factory registration exist. Add the reusable credential/model setup UI; preserve its provider-specific auth options. |
| 1 — Lowest: named provider implementation exists | OpenAI | Dedicated API-key provider and provider-factory registration exist. Its UI must remain distinct from ChatGPT OAuth. |
| 1 — Lowest: named provider implementation exists | Google | Gemini provider and factory exist, with API-key and CLI/managed OAuth paths. Setup UI must choose and explain supported auth rather than assume one credential type. |
| 1 — Lowest: named provider implementation exists | DeepSeek | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | xAI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Mistral | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Ollama | Dedicated local provider exists. Setup needs endpoint/model fields and local reachability handling, not a third-party wrapper. |
| 1 — Lowest: named provider implementation exists | Perplexity | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | LM Studio | Provider factory supports lmstudio, including a local endpoint. The catalog currently labels it Coming Soon; reconcile display status with actual provider support before enabling a setup card. |
| 1 — Lowest: named provider implementation exists | Venice | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Vercel AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Cloudflare AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Moonshot | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Synthetic | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | OpenCode Zen | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Z.AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | GLM | Dedicated provider implementation exists. Its key format is provider-specific, so validate and explain that credential. |
| 1 — Lowest: named provider implementation exists | MiniMax | Dedicated provider implementation and credential handling exist. Add setup UI for its regional/auth choices. |
| 1 — Lowest: named provider implementation exists | Qwen | Dedicated provider implementation exists, including regional and OAuth aliases. Setup must expose only verified supported choices. |
| 1 — Lowest: named provider implementation exists | Qianfan | Dedicated provider implementation exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Groq | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Together AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Fireworks AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Novita AI | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 1 — Lowest: named provider implementation exists | Cohere | Dedicated compatible-provider registration exists. Add provider credential/model selection UI. |
| 2 — Low: existing channel or UI capability | Telegram | Dedicated Channel implementation and config exist. Setup is primarily bot token, allowlist, and channel options. |
| 2 — Low: existing channel or UI capability | Discord | Dedicated Channel implementation and config exist. Setup is primarily bot token, server/channel, and allowlist. |
| 2 — Low: existing channel or UI capability | Slack | Dedicated Channel implementation and config exist. Setup is primarily workspace token, channel, and allowlist. |
| 2 — Low: existing channel or UI capability | Webhooks | Gateway webhook path and configuration exist. Setup should reuse gateway auth and document trigger URL behavior. |
| 2 — Low: existing channel or UI capability | DingTalk | Dedicated Channel implementation and config exist. Setup requires the existing client credentials and allowlist fields. |
| 2 — Low: existing channel or UI capability | QQ Official | Dedicated QQ Channel implementation and config exist. Setup requires app credentials and allowlist fields. |
| 2 — Low: existing channel or UI capability | Email | Dedicated IMAP/SMTP Channel and configuration exist. Setup needs server, account, password, and sender-policy fields. |
| 2 — Low: existing channel or UI capability | Nextcloud Talk | Channel module and configuration exist, but the catalog marks it Coming Soon. Reconcile readiness and expose its base URL/app-token fields. |
| 2 — Low: existing channel or UI capability | Browser | Browser tools and configuration already exist. A setup card can expose the existing enablement and domain/permission settings. |
| 2 — Low: existing channel or UI capability | Cron | Scheduler and cron tools already exist. A card can route to existing schedule configuration rather than add a new backend wrapper. |
| 2 — Low: existing channel or UI capability | Canvas | A Canvas UI route exists, but there is no dedicated integration install/config handler. A route link is easier than a backend integration setup flow. |
| 3 — Medium: existing code with special auth, host, or security needs | WhatsApp | Dedicated channel code supports Cloud API and an optional Web mode. Setup has multiple credential/webhook fields and Web mode is feature-gated. |
| 3 — Medium: existing code with special auth, host, or security needs | Signal | Dedicated Signal channel exists. Setup depends on an external signal-cli-compatible service and account/device configuration. |
| 3 — Medium: existing code with special auth, host, or security needs | iMessage | Dedicated channel exists but is macOS-specific. Setup must explain host permissions and contact allowlists. |
| 3 — Medium: existing code with special auth, host, or security needs | Matrix | Dedicated channel exists behind the channel-matrix Cargo feature. Setup must disclose build availability and homeserver/session fields. |
| 3 — Medium: existing code with special auth, host, or security needs | Nostr | Channel module, configuration, and runtime startup exist, but the catalog marks it Coming Soon. Key handling and relay setup need careful UI treatment. |
| 3 — Medium: existing code with special auth, host, or security needs | Amazon Bedrock | Dedicated provider exists. AWS credential-chain, region, and IAM setup are more involved than a normal API-key form. |
| 3 — Medium: existing code with special auth, host, or security needs | Shell | Shell tool exists. Setup touches security policy and approval boundaries, so enabling it must not broaden permissions silently. |
| 3 — Medium: existing code with special auth, host, or security needs | File System | Read/write tools exist. Setup touches workspace boundaries and permissions and requires explicit scope. |
| 3 — Medium: existing code with special auth, host, or security needs | Voice | TTS and transcription infrastructure exists, but the catalog's voice wake/talk capability is not a complete dedicated setup flow. Treat support as partial. |
| 3 — Medium: existing code with special auth, host, or security needs | Screen Capture | Screenshot and perception-capture tools exist. A capture card needs permission/context handling; it is not a new screenshot backend. |
| 4 — High: service-specific work on top of generic connectors | GitHub | Local Git operations exist, and optional One/Composio tools can cover supported remote actions. No dedicated per-app setup card or GitHub service adapter is registered. |
| 4 — High: service-specific work on top of generic connectors | Notion | Optional One/Composio tools may support it; no dedicated Gloamy Notion adapter or setup form exists. Verify connector availability before promising support. |
| 4 — High: service-specific work on top of generic connectors | Apple Notes | No dedicated Notes connector exists. A local macOS integration would need its own permissions and host behavior. |
| 4 — High: service-specific work on top of generic connectors | Apple Reminders | No dedicated Reminders connector exists. A local OS integration would need explicit permissions and host behavior. |
| 4 — High: service-specific work on top of generic connectors | Obsidian | File tools can access user-approved workspace files, but no vault-aware Obsidian adapter or setup flow exists. |
| 4 — High: service-specific work on top of generic connectors | Things 3 | No dedicated adapter exists. Likely requires service/app-specific automation and platform checks. |
| 4 — High: service-specific work on top of generic connectors | Bear Notes | No dedicated adapter exists. Likely requires service/app-specific automation and platform checks. |
| 4 — High: service-specific work on top of generic connectors | Trello | Optional One/Composio tools may support it; no dedicated Gloamy adapter or setup form exists. Verify connector availability and scopes. |
| 4 — High: service-specific work on top of generic connectors | Linear | Optional One/Composio tools may support it; no dedicated Gloamy adapter or setup form exists. Verify connector availability and scopes. |
| 4 — High: service-specific work on top of generic connectors | Spotify | No dedicated Spotify adapter or setup form exists; optional generic connectors may cover supported actions. |
| 4 — High: service-specific work on top of generic connectors | Sonos | No dedicated adapter exists. Device discovery, local network access, and service behavior need investigation. |
| 4 — High: service-specific work on top of generic connectors | Shazam | No dedicated recognition adapter exists; microphone/audio capture and service/API choice need investigation. |
| 4 — High: service-specific work on top of generic connectors | Home Assistant | No dedicated adapter exists. Remote URL, token, entity discovery, and action safety need integration-specific design. |
| 4 — High: service-specific work on top of generic connectors | Philips Hue | No dedicated adapter exists. Bridge discovery, pairing, credentials, and device control need integration-specific design. |
| 4 — High: service-specific work on top of generic connectors | 8Sleep | No dedicated adapter exists. Verify official API/auth availability before estimating implementation. |
| 4 — High: service-specific work on top of generic connectors | Gmail | Optional One/Composio tools may support it; no dedicated Gmail channel/setup flow is registered. Verify connector scopes and message safety. |
| 4 — High: service-specific work on top of generic connectors | 1Password | No dedicated adapter exists. Secret access has a high security impact; verify supported integration and least-privilege behavior first. |
| 4 — High: service-specific work on top of generic connectors | Weather | No dedicated weather provider exists. Existing web-search/HTTP tools are general capabilities, not a weather integration. |
| 4 — High: service-specific work on top of generic connectors | Image Gen | No dedicated image-generation tool is registered. Model input/vision support is not proof of image output support. |
| 4 — High: service-specific work on top of generic connectors | GIF Search | No dedicated search provider/tool exists. Select and verify a service/API before implementation. |
| 4 — High: service-specific work on top of generic connectors | Twitter/X | No dedicated adapter exists; optional One/Composio support must be verified alongside API access and write scopes. |
| 5 — Highest: no named wrapper or unsupported platform | Microsoft Teams | No dedicated Teams channel module or setup flow is present. Requires new provider/API integration and service configuration. |
| 5 — Highest: no named wrapper or unsupported platform | WebChat | The Vue app exists, but there is no separate WebChat channel/runtime implementation or setup contract. |
| 5 — Highest: no named wrapper or unsupported platform | Zalo | No dedicated Zalo channel module or setup flow is present. |
| 5 — Highest: no named wrapper or unsupported platform | Hugging Face | No named Hugging Face provider/factory registration is present. A custom compatible endpoint may work for some services but is not a dedicated wrapper. |
| 5 — Highest: no named wrapper or unsupported platform | Camera | No dedicated camera capture connector exists. Browser/device permission and capture flows need design. |

## Informational platform entries

These four entries are in the registry but are host/client descriptions rather than installable integrations, so they are not assigned a setup-effort rank.

| Entry | Existing support | Scope |
|---|---|---|
| macOS | Runtime host label; macOS automation exists on the supported Apple target. | Informational; not a separate installable integration. |
| Linux | Runtime host label; generic Linux amd64/arm64 are supported. | Informational; not a separate installable integration. |
| iOS | Can connect through existing chat services such as Telegram/Discord. | No native iOS application or backend host target is supported by this feature. |
| Android | Can connect through existing chat services such as Telegram/Discord. | No native Android application or backend host target is supported by this feature. |

Windows is not returned by the backend integration registry and is outside the supported host target policy; it is not part of the 78-entry inventory.

## Sources and limitations

- The entry count and UI category/name list come from src/integrations/registry.rs.
- Provider support is checked against src/providers/mod.rs and its named provider modules.
- Channel support is checked against src/channels/mod.rs runtime registration and channel modules.
- Local tool support is checked against src/tools/mod.rs and the tool implementations.
- Generic third-party wrappers are src/tools/one.rs and src/tools/composio.rs. They do not establish that every listed service is available from those vendors.
- UI card/setup behavior is in web/src/pages/integrations.vue and web/src/components/integrations/setupRegistry.ts.
- This is an implementation-readiness estimate, not a guarantee of vendor API access, account entitlement, or submodule need for a future selected implementation.
