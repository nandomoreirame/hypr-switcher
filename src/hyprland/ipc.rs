use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use super::types::HyprClient;

/// Builds the socket path from its components.
fn build_socket_path(runtime_dir: &str, instance_sig: &str) -> PathBuf {
    PathBuf::from(runtime_dir)
        .join("hypr")
        .join(instance_sig)
        .join(".socket.sock")
}

/// Constructs the Hyprland IPC socket path from environment variables.
///
/// Path format: `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`
pub fn socket_path() -> Result<PathBuf> {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR not set")?;
    let instance_sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .context("HYPRLAND_INSTANCE_SIGNATURE not set (is Hyprland running?)")?;

    Ok(build_socket_path(&runtime_dir, &instance_sig))
}

/// Sends a raw command to the Hyprland IPC socket and returns the response.
async fn send_command(path: &PathBuf, cmd: &str) -> Result<String> {
    let mut stream = UnixStream::connect(path)
        .await
        .with_context(|| format!("Failed to connect to Hyprland socket at {}", path.display()))?;

    stream
        .write_all(cmd.as_bytes())
        .await
        .context("Failed to write to Hyprland socket")?;

    stream.shutdown().await.context("Failed to shutdown write half")?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .context("Failed to read from Hyprland socket")?;

    Ok(response)
}

/// Fetches all clients from Hyprland, filters to mapped+visible, and sorts by focus history.
pub async fn get_clients() -> Result<Vec<HyprClient>> {
    let path = socket_path()?;
    let response = send_command(&path, "j/clients").await?;

    let clients: Vec<HyprClient> =
        serde_json::from_str(&response).context("Failed to parse Hyprland clients JSON")?;

    Ok(filter_and_sort_clients(clients))
}

/// Focuses a window by its address via Hyprland dispatcher.
pub async fn focus_window(address: &str) -> Result<()> {
    let path = socket_path()?;
    let cmd = format!("dispatch focuswindow address:{}", address);
    let response = send_command(&path, &cmd).await?;

    if response.trim() != "ok" && !response.is_empty() {
        bail!("Hyprland focus_window failed: {}", response.trim());
    }

    Ok(())
}

/// Brings the active (focused) window to the top of the stacking order.
/// This is needed for floating windows to appear above others after focus.
#[allow(dead_code)]
pub async fn bring_active_to_top() -> Result<()> {
    let path = socket_path()?;
    let response = send_command(&path, "dispatch bringactivetotop").await?;

    if response.trim() != "ok" && !response.is_empty() {
        bail!("Hyprland bring_active_to_top failed: {}", response.trim());
    }

    Ok(())
}

/// Filters and sorts raw client data (used for testing without IPC).
pub fn filter_and_sort_clients(mut clients: Vec<HyprClient>) -> Vec<HyprClient> {
    clients.retain(|c| c.mapped && !c.hidden);
    clients.sort_by_key(|c| c.focus_history_id);
    clients
}

/// Returns the path to the IPC socket for inter-instance communication.
/// When a new instance detects a running overlay via this socket, it sends
/// a cycle command instead of killing and restarting the overlay.
pub fn ipc_socket_path() -> PathBuf {
    let runtime_dir =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("hypr-switcher.sock")
}

/// Tries to send a cycle command ("next" or "prev") to a running instance.
/// Returns true if the command was sent successfully (instance is running).
pub fn try_send_cycle_command(command: &str) -> bool {
    let sock_path = ipc_socket_path();
    match std::os::unix::net::UnixStream::connect(&sock_path) {
        Ok(mut stream) => {
            use std::io::Write;
            let ok = stream.write_all(command.as_bytes()).is_ok();
            if ok {
                tracing::info!("Sent IPC command '{command}' to running instance");
            }
            ok
        }
        Err(_) => false,
    }
}

/// Removes the IPC socket file.
pub fn cleanup_ipc_socket() {
    let _ = std::fs::remove_file(ipc_socket_path());
}

