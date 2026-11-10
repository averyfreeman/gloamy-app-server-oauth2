//! Authenticated, server-owned Codex OAuth setup operations.

use super::AppState;
use crate::auth::{AuthService, profiles::AuthProfileKind};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;

const PROVIDER: &str = "openai-codex";
const MODELS_ENDPOINT: &str = "https://chatgpt.com/backend-api/codex/models";
// Codex's backend uses client-version compatibility bands. This is the official
// stable Codex version verified on 2026-11-10, not Gloamy's identity/UA.
// See docs/codex-oauth-sources.md. Discovery remains best-effort.
const CATALOG_COMPATIBILITY_VERSION: &str = "0.167.0";

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}

/// Always require a stored bearer token, including when general pairing is off.
fn owner(state: &AppState, headers: &HeaderMap) -> Result<String, Box<Response>> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|token| state.pairing.is_valid_bearer_token(token))
        .ok_or_else(|| {
            Box::new(error(
                StatusCode::UNAUTHORIZED,
                "A valid paired bearer token is required",
            ))
        })?;
    Ok(hex::encode(Sha256::digest(token.as_bytes())))
}

pub(super) async fn is_connected(config: &crate::config::Config) -> anyhow::Result<bool> {
    let auth = AuthService::new(&crate::auth::state_dir_from_config(config), true);
    Ok(auth
        .get_profile(PROVIDER, None)
        .await?
        .is_some_and(|profile| {
            profile.kind == AuthProfileKind::OAuth
                && profile.token_set.is_some_and(|tokens| {
                    !tokens.access_token.trim().is_empty()
                        && (!tokens.is_expiring_within(Duration::ZERO)
                            || tokens
                                .refresh_token
                                .is_some_and(|token| !token.trim().is_empty()))
                })
        }))
}

pub(super) async fn start(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let owner = match owner(&state, &headers) {
        Ok(owner) => owner,
        Err(response) => return *response,
    };
    let config = state.config.lock().clone();
    match state.codex_setup.start(&owner, &config).await {
        Ok(view) => Json(view).into_response(),
        Err(_) => error(
            StatusCode::BAD_GATEWAY,
            "Unable to start authorization; retry shortly",
        ),
    }
}

pub(super) async fn inspect(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let owner = match owner(&state, &headers) {
        Ok(owner) => owner,
        Err(response) => return *response,
    };
    match state.codex_setup.inspect(&id, &owner).await {
        Some(view) => Json(view).into_response(),
        None => error(
            StatusCode::NOT_FOUND,
            "Authorization is unavailable or has been cleaned up",
        ),
    }
}

