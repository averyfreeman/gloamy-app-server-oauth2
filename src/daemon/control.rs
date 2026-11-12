use crate::security::pairing::PairingGuard;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::watch;
use tokio::task::JoinHandle;

const CONTROL_DIRECTORY_MODE: u32 = 0o700;
const CONTROL_SOCKET_MODE: u32 = 0o600;
const MAX_CONTROL_REQUEST_BYTES: usize = 4096;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ControlRequest {
    Status,
    RequestPairingCode,
    Shutdown,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ControlResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desktop_managed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ControlResponse {
    fn status(ready: bool, api_base_url: Option<String>, desktop_managed: bool) -> Self {
        Self {
            ok: true,
            ready: Some(ready),
            api_base_url,
            desktop_managed: Some(desktop_managed),
            code: None,
            error: None,
        }
    }

    fn pairing_code(code: String) -> Self {
        Self {
            ok: true,
            ready: None,
            api_base_url: None,
            desktop_managed: None,
            code: Some(code),
            error: None,
        }
    }

    fn acknowledged() -> Self {
        Self {
            ok: true,
            ready: None,
            api_base_url: None,
            desktop_managed: None,
            code: None,
            error: None,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            ready: None,
            api_base_url: None,
            desktop_managed: None,
            code: None,
            error: Some(message.into()),
        }
    }
}

pub struct ControlServer {
    pub socket_path: PathBuf,
    task: JoinHandle<()>,
}

impl ControlServer {
    pub async fn start(
        pairing: Arc<PairingGuard>,
        desktop_managed: bool,
        gateway_addr: watch::Receiver<Option<SocketAddr>>,
        shutdown: watch::Sender<bool>,
    ) -> Result<Self> {
        Self::start_in(
            &std::env::temp_dir(),
            pairing,
            desktop_managed,
            gateway_addr,
            shutdown,
        )
        .await
    }

    async fn start_in(
        temp_root: &Path,
        pairing: Arc<PairingGuard>,
        desktop_managed: bool,
        gateway_addr: watch::Receiver<Option<SocketAddr>>,
        shutdown: watch::Sender<bool>,
    ) -> Result<Self> {
        let directory = ensure_control_directory(temp_root)?;
        let socket_path = directory.join("control.sock");
        let listener = bind_control_socket(&socket_path).await?;
        let cleanup_path = socket_path.clone();
        let task = tokio::spawn(async move {
            let _cleanup = SocketCleanup(cleanup_path);
            serve_control_requests(listener, pairing, desktop_managed, gateway_addr, shutdown)
                .await;
        });

        Ok(Self { socket_path, task })
    }

    pub async fn stop(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
        remove_owned_socket(&self.socket_path);
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        self.task.abort();
        remove_owned_socket(&self.socket_path);
    }
}

struct SocketCleanup(PathBuf);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        remove_owned_socket(&self.0);
    }
}

pub fn ensure_control_directory(temp_root: &Path) -> Result<PathBuf> {
    let uid = effective_uid();
    let directory = temp_root.join(format!("gloamy-{uid}"));

    match fs::symlink_metadata(&directory) {
        Ok(metadata) => {
            if !metadata.file_type().is_dir() || metadata.uid() != uid {
                bail!("daemon control directory is not owned by the current user");
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&directory).context("create daemon control directory")?;
        }
        Err(error) => return Err(error).context("inspect daemon control directory"),
    }

    fs::set_permissions(
        &directory,
        fs::Permissions::from_mode(CONTROL_DIRECTORY_MODE),
    )
    .context("restrict daemon control directory permissions")?;

    let metadata = fs::symlink_metadata(&directory).context("verify daemon control directory")?;
    if !metadata.file_type().is_dir()
        || metadata.uid() != uid
        || metadata.permissions().mode() & 0o777 != CONTROL_DIRECTORY_MODE
    {
        bail!("daemon control directory failed ownership or permission checks");
    }

    Ok(directory)
}

