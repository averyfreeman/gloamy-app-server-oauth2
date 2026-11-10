use super::tests::{MockMemory, MockProvider};
use super::*;
use axum::http::HeaderValue;
use http_body_util::BodyExt;

fn state(config: Config) -> AppState {
    AppState {
        codex_setup: Arc::new(crate::auth::codex_setup::CodexOAuthSetup::new()),
        config_update_lock: Arc::new(tokio::sync::Mutex::new(())),
        config: Arc::new(Mutex::new(config)),
        provider: Arc::new(MockProvider::default()),
        model: "test-model".into(),
        temperature: 0.0,
        mem: Arc::new(MockMemory),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: Arc::new(PairingGuard::new(false, &["fixture-gateway-token".into()])),
        trust_forwarded_headers: false,
        rate_limiter: Arc::new(GatewayRateLimiter::new(100, 100, 100)),
        idempotency_store: Arc::new(IdempotencyStore::new(Duration::from_secs(300), 1000)),
        whatsapp: None,
        whatsapp_app_secret: None,
        linq: None,
        linq_signing_secret: None,
        nextcloud_talk: None,
        nextcloud_talk_webhook_secret: None,
        wati: None,
        observer: Arc::new(crate::observability::NoopObserver),
        tools_registry: Arc::new(Vec::new()),
        cost_tracker: None,
        event_tx: tokio::sync::broadcast::channel(16).0,
    }
}

fn headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_static("Bearer fixture-gateway-token"),
    );
    headers
}

async fn seed_auth(config: &Config) {
    crate::auth::AuthService::new(&crate::auth::state_dir_from_config(config), true)
        .store_openai_tokens(
            "fixture",
            crate::auth::profiles::TokenSet {
                access_token: "fixture-access-token".into(),
                refresh_token: Some("fixture-refresh-token".into()),
                id_token: None,
                expires_at: None,
                token_type: None,
                scope: None,
            },
            Some("fixture-account".into()),
            true,
        )
        .await
        .unwrap();
}

#[tokio::test]
async fn all_setup_operations_enforce_bearer_auth_with_pairing_disabled() {
    let directory = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.config_path = directory.path().join("config.toml");
    let state = state(config);
    let request = serde_json::from_value(serde_json::json!({"model":"custom-model"})).unwrap();
    let responses = [
        codex::start(State(state.clone()), HeaderMap::new()).await,
        codex::inspect(
            State(state.clone()),
            HeaderMap::new(),
            axum::extract::Path("unknown".into()),
        )
        .await,
        codex::cancel(
            State(state.clone()),
            HeaderMap::new(),
            axum::extract::Path("unknown".into()),
        )
        .await,
        codex::models(State(state.clone()), HeaderMap::new()).await,
        codex::apply_model(State(state), HeaderMap::new(), Ok(Json(request))).await,
    ];
    for response in responses {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn connected_state_is_independent_and_catalog_fields_are_additive() {
    let directory = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.config_path = directory.path().join("config.toml");
    config.default_provider = Some("openai".into());
    seed_auth(&config).await;
    let state = state(config);
    let response = api::handle_api_integrations(State(state.clone()), headers())
        .await
        .into_response();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entries = value["integrations"].as_array().unwrap();
    let entry = entries
        .iter()
        .find(|entry| entry["id"] == "openai-codex")
        .unwrap();
    assert_eq!(entry["connected"], true);
    assert_eq!(entry["status"], "Active");
    assert_eq!(entry["setup_capability"], "oauth_device_code");
    for field in ["name", "description", "category", "status"] {
        assert!(entry.get(field).is_some());
    }
    assert!(entries.iter().any(|entry| entry["id"] == "openai-api"));
    assert_eq!(
        state.config.lock().default_provider.as_deref(),
        Some("openai")
    );
    let encoded = String::from_utf8(body.to_vec()).unwrap();
    assert!(!encoded.contains("fixture-access-token"));
    assert!(!encoded.contains("fixture-refresh-token"));
}

#[tokio::test]
async fn apply_persists_only_selection_and_failed_save_keeps_memory() {
    let directory = tempfile::TempDir::new().unwrap();
    let mut config = Config::default();
    config.config_path = directory.path().join("config.toml");
    config.workspace_dir = directory.path().join("workspace");
    config.default_model = Some("original-model".into());
    config.api_key = Some("fixture-unrelated-secret".into());
    config.reliability.fallback_providers = vec!["anthropic".into()];
    config.save().await.unwrap();
    seed_auth(&config).await;
    let before = serde_json::to_value(&config).unwrap();
    let state = state(config);
    let original_file = std::fs::read(state.config.lock().config_path.clone()).unwrap();
    let request = serde_json::from_value(serde_json::json!({"model":"custom/model-a"})).unwrap();
    let response = codex::apply_model(State(state.clone()), headers(), Ok(Json(request))).await;
    assert_eq!(response.status(), StatusCode::CONFLICT);
    assert_eq!(serde_json::to_value(&*state.config.lock()).unwrap(), before);
    assert_eq!(
        std::fs::read(state.config.lock().config_path.clone()).unwrap(),
        original_file
    );
    let request = serde_json::from_value(
        serde_json::json!({"model":"custom/model-a", "use_as_default":true}),
    )
    .unwrap();
    let response = codex::apply_model(State(state.clone()), headers(), Ok(Json(request))).await;
    assert_eq!(response.status(), StatusCode::OK);
    let mut expected = before;
    expected["default_provider"] = serde_json::json!("openai-codex");
    expected["default_model"] = serde_json::json!("custom/model-a");
    assert_eq!(
        serde_json::to_value(&*state.config.lock()).unwrap(),
        expected
    );
    let saved: Config =
        toml::from_str(&std::fs::read_to_string(state.config.lock().config_path.clone()).unwrap())
            .unwrap();
    assert_eq!(saved.default_model.as_deref(), Some("custom/model-a"));
    assert_eq!(saved.reliability.fallback_providers, vec!["anthropic"]);
    assert!(crate::security::SecretStore::is_encrypted(
        saved.api_key.as_deref().unwrap()
    ));
    // Already-default selection requires no provider-switch checkbox.
    let request = serde_json::from_value(serde_json::json!({"model":"another-custom-id"})).unwrap();
    let response = codex::apply_model(State(state.clone()), headers(), Ok(Json(request))).await;
    assert_eq!(response.status(), StatusCode::OK);
    expected["default_model"] = serde_json::json!("another-custom-id");
    assert_eq!(
        serde_json::to_value(&*state.config.lock()).unwrap(),
        expected
    );
    // This config destination is a directory; the auth directory is unchanged.
    let invalid_path = directory.path().join("directory-config");
    std::fs::create_dir(&invalid_path).unwrap();
    state.config.lock().config_path = invalid_path;
    let request = serde_json::from_value(serde_json::json!({"model":"must-not-apply"})).unwrap();
    let response = codex::apply_model(State(state.clone()), headers(), Ok(Json(request))).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        serde_json::to_value(&*state.config.lock()).unwrap(),
        expected
    );
}
