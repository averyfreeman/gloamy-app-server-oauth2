//! Server-side lifecycle for Codex ChatGPT-account device-code authorization.
//!
//! Device codes and returned tokens are intentionally kept in task-local state.
//! Only [`FlowView`] is returned to callers.

use crate::auth::openai_oauth::{
    DeviceCodeStart, extract_account_id_from_jwt, poll_device_code_tokens, start_device_code_flow,
};
use crate::auth::profiles::TokenSet;
use crate::auth::{AuthService, state_dir_from_config};
use crate::config::Config;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

const MAX_FLOWS: usize = 8;
const MAX_RETAINED_FLOWS: usize = 32;
const MAX_FLOW_TTL: Duration = Duration::from_secs(15 * 60);
const START_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_TERMINAL_RETENTION: Duration = Duration::from_secs(5 * 60);
const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(1);

const START_ERROR: &str = "Could not start Codex authorization. Try again.";
const INVALID_DEVICE_ERROR: &str = "OpenAI returned invalid verification information.";
const POLL_ERROR: &str = "Authorization could not be completed. Try again.";
const SAVE_ERROR: &str = "Authorization succeeded, but credentials could not be saved.";

/// Device-code backend. Implementations must not log device codes or tokens.
#[async_trait]
pub trait DeviceOAuthBackend: Send + Sync {
    async fn start(&self) -> Result<DeviceCodeStart>;
    async fn poll(&self, device: &DeviceCodeStart) -> Result<TokenSet>;
}

struct OpenAiDeviceOAuthBackend {
    client: reqwest::Client,
}

#[async_trait]
impl DeviceOAuthBackend for OpenAiDeviceOAuthBackend {
    async fn start(&self) -> Result<DeviceCodeStart> {
        start_device_code_flow(&self.client).await
    }

    async fn poll(&self, device: &DeviceCodeStart) -> Result<TokenSet> {
        poll_device_code_tokens(&self.client, device).await
    }
}

/// Public, non-secret progress returned by the authenticated setup API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowView {
    pub id: String,
    pub status: FlowStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowStatus {
    Pending,
    Success,
    Error,
    Expired,
    Cancelled,
}

struct FlowState {
    view: FlowView,
    deadline: Instant,
    terminal_at: Option<Instant>,
    commit_started: bool,
}

struct Flow {
    state: Mutex<FlowState>,
    commit_lock: AsyncMutex<()>,
    cancellation: CancellationToken,
}

struct FlowEntry {
    owner_hash: String,
    flow: Arc<Flow>,
}

#[derive(Clone, Copy)]
struct Lifecycle {
    max_ttl: Duration,
    start_timeout: Duration,
    terminal_retention: Duration,
    cleanup_interval: Duration,
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self {
            max_ttl: MAX_FLOW_TTL,
            start_timeout: START_TIMEOUT,
            terminal_retention: DEFAULT_TERMINAL_RETENTION,
            cleanup_interval: DEFAULT_CLEANUP_INTERVAL,
        }
    }
}

/// Manages bounded, per-owner Codex OAuth flows and encrypted profile writes.
pub struct CodexOAuthSetup {
    backend: Arc<dyn DeviceOAuthBackend>,
    flows: Mutex<HashMap<String, FlowEntry>>,
    start_gate: AsyncMutex<()>,
    cleanup_started: AtomicBool,
    lifecycle: Lifecycle,
}

impl CodexOAuthSetup {
    /// Build a setup service backed by OpenAI's existing device-code OAuth flow.
    pub fn new() -> Self {
        Self::with_backend(Arc::new(OpenAiDeviceOAuthBackend {
            client: crate::config::build_runtime_proxy_client("provider.auth"),
        }))
    }

    /// Build a setup service with an injected backend, useful for deterministic callers and tests.
    pub fn with_backend(backend: Arc<dyn DeviceOAuthBackend>) -> Self {
        Self::with_backend_and_lifecycle(backend, Lifecycle::default())
    }

    fn with_backend_and_lifecycle(
        backend: Arc<dyn DeviceOAuthBackend>,
        lifecycle: Lifecycle,
    ) -> Self {
        Self {
            backend,
            flows: Mutex::new(HashMap::new()),
            start_gate: AsyncMutex::new(()),
            cleanup_started: AtomicBool::new(false),
            lifecycle,
        }
    }

