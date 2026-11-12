use super::{IntegrationType, SetupCapability, registry};

/// Metadata for an installed setup handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupHandler {
    pub integration_id: &'static str,
    pub implementation_type: IntegrationType,
    pub capability: SetupCapability,
}

/// Resolves setup requests through one catalog-backed registry.
///
/// The four integration families are stable dispatch boundaries. A family only
/// becomes interactive when a concrete handler is registered in the catalog;
/// Codex device-code authorization is the only such flow today.
pub struct IntegrationSetupDispatcher;

impl IntegrationSetupDispatcher {
    pub fn resolve(integration_id: &str, capability: SetupCapability) -> Option<SetupHandler> {
        let entry = registry::all_integrations()
            .into_iter()
            .find(|entry| entry.id == integration_id)?;
        (entry.setup_capability == Some(capability)).then_some(SetupHandler {
            integration_id: entry.id,
            implementation_type: entry.implementation_type,
            capability,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatcher_resolves_only_registered_setup_capabilities() {
        let codex =
            IntegrationSetupDispatcher::resolve("openai-codex", SetupCapability::OAuthDeviceCode)
                .unwrap();
        assert_eq!(codex.integration_id, "openai-codex");
        assert_eq!(codex.implementation_type, IntegrationType::AiProvider);
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
    }
}