pub(super) async fn cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let owner = match owner(&state, &headers) {
        Ok(owner) => owner,
        Err(response) => return *response,
    };
    match state.codex_setup.cancel(&id, &owner).await {
        Some(view) => Json(view).into_response(),
        None => error(
            StatusCode::NOT_FOUND,
            "Authorization is unavailable or has been cleaned up",
        ),
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub(super) struct ModelChoice {
    id: String,
    label: String,
}

#[derive(Serialize)]
struct ModelChoices {
    models: Vec<ModelChoice>,
    source: &'static str,
    warning: Option<&'static str>,
    selected_model: Option<String>,
    is_default: bool,
    connected: bool,
}

#[derive(Deserialize)]
struct AccountModels {
    models: Vec<AccountModel>,
}

#[derive(Deserialize)]
struct AccountModel {
    slug: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
}

async fn request_account_models(
    client: &reqwest::Client,
    endpoint: &str,
    token: &str,
    account: &str,
) -> anyhow::Result<Vec<ModelChoice>> {
    let response = client
        .get(endpoint)
        .query(&[("client_version", CATALOG_COMPATIBILITY_VERSION)])
        .bearer_auth(token)
        .header("ChatGPT-Account-Id", account)
        .header("originator", "gloamy")
        .header(
            reqwest::header::USER_AGENT,
            concat!("gloamy/", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await?
        .error_for_status()?;
    let document: AccountModels = response.json().await?;
    let mut seen = std::collections::HashSet::new();
    Ok(document
        .models
        .into_iter()
        .filter(|model| matches!(model.visibility.as_deref(), None | Some("list")))
        .filter(|model| !model.slug.trim().is_empty() && seen.insert(model.slug.clone()))
        .take(512)
        .map(|model| ModelChoice {
            label: model
                .display_name
                .filter(|name| !name.trim().is_empty())
                .unwrap_or_else(|| model.slug.clone()),
            id: model.slug,
        })
        .collect())
}

async fn account_models(config: &crate::config::Config) -> anyhow::Result<Vec<ModelChoice>> {
    let auth = AuthService::new(&crate::auth::state_dir_from_config(config), true);
    let token = auth
        .get_valid_openai_access_token(None)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No connected account"))?;
    let profile = auth.get_profile(PROVIDER, None).await?;
    let account = profile
        .and_then(|profile| profile.account_id)
        .or_else(|| crate::auth::openai_oauth::extract_account_id_from_jwt(&token))
        .ok_or_else(|| anyhow::anyhow!("Connected account identifier unavailable"))?;
    let builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none());
    let client =
        crate::config::apply_runtime_proxy_to_builder(builder, "provider.openai").build()?;
    request_account_models(&client, MODELS_ENDPOINT, &token, &account).await
}

fn choose_models(
    catalog: &crate::model_catalog::Catalog,
    as_of: chrono::NaiveDate,
    account: Option<Vec<ModelChoice>>,
) -> (Vec<ModelChoice>, &'static str) {
    let curated = catalog.curated(PROVIDER, as_of);
    if let Some(mut models) = account.filter(|models| !models.is_empty()) {
        for model in &mut models {
            if let Some(entry) = curated
                .iter()
                .find(|entry| entry.id == model.id || entry.aliases.contains(&model.id))
            {
                model.label.clone_from(&entry.name);
            }
        }
        return (models, "account");
    }
    let models: Vec<_> = curated
        .into_iter()
        .map(|entry| ModelChoice {
            id: entry.id,
            label: entry.name,
        })
        .collect();
    let source = if models.is_empty() {
        "custom"
    } else {
        "catalog"
    };
    (models, source)
}

pub(super) async fn models(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Err(response) = owner(&state, &headers) {
        return *response;
    }
    let config = state.config.lock().clone();
    let connected = match is_connected(&config).await {
        Ok(connected) => connected,
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to read integration credentials",
            );
        }
    };
    let account = if connected {
        account_models(&config).await.ok()
    } else {
        None
    };
    let (models, source) = choose_models(
        &crate::model_catalog::global().snapshot(),
        crate::model_catalog::today(),
        account,
    );
    Json(ModelChoices {
        models, source, connected,
        warning: if source == "account" { None } else { Some("Account model discovery is unavailable. Catalog choices may differ from your account; custom IDs are always allowed.") },
        selected_model: if crate::providers::canonical_provider_name(config.default_provider.as_deref().unwrap_or("")) == PROVIDER { config.default_model.clone() } else { None },
        is_default: crate::providers::canonical_provider_name(config.default_provider.as_deref().unwrap_or("")) == PROVIDER,
    }).into_response()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ApplyModel {
    model: String,
    #[serde(default)]
    use_as_default: bool,
}

fn model_update(
    current: &crate::config::Config,
    request: &ApplyModel,
) -> Result<crate::config::Config, &'static str> {
    let model = request.model.trim();
    if model.is_empty() {
        return Err("Enter a model ID");
    }
    let already_default = crate::providers::canonical_provider_name(
        current.default_provider.as_deref().unwrap_or(""),
    ) == PROVIDER;
    if !already_default && !request.use_as_default {
        return Err("Choose Use Codex as default before changing providers");
    }
    // A provider switch must not silently reuse another provider's endpoint or
    // erase its override. Let the user remove the conflicting override explicitly.
    if !already_default
        && current
            .api_url
            .as_ref()
            .is_some_and(|url| !url.trim().is_empty())
    {
        return Err(
            "Remove the global API URL override in Configuration before using Codex as default",
        );
    }
    let mut updated = current.clone();
    if !already_default {
        updated.default_provider = Some(PROVIDER.into());
    }
    updated.default_model = Some(model.into());
    Ok(updated)
}