    /// Start a device-code flow for an authenticated owner.
    ///
    /// Provider failures become safe error progress views. Capacity errors use a fixed
    /// local message and never include provider response data.
    pub async fn start(self: &Arc<Self>, owner_hash: &str, config: &Config) -> Result<FlowView> {
        if owner_hash.is_empty() {
            return Err(anyhow!("Authenticated owner is required"));
        }

        // Serialize starts so a slow provider endpoint cannot create an unbounded
        // number of concurrent outbound start requests.
        let _start_guard = self
            .start_gate
            .try_lock()
            .map_err(|_| anyhow!("An authorization start is already in progress"))?;
        self.cleanup_expired();

        let id = uuid::Uuid::new_v4().to_string();
        let flow = Arc::new(Flow {
            state: Mutex::new(FlowState {
                view: FlowView {
                    id: id.clone(),
                    status: FlowStatus::Pending,
                    verification_uri: None,
                    user_code: None,
                    expires_at: None,
                    error: None,
                },
                deadline: Instant::now() + self.lifecycle.start_timeout,
                terminal_at: None,
                commit_started: false,
            }),
            commit_lock: AsyncMutex::new(()),
            cancellation: CancellationToken::new(),
        });

        {
            let mut flows = lock(&self.flows);
            let pending = flows
                .values()
                .filter(|entry| lock(&entry.flow.state).view.status == FlowStatus::Pending)
                .count();
            if pending >= MAX_FLOWS {
                return Err(anyhow!("Too many Codex authorization flows are active"));
            }
            if flows.values().any(|entry| {
                entry.owner_hash == owner_hash
                    && lock(&entry.flow.state).view.status == FlowStatus::Pending
            }) {
                return Err(anyhow!(
                    "Cancel this client's pending authorization before starting another"
                ));
            }
            if flows.len() >= MAX_RETAINED_FLOWS {
                let oldest = flows
                    .iter()
                    .filter_map(|(id, entry)| {
                        lock(&entry.flow.state)
                            .terminal_at
                            .map(|time| (id.clone(), time))
                    })
                    .min_by_key(|(_, time)| *time)
                    .map(|(id, _)| id);
                if let Some(oldest) = oldest {
                    flows.remove(&oldest);
                }
            }
            flows.insert(
                id.clone(),
                FlowEntry {
                    owner_hash: owner_hash.to_string(),
                    flow: Arc::clone(&flow),
                },
            );
        }

        self.ensure_cleanup_task();
        let mut reservation = FlowReservation {
            service: Arc::downgrade(self),
            id: id.clone(),
            armed: true,
        };

        let start_result =
            tokio::time::timeout(self.lifecycle.start_timeout, self.backend.start()).await;
        let device = match start_result {
            Ok(Ok(device)) => device,
            Ok(Err(_)) | Err(_) => {
                set_terminal(&flow, FlowStatus::Error, Some(START_ERROR));
                reservation.armed = false;
                return Ok(view(&flow));
            }
        };

        let Some(verification_uri) = safe_verification_uri(&device.verification_uri) else {
            set_terminal(&flow, FlowStatus::Error, Some(INVALID_DEVICE_ERROR));
            reservation.armed = false;
            return Ok(view(&flow));
        };
        if !safe_user_code(&device.user_code) {
            set_terminal(&flow, FlowStatus::Error, Some(INVALID_DEVICE_ERROR));
            reservation.armed = false;
            return Ok(view(&flow));
        }

        let ttl = self
            .lifecycle
            .max_ttl
            .min(Duration::from_secs(device.expires_in));
        let now = Utc::now();
        {
            let mut state = lock(&flow.state);
            if state.view.status != FlowStatus::Pending {
                reservation.armed = false;
                return Ok(state.view.clone());
            }
            if ttl.is_zero() {
                state.deadline = Instant::now();
                state.view.expires_at = Some(now);
                set_terminal_locked(&mut state, FlowStatus::Expired, None);
                flow.cancellation.cancel();
                reservation.armed = false;
                return Ok(state.view.clone());
            }

            state.deadline = Instant::now() + ttl;
            state.view.verification_uri = Some(verification_uri);
            state.view.user_code = Some(device.user_code.clone());
            state.view.expires_at = Some(now + ChronoDuration::from_std(ttl).unwrap_or_default());
        }

        let state_dir = state_dir_from_config(config);
        let auth_service = AuthService::new(&state_dir, true);
        let backend = Arc::clone(&self.backend);
        let poll_flow = Arc::clone(&flow);
        tokio::spawn(async move {
            poll_and_persist(backend, device, auth_service, poll_flow).await;
        });

        reservation.armed = false;
        Ok(view(&flow))
    }

