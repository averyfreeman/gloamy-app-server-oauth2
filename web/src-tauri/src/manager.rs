use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const CONTROL_DIRECTORY_MODE: u32 = 0o700;
const CONTROL_SOCKET_MODE: u32 = 0o600;
const KEEP_RUNNING_DEFAULT: bool = false;
const STARTUP_TIMEOUT: Duration = Duration::from_secs(45);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "op", rename_all = "snake_case")]
enum ControlRequest {
    Status,
    RequestPairingCode,
    Shutdown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ControlResponse {
    ok: bool,
    ready: Option<bool>,
    api_base_url: Option<String>,
    desktop_managed: Option<bool>,
    code: Option<String>,
    error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct DesktopInfo {
    gateway_host: String,
    gateway_port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DesktopRuntime {
    pub ready: bool,
    pub api_base_url: Option<String>,
    pub desktop_managed: bool,
    pub error: Option<String>,
}

impl DesktopRuntime {
    fn unavailable(error: impl Into<String>) -> Self {
        Self {
            ready: false,
            api_base_url: None,
            desktop_managed: false,
            error: Some(error.into()),
        }
    }
}

pub struct DesktopState(pub std::sync::Mutex<DesktopManager>);

pub struct DesktopManager {
    binary: Option<PathBuf>,
    socket_path: PathBuf,
    preference_path: Option<PathBuf>,
    keep_running: bool,
    runtime: DesktopRuntime,
    child: Option<Child>,
    allow_exit: bool,
}

impl DesktopManager {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let binary = resolve_daemon_binary(app)?;
        let preference_path = app
            .path()
            .app_config_dir()
            .map_err(|error| format!("cannot locate desktop preferences: {error}"))?
            .join("desktop-preferences.json");
        ensure_private_preferences_directory(
            preference_path
                .parent()
                .ok_or_else(|| "desktop preferences have no parent directory".to_owned())?,
        )?;
        let keep_running = load_keep_running(&preference_path)?;
        let socket_path = control_socket_path();

        let mut manager = Self {
            binary: Some(binary),
            socket_path,
            preference_path: Some(preference_path),
            keep_running,
            runtime: DesktopRuntime::unavailable("Starting Gloamy daemon…"),
            child: None,
            allow_exit: false,
        };
        manager.start_or_attach()?;
        Ok(manager)
    }

    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            binary: None,
            socket_path: control_socket_path(),
            preference_path: None,
            keep_running: KEEP_RUNNING_DEFAULT,
            runtime: DesktopRuntime::unavailable(error),
            child: None,
            allow_exit: false,
        }
    }

    pub fn runtime(&mut self) -> DesktopRuntime {
        if let Ok(Some(response)) = self.request(ControlRequest::Status)
            && response.ok
            && response.ready == Some(true)
            && let Some(api_base_url) = response.api_base_url
        {
            if validate_loopback_url(&api_base_url).is_ok() {
                self.runtime = DesktopRuntime {
                    ready: true,
                    api_base_url: Some(api_base_url),
                    desktop_managed: response.desktop_managed.unwrap_or(false),
                    error: None,
                };
                return self.runtime.clone();
            }
            self.runtime = DesktopRuntime::unavailable(
                "The daemon is bound to a non-loopback address that the desktop app cannot reach.",
            );
        }
        self.runtime.clone()
    }

    pub fn request_pairing_code(&mut self) -> Result<String, String> {
        let response = self
            .request(ControlRequest::RequestPairingCode)?
            .ok_or_else(|| "Gloamy daemon control socket is unavailable".to_owned())?;
        if !response.ok {
            return Err(response
                .error
                .unwrap_or_else(|| "pairing code request failed".to_owned()));
        }
        response
            .code
            .ok_or_else(|| "daemon returned no pairing code".to_owned())
    }

    pub fn keep_running(&self) -> bool {
        self.keep_running
    }

    pub fn set_keep_running(&mut self, keep_running: bool) -> Result<(), String> {
        let path = self
            .preference_path
            .as_ref()
            .ok_or_else(|| "desktop preferences are unavailable".to_owned())?;
        save_keep_running(path, keep_running)?;
        self.keep_running = keep_running;
        Ok(())
    }

    pub fn shutdown_on_exit(&self) -> bool {
        should_shutdown_managed_daemon(self.runtime.desktop_managed, self.keep_running)
    }

