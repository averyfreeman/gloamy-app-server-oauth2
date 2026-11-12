use super::{
    IntegrationCategory, IntegrationEntry, IntegrationStatus, IntegrationType, SetupCapability,
};
use crate::providers::{
    is_glm_alias, is_minimax_alias, is_moonshot_alias, is_qianfan_alias, is_qwen_alias,
    is_zai_alias,
};

/// Returns the full catalog of integrations
#[allow(clippy::too_many_lines)]
pub fn all_integrations() -> Vec<IntegrationEntry> {
    vec![
        // ── Chat Providers ──────────────────────────────────────
        IntegrationEntry {
            id: "telegram",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Telegram",
            description: "Bot API — long-polling",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.telegram.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "discord",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Discord",
            description: "Servers, channels & DMs",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.discord.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "slack",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Slack",
            description: "Workspace apps via Web API",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.slack.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "webhooks",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Webhooks",
            description: "HTTP endpoint for triggers",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.webhook.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "whatsapp",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "WhatsApp",
            description: "Meta Cloud API via webhook",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.whatsapp.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "signal",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Signal",
            description: "Privacy-focused via signal-cli",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.signal.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "imessage",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "iMessage",
            description: "macOS AppleScript bridge",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.imessage.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "microsoft-teams",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Microsoft Teams",
            description: "Enterprise chat support",
            category: IntegrationCategory::Chat,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "matrix",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Matrix",
            description: "Matrix protocol (Element)",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.matrix.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "nostr",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Nostr",
            description: "Decentralized DMs (NIP-04)",
            category: IntegrationCategory::Chat,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "webchat",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "WebChat",
            description: "Browser-based chat UI",
            category: IntegrationCategory::Chat,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "nextcloud-talk",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Nextcloud Talk",
            description: "Self-hosted Nextcloud chat",
            category: IntegrationCategory::Chat,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "zalo",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "Zalo",
            description: "Zalo Bot API",
            category: IntegrationCategory::Chat,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "dingtalk",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "DingTalk",
            description: "DingTalk Stream Mode",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.dingtalk.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "qq-official",
            implementation_type: IntegrationType::CommunicationChannel,
            setup_capability: None,
            name: "QQ Official",
            description: "Tencent QQ Bot SDK",
            category: IntegrationCategory::Chat,
            status_fn: |c| {
                if c.channels_config.qq.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        // ── AI Models ───────────────────────────────────────────
        IntegrationEntry {
            id: "openrouter",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "OpenRouter",
            description: "200+ models, 1 API key",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("openrouter") && c.api_key.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "anthropic",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Anthropic",
            description: "Anthropic language models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("anthropic") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "openai-codex",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: Some(SetupCapability::OAuthDeviceCode),
            name: "OpenAI Codex (ChatGPT OAuth)",
            description: "Connect your ChatGPT account for Codex; separate from OpenAI API keys",
            category: IntegrationCategory::AiModel,
            // AuthService supplies connected state at the gateway boundary.
            status_fn: |_| IntegrationStatus::Available,
        },
        IntegrationEntry {
            id: "openai-api",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "OpenAI",
            description: "OpenAI API language models using an API key (separate API billing)",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("openai") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "google",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Google",
            description: "Google Gemini language models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_model
                    .as_deref()
                    .is_some_and(|m| m.starts_with("google/"))
                {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "deepseek",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "DeepSeek",
            description: "DeepSeek V3 & R1",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_model
                    .as_deref()
                    .is_some_and(|m| m.starts_with("deepseek/"))
                {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "xai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "xAI",
            description: "xAI language models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_model
                    .as_deref()
                    .is_some_and(|m| m.starts_with("x-ai/"))
                {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "mistral",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Mistral",
            description: "Mistral language models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_model
                    .as_deref()
                    .is_some_and(|m| m.starts_with("mistral"))
                {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "ollama",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Ollama",
            description: "Local models (Llama, etc.)",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("ollama") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "perplexity",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Perplexity",
            description: "Search-augmented AI",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("perplexity") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "hugging-face",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Hugging Face",
            description: "Open-source models",
            category: IntegrationCategory::AiModel,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "lm-studio",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "LM Studio",
            description: "Local model server",
            category: IntegrationCategory::AiModel,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "venice",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Venice",
            description: "Privacy-first inference (Llama, Opus)",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("venice") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "vercel-ai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Vercel AI",
            description: "Vercel AI Gateway",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("vercel") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "cloudflare-ai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Cloudflare AI",
            description: "Cloudflare AI Gateway",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("cloudflare") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "moonshot",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Moonshot",
            description: "Kimi & Kimi Coding",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_moonshot_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "synthetic",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Synthetic",
            description: "Synthetic AI models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("synthetic") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "opencode-zen",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "OpenCode Zen",
            description: "Code-focused AI models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("opencode") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "zai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Z.AI",
            description: "Z.AI inference",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_zai_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "glm",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "GLM",
            description: "ChatGLM / Zhipu models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_glm_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "minimax",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "MiniMax",
            description: "MiniMax AI models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_minimax_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "qwen",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Qwen",
            description: "Alibaba DashScope Qwen models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_qwen_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "amazon-bedrock",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Amazon Bedrock",
            description: "AWS managed model access",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("bedrock") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "qianfan",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Qianfan",
            description: "Baidu AI models",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref().is_some_and(is_qianfan_alias) {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "groq",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Groq",
            description: "Ultra-fast LPU inference",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("groq") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "together-ai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Together AI",
            description: "Open-source model hosting",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("together") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "fireworks-ai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Fireworks AI",
            description: "Fast open-source inference",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("fireworks") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "novita-ai",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Novita AI",
            description: "Affordable open-source inference",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("novita") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "cohere",
            implementation_type: IntegrationType::AiProvider,
            setup_capability: None,
            name: "Cohere",
            description: "Command R+ & embeddings",
            category: IntegrationCategory::AiModel,
            status_fn: |c| {
                if c.default_provider.as_deref() == Some("cohere") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        // ── Productivity ────────────────────────────────────────
        IntegrationEntry {
            id: "github",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "GitHub",
            description: "Code, issues, PRs",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "notion",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Notion",
            description: "Workspace & databases",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "apple-notes",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Apple Notes",
            description: "Native macOS/iOS notes",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "apple-reminders",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Apple Reminders",
            description: "Task management",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "obsidian",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Obsidian",
            description: "Knowledge graph notes",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "things-3",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Things 3",
            description: "GTD task manager",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "bear-notes",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Bear Notes",
            description: "Markdown notes",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "trello",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Trello",
            description: "Kanban boards",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "linear",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Linear",
            description: "Issue tracking",
            category: IntegrationCategory::Productivity,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        // ── Music & Audio ───────────────────────────────────────
        IntegrationEntry {
            id: "spotify",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Spotify",
            description: "Music playback control",
            category: IntegrationCategory::MusicAudio,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "sonos",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Sonos",
            description: "Multi-room audio",
            category: IntegrationCategory::MusicAudio,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "shazam",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Shazam",
            description: "Song recognition",
            category: IntegrationCategory::MusicAudio,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        // ── Smart Home ──────────────────────────────────────────
        IntegrationEntry {
            id: "home-assistant",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Home Assistant",
            description: "Home automation hub",
            category: IntegrationCategory::SmartHome,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "philips-hue",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Philips Hue",
            description: "Smart lighting",
            category: IntegrationCategory::SmartHome,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "8sleep",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "8Sleep",
            description: "Smart mattress",
            category: IntegrationCategory::SmartHome,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        // ── Tools & Automation ──────────────────────────────────
        IntegrationEntry {
            id: "browser",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Browser",
            description: "Chrome/Chromium control",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::Available,
        },
        IntegrationEntry {
            id: "shell",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Shell",
            description: "Terminal command execution",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::Active,
        },
        IntegrationEntry {
            id: "file-system",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "File System",
            description: "Read/write files",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::Active,
        },
        IntegrationEntry {
            id: "cron",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Cron",
            description: "Scheduled tasks",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::Available,
        },
        IntegrationEntry {
            id: "voice",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Voice",
            description: "Voice wake + talk mode",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "gmail",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Gmail",
            description: "Email triggers & send",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "1password",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "1Password",
            description: "Secure credentials",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "weather",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Weather",
            description: "Forecasts & conditions",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "canvas",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Canvas",
            description: "Visual workspace + A2UI",
            category: IntegrationCategory::ToolsAutomation,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        // ── Media & Creative ────────────────────────────────────
        IntegrationEntry {
            id: "image-gen",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Image Gen",
            description: "AI image generation",
            category: IntegrationCategory::MediaCreative,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "gif-search",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "GIF Search",
            description: "Find the perfect GIF",
            category: IntegrationCategory::MediaCreative,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "screen-capture",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Screen Capture",
            description: "Screenshot & screen control",
            category: IntegrationCategory::MediaCreative,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "camera",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Camera",
            description: "Photo/video capture",
            category: IntegrationCategory::MediaCreative,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        // ── Social ──────────────────────────────────────────────
        IntegrationEntry {
            id: "twitter-x",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Twitter/X",
            description: "Tweet, reply, search",
            category: IntegrationCategory::Social,
            status_fn: |_| IntegrationStatus::ComingSoon,
        },
        IntegrationEntry {
            id: "email",
            implementation_type: IntegrationType::RemoteService,
            setup_capability: None,
            name: "Email",
            description: "IMAP/SMTP email channel",
            category: IntegrationCategory::Social,
            status_fn: |c| {
                if c.channels_config.email.is_some() {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        // ── Platforms ───────────────────────────────────────────
        IntegrationEntry {
            id: "macos",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "macOS",
            description: "Native support + AppleScript",
            category: IntegrationCategory::Platform,
            status_fn: |_| {
                if cfg!(target_os = "macos") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "linux",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Linux",
            description: "Native support",
            category: IntegrationCategory::Platform,
            status_fn: |_| {
                if cfg!(target_os = "linux") {
                    IntegrationStatus::Active
                } else {
                    IntegrationStatus::Available
                }
            },
        },
        IntegrationEntry {
            id: "ios",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "iOS",
            description: "Chat via Telegram/Discord",
            category: IntegrationCategory::Platform,
            status_fn: |_| IntegrationStatus::Available,
        },
        IntegrationEntry {
            id: "android",
            implementation_type: IntegrationType::LocalRuntime,
            setup_capability: None,
            name: "Android",
            description: "Chat via Telegram/Discord",
            category: IntegrationCategory::Platform,
            status_fn: |_| IntegrationStatus::Available,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::config::schema::{IMessageConfig, MatrixConfig, StreamMode, TelegramConfig};

    #[test]
    fn registry_has_entries() {
        let entries = all_integrations();
        assert!(
            entries.len() >= 50,
            "Expected 50+ integrations, got {}",
            entries.len()
        );
    }

    #[test]
    fn all_categories_represented() {
        let entries = all_integrations();
        for cat in IntegrationCategory::all() {
            let count = entries.iter().filter(|e| e.category == *cat).count();
            assert!(count > 0, "Category {cat:?} has no entries");
        }
    }

    #[test]
    fn status_functions_dont_panic() {
        let config = Config::default();
        let entries = all_integrations();
        for entry in &entries {
            let _ = (entry.status_fn)(&config);
        }
    }

    #[test]
    fn no_duplicate_names() {
        let entries = all_integrations();
        let mut seen = std::collections::HashSet::new();
        for entry in &entries {
            assert!(
                seen.insert(entry.name),
                "Duplicate integration name: {}",
                entry.name
            );
        }
    }

    #[test]
    fn no_empty_names_or_descriptions() {
        let entries = all_integrations();
        for entry in &entries {
            assert!(!entry.name.is_empty(), "Found integration with empty name");
            assert!(
                !entry.description.is_empty(),
                "Integration '{}' has empty description",
                entry.name
            );
        }
    }

    #[test]
    fn telegram_active_when_configured() {
        let mut config = Config::default();
        config.channels_config.telegram = Some(TelegramConfig {
            bot_token: "123:ABC".into(),
            allowed_users: vec!["user".into()],
            stream_mode: StreamMode::default(),
            draft_update_interval_ms: 1000,
            interrupt_on_new_message: false,
            mention_only: false,
        });
        let entries = all_integrations();
        let tg = entries.iter().find(|e| e.name == "Telegram").unwrap();
        assert!(matches!((tg.status_fn)(&config), IntegrationStatus::Active));
    }

    #[test]
    fn telegram_available_when_not_configured() {
        let config = Config::default();
        let entries = all_integrations();
        let tg = entries.iter().find(|e| e.name == "Telegram").unwrap();
        assert!(matches!(
            (tg.status_fn)(&config),
            IntegrationStatus::Available
        ));
    }

    #[test]
    fn imessage_active_when_configured() {
        let mut config = Config::default();
        config.channels_config.imessage = Some(IMessageConfig {
            allowed_contacts: vec!["*".into()],
        });
        let entries = all_integrations();
        let im = entries.iter().find(|e| e.name == "iMessage").unwrap();
        assert!(matches!((im.status_fn)(&config), IntegrationStatus::Active));
    }

    #[test]
    fn imessage_available_when_not_configured() {
        let config = Config::default();
        let entries = all_integrations();
        let im = entries.iter().find(|e| e.name == "iMessage").unwrap();
        assert!(matches!(
            (im.status_fn)(&config),
            IntegrationStatus::Available
        ));
    }

    #[test]
    fn matrix_active_when_configured() {
        let mut config = Config::default();
        config.channels_config.matrix = Some(MatrixConfig {
            homeserver: "https://m.org".into(),
            access_token: "tok".into(),
            user_id: None,
            device_id: None,
            room_id: "!r:m".into(),
            allowed_users: vec![],
        });
        let entries = all_integrations();
        let mx = entries.iter().find(|e| e.name == "Matrix").unwrap();
        assert!(matches!((mx.status_fn)(&config), IntegrationStatus::Active));
    }

    #[test]
    fn matrix_available_when_not_configured() {
        let config = Config::default();
        let entries = all_integrations();
        let mx = entries.iter().find(|e| e.name == "Matrix").unwrap();
        assert!(matches!(
            (mx.status_fn)(&config),
            IntegrationStatus::Available
        ));
    }

    #[test]
    fn coming_soon_integrations_stay_coming_soon() {
        let config = Config::default();
        let entries = all_integrations();
        for name in ["Nostr", "Spotify", "Home Assistant"] {
            let entry = entries.iter().find(|e| e.name == name).unwrap();
            assert!(
                matches!((entry.status_fn)(&config), IntegrationStatus::ComingSoon),
                "{name} should be ComingSoon"
            );
        }
    }

    #[test]
    fn whatsapp_available_when_not_configured() {
        let config = Config::default();
        let entries = all_integrations();
        let wa = entries.iter().find(|e| e.name == "WhatsApp").unwrap();
        assert!(matches!(
            (wa.status_fn)(&config),
            IntegrationStatus::Available
        ));
    }

    #[test]
    fn email_available_when_not_configured() {
        let config = Config::default();
        let entries = all_integrations();
        let email = entries.iter().find(|e| e.name == "Email").unwrap();
        assert!(matches!(
            (email.status_fn)(&config),
            IntegrationStatus::Available
        ));
    }

    #[test]
    fn shell_and_filesystem_always_active() {
        let config = Config::default();
        let entries = all_integrations();
        for name in ["Shell", "File System"] {
            let entry = entries.iter().find(|e| e.name == name).unwrap();
            assert!(
                matches!((entry.status_fn)(&config), IntegrationStatus::Active),
                "{name} should always be Active"
            );
        }
    }

    #[test]
    fn macos_active_on_macos() {
        let config = Config::default();
        let entries = all_integrations();
        let macos = entries.iter().find(|e| e.name == "macOS").unwrap();
        let status = (macos.status_fn)(&config);
        if cfg!(target_os = "macos") {
            assert!(matches!(status, IntegrationStatus::Active));
        } else {
            assert!(matches!(status, IntegrationStatus::Available));
        }
    }

    #[test]
    fn category_counts_reasonable() {
        let entries = all_integrations();
        let chat_count = entries
            .iter()
            .filter(|e| e.category == IntegrationCategory::Chat)
            .count();
        let ai_count = entries
            .iter()
            .filter(|e| e.category == IntegrationCategory::AiModel)
            .count();
        assert!(
            chat_count >= 5,
            "Expected 5+ chat integrations, got {chat_count}"
        );
        assert!(
            ai_count >= 5,
            "Expected 5+ AI model integrations, got {ai_count}"
        );
    }

    #[test]
    fn every_catalog_entry_has_a_unique_stable_id_and_supported_family() {
        let entries = all_integrations();
        let mut ids = std::collections::HashSet::new();
        for entry in &entries {
            assert!(!entry.id.is_empty(), "{} must have a stable ID", entry.name);
            assert!(
                ids.insert(entry.id),
                "duplicate integration ID: {}",
                entry.id
            );
            assert!(matches!(
                entry.implementation_type,
                super::super::IntegrationType::AiProvider
                    | super::super::IntegrationType::CommunicationChannel
                    | super::super::IntegrationType::RemoteService
                    | super::super::IntegrationType::LocalRuntime
            ));
        }
        assert_eq!(entries.len(), 78);
        assert_eq!(ids.len(), entries.len());
    }

    #[test]
    fn regional_provider_aliases_activate_expected_ai_integrations() {
        let entries = all_integrations();
        let mut config = Config {
            default_provider: Some("minimax-cn".to_string()),
            ..Config::default()
        };

        let minimax = entries.iter().find(|e| e.name == "MiniMax").unwrap();
        assert!(matches!(
            (minimax.status_fn)(&config),
            IntegrationStatus::Active
        ));

        config.default_provider = Some("glm-cn".to_string());
        let glm = entries.iter().find(|e| e.name == "GLM").unwrap();
        assert!(matches!(
            (glm.status_fn)(&config),
            IntegrationStatus::Active
        ));

        config.default_provider = Some("moonshot-intl".to_string());
        let moonshot = entries.iter().find(|e| e.name == "Moonshot").unwrap();
        assert!(matches!(
            (moonshot.status_fn)(&config),
            IntegrationStatus::Active
        ));

        config.default_provider = Some("qwen-intl".to_string());
        let qwen = entries.iter().find(|e| e.name == "Qwen").unwrap();
        assert!(matches!(
            (qwen.status_fn)(&config),
            IntegrationStatus::Active
        ));

        config.default_provider = Some("zai-cn".to_string());
        let zai = entries.iter().find(|e| e.name == "Z.AI").unwrap();
        assert!(matches!(
            (zai.status_fn)(&config),
            IntegrationStatus::Active
        ));

        config.default_provider = Some("baidu".to_string());
        let qianfan = entries.iter().find(|e| e.name == "Qianfan").unwrap();
        assert!(matches!(
            (qianfan.status_fn)(&config),
            IntegrationStatus::Active
        ));
    }
}
