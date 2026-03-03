use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Resolves window class names to icon file paths.
pub struct IconResolver {
    /// Maps lowercase window class -> icon name from .desktop files
    class_to_icon: HashMap<String, String>,
    /// Caches resolved icon paths
    cache: HashMap<String, Option<PathBuf>>,
    /// Icon theme to use (e.g., "Yaru-purple")
    theme: Option<String>,
}

impl IconResolver {
    /// Creates a new IconResolver by scanning system .desktop files.
    pub fn new(theme: Option<String>) -> Self {
        let mut resolver = Self {
            class_to_icon: HashMap::new(),
            cache: HashMap::new(),
            theme,
        };
        resolver.scan_desktop_files();
        resolver
    }

    /// Creates an IconResolver from pre-built data (for testing).
    #[cfg(test)]
    pub fn from_map(class_to_icon: HashMap<String, String>, theme: Option<String>) -> Self {
        Self {
            class_to_icon,
            cache: HashMap::new(),
            theme,
        }
    }

    /// Scans standard XDG application directories for .desktop files.
    fn scan_desktop_files(&mut self) {
        let dirs = desktop_file_dirs();
        for dir in dirs {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|ext| ext == "desktop") {
                        self.parse_desktop_file(&path);
                    }
                }
            }
        }
    }

    /// Parses a .desktop file to extract StartupWMClass and Icon fields.
    fn parse_desktop_file(&mut self, path: &Path) {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut icon_name = None;
        let mut wm_class = None;

        for line in content.lines() {
            let line = line.trim();
            if let Some(value) = line.strip_prefix("Icon=") {
                icon_name = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("StartupWMClass=") {
                wm_class = Some(value.to_string());
            }
        }

        let icon = match icon_name {
            Some(i) => i,
            None => return,
        };

        // Map by StartupWMClass (primary)
        if let Some(ref wmc) = wm_class {
            self.class_to_icon
                .insert(wmc.to_lowercase(), icon.clone());
        }

        // Map by desktop filename without extension (fallback)
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            self.class_to_icon
                .entry(stem.to_lowercase())
                .or_insert(icon);
        }
    }

    /// Resolves a window class to an icon file path.
    ///
    /// Fallback chain:
    /// 1. Look up icon name from class_to_icon map
    /// 2. Try freedesktop_icons::lookup with theme
    /// 3. Try freedesktop_icons::lookup without theme (hicolor)
    /// 4. Try /usr/share/pixmaps/{name}.{svg,png}
    /// 5. Return None (caller uses embedded default)
    pub fn resolve(&mut self, window_class: &str) -> Option<PathBuf> {
        let key = window_class.to_lowercase();

        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }

        let result = self.resolve_uncached(&key);
        self.cache.insert(key, result.clone());
        result
    }

    fn resolve_uncached(&self, class_lower: &str) -> Option<PathBuf> {
        // Step 1: Look up icon name from .desktop file data
        let icon_name = self
            .class_to_icon
            .get(class_lower)
            .cloned()
            .unwrap_or_else(|| class_lower.to_string());

        // If icon_name is already an absolute path, use it directly
        if icon_name.starts_with('/') {
            let path = PathBuf::from(&icon_name);
            if path.exists() {
                return Some(path);
            }
        }

        // Step 2: freedesktop lookup with theme
        if let Some(ref theme) = self.theme {
            if let Some(path) = freedesktop_icons::lookup(&icon_name)
                .with_theme(theme)
                .with_size(48)
                .with_cache()
                .find()
            {
                return Some(path);
            }
        }

        // Step 3: freedesktop lookup without theme (hicolor fallback)
        if let Some(path) = freedesktop_icons::lookup(&icon_name)
            .with_size(48)
            .with_cache()
            .find()
        {
            return Some(path);
        }

        // Step 4: pixmaps fallback
        for ext in &["svg", "png"] {
            let pixmap = PathBuf::from(format!("/usr/share/pixmaps/{icon_name}.{ext}"));
            if pixmap.exists() {
                return Some(pixmap);
            }
        }

        None
    }

    /// Returns the icon name mapping for a given class (for testing/debugging).
    #[allow(dead_code)]
    pub fn icon_name_for_class(&self, window_class: &str) -> Option<&String> {
        self.class_to_icon.get(&window_class.to_lowercase())
    }
}

/// Returns the list of directories to scan for .desktop files.
fn desktop_file_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("/usr/share/applications")];

    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    if let Ok(data_dirs) = std::env::var("XDG_DATA_DIRS") {
        for dir in data_dirs.split(':') {
            let app_dir = PathBuf::from(dir).join("applications");
            if !dirs.contains(&app_dir) {
                dirs.push(app_dir);
            }
        }
    }

    dirs
}