    pub fn begin_exit(&mut self) -> Result<(), String> {
        if !self.shutdown_on_exit() {
            return Ok(());
        }
        let response = self
            .request(ControlRequest::Shutdown)?
            .ok_or_else(|| "managed daemon control socket is unavailable".to_owned())?;
        if !response.ok {
            return Err(response
                .error
                .unwrap_or_else(|| "daemon rejected shutdown request".to_owned()));
        }

        let deadline = Instant::now() + Duration::from_secs(12);
        let mut stopped = false;
        while Instant::now() < deadline {
            match self.request(ControlRequest::Status) {
                Ok(Some(status)) if status.ok => thread::sleep(Duration::from_millis(100)),
                Ok(Some(_)) => return Err("managed daemon rejected its status request".to_owned()),
                Ok(None) if !self.socket_path.exists() => {
                    stopped = true;
                    break;
                }
                Ok(None) => thread::sleep(Duration::from_millis(100)),
                Err(error) => return Err(error),
            }
        }
        if !stopped {
            return Err("managed daemon did not stop within 12 seconds".to_owned());
        }
        if let Some(child) = self.child.as_mut() {
            let child_deadline = Instant::now() + Duration::from_secs(1);
            while Instant::now() < child_deadline {
                if child
                    .try_wait()
                    .map_err(|error| format!("could not inspect stopped daemon: {error}"))?
                    .is_some()
                {
                    break;
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
        self.runtime.ready = false;
        self.allow_exit = true;
        Ok(())
    }

    pub fn exit_is_allowed(&self) -> bool {
        self.allow_exit || !self.shutdown_on_exit()
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.runtime.error = Some(error.into());
    }

    fn start_or_attach(&mut self) -> Result<(), String> {
        match self.request(ControlRequest::Status) {
            Ok(Some(response)) => match startup_action(Some(&response), None)? {
                StartupAction::AttachControlled => {
                    self.attach_control_status(response)?;
                    return Ok(());
                }
                StartupAction::WaitForExistingControl => {
                    let deadline = Instant::now() + STARTUP_TIMEOUT;
                    while Instant::now() < deadline {
                        if let Ok(Some(response)) = self.request(ControlRequest::Status) {
                            if response.ready == Some(true) {
                                self.attach_control_status(response)?;
                                return Ok(());
                            }
                        } else {
                            break;
                        }
                        thread::sleep(Duration::from_millis(150));
                    }
                    return Err(
                        "an existing Gloamy daemon did not become ready within 45 seconds"
                            .to_owned(),
                    );
                }
                _ => unreachable!("control status only permits controlled startup actions"),
            },
            Ok(None) => {}
            Err(error) => return Err(error),
        }

        let binary = self
            .binary
            .as_ref()
            .ok_or_else(|| "bundled daemon binary is unavailable".to_owned())?;
        let info = read_desktop_info(binary)?;
        let target = local_socket_addr(&info.gateway_host, info.gateway_port)?;

        match startup_action(None, Some(probe_gloamy(&target)?))? {
            StartupAction::AttachUnmanaged => {
                self.runtime = DesktopRuntime {
                    ready: true,
                    api_base_url: Some(http_base_url(target)),
                    desktop_managed: false,
                    error: None,
                };
                return Ok(());
            }
            StartupAction::RejectConflict => {
                return Err(format!(
                    "Another service is using the configured Gloamy address {}. Stop it or change gateway.host/gateway.port.",
                    target
                ));
            }
            StartupAction::StartBundled => {}
            _ => unreachable!("HTTP probe only permits unmanaged startup actions"),
        }

        self.child = Some(
            Command::new(binary)
                .args(["daemon", "--desktop-managed"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|error| format!("could not start bundled Gloamy daemon: {error}"))?,
        );

        let deadline = Instant::now() + STARTUP_TIMEOUT;
        while Instant::now() < deadline {
            if let Ok(Some(response)) = self.request(ControlRequest::Status)
                && response.ready == Some(true)
            {
                self.attach_control_status(response)?;
                return Ok(());
            }

            if matches!(probe_gloamy(&target)?, ProbeResult::Occupied) {
                let _ = self.request(ControlRequest::Shutdown);
                return Err(format!(
                    "Another service is using the configured Gloamy address {}. The desktop-launched daemon was asked to stop.",
                    target
                ));
            }

            if let Some(child) = self.child.as_mut()
                && child
                    .try_wait()
                    .map_err(|error| format!("could not inspect daemon process: {error}"))?
                    .is_some()
            {
                return Err("bundled Gloamy daemon exited before it became ready".to_owned());
            }
            thread::sleep(Duration::from_millis(150));
        }

        let _ = self.request(ControlRequest::Shutdown);
        Err("Gloamy daemon did not become ready within 45 seconds".to_owned())
    }

    fn attach_control_status(&mut self, response: ControlResponse) -> Result<(), String> {
        if !response.ok || response.ready != Some(true) {
            return Err(response
                .error
                .unwrap_or_else(|| "Gloamy daemon is not ready".to_owned()));
        }
        let api_base_url = response
            .api_base_url
            .ok_or_else(|| "daemon control socket did not report an API address".to_owned())?;
        validate_loopback_url(&api_base_url)?;
        self.runtime = DesktopRuntime {
            ready: true,
            api_base_url: Some(api_base_url),
            desktop_managed: response.desktop_managed.unwrap_or(false),
            error: None,
        };
        Ok(())
    }

    fn request(&self, request: ControlRequest) -> Result<Option<ControlResponse>, String> {
        match request_control(&self.socket_path, request) {
            Ok(response) => Ok(Some(response)),
            Err(error) if is_socket_unavailable(&error) => Ok(None),
            Err(error) => Err(format!("daemon control request failed: {error}")),
        }
    }
}

pub fn should_shutdown_managed_daemon(desktop_managed: bool, keep_running: bool) -> bool {
    desktop_managed && !keep_running
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProbeResult {
    Gloamy,
    Occupied,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupAction {
    AttachControlled,
    WaitForExistingControl,
    AttachUnmanaged,
    StartBundled,
    RejectConflict,
}

fn startup_action(
    control_status: Option<&ControlResponse>,
    probe: Option<ProbeResult>,
) -> Result<StartupAction, String> {
    if let Some(status) = control_status {
        if !status.ok {
            return Err(status
                .error
                .clone()
                .unwrap_or_else(|| "daemon control status failed".to_owned()));
        }
        return Ok(if status.ready == Some(true) {
            StartupAction::AttachControlled
        } else {
            StartupAction::WaitForExistingControl
        });
    }

    match probe {
        Some(ProbeResult::Gloamy) => Ok(StartupAction::AttachUnmanaged),
        Some(ProbeResult::Occupied) => Ok(StartupAction::RejectConflict),
        Some(ProbeResult::Unavailable) => Ok(StartupAction::StartBundled),
        None => Err("daemon startup probe did not return a result".to_owned()),
    }
}

fn resolve_daemon_binary(app: &AppHandle) -> Result<PathBuf, String> {
    let packaged = app
        .path()
        .resource_dir()
        .map_err(|error| format!("cannot locate bundled resources: {error}"))?
        .join("gloamy");
    if packaged.is_file() {
        return Ok(packaged);
    }

    let development = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("gloamy");
    if cfg!(debug_assertions) && development.is_file() {
        return Ok(development);
    }
    Err("bundled Gloamy daemon binary is missing".to_owned())
}

fn read_desktop_info(binary: &Path) -> Result<DesktopInfo, String> {
    let output = Command::new(binary)
        .arg("desktop-info")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|error| format!("could not read Gloamy configuration: {error}"))?;
    if !output.status.success() {
        return Err("bundled Gloamy could not load its configuration".to_owned());
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|_| "bundled Gloamy returned invalid desktop connection settings".to_owned())
}

fn control_socket_path() -> PathBuf {
    let uid = effective_uid();
    std::env::temp_dir()
        .join(format!("gloamy-{uid}"))
        .join("control.sock")
}

fn verify_control_path(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "daemon control socket has no parent directory".to_owned())?;
    let directory = fs::symlink_metadata(parent)
        .map_err(|error| format!("inspect daemon control directory: {error}"))?;
    if !directory.file_type().is_dir()
        || directory.uid() != effective_uid()
        || directory.permissions().mode() & 0o777 != CONTROL_DIRECTORY_MODE
    {
        return Err("daemon control directory failed ownership or permission checks".to_owned());
    }
    let socket = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect daemon control socket: {error}"))?;
    if !socket.file_type().is_socket()
        || socket.uid() != effective_uid()
        || socket.permissions().mode() & 0o777 != CONTROL_SOCKET_MODE
    {
        return Err("daemon control socket failed ownership or permission checks".to_owned());
    }
    Ok(())
}

fn request_control(path: &Path, request: ControlRequest) -> std::io::Result<ControlResponse> {
    fs::symlink_metadata(path)?;
    verify_control_path(path).map_err(std::io::Error::other)?;
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(CONTROL_TIMEOUT))?;
    stream.set_write_timeout(Some(CONTROL_TIMEOUT))?;
    let mut message = serde_json::to_vec(&request).map_err(std::io::Error::other)?;
    message.push(b'\n');
    stream.write_all(&message)?;
    stream.shutdown(Shutdown::Write)?;
    let mut response = Vec::with_capacity(256);
    stream.read_to_end(&mut response)?;
    serde_json::from_slice(&response).map_err(std::io::Error::other)
}

fn is_socket_unavailable(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::NotFound
            | std::io::ErrorKind::ConnectionRefused
            | std::io::ErrorKind::TimedOut
    )
}

fn local_socket_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let candidates = (host, port)
        .to_socket_addrs()
        .map_err(|error| format!("invalid gateway.host '{host}': {error}"))?;
    for candidate in candidates {
        let ip = candidate.ip();
        if is_supported_loopback(ip) {
            return Ok(candidate);
        }
        if ip.is_unspecified() {
            return Ok(SocketAddr::new(loopback_for(ip), port));
        }
    }
    Err(format!(
        "gateway.host '{host}' is not loopback or wildcard; the desktop app only connects to a local daemon"
    ))
}

fn loopback_for(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        IpAddr::V6(_) => IpAddr::V6(std::net::Ipv6Addr::LOCALHOST),
    }
}