    /// Inspect a flow owned by the authenticated caller.
    pub async fn inspect(&self, id: &str, owner_hash: &str) -> Option<FlowView> {
        let flow = self.find_owned(id, owner_hash)?;
        expire_if_needed(&flow);
        Some(view(&flow))
    }

    /// Cancel a flow owned by the authenticated caller.
    pub async fn cancel(&self, id: &str, owner_hash: &str) -> Option<FlowView> {
        let flow = self.find_owned(id, owner_hash)?;
        {
            let state = lock(&flow.state);
            if state.view.status == FlowStatus::Pending && !state.commit_started {
                // Signal before waiting on the persistence gate. A poll result that
                // has just arrived must observe this cancellation before it can commit.
                flow.cancellation.cancel();
            }
        }
        let _commit_guard = flow.commit_lock.lock().await;
        {
            let mut state = lock(&flow.state);
            if state.view.status == FlowStatus::Pending {
                if !state.commit_started && Instant::now() >= state.deadline {
                    set_terminal_locked(&mut state, FlowStatus::Expired, None);
                } else {
                    set_terminal_locked(&mut state, FlowStatus::Cancelled, None);
                }
            }
        }
        Some(view(&flow))
    }

    fn find_owned(&self, id: &str, owner_hash: &str) -> Option<Arc<Flow>> {
        let flows = lock(&self.flows);
        flows
            .get(id)
            .filter(|entry| entry.owner_hash == owner_hash)
            .map(|entry| Arc::clone(&entry.flow))
    }

    fn ensure_cleanup_task(self: &Arc<Self>) {
        if self
            .cleanup_started
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }

        let service = Arc::downgrade(self);
        let interval = self
            .lifecycle
            .cleanup_interval
            .max(Duration::from_millis(1));
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                let Some(service) = service.upgrade() else {
                    break;
                };
                service.cleanup_expired();
            }
        });
    }

    fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut flows = lock(&self.flows);
        let mut remove_ids = Vec::new();

        for (id, entry) in flows.iter() {
            let mut state = lock(&entry.flow.state);
            if state.view.status == FlowStatus::Pending
                && !state.commit_started
                && now >= state.deadline
            {
                set_terminal_locked(&mut state, FlowStatus::Expired, None);
                entry.flow.cancellation.cancel();
            }

            if state.terminal_at.is_some_and(|terminal_at| {
                now.duration_since(terminal_at) >= self.lifecycle.terminal_retention
            }) {
                remove_ids.push(id.clone());
            }
        }

        for id in remove_ids {
            flows.remove(&id);
        }
    }
}

impl Default for CodexOAuthSetup {
    fn default() -> Self {
        Self::new()
    }
}

struct FlowReservation {
    service: Weak<CodexOAuthSetup>,
    id: String,
    armed: bool,
}

impl Drop for FlowReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if let Some(service) = self.service.upgrade() {
            lock(&service.flows).remove(&self.id);
        }
    }
}

