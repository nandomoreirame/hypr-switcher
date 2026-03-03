# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

hypr-switcher is a visual Alt+Tab window switcher for Hyprland, written in Rust. It renders a fullscreen overlay via Wayland layer-shell showing all windows across all workspaces, with keyboard navigation to select and focus a window. It runs as an exec-on-keybind binary (not a daemon), launched by Hyprland on ALT+Tab.

## Architecture

The project follows a modular Rust architecture with four main subsystems:

- **Hyprland IPC** (`src/hyprland/`): Communicates with Hyprland via Unix socket at `$XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/.socket.sock`. Uses raw socket commands (`j/clients` for window list, `dispatch focuswindow` for focusing). The `HyprClient` struct uses `#[serde(alias = "focusHistoryID")]` because Hyprland's JSON uses non-standard camelCase for that field.
- **Icon Resolver** (`src/icons/`): Maps window class names to icon paths by parsing `.desktop` files from XDG application directories, then resolving icons via `freedesktop-icons` crate with theme support. Results are cached in a HashMap. Fallback chain: theme -> hicolor -> pixmaps -> None.
- **UI Layer** (`src/ui/`): Iced widgets rendered as a Wayland layer-shell overlay (Layer::Overlay, KeyboardInteractivity::Exclusive). `style.rs` holds design tokens (colors, dimensions), `window_list.rs` renders the window card list.
- **App State** (`src/app.rs`): Iced Application using the `iced_layershell::application()` builder pattern with `#[to_layer_message]` macro on the Message enum. Handles Tab/Shift+Tab cycling, Enter to confirm (focuses window via IPC then exits), Escape to dismiss.

## Startup Flow (`src/main.rs`)

1. Init tracing with env-filter
2. Kill previous instance via PID file (`$XDG_RUNTIME_DIR/hypr-switcher.pid`)
3. Write current PID file
4. Fetch clients from Hyprland IPC socket
5. If 1 window: auto-focus and exit
6. Resolve icons for all windows
7. Launch iced-layershell overlay
8. Cleanup PID file on exit

## Tech Stack

| Component | Choice |
|-----------|--------|
| Language | Rust (edition 2024) |
| UI Framework | `iced` 0.14 + `iced_layershell` 0.15 |
| Async Runtime | `tokio` (for Unix socket IPC) |
| Serialization | `serde` + `serde_json` (Hyprland JSON responses) |
| Icons | `freedesktop-icons` + XDG `.desktop` file parsing |
| Logging | `tracing` + `tracing-subscriber` with env-filter |

## Build and Test Commands

```bash
# Build
cargo build
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests in a specific module
cargo test hyprland::
cargo test icons::
cargo test app::

# Run with logging
RUST_LOG=debug cargo run

# Install locally
cargo build --release && cp target/release/hypr-switcher ~/.local/bin/
```

## Hyprland Keybinding

Add to `~/.config/hypr/bindings.conf`:

```
unbind = ALT, TAB
unbind = ALT SHIFT, TAB
bindd = ALT, TAB, Window switcher, exec, hypr-switcher
```

## Key Design Decisions

- **Exec-on-keybind, not daemon**: Zero resource usage when inactive. Hyprland spawns the binary each time ALT+Tab is pressed.
- **Raw Unix socket, not hyprland-rs**: Only two IPC commands are needed, so a lightweight raw socket client avoids a heavy dependency.
- **freedesktop-icons crate**: Resolves icons following the XDG icon theme spec with multiple fallback levels.
- **iced + iced_layershell**: Pure Rust UI stack with native layer-shell support, no GTK/FFI required.
- **`Mutex<Option<Vec<WindowEntry>>>` for init**: The `iced_layershell::application()` requires `Fn` (not `FnOnce`), so window data is passed via `Mutex::take()` to the init closure.
- **PID file dedup**: Previous instance is killed on startup to avoid stacking overlays.
- **Rust 2024 edition**: `std::env::set_var`/`remove_var` are unsafe; tests use pure function testing instead of env var manipulation.