fn is_supported_loopback(ip: IpAddr) -> bool {
    ip == IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
        || ip == IpAddr::V6(std::net::Ipv6Addr::LOCALHOST)
}

fn probe_gloamy(address: &SocketAddr) -> Result<ProbeResult, String> {
    if address.port() == 0 {
        return Ok(ProbeResult::Unavailable);
    }
    let mut stream = match TcpStream::connect_timeout(address, Duration::from_millis(250)) {
        Ok(stream) => stream,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::AddrNotAvailable
            ) =>
        {
            return Ok(ProbeResult::Unavailable);
        }
        Err(error) => {
            return Err(format!(
                "could not probe configured daemon address: {error}"
            ));
        }
    };
    stream
        .set_read_timeout(Some(Duration::from_millis(750)))
        .map_err(|error| format!("configure local health probe: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_millis(750)))
        .map_err(|error| format!("configure local health probe: {error}"))?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .map_err(|error| format!("send local health probe: {error}"))?;
    let mut response = Vec::new();
    let _ = stream.take(16 * 1024).read_to_end(&mut response);
    let response = String::from_utf8_lossy(&response);
    if response.starts_with("HTTP/1.1 200")
        && response.contains("\"status\":\"ok\"")
        && response.contains("\"paired\":")
    {
        Ok(ProbeResult::Gloamy)
    } else {
        Ok(ProbeResult::Occupied)
    }
}

fn http_base_url(address: SocketAddr) -> String {
    match address {
        SocketAddr::V4(address) => format!("http://{}:{}", address.ip(), address.port()),
        SocketAddr::V6(address) => format!("http://[{}]:{}", address.ip(), address.port()),
    }
}

fn validate_loopback_url(url: &str) -> Result<(), String> {
    let authority = url
        .strip_prefix("http://")
        .ok_or_else(|| "daemon reported a non-HTTP API address".to_owned())?;
    if authority.contains('/') {
        return Err("daemon reported an invalid API address".to_owned());
    }
    let address: SocketAddr = authority
        .parse()
        .map_err(|_| "daemon reported an invalid API address".to_owned())?;
    if !is_supported_loopback(address.ip()) || address.port() == 0 {
        return Err("desktop app requires the daemon API to use a loopback address".to_owned());
    }
    Ok(())
}

fn load_keep_running(path: &Path) -> Result<bool, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(KEEP_RUNNING_DEFAULT);
        }
        Err(error) => return Err(format!("inspect desktop preferences: {error}")),
    };
    if !metadata.file_type().is_file() || metadata.uid() != effective_uid() {
        return Err(
            "desktop preferences must be a regular file owned by the current user".to_owned(),
        );
    }
    if metadata.permissions().mode() & 0o777 != 0o600 {
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .map_err(|error| format!("restrict desktop preferences permissions: {error}"))?;
    }

    let bytes = fs::read(path).map_err(|error| format!("read desktop preferences: {error}"))?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "desktop preferences are invalid JSON".to_owned())?;
    Ok(value
        .get("keep_daemon_running")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(KEEP_RUNNING_DEFAULT))
}