async fn bind_control_socket(path: &Path) -> Result<UnixListener> {
    match UnixListener::bind(path) {
        Ok(listener) => {
            fs::set_permissions(path, fs::Permissions::from_mode(CONTROL_SOCKET_MODE))
                .context("restrict daemon control socket permissions")?;
            verify_owned_socket(path)?;
            Ok(listener)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AddrInUse => {
            if !remove_stale_socket(path).await? {
                bail!("another Gloamy daemon already owns the local control socket");
            }
            let listener =
                UnixListener::bind(path).context("rebind stale daemon control socket")?;
            fs::set_permissions(path, fs::Permissions::from_mode(CONTROL_SOCKET_MODE))
                .context("restrict daemon control socket permissions")?;
            verify_owned_socket(path)?;
            Ok(listener)
        }
        Err(error) => Err(error).context("bind daemon control socket"),
    }
}

async fn remove_stale_socket(path: &Path) -> Result<bool> {
    let metadata = fs::symlink_metadata(path).context("inspect existing daemon control socket")?;
    if !metadata.file_type().is_socket() || metadata.uid() != effective_uid() {
        bail!("existing daemon control path is not a current-user socket");
    }

    match UnixStream::connect(path).await {
        Ok(_) => Ok(false),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound
            ) =>
        {
            verify_owned_socket(path)?;
            fs::remove_file(path).context("remove stale daemon control socket")?;
            Ok(true)
        }
        Err(error) => Err(error).context("check existing daemon control socket"),
    }
}

fn verify_owned_socket(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("inspect daemon control socket")?;
    if !metadata.file_type().is_socket()
        || metadata.uid() != effective_uid()
        || metadata.permissions().mode() & 0o777 != CONTROL_SOCKET_MODE
    {
        bail!("daemon control socket failed ownership or permission checks");
    }
    Ok(())
}

fn remove_owned_socket(path: &Path) {
    if verify_owned_socket(path).is_ok() {
        let _ = fs::remove_file(path);
    }
}

fn effective_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and returns the current process UID.
    unsafe { libc::geteuid() }
}

async fn serve_control_requests(
    listener: UnixListener,
    pairing: Arc<PairingGuard>,
    desktop_managed: bool,
    gateway_addr: watch::Receiver<Option<SocketAddr>>,
    shutdown: watch::Sender<bool>,
) {
    let mut shutdown_rx = shutdown.subscribe();
    loop {
        tokio::select! {
            changed = shutdown_rx.changed() => {
                if changed.is_err() || *shutdown_rx.borrow() {
                    break;
                }
            }
            accepted = listener.accept() => {
                let Ok((stream, _)) = accepted else {
                    break;
                };
                let response = handle_request(
                    stream,
                    &pairing,
                    desktop_managed,
                    &gateway_addr,
                    &shutdown,
                ).await;
                if let Err(error) = response {
                    tracing::debug!("daemon control request failed: {error}");
                }
            }
        }
    }
}

async fn handle_request(
    mut stream: UnixStream,
    pairing: &PairingGuard,
    desktop_managed: bool,
    gateway_addr: &watch::Receiver<Option<SocketAddr>>,
    shutdown: &watch::Sender<bool>,
) -> Result<()> {
    let Some(request) = read_control_request(&mut stream).await? else {
        write_response(
            &mut stream,
            ControlResponse::error("invalid control request"),
        )
        .await?;
        return Ok(());
    };

    let (response, request_shutdown) = match request {
        ControlRequest::Status => {
            let address = *gateway_addr.borrow();
            (
                ControlResponse::status(
                    address.is_some(),
                    address.map(api_base_url),
                    desktop_managed,
                ),
                false,
            )
        }
        ControlRequest::RequestPairingCode => match pairing.request_pairing_code() {
            Some(code) => (ControlResponse::pairing_code(code), false),
            None => (
                ControlResponse::error("pairing is disabled for this daemon"),
                false,
            ),
        },
        ControlRequest::Shutdown => (ControlResponse::acknowledged(), true),
    };

    write_response(&mut stream, response).await?;
    if request_shutdown {
        shutdown.send_replace(true);
    }
    Ok(())
}

async fn read_control_request(stream: &mut UnixStream) -> Result<Option<ControlRequest>> {
    let mut line = Vec::with_capacity(128);
    let mut byte = [0_u8; 1];
    loop {
        let read = stream.read(&mut byte).await?;
        if read == 0 {
            return Ok(None);
        }
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
        if line.len() > MAX_CONTROL_REQUEST_BYTES {
            return Ok(None);
        }
    }
    Ok(serde_json::from_slice(&line).ok())
}

async fn write_response(stream: &mut UnixStream, response: ControlResponse) -> Result<()> {
    let mut bytes = serde_json::to_vec(&response).context("serialize daemon control response")?;
    bytes.push(b'\n');
    stream.write_all(&bytes).await?;
    stream.shutdown().await?;
    Ok(())
}