/// Parses a single .desktop file content and extracts StartupWMClass and Icon.
/// Exposed for testing.
#[cfg(test)]
pub fn parse_desktop_content(content: &str) -> (Option<String>, Option<String>) {
    let mut icon_name = None;
    let mut wm_class = None;

    for line in content.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("Icon=") {
            icon_name = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("StartupWMClass=") {
            wm_class = Some(value.to_string());
        }
    }

    (wm_class, icon_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_desktop_content_alacritty() {
        let content = "\
[Desktop Entry]
Type=Application
Name=Alacritty
Comment=A fast, cross-platform, OpenGL terminal emulator
Exec=alacritty
Icon=Alacritty
Terminal=false
Categories=System;TerminalEmulator;
StartupWMClass=Alacritty
";
        let (wm_class, icon) = parse_desktop_content(content);
        assert_eq!(wm_class.as_deref(), Some("Alacritty"));
        assert_eq!(icon.as_deref(), Some("Alacritty"));
    }

    #[test]
    fn test_parse_desktop_content_firefox() {
        let content = "\
[Desktop Entry]
Name=Firefox
Exec=firefox %u
Icon=firefox
Type=Application
StartupWMClass=firefox
";
        let (wm_class, icon) = parse_desktop_content(content);
        assert_eq!(wm_class.as_deref(), Some("firefox"));
        assert_eq!(icon.as_deref(), Some("firefox"));
    }

    #[test]
    fn test_parse_desktop_content_no_wm_class() {
        let content = "\
[Desktop Entry]
Name=SomeApp
Icon=some-icon
Type=Application
";
        let (wm_class, icon) = parse_desktop_content(content);
        assert!(wm_class.is_none());
        assert_eq!(icon.as_deref(), Some("some-icon"));
    }

    #[test]
    fn test_parse_desktop_content_no_icon() {
        let content = "\
[Desktop Entry]
Name=NoIcon
Type=Application
";
        let (wm_class, icon) = parse_desktop_content(content);
        assert!(wm_class.is_none());
        assert!(icon.is_none());
    }

    #[test]
    fn test_parse_desktop_content_absolute_icon_path() {
        let content = "\
[Desktop Entry]
Name=Custom
Icon=/opt/custom/icon.png
StartupWMClass=custom-app
";
        let (wm_class, icon) = parse_desktop_content(content);
        assert_eq!(wm_class.as_deref(), Some("custom-app"));
        assert_eq!(icon.as_deref(), Some("/opt/custom/icon.png"));
    }

    #[test]
    fn test_resolver_from_map_lookup() {
        let mut map = HashMap::new();
        map.insert("alacritty".to_string(), "Alacritty".to_string());
        map.insert("firefox".to_string(), "firefox".to_string());

        let resolver = IconResolver::from_map(map, None);

        assert_eq!(
            resolver.icon_name_for_class("Alacritty"),
            Some(&"Alacritty".to_string())
        );
        assert_eq!(
            resolver.icon_name_for_class("firefox"),
            Some(&"firefox".to_string())
        );
        assert_eq!(resolver.icon_name_for_class("unknown"), None);
    }

    #[test]
    fn test_resolver_case_insensitive_lookup() {
        let mut map = HashMap::new();
        map.insert("alacritty".to_string(), "Alacritty".to_string());

        let resolver = IconResolver::from_map(map, None);

        // Should find via case-insensitive lookup
        assert_eq!(
            resolver.icon_name_for_class("ALACRITTY"),
            Some(&"Alacritty".to_string())
        );
        assert_eq!(
            resolver.icon_name_for_class("alacritty"),
            Some(&"Alacritty".to_string())
        );
    }

    #[test]
    fn test_resolver_caching() {
        let mut map = HashMap::new();
        map.insert("nonexistent".to_string(), "nonexistent-icon".to_string());

        let mut resolver = IconResolver::from_map(map, None);

        // First call - misses cache
        let result1 = resolver.resolve("nonexistent");
        // Second call - hits cache
        let result2 = resolver.resolve("nonexistent");

        assert_eq!(result1, result2);
        assert!(resolver.cache.contains_key("nonexistent"));
    }

    #[test]
    fn test_desktop_file_dirs_includes_system() {
        let dirs = desktop_file_dirs();
        assert!(dirs.contains(&PathBuf::from("/usr/share/applications")));
    }
}