fn save_keep_running(path: &Path, keep_running: bool) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "desktop preference path has no parent directory".to_owned())?;
    ensure_private_preferences_directory(parent)?;

    let temporary = path.with_file_name(format!(".desktop-preferences-{}.tmp", std::process::id()));
    let bytes = serde_json::to_vec(&serde_json::json!({
        "keep_daemon_running": keep_running,
    }))
    .map_err(|error| format!("serialize desktop preferences: {error}"))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&temporary)
        .map_err(|error| format!("write desktop preferences: {error}"))?;
    file.write_all(&bytes)
        .map_err(|error| format!("write desktop preferences: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("flush desktop preferences: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("save desktop preferences: {error}"))?;
    Ok(())
}

fn ensure_private_preferences_directory(directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory)
        .map_err(|error| format!("create desktop preferences directory: {error}"))?;
    let metadata = fs::symlink_metadata(directory)
        .map_err(|error| format!("inspect desktop preferences directory: {error}"))?;
    if !metadata.file_type().is_dir() || metadata.uid() != effective_uid() {
        return Err("desktop preferences directory must be owned by the current user".to_owned());
    }
    fs::set_permissions(
        directory,
        fs::Permissions::from_mode(CONTROL_DIRECTORY_MODE),
    )
    .map_err(|error| format!("restrict desktop preferences directory: {error}"))?;
    let metadata = fs::symlink_metadata(directory)
        .map_err(|error| format!("verify desktop preferences directory: {error}"))?;
    if !metadata.file_type().is_dir()
        || metadata.uid() != effective_uid()
        || metadata.permissions().mode() & 0o777 != CONTROL_DIRECTORY_MODE
    {
        return Err(
            "desktop preferences directory failed ownership or permission checks".to_owned(),
        );
    }
    Ok(())
}