fn api_base_url(address: SocketAddr) -> String {
    let ip = match address.ip() {
        std::net::IpAddr::V4(ip) if ip.is_unspecified() => {
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        }
        std::net::IpAddr::V6(ip) if ip.is_unspecified() => {
            std::net::IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
        }
        ip => ip,
    };
    let host = match ip {
        std::net::IpAddr::V4(ip) => ip.to_string(),
        std::net::IpAddr::V6(ip) => format!("[{ip}]"),
    };
    format!("http://{host}:{}", address.port())
}

#[cfg(test)]
async fn request(path: &Path, request: ControlRequest) -> Result<ControlResponse> {
    let mut stream = UnixStream::connect(path).await?;
    let mut bytes = serde_json::to_vec(&request)?;
    bytes.push(b'\n');
    stream.write_all(&bytes).await?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await?;
    Ok(serde_json::from_slice(&response)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::pairing::PairingGuard;
    use std::net::SocketAddr;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::watch;

    #[tokio::test]
    async fn control_server_reports_gateway_readiness_and_management() {
        let temp = tempdir().unwrap();
        let (gateway_tx, gateway_rx) = watch::channel(None::<SocketAddr>);
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);
        let pairing = Arc::new(PairingGuard::new(true, &[]));
        let server = ControlServer::start_in(temp.path(), pairing, true, gateway_rx, shutdown_tx)
            .await
            .unwrap();

        let status = request(&server.socket_path, ControlRequest::Status)
            .await
            .unwrap();
        assert_eq!(status.ready, Some(false));
        assert_eq!(status.desktop_managed, Some(true));
        assert_eq!(status.api_base_url, None);

        gateway_tx
            .send(Some("0.0.0.0:42617".parse().unwrap()))
            .unwrap();
        let status = request(&server.socket_path, ControlRequest::Status)
            .await
            .unwrap();
        assert_eq!(status.ready, Some(true));
        assert_eq!(
            status.api_base_url.as_deref(),
            Some("http://127.0.0.1:42617")
        );

        let _ = request(&server.socket_path, ControlRequest::Shutdown)
            .await
            .unwrap();
        shutdown_rx.changed().await.unwrap();
        assert!(*shutdown_rx.borrow());
    }

    #[tokio::test]
    async fn control_server_issues_code_without_replacing_paired_tokens() {
        let temp = tempdir().unwrap();
        let (_, gateway_rx) = watch::channel(Some("127.0.0.1:42617".parse().unwrap()));
        let (shutdown_tx, _) = watch::channel(false);
        let pairing = Arc::new(PairingGuard::new(true, &["zc_existing".into()]));
        let server =
            ControlServer::start_in(temp.path(), pairing.clone(), false, gateway_rx, shutdown_tx)
                .await
                .unwrap();

        let response = request(&server.socket_path, ControlRequest::RequestPairingCode)
            .await
            .unwrap();
        let code = response.code.expect("recovery code");
        assert_eq!(pairing.pairing_code().as_deref(), Some(code.as_str()));
        assert_eq!(pairing.tokens().len(), 1);
        assert!(pairing.is_authenticated("zc_existing"));
    }

    #[tokio::test]
    async fn control_directory_is_private() {
        let temp = tempdir().unwrap();

        let directory = ensure_control_directory(temp.path()).unwrap();

        assert_eq!(
            std::fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }

    #[tokio::test]
    async fn control_directory_refuses_to_follow_a_non_directory() {
        let temp = tempdir().unwrap();
        let directory = temp
            .path()
            .join(format!("gloamy-{}", unsafe { libc::geteuid() }));
        std::fs::write(&directory, "preserve").unwrap();

        assert!(ensure_control_directory(temp.path()).is_err());
        assert_eq!(std::fs::read_to_string(directory).unwrap(), "preserve");
    }

    #[tokio::test]
    async fn stale_owned_socket_is_removed_only_after_connect_refuses() {
        let temp = tempdir().unwrap();
        let directory = ensure_control_directory(temp.path()).unwrap();
        let socket_path = directory.join("stale.sock");
        let listener = UnixListener::bind(&socket_path).unwrap();
        drop(listener);
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600)).unwrap();

        assert!(socket_path.exists());
        assert!(remove_stale_socket(&socket_path).await.unwrap());
        assert!(!socket_path.exists());
    }

    #[tokio::test]
    async fn control_request_uses_a_tagged_operation_name() {
        let json = serde_json::to_value(ControlRequest::RequestPairingCode).unwrap();

        assert_eq!(json, serde_json::json!({ "op": "request_pairing_code" }));
    }
}
