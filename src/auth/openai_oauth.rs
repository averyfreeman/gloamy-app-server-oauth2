use crate::auth::oauth_common::{parse_query_params, url_encode};

use crate::auth::profiles::TokenSet;
use anyhow::{Context, Result};
use base64::Engine;
use chrono::Utc;
use reqwest::Client;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

// Re-export for external use (used by main.rs)
#[allow(unused_imports)]
pub use crate::auth::oauth_common::{PkceState, generate_pkce_state};

pub const OPENAI_OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const OPENAI_OAUTH_AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
pub const OPENAI_OAUTH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub const OPENAI_OAUTH_DEVICE_CODE_URL: &str =
    "https://auth.openai.com/api/accounts/deviceauth/usercode";
const OPENAI_DEVICE_POLL_URL: &str = "https://auth.openai.com/api/accounts/deviceauth/token";
const OPENAI_DEVICE_CALLBACK: &str = "https://auth.openai.com/deviceauth/callback";
pub const OPENAI_OAUTH_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";

#[derive(Clone)]
pub struct DeviceCodeStart {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
    pub message: Option<String>,
}

impl std::fmt::Debug for DeviceCodeStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeviceCodeStart")
            .field("expires_in", &self.expires_in)
            .field("interval", &self.interval)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    token_type: Option<String>,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_auth_id: String,
    #[serde(alias = "usercode")]
    user_code: String,
    #[serde(
        default = "default_device_interval",
        deserialize_with = "device_interval"
    )]
    interval: u64,
}

#[derive(Deserialize)]
struct DeviceAuthorization {
    authorization_code: String,
    code_verifier: String,
}

fn default_device_interval() -> u64 {
    5
}

fn device_interval<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Interval {
        Seconds(u64),
        Text(String),
    }
    match Interval::deserialize(deserializer)? {
        Interval::Seconds(value) => Ok(value.max(1)),
        Interval::Text(value) => value
            .trim()
            .parse::<u64>()
            .map(|value| value.max(1))
            .map_err(|_| serde::de::Error::custom("invalid device polling interval")),
    }
}

pub fn build_authorize_url(pkce: &PkceState) -> String {
    let mut params = BTreeMap::new();
    params.insert("response_type", "code");
    params.insert("client_id", OPENAI_OAUTH_CLIENT_ID);
    params.insert("redirect_uri", OPENAI_OAUTH_REDIRECT_URI);
    params.insert("scope", "openid profile email offline_access");
    params.insert("code_challenge", pkce.code_challenge.as_str());
    params.insert("code_challenge_method", "S256");
    params.insert("state", pkce.state.as_str());
    params.insert("codex_cli_simplified_flow", "true");
    params.insert("id_token_add_organizations", "true");

    let mut encoded: Vec<String> = Vec::with_capacity(params.len());
    for (k, v) in params {
        encoded.push(format!("{}={}", url_encode(k), url_encode(v)));
    }

    format!("{OPENAI_OAUTH_AUTHORIZE_URL}?{}", encoded.join("&"))
}

pub async fn exchange_code_for_tokens(
    client: &Client,
    code: &str,
    pkce: &PkceState,
) -> Result<TokenSet> {
    exchange_code_for_tokens_at(
        client,
        OPENAI_OAUTH_TOKEN_URL,
        code,
        &pkce.code_verifier,
        OPENAI_OAUTH_REDIRECT_URI,
    )
    .await
}

async fn exchange_code_for_tokens_at(
    client: &Client,
    token_url: &str,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
) -> Result<TokenSet> {
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", OPENAI_OAUTH_CLIENT_ID),
        ("redirect_uri", redirect_uri),
        ("code_verifier", verifier),
    ];

    let response = client
        .post(token_url)
        .form(&form)
        .send()
        .await
        .context("Failed to exchange OpenAI OAuth authorization code")?;

    parse_token_response(response).await
}

pub async fn refresh_access_token(client: &Client, refresh_token: &str) -> Result<TokenSet> {
    let form = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", OPENAI_OAUTH_CLIENT_ID),
    ];

    let response = client
        .post(OPENAI_OAUTH_TOKEN_URL)
        .form(&form)
        .send()
        .await
        .context("Failed to refresh OpenAI OAuth token")?;

    parse_token_response(response).await
}