async fn poll_and_persist(
    backend: Arc<dyn DeviceOAuthBackend>,
    device: DeviceCodeStart,
    auth_service: AuthService,
    flow: Arc<Flow>,
) {
    let deadline = lock(&flow.state).deadline;
    let result = tokio::select! {
        _ = flow.cancellation.cancelled() => return,
        _ = tokio::time::sleep_until(deadline) => {
            Err(anyhow!("device-code flow expired"))
        }
        result = backend.poll(&device) => result,
    };

    let _commit_guard = flow.commit_lock.lock().await;
    let token_set = {
        let mut state = lock(&flow.state);
        if state.view.status != FlowStatus::Pending {
            return;
        }
        if flow.cancellation.is_cancelled() {
            let status = if Instant::now() >= state.deadline {
                FlowStatus::Expired
            } else {
                FlowStatus::Cancelled
            };
            set_terminal_locked(&mut state, status, None);
            return;
        }
        if Instant::now() >= state.deadline {
            set_terminal_locked(&mut state, FlowStatus::Expired, None);
            flow.cancellation.cancel();
            return;
        }

        match result {
            Ok(token_set) if !token_set.access_token.trim().is_empty() => {
                state.commit_started = true;
                Some(token_set)
            }
            Ok(_) | Err(_) => {
                set_terminal_locked(&mut state, FlowStatus::Error, Some(POLL_ERROR));
                None
            }
        }
    };

    if let Some(token_set) = token_set {
        match persist_tokens(&auth_service, token_set).await {
            Ok(()) => set_terminal(&flow, FlowStatus::Success, None),
            Err(()) => set_terminal(&flow, FlowStatus::Error, Some(SAVE_ERROR)),
        }
    }
}

async fn persist_tokens(
    auth_service: &AuthService,
    token_set: TokenSet,
) -> std::result::Result<(), ()> {
    let profiles = auth_service.load_profiles().await.map_err(|_| ())?;
    let profile_name =
        loop {
            let candidate = format!("integration-{}", uuid::Uuid::new_v4());
            if !profiles.profiles.values().any(|profile| {
                profile.provider == "openai-codex" && profile.profile_name == candidate
            }) {
                break candidate;
            }
        };
    let account_id = extract_account_id_from_jwt(&token_set.access_token).or_else(|| {
        token_set
            .id_token
            .as_deref()
            .and_then(extract_account_id_from_jwt)
    });
    auth_service
        .store_openai_tokens(&profile_name, token_set, account_id, true)
        .await
        .map_err(|_| ())?;
    Ok(())
}

fn set_terminal(flow: &Flow, status: FlowStatus, error: Option<&'static str>) {
    let mut state = lock(&flow.state);
    set_terminal_locked(&mut state, status, error);
}

fn set_terminal_locked(state: &mut FlowState, status: FlowStatus, error: Option<&'static str>) {
    state.view.status = status;
    state.view.verification_uri = None;
    state.view.user_code = None;
    state.view.error = error.map(ToString::to_string);
    state.terminal_at = Some(Instant::now());
}

fn expire_if_needed(flow: &Flow) {
    let mut state = lock(&flow.state);
    if state.view.status == FlowStatus::Pending
        && !state.commit_started
        && Instant::now() >= state.deadline
    {
        set_terminal_locked(&mut state, FlowStatus::Expired, None);
        flow.cancellation.cancel();
    }
}

fn view(flow: &Flow) -> FlowView {
    lock(&flow.state).view.clone()
}

fn safe_verification_uri(candidate: &str) -> Option<String> {
    let parsed = reqwest::Url::parse(candidate).ok()?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("auth.openai.com")
        || parsed.port_or_known_default() != Some(443)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.path() != "/codex/device"
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return None;
    }
    Some(parsed.to_string())
}