fn effective_uid() -> u32 {
    // SAFETY: geteuid has no preconditions and returns the current process UID.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_is_limited_to_managed_daemons_when_keep_running_is_off() {
        assert!(should_shutdown_managed_daemon(true, false));
        assert!(!should_shutdown_managed_daemon(true, true));
        assert!(!should_shutdown_managed_daemon(false, false));
        assert!(!should_shutdown_managed_daemon(false, true));
    }

    #[test]
    fn keep_running_preference_defaults_off_and_persists() {
        let root =
            std::env::temp_dir().join(format!("gloamy-preferences-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir(&root).unwrap();
        let path = root.join("desktop-preferences.json");

        assert!(!load_keep_running(&path).unwrap());
        save_keep_running(&path, true).unwrap();
        assert!(load_keep_running(&path).unwrap());
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn only_local_or_wildcard_gateway_addresses_are_accepted() {
        assert_eq!(
            local_socket_addr("127.0.0.1", 42617).unwrap(),
            "127.0.0.1:42617".parse().unwrap()
        );
        assert_eq!(
            local_socket_addr("0.0.0.0", 42617).unwrap(),
            "127.0.0.1:42617".parse().unwrap()
        );
        assert!(local_socket_addr("192.0.2.4", 42617).is_err());
        assert!(local_socket_addr("127.0.0.2", 42617).is_err());
    }

    #[test]
    fn control_protocol_uses_only_fixed_operations() {
        let status = serde_json::to_value(ControlRequest::Status).unwrap();
        let code = serde_json::to_value(ControlRequest::RequestPairingCode).unwrap();
        let shutdown = serde_json::to_value(ControlRequest::Shutdown).unwrap();
        assert_eq!(status["op"], "status");
        assert_eq!(code["op"], "request_pairing_code");
        assert_eq!(shutdown["op"], "shutdown");
    }

    #[test]
    fn startup_attaches_or_starts_without_duplicating_a_daemon() {
        let managed = ControlResponse {
            ok: true,
            ready: Some(true),
            api_base_url: Some("http://127.0.0.1:42617".to_owned()),
            desktop_managed: Some(true),
            code: None,
            error: None,
        };
        let starting = ControlResponse {
            ready: Some(false),
            ..managed.clone()
        };

        assert_eq!(
            startup_action(Some(&managed), None).unwrap(),
            StartupAction::AttachControlled
        );
        assert_eq!(
            startup_action(Some(&starting), None).unwrap(),
            StartupAction::WaitForExistingControl
        );
        assert_eq!(
            startup_action(None, Some(ProbeResult::Gloamy)).unwrap(),
            StartupAction::AttachUnmanaged
        );
        assert_eq!(
            startup_action(None, Some(ProbeResult::Unavailable)).unwrap(),
            StartupAction::StartBundled
        );
        assert_eq!(
            startup_action(None, Some(ProbeResult::Occupied)).unwrap(),
            StartupAction::RejectConflict
        );
    }
}
