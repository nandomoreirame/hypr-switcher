use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct HyprWorkspace {
    pub id: i32,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyprClient {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace: HyprWorkspace,
    pub monitor: i32,
    pub mapped: bool,
    pub hidden: bool,
    #[serde(alias = "focusHistoryID")]
    pub focus_history_id: i32,
    pub pid: u32,
    pub floating: bool,
}

/// Filtered and enriched window entry used by the UI.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WindowEntry {
    pub address: String,
    pub class: String,
    pub title: String,
    pub workspace_id: i32,
    pub workspace_name: String,
    pub icon_path: Option<std::path::PathBuf>,
}

impl From<HyprClient> for WindowEntry {
    fn from(client: HyprClient) -> Self {
        Self {
            address: client.address,
            class: client.class,
            title: client.title,
            workspace_id: client.workspace.id,
            workspace_name: client.workspace.name,
            icon_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HYPRCTL_CLIENTS_JSON: &str = r#"[
        {
            "address": "0x55a3c6e4d0",
            "mapped": true,
            "hidden": false,
            "at": [0, 0],
            "size": [1920, 1080],
            "workspace": { "id": 1, "name": "1" },
            "floating": false,
            "pseudo": false,
            "monitor": 0,
            "class": "Alacritty",
            "title": "~/projects",
            "initialClass": "Alacritty",
            "initialTitle": "Alacritty",
            "pid": 12345,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "grouped": [],
            "tags": [],
            "swallowing": "0x0",
            "focusHistoryID": 0
        },
        {
            "address": "0x55a3c6f8b0",
            "mapped": true,
            "hidden": false,
            "at": [0, 0],
            "size": [1920, 1080],
            "workspace": { "id": 2, "name": "2" },
            "floating": false,
            "pseudo": false,
            "monitor": 1,
            "class": "firefox",
            "title": "GitHub - Mozilla Firefox",
            "initialClass": "firefox",
            "initialTitle": "Mozilla Firefox",
            "pid": 12346,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "grouped": [],
            "tags": [],
            "swallowing": "0x0",
            "focusHistoryID": 1
        },
        {
            "address": "0x55a3c70a90",
            "mapped": false,
            "hidden": true,
            "at": [0, 0],
            "size": [0, 0],
            "workspace": { "id": -1, "name": "special" },
            "floating": false,
            "pseudo": false,
            "monitor": 0,
            "class": "hidden-app",
            "title": "Hidden Window",
            "initialClass": "hidden-app",
            "initialTitle": "Hidden",
            "pid": 12347,
            "xwayland": false,
            "pinned": false,
            "fullscreen": 0,
            "fullscreenClient": 0,
            "grouped": [],
            "tags": [],
            "swallowing": "0x0",
            "focusHistoryID": 2
        }
    ]"#;

    #[test]
    fn test_deserialize_hyprctl_clients() {
        let clients: Vec<HyprClient> = serde_json::from_str(HYPRCTL_CLIENTS_JSON).unwrap();
        assert_eq!(clients.len(), 3);

        assert_eq!(clients[0].class, "Alacritty");
        assert_eq!(clients[0].title, "~/projects");
        assert_eq!(clients[0].workspace.id, 1);
        assert_eq!(clients[0].workspace.name, "1");
        assert_eq!(clients[0].address, "0x55a3c6e4d0");
        assert_eq!(clients[0].monitor, 0);
        assert!(clients[0].mapped);
        assert!(!clients[0].hidden);
        assert_eq!(clients[0].focus_history_id, 0);
        assert_eq!(clients[0].pid, 12345);
        assert!(!clients[0].floating);
    }

    #[test]
    fn test_deserialize_firefox_client() {
        let clients: Vec<HyprClient> = serde_json::from_str(HYPRCTL_CLIENTS_JSON).unwrap();
        let firefox = &clients[1];

        assert_eq!(firefox.class, "firefox");
        assert_eq!(firefox.title, "GitHub - Mozilla Firefox");
        assert_eq!(firefox.workspace.id, 2);
        assert_eq!(firefox.pid, 12346);
        assert_eq!(firefox.focus_history_id, 1);
    }

    #[test]
    fn test_deserialize_hidden_client() {
        let clients: Vec<HyprClient> = serde_json::from_str(HYPRCTL_CLIENTS_JSON).unwrap();
        let hidden = &clients[2];

        assert!(!hidden.mapped);
        assert!(hidden.hidden);
        assert_eq!(hidden.workspace.id, -1);
    }

    #[test]
    fn test_window_entry_from_hypr_client() {
        let client = HyprClient {
            address: "0xabc".to_string(),
            class: "Alacritty".to_string(),
            title: "Terminal".to_string(),
            workspace: HyprWorkspace {
                id: 1,
                name: "1".to_string(),
            },
            monitor: 0,
            mapped: true,
            hidden: false,
            focus_history_id: 0,
            pid: 1000,
            floating: false,
        };

        let entry = WindowEntry::from(client);
        assert_eq!(entry.address, "0xabc");
        assert_eq!(entry.class, "Alacritty");
        assert_eq!(entry.title, "Terminal");
        assert_eq!(entry.workspace_id, 1);
        assert_eq!(entry.workspace_name, "1");
        assert!(entry.icon_path.is_none());
    }
}