/// Returns the initial selected index for the switcher.
/// Alt+Tab should pre-select the second window (index 1 = previous window),
/// mimicking standard Alt+Tab behavior where the previously focused window
/// is highlighted immediately on first press.
pub fn initial_selected_index(window_count: usize) -> usize {
    if window_count > 1 { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{HyprClient, HyprWorkspace};

    #[test]
    fn test_build_socket_path() {
        let path = build_socket_path("/run/user/1000", "abc123def");
        assert_eq!(
            path,
            PathBuf::from("/run/user/1000/hypr/abc123def/.socket.sock")
        );
    }

    #[test]
    fn test_build_socket_path_different_values() {
        let path = build_socket_path("/tmp/runtime", "xyz789");
        assert_eq!(
            path,
            PathBuf::from("/tmp/runtime/hypr/xyz789/.socket.sock")
        );
    }

    #[test]
    fn test_filter_removes_unmapped_and_hidden() {
        let clients = vec![
            make_client("0x1", "visible", true, false, 0),
            make_client("0x2", "hidden", true, true, 1),
            make_client("0x3", "unmapped", false, false, 2),
            make_client("0x4", "both-bad", false, true, 3),
        ];

        let filtered = filter_and_sort_clients(clients);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].class, "visible");
    }

    #[test]
    fn test_sort_by_focus_history_id() {
        let clients = vec![
            make_client("0x1", "firefox", true, false, 2),
            make_client("0x2", "alacritty", true, false, 0),
            make_client("0x3", "vscode", true, false, 1),
        ];

        let sorted = filter_and_sort_clients(clients);
        assert_eq!(sorted[0].class, "alacritty");
        assert_eq!(sorted[1].class, "vscode");
        assert_eq!(sorted[2].class, "firefox");
    }

    #[test]
    fn test_filter_empty_input() {
        let clients: Vec<HyprClient> = vec![];
        let filtered = filter_and_sort_clients(clients);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_all_hidden() {
        let clients = vec![
            make_client("0x1", "a", false, true, 0),
            make_client("0x2", "b", false, false, 1),
        ];

        let filtered = filter_and_sort_clients(clients);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_initial_selected_index_zero_windows() {
        assert_eq!(initial_selected_index(0), 0);
    }

    #[test]
    fn test_initial_selected_index_one_window() {
        // With 1 window, there's no "previous" window, so stay at 0.
        assert_eq!(initial_selected_index(1), 0);
    }

    #[test]
    fn test_initial_selected_index_two_windows() {
        // With 2+ windows, pre-select index 1 (the previously focused window).
        assert_eq!(initial_selected_index(2), 1);
    }

    #[test]
    fn test_initial_selected_index_many_windows() {
        // With many windows, always pre-select index 1.
        assert_eq!(initial_selected_index(10), 1);
    }

    #[test]
    fn test_sort_by_focus_history_id_current_window_first() {
        // focusHistoryID=0 is the most recently focused window (current).
        // It should appear first so selected_index=1 picks the previous window.
        let clients = vec![
            make_client("0x1", "firefox", true, false, 1),   // previous
            make_client("0x2", "alacritty", true, false, 0), // current (most recent)
            make_client("0x3", "vscode", true, false, 2),    // older
        ];

        let sorted = filter_and_sort_clients(clients);
        assert_eq!(sorted[0].class, "alacritty", "current window (focusHistoryID=0) should be first");
        assert_eq!(sorted[1].class, "firefox", "previous window (focusHistoryID=1) should be second");
        assert_eq!(sorted[2].class, "vscode", "oldest window should be last");
    }

    fn make_client(
        address: &str,
        class: &str,
        mapped: bool,
        hidden: bool,
        focus_history_id: i32,
    ) -> HyprClient {
        HyprClient {
            address: address.to_string(),
            class: class.to_string(),
            title: format!("{class} window"),
            workspace: HyprWorkspace {
                id: 1,
                name: "1".to_string(),
            },
            monitor: 0,
            mapped,
            hidden,
            focus_history_id,
            pid: 1000,
            floating: false,
        }
    }
}