pub(super) async fn apply_model(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Json<ApplyModel>, axum::extract::rejection::JsonRejection>,
) -> Response {
    if let Err(response) = owner(&state, &headers) {
        return *response;
    }
    let request = match body {
        Ok(Json(request)) => request,
        Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid model selection request"),
    };
    let _write = state.config_update_lock.lock().await;
    let current = state.config.lock().clone();
    match is_connected(&current).await {
        Ok(true) => (),
        Ok(false) => return error(StatusCode::CONFLICT, "Connect your ChatGPT account first"),
        Err(_) => {
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to read integration credentials",
            );
        }
    }
    let updated = match model_update(&current, &request) {
        Ok(updated) => updated,
        Err(message) => return error(StatusCode::CONFLICT, message),
    };
    if updated.validate().is_err() {
        return error(
            StatusCode::BAD_REQUEST,
            "Configuration is invalid; fix it before applying a model",
        );
    }
    if updated.save().await.is_err() {
        return error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to save configuration; selection was not changed",
        );
    }
    let selected = updated.default_model.clone();
    *state.config.lock() = updated;
    Json(serde_json::json!({"applied":true,"is_default":true,"selected_model":selected}))
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn catalog() -> crate::model_catalog::Catalog {
        crate::model_catalog::Catalog::parse(
            r#"schema_version = 1
[[models]]
provider = "openai-codex"
id = "synthetic-codex"
name = "Curated synthetic label"
released = "2026-10-01"
verified = "2026-11-09"
sources = ["https://example.com/release"]
"#,
        )
        .unwrap()
    }

    #[test]
    fn models_use_account_ids_with_curated_labels_then_eligible_fallback() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 9).unwrap();
        let catalog = catalog();
        let (choices, source) = choose_models(
            &catalog,
            date,
            Some(vec![
                ModelChoice {
                    id: "synthetic-codex".into(),
                    label: "Account label".into(),
                },
                ModelChoice {
                    id: "unlisted-account-model".into(),
                    label: "Account-only model".into(),
                },
            ]),
        );
        assert_eq!(source, "account");
        assert_eq!(choices[0].label, "Curated synthetic label");
        assert_eq!(choices[1].id, "unlisted-account-model");
        let (choices, source) = choose_models(&catalog, date, Some(vec![]));
        assert_eq!(source, "catalog");
        assert_eq!(choices[0].id, "synthetic-codex");
        let expired = chrono::NaiveDate::from_ymd_opt(2027, 7, 1).unwrap();
        assert_eq!(choose_models(&catalog, expired, None), (vec![], "custom"));
    }

    #[test]
    fn model_update_requires_explicit_switch_and_preserves_all_other_values() {
        let mut config = Config::default();
        config.default_model = Some("original-model".into());
        config.api_key = Some("fixture-secret".into());
        config.reliability.fallback_providers = vec!["anthropic".into()];
        config.identity.aieos_inline = Some("fixture-personalization".into());
        config.model_routes = vec![crate::config::ModelRouteConfig {
            hint: "coding".into(),
            provider: "openai".into(),
            model: "route-model".into(),
            api_key: Some("fixture-route-key".into()),
        }];
        let mut request = ApplyModel {
            model: "custom/arbitrary-id".into(),
            use_as_default: false,
        };
        assert!(model_update(&config, &request).is_err());
        request.use_as_default = true;
        let updated = model_update(&config, &request).unwrap();
        let mut expected = serde_json::to_value(&config).unwrap();
        expected["default_provider"] = serde_json::json!(PROVIDER);
        expected["default_model"] = serde_json::json!("custom/arbitrary-id");
        assert_eq!(serde_json::to_value(&updated).unwrap(), expected);
        request.use_as_default = false;
        request.model = "another-custom-id".into();
        let updated = model_update(&updated, &request).unwrap();
        assert_eq!(updated.default_model.as_deref(), Some("another-custom-id"));
        config.api_url = Some("https://example.com/provider-endpoint".into());
        request.use_as_default = true;
        assert!(
            model_update(&config, &request)
                .unwrap_err()
                .contains("API URL override")
        );
    }

    #[tokio::test]
    async fn account_catalog_uses_codex_headers_and_filters_picker_visibility() {
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{header, method, path, query_param},
        };
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .and(query_param("client_version", CATALOG_COMPATIBILITY_VERSION))
            .and(header("Authorization", "Bearer fixture-access"))
            .and(header("ChatGPT-Account-Id", "fixture-account"))
            .and(header("originator", "gloamy"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"models":[
                    {"slug":"account-model","display_name":"Account label","visibility":"list"},
                    {"slug":"hidden-model","visibility":"hide"},
                    {"slug":"non-picker-model","visibility":"none"},
                    {"slug":"account-model","visibility":"list"},
                    {"slug":"","visibility":"list"}
                ]})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let choices = request_account_models(
            &reqwest::Client::new(),
            &format!("{}/models", server.uri()),
            "fixture-access",
            "fixture-account",
        )
        .await
        .unwrap();
        assert_eq!(
            choices,
            vec![ModelChoice {
                id: "account-model".into(),
                label: "Account label".into()
            }]
        );
    }
}
