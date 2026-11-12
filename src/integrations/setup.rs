use super::{IntegrationType, SetupCapability, registry};
use crate::auth::codex_setup::{CodexOAuthSetup, FlowView};
use crate::config::Config;
use anyhow::Result;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupHandlerKind {
    CodexOAuthDeviceCode,
}

/// A setup handler registered for one catalog entry and capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupHandler {
    pub integration_id: &'static str,
    pub implementation_type: IntegrationType,
    pub capability: SetupCapability,
    kind: SetupHandlerKind,
}

const REGISTERED_HANDLERS: &[SetupHandler] = &[SetupHandler {
    integration_id: "openai-codex",
    implementation_type: IntegrationType::AiProvider,
    capability: SetupCapability::OAuthDeviceCode,
    kind: SetupHandlerKind::CodexOAuthDeviceCode,
}];

/// Central registry and executable dispatcher for integration setup operations.
///
/// A catalog capability is usable only when its ID and capability resolve to one
/// of these executable handler registrations. Codex device-code authorization is
/// the only registered flow today.
pub struct IntegrationSetupDispatcher;

impl IntegrationSetupDispatcher {
    pub fn resolve(integration_id: &str, capability: SetupCapability) -> Option<SetupHandler> {
        let handler = REGISTERED_HANDLERS
            .iter()
            .find(|handler| {
                handler.integration_id == integration_id && handler.capability == capability
            })
            .copied()?;
        let entry = registry::all_integrations()
            .into_iter()
            .find(|entry| entry.id == integration_id)?;
        (entry.setup_capability == Some(capability)
            && entry.implementation_type == handler.implementation_type)
            .then_some(handler)
    }

    pub async fn start(
        handler: SetupHandler,
        setup: &Arc<CodexOAuthSetup>,
        owner_hash: &str,
        config: &Config,
    ) -> Result<FlowView> {
        match handler.kind {
            SetupHandlerKind::CodexOAuthDeviceCode => setup.start(owner_hash, config).await,
        }
    }

    pub async fn inspect(
        handler: SetupHandler,
        setup: &Arc<CodexOAuthSetup>,
        flow_id: &str,
        owner_hash: &str,
    ) -> Option<FlowView> {
        match handler.kind {
            SetupHandlerKind::CodexOAuthDeviceCode => setup.inspect(flow_id, owner_hash).await,
        }
    }

    pub async fn cancel(
        handler: SetupHandler,
        setup: &Arc<CodexOAuthSetup>,
        flow_id: &str,
        owner_hash: &str,
    ) -> Option<FlowView> {
        match handler.kind {
            SetupHandlerKind::CodexOAuthDeviceCode => setup.cancel(flow_id, owner_hash).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_only_executable_handlers_consistent_with_catalog() {
        let handler =
            IntegrationSetupDispatcher::resolve("openai-codex", SetupCapability::OAuthDeviceCode)
                .unwrap();
        assert_eq!(handler.integration_id, "openai-codex");
        assert_eq!(handler.implementation_type, IntegrationType::AiProvider);
        assert_eq!(handler.capability, SetupCapability::OAuthDeviceCode);

        assert!(
            IntegrationSetupDispatcher::resolve("openai-api", SetupCapability::OAuthDeviceCode)
                .is_none()
        );
        assert!(
            IntegrationSetupDispatcher::resolve(
                "missing-provider",
                SetupCapability::OAuthDeviceCode
            )
            .is_none()
        );

        for entry in registry::all_integrations() {
            if let Some(capability) = entry.setup_capability {
                assert!(IntegrationSetupDispatcher::resolve(entry.id, capability).is_some());
            }
        }
    }
}
