mod app;
mod hyprland;
mod icons;
mod ui;

use std::fs;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::Ordering;

use anyhow::Result;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use crate::hyprland::ipc::{self, initial_selected_index};
use crate::hyprland::types::WindowEntry;
use crate::icons::resolver::IconResolver;
use crate::ui::window_list;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Determine cycle direction from args
    let is_reverse = std::env::args().any(|a| a == "--reverse");
    let command = if is_reverse { "prev" } else { "next" };

    // If an overlay is already running, send a cycle command and exit.
    // Hyprland re-launches the binary on each Alt+Tab press, so subsequent
    // launches just send an IPC command to the running overlay and exit fast.
    if ipc::try_send_cycle_command(command) {
        return Ok(());
    }

    // No running instance found — start fresh
    kill_previous_instance();
    write_pid_file();

    // Fetch window list from Hyprland
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let clients = rt.block_on(ipc::get_clients())?;

    // 0 windows: show empty UI (will dismiss on any key)
    // 1 window: auto-focus and exit
    if clients.len() == 1 {
        tracing::info!("Only 1 window, auto-focusing: {}", clients[0].class);
        rt.block_on(ipc::focus_window(&clients[0].address))?;
        cleanup_pid_file();
        return Ok(());
    }

    // Resolve icons
    let mut icon_resolver = IconResolver::new(Some("Yaru-purple".to_string()));
    let windows: Vec<WindowEntry> = clients
        .into_iter()
        .map(|client| {
            let icon_path = icon_resolver.resolve(&client.class);
            let mut entry = WindowEntry::from(client);
            entry.icon_path = icon_path;
            entry
        })
        .collect();

    tracing::info!("Launching switcher with {} windows", windows.len());

    // Start the IPC listener thread. This listens on a Unix socket for
    // cycle commands from new instances and sets the IPC_SIGNAL atomic.
    start_ipc_listener();

    // Launch UI - Mutex needed because application() requires Fn, not FnOnce.
    // selected_index starts at 1 (the previous window) to match standard Alt+Tab
    // behavior: the previously focused window is pre-selected on first press.
    let initial_index = initial_selected_index(windows.len());
    let per_row = window_list::items_per_row();
    let state_cell = Mutex::new(Some(windows));

    let result = iced_layershell::application(
        move || {
            let windows = state_cell.lock().unwrap().take().unwrap_or_default();
            app::AppState {
                windows,
                selected_index: initial_index,
                items_per_row: per_row,
            }
        },
        app::namespace,
        app::update,
        app::view,
    )
    .style(app::app_style)
    .subscription(app::subscription)
    .settings(Settings {
        layer_settings: LayerShellSettings {
            anchor: Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right,
            layer: Layer::Overlay,
            keyboard_interactivity: KeyboardInteractivity::Exclusive,
            start_mode: StartMode::Active,
            ..Default::default()
        },
        ..Default::default()
    })
    .run()
    .map_err(|e| anyhow::anyhow!("iced_layershell error: {e}"));

    // Focus the selected window AFTER the overlay is fully destroyed.
    // This is critical: while the overlay has KeyboardInteractivity::Exclusive,
    // focuswindow doesn't transfer keyboard focus. By waiting until iced exits
    // and the layer shell surface is gone, the focus transfer works correctly.
    if let Some(address) = app::SELECTED_WINDOW.lock().unwrap().take() {
        focus_window_sync(&address);
    }

    ipc::cleanup_ipc_socket();
    cleanup_pid_file();
    result
}

/// Focuses a Hyprland window by address and brings it to the top of the stack.
fn focus_window_sync(address: &str) {
    let cmd = format!("address:{address}");
    tracing::info!("Focusing window: {cmd}");

    match std::process::Command::new("hyprctl")
        .args(["dispatch", "focuswindow", &cmd])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::info!("hyprctl focuswindow: stdout={stdout}, stderr={stderr}, status={}", output.status);
        }
        Err(e) => {
            tracing::error!("Failed to run hyprctl focuswindow: {e}");
        }
    }

    match std::process::Command::new("hyprctl")
        .args(["dispatch", "bringactivetotop"])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::debug!("hyprctl bringactivetotop: {stdout}");
        }
        Err(e) => {
            tracing::warn!("Failed to run hyprctl bringactivetotop: {e}");
        }
    }
}

/// Starts a background thread that listens on a Unix socket for cycle
/// commands from new instances. When a command is received, it sets
/// the IPC_SIGNAL atomic which is polled by the iced event loop.
fn start_ipc_listener() {
    std::thread::spawn(|| {
        let sock_path = ipc::ipc_socket_path();
        // Remove stale socket from a previous crashed instance
        let _ = std::fs::remove_file(&sock_path);

        let listener = match std::os::unix::net::UnixListener::bind(&sock_path) {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to bind IPC socket at {}: {e}", sock_path.display());
                return;
            }
        };

        tracing::info!("IPC listener started at {}", sock_path.display());

        for conn in listener.incoming().flatten() {
            let mut conn = conn;
            let mut buf = [0u8; 16];
            if let Ok(n) = std::io::Read::read(&mut conn, &mut buf) {
                let cmd = std::str::from_utf8(&buf[..n]).unwrap_or("").trim();
                match cmd {
                    "next" => {
                        tracing::debug!("IPC received: next");
                        app::IPC_SIGNAL.store(1, Ordering::SeqCst);
                    }
                    "prev" => {
                        tracing::debug!("IPC received: prev");
                        app::IPC_SIGNAL.store(2, Ordering::SeqCst);
                    }
                    _ => {
                        tracing::warn!("IPC received unknown command: {cmd}");
                    }
                }
            }
        }
    });
}

/// Returns the path to the PID file.
fn pid_file_path() -> std::path::PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".to_string());
    std::path::PathBuf::from(runtime_dir).join("hypr-switcher.pid")
}

/// Writes the current process PID to a file.
fn write_pid_file() {
    let path = pid_file_path();
    if let Ok(mut file) = fs::File::create(&path) {
        let _ = write!(file, "{}", std::process::id());
    }
}

/// Kills a previous instance if its PID file exists and the process is alive.
fn kill_previous_instance() {
    let path = pid_file_path();
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            let current_pid = std::process::id();
            if pid != current_pid {
                #[cfg(unix)]
                {
                    let _ = std::process::Command::new("kill")
                        .arg(pid.to_string())
                        .status();
                    tracing::info!("Killed previous instance (pid: {})", pid);
                }
            }
        }
    }
}

/// Removes the PID file on exit.
fn cleanup_pid_file() {
    let _ = fs::remove_file(pid_file_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pid_file_path_uses_runtime_dir() {
        let path = pid_file_path();
        assert!(path.to_str().unwrap().ends_with("hypr-switcher.pid"));
    }
}
