# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] - 2026-03-03

### Added

- Hyprland IPC client with Unix socket communication for window listing and focusing (`hyprland/ipc`)
- Window and client data types with serde deserialization (`hyprland/types`)
- XDG icon resolver with theme support and multi-level fallback chain (`icons/resolver`)
- Window list UI with layer-shell overlay rendering via iced + iced_layershell (`ui/window_list`)
- Application entry point with state management, keyboard navigation, and IPC polling (`app`)
- Grid layout replacing horizontal scroll, with responsive items-per-row based on monitor width
- 2D keyboard navigation: Tab/Shift+Tab for linear cycling, Arrow keys for directional grid navigation
- Exec-on-keybind design with PID file deduplication and IPC-based fast cycling
- Standard Alt+Tab behavior: releasing Alt confirms the selection
- Auto-focus when only one window is open
- README with installation guide, Hyprland configuration, and architecture docs
- CONTRIBUTING guide with step-by-step contribution workflow
- LICENSE (MIT)

### Fixed

- Grid layout now adapts dynamically to actual monitor width instead of using a fixed max width