fn safe_user_code(candidate: &str) -> bool {
    !candidate.is_empty()
        && candidate.len() <= 64
        && candidate
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::profiles::AuthProfileKind;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use tempfile::TempDir;

    #[derive(Clone)]
    enum PollMode {
        Success(TokenSet),
        Error(&'static str),
        Pending,
    }

    struct MockBackend {
        mode: Mutex<PollMode>,
        starts: AtomicUsize,
    }

    impl MockBackend {
        fn new(mode: PollMode) -> Self {
            Self {
                mode: Mutex::new(mode),
                starts: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl DeviceOAuthBackend for MockBackend {
        async fn start(&self) -> Result<DeviceCodeStart> {
            self.starts.fetch_add(1, AtomicOrdering::Relaxed);
            Ok(DeviceCodeStart {
                device_code: "server-only-device-code".to_string(),
                user_code: "ABCD-EFGH".to_string(),
                verification_uri: "https://auth.openai.com/codex/device".to_string(),
                verification_uri_complete: Some(
                    "https://auth.openai.com/codex/device?user_code=ABCD-EFGH".to_string(),
                ),
                expires_in: 60,
                interval: 1,
                message: None,
            })
        }

        async fn poll(&self, _device: &DeviceCodeStart) -> Result<TokenSet> {
            let mode = lock(&self.mode).clone();
            match mode {
                PollMode::Success(tokens) => Ok(tokens),
                PollMode::Error(message) => Err(anyhow!(message)),
                PollMode::Pending => std::future::pending().await,
            }
        }
    }

    fn test_config(temp_dir: &TempDir) -> Config {
        let mut config = Config::default();
        config.config_path = temp_dir.path().join("config.toml");
        config
    }

    fn tokens(access: &str, refresh: &str) -> TokenSet {
        TokenSet {
            access_token: access.to_string(),
            refresh_token: Some(refresh.to_string()),
            id_token: None,
            expires_at: None,
            token_type: Some("Bearer".to_string()),
            scope: Some("openid offline_access".to_string()),
        }
    }

    async fn wait_for_status(
        service: &CodexOAuthSetup,
        flow_id: &str,
        owner: &str,
        expected: FlowStatus,
    ) -> FlowView {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(view) = service.inspect(flow_id, owner).await
                    && view.status == expected
                {
                    return view;
                }
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .expect("flow should reach expected status")
    }

    #[tokio::test]
    async fn device_flow_transitions_to_success_and_persists_encrypted_credentials() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        let config = test_config(&temp_dir);
        let backend = Arc::new(MockBackend::new(PollMode::Success(tokens(
            "codex-access-secret",
            "codex-refresh-secret",
        ))));
        let service = Arc::new(CodexOAuthSetup::with_backend(backend));

        let started = service.start("owner-a", &config).await.expect("flow start");
        assert_eq!(started.status, FlowStatus::Pending);
        assert_eq!(
            started.verification_uri.as_deref(),
            Some("https://auth.openai.com/codex/device")
        );
        assert_eq!(started.user_code.as_deref(), Some("ABCD-EFGH"));
        let completed =
            wait_for_status(&service, &started.id, "owner-a", FlowStatus::Success).await;
        assert!(completed.verification_uri.is_none());
        assert!(completed.user_code.is_none());

        let raw = tokio::fs::read_to_string(temp_dir.path().join("auth-profiles.json"))
            .await
            .expect("persisted auth store");
        assert!(!raw.contains("codex-access-secret"));
        assert!(!raw.contains("codex-refresh-secret"));
        assert!(raw.contains("enc2:"));

        let loaded = AuthService::new(temp_dir.path(), false)
            .load_profiles()
            .await
            .expect("decrypt encrypted profiles regardless of write setting");
        let active_id = loaded
            .active_profiles
            .get("openai-codex")
            .expect("Codex active profile");
        let profile = loaded
            .profiles
            .get(active_id)
            .expect("active profile exists");
        assert_eq!(profile.kind, AuthProfileKind::OAuth);
        assert!(profile.profile_name.starts_with("integration-"));
        assert_eq!(
            profile
                .token_set
                .as_ref()
                .map(|tokens| tokens.access_token.as_str()),
            Some("codex-access-secret")
        );

        // A later runtime save created with `encrypt = false` must not downgrade
        // OAuth credentials that were deliberately encrypted during setup.
        AuthService::new(temp_dir.path(), false)
            .store_provider_token(
                "anthropic",
                "later-profile",
                "later-profile-secret",
                HashMap::new(),
                false,
            )
            .await
            .expect("save unrelated profile with encryption disabled");
        let rewritten = tokio::fs::read_to_string(temp_dir.path().join("auth-profiles.json"))
            .await
            .expect("rewritten auth store");
        assert!(!rewritten.contains("codex-access-secret"));
        assert!(!rewritten.contains("codex-refresh-secret"));
        let reloaded = AuthService::new(temp_dir.path(), false)
            .load_profiles()
            .await
            .expect("reload after unrelated profile save");
        let active_id = reloaded
            .active_profiles
            .get("openai-codex")
            .expect("Codex active profile remains");
        assert_eq!(
            reloaded.profiles[active_id]
                .token_set
                .as_ref()
                .map(|tokens| tokens.access_token.as_str()),
            Some("codex-access-secret")
        );
    }

    #[tokio::test]
    async fn device_flow_cancellation_is_owner_scoped_and_redacts_server_secrets() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        let service = Arc::new(CodexOAuthSetup::with_backend(Arc::new(MockBackend::new(
            PollMode::Pending,
        ))));
        let started = service
            .start("owner-a", &test_config(&temp_dir))
            .await
            .expect("flow start");

        assert!(service.inspect(&started.id, "owner-b").await.is_none());
        assert!(service.cancel(&started.id, "owner-b").await.is_none());
        let serialized = serde_json::to_string(&started).expect("safe view serializes");
        assert!(!serialized.contains("server-only-device-code"));
        assert!(!serialized.contains("verification_uri_complete"));
        assert!(!serialized.contains("codex-access-secret"));

        let cancelled = service
            .cancel(&started.id, "owner-a")
            .await
            .expect("owner can cancel flow");
        assert_eq!(cancelled.status, FlowStatus::Cancelled);
        assert!(cancelled.user_code.is_none());
    }

    #[tokio::test]
    async fn device_flow_expires_and_is_removed_without_continuous_inspection() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        let service = Arc::new(CodexOAuthSetup::with_backend_and_lifecycle(
            Arc::new(MockBackend::new(PollMode::Pending)),
            Lifecycle {
                cleanup_interval: Duration::from_millis(2),
                ..Lifecycle::default()
            },
        ));
        let started = service
            .start("owner-a", &test_config(&temp_dir))
            .await
            .unwrap();
        let flow = service.find_owned(&started.id, "owner-a").unwrap();
        // Inject past deadlines instead of depending on a tiny expiry/retention
        // window while thousands of other tests contend for scheduler time.
        lock(&flow.state).deadline = Instant::now() - Duration::from_secs(1);
        tokio::time::timeout(Duration::from_secs(5), async {
            while view(&flow).status == FlowStatus::Pending {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(view(&flow).status, FlowStatus::Expired);
        assert!(view(&flow).user_code.is_none());
        lock(&flow.state).terminal_at =
            Some(Instant::now() - DEFAULT_TERMINAL_RETENTION - Duration::from_secs(1));
        tokio::time::timeout(Duration::from_secs(5), async {
            while service.find_owned(&started.id, "owner-a").is_some() {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        })
        .await
        .unwrap();
        assert!(service.inspect(&started.id, "owner-a").await.is_none());
    }

    #[tokio::test]
    async fn device_flow_provider_errors_are_safe_and_never_echo_provider_payloads() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        let backend = Arc::new(MockBackend::new(PollMode::Error(
            "provider payload included access-token-secret",
        )));
        let service = Arc::new(CodexOAuthSetup::with_backend(backend));
        let started = service
            .start("owner-a", &test_config(&temp_dir))
            .await
            .expect("flow start");
        let failed = wait_for_status(&service, &started.id, "owner-a", FlowStatus::Error).await;
        let serialized = serde_json::to_string(&failed).expect("safe view serializes");
        assert_eq!(failed.error.as_deref(), Some(POLL_ERROR));
        assert!(!serialized.contains("provider payload"));
        assert!(!serialized.contains("access-token-secret"));
    }

    #[tokio::test]
    async fn successful_codex_setup_preserves_other_profiles_and_active_namespaces() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        let config = test_config(&temp_dir);
        let existing = AuthService::new(temp_dir.path(), true);
        existing
            .store_provider_token(
                "anthropic",
                "existing",
                "anthropic-secret",
                HashMap::new(),
                true,
            )
            .await
            .expect("seed unrelated profile");
        existing
            .store_provider_token(
                "openai-codex",
                "existing",
                "existing-codex-secret",
                HashMap::new(),
                true,
            )
            .await
            .expect("seed existing Codex profile");
        let before = existing.load_profiles().await.expect("profiles before");
        let config_snapshot = (
            config.default_provider.clone(),
            config.default_model.clone(),
        );

        let backend = Arc::new(MockBackend::new(PollMode::Success(tokens(
            "codex-access-secret",
            "codex-refresh-secret",
        ))));
        let service = Arc::new(CodexOAuthSetup::with_backend(backend));
        let started = service.start("owner-a", &config).await.expect("flow start");
        wait_for_status(&service, &started.id, "owner-a", FlowStatus::Success).await;

        let after = existing.load_profiles().await.expect("profiles after");
        assert!(after.profiles.contains_key("anthropic:existing"));
        assert!(after.profiles.contains_key("openai-codex:existing"));
        assert_eq!(
            after.active_profiles.get("anthropic"),
            before.active_profiles.get("anthropic")
        );
        assert_ne!(
            after.active_profiles.get("openai-codex"),
            before.active_profiles.get("openai-codex")
        );
        assert_eq!(after.profiles.len(), before.profiles.len() + 1);
        assert_eq!(
            (
                config.default_provider.clone(),
                config.default_model.clone()
            ),
            config_snapshot
        );
    }

    #[tokio::test]
    async fn failed_encrypted_save_never_reports_success() {
        let temp_dir = TempDir::new().expect("temporary state dir");
        std::fs::create_dir(temp_dir.path().join(".secret_key")).expect("block secret key file");
        let backend = Arc::new(MockBackend::new(PollMode::Success(tokens(
            "codex-access-secret",
            "codex-refresh-secret",
        ))));
        let service = Arc::new(CodexOAuthSetup::with_backend(backend));
        let started = service
            .start("owner-a", &test_config(&temp_dir))
            .await
            .expect("flow start");
        let failed = wait_for_status(&service, &started.id, "owner-a", FlowStatus::Error).await;
        assert_eq!(failed.error.as_deref(), Some(SAVE_ERROR));
        assert!(
            !failed
                .error
                .as_deref()
                .unwrap_or_default()
                .contains("secret")
        );
        let profiles = AuthService::new(temp_dir.path(), true)
            .load_profiles()
            .await
            .expect("load profiles after failed save");
        assert!(profiles.profiles.is_empty());
    }

    #[tokio::test]
    async fn unsafe_verification_locations_are_rejected() {
        assert!(safe_verification_uri("https://auth.openai.com/codex/device").is_some());
        assert!(safe_verification_uri("http://auth.openai.com/codex/device").is_none());
        assert!(safe_verification_uri("https://attacker.example/codex/device").is_none());
        assert!(safe_verification_uri("https://auth.openai.com/other").is_none());
        assert!(!safe_user_code("<img src=x>"));
    }

    #[tokio::test]
    async fn terminal_history_does_not_block_new_authorizations() {
        let directory = TempDir::new().unwrap();
        let service = Arc::new(CodexOAuthSetup::with_backend(Arc::new(MockBackend::new(
            PollMode::Error("fixture error"),
        ))));
        for _ in 0..MAX_RETAINED_FLOWS + 3 {
            let started = service
                .start("owner-a", &test_config(&directory))
                .await
                .unwrap();
            wait_for_status(&service, &started.id, "owner-a", FlowStatus::Error).await;
        }
        assert!(lock(&service.flows).len() <= MAX_RETAINED_FLOWS);
    }

    #[tokio::test]
    async fn concurrent_starts_are_rejected_instead_of_queued() {
        struct SlowStart {
            entered: tokio::sync::Notify,
            release: tokio::sync::Notify,
        }
        #[async_trait]
        impl DeviceOAuthBackend for SlowStart {
            async fn start(&self) -> Result<DeviceCodeStart> {
                self.entered.notify_one();
                self.release.notified().await;
                MockBackend::new(PollMode::Pending).start().await
            }
            async fn poll(&self, _: &DeviceCodeStart) -> Result<TokenSet> {
                std::future::pending().await
            }
        }
        let directory = TempDir::new().unwrap();
        let config = test_config(&directory);
        let backend = Arc::new(SlowStart {
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
        });
        let service = Arc::new(CodexOAuthSetup::with_backend(backend.clone()));
        let first = Arc::clone(&service);
        let task = tokio::spawn(async move { first.start("owner-a", &config).await });
        tokio::time::timeout(Duration::from_secs(1), backend.entered.notified())
            .await
            .unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(1),
            service.start("owner-b", &test_config(&directory)),
        )
        .await
        .unwrap();
        assert!(result.is_err());
        backend.release.notify_one();
        let started = task.await.unwrap().unwrap();
        assert_eq!(
            service.cancel(&started.id, "owner-a").await.unwrap().status,
            FlowStatus::Cancelled
        );
    }
}