pub async fn start_device_code_flow(client: &Client) -> Result<DeviceCodeStart> {
    start_device_code_flow_at(client, OPENAI_OAUTH_DEVICE_CODE_URL).await
}

// Current first-party protocol: deviceauth/usercode -> deviceauth/token ->
// OAuth authorization-code exchange. Never expose the private authorization ID
// or verifier to the browser. See docs/codex-oauth-sources.md.
async fn start_device_code_flow_at(client: &Client, endpoint: &str) -> Result<DeviceCodeStart> {
    let response = client
        .post(endpoint)
        .json(&serde_json::json!({"client_id": OPENAI_OAUTH_CLIENT_ID}))
        .send()
        .await
        .context("Failed to start OpenAI device authorization")?;
    if !response.status().is_success() {
        anyhow::bail!(
            "OpenAI device authorization start failed ({})",
            response.status()
        );
    }
    let parsed: DeviceCodeResponse = response
        .json()
        .await
        .context("Invalid OpenAI device authorization response")?;
    if parsed.device_auth_id.trim().is_empty() || parsed.user_code.trim().is_empty() {
        anyhow::bail!("OpenAI returned incomplete device authorization");
    }
    Ok(DeviceCodeStart {
        device_code: parsed.device_auth_id,
        user_code: parsed.user_code,
        verification_uri: "https://auth.openai.com/codex/device".into(),
        verification_uri_complete: None,
        expires_in: 15 * 60,
        interval: parsed.interval.max(1),
        message: None,
    })
}

pub async fn poll_device_code_tokens(
    client: &Client,
    device: &DeviceCodeStart,
) -> Result<TokenSet> {
    poll_device_code_tokens_at(
        client,
        device,
        OPENAI_DEVICE_POLL_URL,
        OPENAI_OAUTH_TOKEN_URL,
    )
    .await
}

async fn poll_device_code_tokens_at(
    client: &Client,
    device: &DeviceCodeStart,
    poll_url: &str,
    token_url: &str,
) -> Result<TokenSet> {
    let lifetime = Duration::from_secs(device.expires_in.min(15 * 60));
    tokio::time::timeout(lifetime, async {
        loop {
            let response = client
                .post(poll_url)
                .json(&serde_json::json!({
                    "device_auth_id": device.device_code,
                    "user_code": device.user_code,
                }))
                .send()
                .await
                .context("Failed polling OpenAI device authorization")?;
            let status = response.status();
            if status.is_success() {
                let authorization: DeviceAuthorization = response
                    .json()
                    .await
                    .context("Invalid OpenAI device authorization result")?;
                return exchange_code_for_tokens_at(
                    client,
                    token_url,
                    &authorization.authorization_code,
                    &authorization.code_verifier,
                    OPENAI_DEVICE_CALLBACK,
                )
                .await;
            }
            if status == reqwest::StatusCode::FORBIDDEN || status == reqwest::StatusCode::NOT_FOUND
            {
                tokio::time::sleep(Duration::from_secs(device.interval.max(1))).await;
                continue;
            }
            anyhow::bail!("OpenAI device authorization failed ({status})");
        }
    })
    .await
    .context("Device authorization expired")?
}

pub async fn receive_loopback_code(expected_state: &str, timeout: Duration) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:1455")
        .await
        .context("Failed to bind callback listener at 127.0.0.1:1455")?;

    let accepted = tokio::time::timeout(timeout, listener.accept())
        .await
        .context("Timed out waiting for browser callback")?
        .context("Failed to accept callback connection")?;

    let (mut stream, _) = accepted;
    let mut buffer = vec![0_u8; 8192];
    let bytes_read = stream
        .read(&mut buffer)
        .await
        .context("Failed to read callback request")?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Malformed callback request"))?;

    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Callback request missing path"))?;

    let code = parse_code_from_redirect(path, Some(expected_state))?;

    let body =
        "<html><body><h2>Gloamy login complete</h2><p>You can close this tab.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes()).await;

    Ok(code)
}

