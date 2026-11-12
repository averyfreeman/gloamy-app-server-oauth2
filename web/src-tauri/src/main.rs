mod manager;

use manager::{DesktopManager, DesktopRuntime, DesktopState};
use std::sync::Mutex;
use tauri::{Manager, RunEvent, State};

#[tauri::command]
fn get_daemon_status(state: State<'_, DesktopState>) -> DesktopRuntime {
    let mut manager = state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    manager.runtime()
}

#[tauri::command]
fn request_pairing_code(state: State<'_, DesktopState>) -> Result<String, String> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .request_pairing_code()
}

#[tauri::command]
fn get_keep_daemon_running(state: State<'_, DesktopState>) -> bool {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .keep_running()
}

#[tauri::command]
fn set_keep_daemon_running(
    keep_running: bool,
    state: State<'_, DesktopState>,
) -> Result<(), String> {
    state
        .0
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .set_keep_running(keep_running)
}

fn main() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let manager = DesktopManager::initialize(app.handle())
                .unwrap_or_else(DesktopManager::unavailable);
            app.manage(DesktopState(Mutex::new(manager)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_daemon_status,
            request_pairing_code,
            get_keep_daemon_running,
            set_keep_daemon_running
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Gloamy desktop application");

    app.run(|app_handle, event| match event {
        RunEvent::ExitRequested { api, .. } => {
            let state = app_handle.state::<DesktopState>();
            let mut manager = state
                .0
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if manager.exit_is_allowed() {
                return;
            }

            api.prevent_exit();
            match manager.begin_exit() {
                Ok(()) => app_handle.exit(0),
                Err(error) => manager.set_error(error),
            }
        }
        // Some native quit routes reach event-loop teardown without a usable
        // exit-request callback. Stop an owned daemon as a final cleanup path.
        RunEvent::Exit => {
            let state = app_handle.state::<DesktopState>();
            let mut manager = state
                .0
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if !manager.exit_is_allowed()
                && let Err(error) = manager.begin_exit()
            {
                eprintln!("could not stop the managed Gloamy daemon during app exit: {error}");
            }
        }
        _ => {}
    });
}