pub fn parse_code_from_redirect(input: &str, expected_state: Option<&str>) -> Result<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!("No OAuth code provided");
    }

    let query = if let Some((_, right)) = trimmed.split_once('?') {
        right
    } else {
        trimmed
    };

    let params = parse_query_params(query);
    let is_callback_payload = trimmed.contains('?')
        || params.contains_key("code")
        || params.contains_key("state")
        || params.contains_key("error");

    if let Some(err) = params.get("error") {
        let desc = params
            .get("error_description")
            .cloned()
            .unwrap_or_else(|| "OAuth authorization failed".to_string());
        anyhow::bail!("OpenAI OAuth error: {err} ({desc})");
    }

    if let Some(expected_state) = expected_state {
        if let Some(got) = params.get("state") {
            if got != expected_state {
                anyhow::bail!("OAuth state mismatch");
            }
        } else if is_callback_payload {
            anyhow::bail!("Missing OAuth state in callback");
        }
    }

    if let Some(code) = params.get("code").cloned() {
        return Ok(code);
    }

    if !is_callback_payload {
        return Ok(trimmed.to_string());
    }

    anyhow::bail!("Missing OAuth code in callback")
}

pub fn extract_account_id_from_jwt(token: &str) -> Option<String> {
    let payload = token.split('.').nth(1)?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;

    if let Some(account) = claims
        .pointer("/https:~1~1api.openai.com~1auth/chatgpt_account_id")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        return Some(account.to_string());
    }

    for key in [
        "account_id",
        "accountId",
        "acct",
        "https://api.openai.com/account_id",
    ] {
        if let Some(value) = claims.get(key).and_then(|v| v.as_str()) {
            if !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

async fn parse_token_response(response: reqwest::Response) -> Result<TokenSet> {
    if !response.status().is_success() {
        let status = response.status();
        anyhow::bail!("OpenAI OAuth token request failed ({status})");
    }

    let token: TokenResponse = response
        .json()
        .await
        .context("Failed to parse OpenAI token response")?;

    let expires_at = token.expires_in.and_then(|seconds| {
        if seconds <= 0 {
            None
        } else {
            Some(Utc::now() + chrono::Duration::seconds(seconds))
        }
    });

    Ok(TokenSet {
        access_token: token.access_token,
        refresh_token: token.refresh_token,
        id_token: token.id_token,
        expires_at,
        token_type: token.token_type,
        scope: token.scope,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_generation_is_valid() {
        let pkce = generate_pkce_state();
        assert!(pkce.code_verifier.len() >= 43);
        assert!(!pkce.code_challenge.is_empty());
        assert!(!pkce.state.is_empty());
    }

    #[test]
    fn parse_redirect_url_extracts_code() {
        let code = parse_code_from_redirect(
            "http://127.0.0.1:1455/auth/callback?code=abc123&state=xyz",
            Some("xyz"),
        )
        .unwrap();
        assert_eq!(code, "abc123");
    }

    #[test]
    fn parse_redirect_accepts_raw_code() {
        let code = parse_code_from_redirect("raw-code", None).unwrap();
        assert_eq!(code, "raw-code");
    }

    #[test]
    fn parse_redirect_rejects_state_mismatch() {
        let err = parse_code_from_redirect("/auth/callback?code=x&state=a", Some("b")).unwrap_err();
        assert!(err.to_string().contains("state mismatch"));
    }

    #[test]
    fn parse_redirect_rejects_error_without_code() {
        let err = parse_code_from_redirect(
            "/auth/callback?error=access_denied&error_description=user+cancelled",
            Some("xyz"),
        )
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("OpenAI OAuth error: access_denied")
        );
    }

    #[test]
    fn extract_account_id_from_jwt_payload() {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("{}");
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode("{\"account_id\":\"acct_123\"}");
        let token = format!("{header}.{payload}.sig");

        let account = extract_account_id_from_jwt(&token);
        assert_eq!(account.as_deref(), Some("acct_123"));
    }

    #[test]
    fn chatgpt_account_claim_precedes_user_subject_and_debug_omits_secrets() {
        let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode("{}");
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            r#"{"sub":"fixture-user","https://api.openai.com/auth":{"chatgpt_account_id":"fixture-account"}}"#,
        );
        assert_eq!(
            extract_account_id_from_jwt(&format!("{header}.{payload}.sig")).as_deref(),
            Some("fixture-account")
        );
        let subject =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"sub":"fixture-user"}"#);
        assert!(extract_account_id_from_jwt(&format!("{header}.{subject}.sig")).is_none());
        let device = DeviceCodeStart {
            device_code: "fixture-private-device".into(),
            user_code: "ABCD-EFGH".into(),
            verification_uri: "https://auth.openai.com/codex/device".into(),
            verification_uri_complete: None,
            expires_in: 60,
            interval: 5,
            message: None,
        };
        let debug = format!("{device:?}");
        assert!(!debug.contains("fixture-private-device"));
        assert!(!debug.contains("ABCD-EFGH"));
        let tokens = TokenSet {
            access_token: "fixture-private-access".into(),
            refresh_token: Some("fixture-private-refresh".into()),
            id_token: None,
            expires_at: None,
            token_type: None,
            scope: None,
        };
        let debug = format!("{tokens:?}");
        assert!(!debug.contains("fixture-private-access"));
        assert!(!debug.contains("fixture-private-refresh"));
    }

    #[tokio::test]
    async fn token_error_never_echoes_response_payload() {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .respond_with(
                wiremock::ResponseTemplate::new(400)
                    .set_body_string("fixture-private-access fixture-private-refresh"),
            )
            .mount(&server)
            .await;
        let response = Client::new().get(server.uri()).send().await.unwrap();
        let error = parse_token_response(response)
            .await
            .unwrap_err()
            .to_string();
        assert!(!error.contains("fixture-private-access"));
        assert!(!error.contains("fixture-private-refresh"));
        assert!(error.contains("400"));
    }

    #[tokio::test]
    async fn device_protocol_requests_user_code_polls_and_exchanges_authorization() {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        use wiremock::{
            Mock, MockServer, ResponseTemplate,
            matchers::{body_json, body_string_contains, method, path},
        };
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/usercode"))
            .and(body_json(
                serde_json::json!({"client_id": OPENAI_OAUTH_CLIENT_ID}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "device_auth_id":"fixture-private-device", "user_code":"ABCD-EFGH", "interval":"1"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let polls = Arc::new(AtomicUsize::new(0));
        let calls = Arc::clone(&polls);
        Mock::given(method("POST")).and(path("/device-token"))
            .and(body_json(serde_json::json!({"device_auth_id":"fixture-private-device", "user_code":"ABCD-EFGH"})))
            .respond_with(move |_: &wiremock::Request| {
                if calls.fetch_add(1, Ordering::SeqCst) == 0 { ResponseTemplate::new(403) }
                else { ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "authorization_code":"fixture-code", "code_verifier":"fixture-verifier", "code_challenge":"fixture-challenge"
                })) }
            }).expect(2).mount(&server).await;
        Mock::given(method("POST")).and(path("/oauth-token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .and(body_string_contains("code_verifier=fixture-verifier"))
            .and(body_string_contains("redirect_uri=https%3A%2F%2Fauth.openai.com%2Fdeviceauth%2Fcallback"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token":"fixture-access", "refresh_token":"fixture-refresh", "expires_in":3600
            }))).expect(1).mount(&server).await;
        let client = Client::new();
        let device = start_device_code_flow_at(&client, &format!("{}/usercode", server.uri()))
            .await
            .unwrap();
        assert_eq!(device.user_code, "ABCD-EFGH");
        assert_eq!(device.interval, 1);
        assert_eq!(
            device.verification_uri,
            "https://auth.openai.com/codex/device"
        );
        let tokens = poll_device_code_tokens_at(
            &client,
            &device,
            &format!("{}/device-token", server.uri()),
            &format!("{}/oauth-token", server.uri()),
        )
        .await
        .unwrap();
        assert_eq!(tokens.access_token, "fixture-access");
        assert_eq!(tokens.refresh_token.as_deref(), Some("fixture-refresh"));
        assert_eq!(polls.load(Ordering::SeqCst), 2);
    }
}
