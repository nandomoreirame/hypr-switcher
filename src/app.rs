use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use iced::keyboard::{self, Key};
use iced::widget::{center, container};
use iced::{Color, Element, Event, Length, Subscription, Task};
use iced_layershell::to_layer_message;

use crate::hyprland::types::WindowEntry;
use crate::ui::{style, window_list};

/// Signal from the IPC listener thread: 0=none, 1=next, 2=prev.
pub static IPC_SIGNAL: AtomicI32 = AtomicI32::new(0);

/// Address of the window to focus after the overlay closes.
/// Set by ConfirmSelection, read by main() after iced exits.
pub static SELECTED_WINDOW: Mutex<Option<String>> = Mutex::new(None);

pub struct AppState {
    pub windows: Vec<WindowEntry>,
    pub selected_index: usize,
}

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    CycleNext,
    CyclePrev,
    ConfirmSelection,
    Dismiss,
    PollIpc,
    IcedEvent(Event),
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            selected_index: 0,
        }
    }
}

pub fn namespace() -> String {
    "hypr-switcher".to_string()
}

pub fn subscription(_state: &AppState) -> Subscription<Message> {
    Subscription::batch([
        iced::event::listen().map(Message::IcedEvent),
        iced::time::every(Duration::from_millis(16)).map(|_| Message::PollIpc),
    ])
}

/// Returns the address of the selected window, if any.
pub fn selected_address(state: &AppState) -> Option<String> {
    state
        .windows
        .get(state.selected_index)
        .map(|w| w.address.clone())
}

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::CycleNext => {
            if !state.windows.is_empty() {
                state.selected_index = (state.selected_index + 1) % state.windows.len();
            }
            Task::none()
        }
        Message::CyclePrev => {
            if !state.windows.is_empty() {
                state.selected_index = if state.selected_index == 0 {
                    state.windows.len() - 1
                } else {
                    state.selected_index - 1
                };
            }
            Task::none()
        }
        Message::ConfirmSelection => {
            // Store the address to focus AFTER the overlay is destroyed.
            // Focusing while the overlay has KeyboardInteractivity::Exclusive
            // doesn't transfer keyboard focus; Hyprland restores focus to the
            // previous window when the exclusive surface dies.
            if let Some(address) = selected_address(state) {
                *SELECTED_WINDOW.lock().unwrap() = Some(address);
            }
            iced::exit()
        }
        Message::Dismiss => {
            iced::exit()
        }
        Message::PollIpc => {
            let signal = IPC_SIGNAL.swap(0, Ordering::SeqCst);
            match signal {
                1 => return update(state, Message::CycleNext),
                2 => return update(state, Message::CyclePrev),
                _ => {}
            }
            Task::none()
        }
        Message::IcedEvent(event) => {
            match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) => {
                    if state.windows.is_empty() {
                        return iced::exit();
                    }

                    match key {
                        Key::Named(keyboard::key::Named::Tab)
                        | Key::Named(keyboard::key::Named::ArrowRight) => {
                            if modifiers.shift() {
                                return update(state, Message::CyclePrev);
                            } else {
                                return update(state, Message::CycleNext);
                            }
                        }
                        Key::Named(keyboard::key::Named::ArrowLeft) => {
                            return update(state, Message::CyclePrev);
                        }
                        Key::Named(keyboard::key::Named::Enter) => {
                            return update(state, Message::ConfirmSelection);
                        }
                        Key::Named(keyboard::key::Named::Escape) => {
                            return update(state, Message::Dismiss);
                        }
                        _ => {}
                    }
                }
                // Release ALT -> confirm selection (standard Alt+Tab behavior).
                Event::Keyboard(keyboard::Event::KeyReleased {
                    key: Key::Named(keyboard::key::Named::Alt),
                    ..
                }) => {
                    return update(state, Message::ConfirmSelection);
                }
                _ => {}
            }
            Task::none()
        }
        _ => Task::none(),
    }
}


pub fn view(state: &AppState) -> Element<'_, Message> {
    if state.windows.is_empty() {
        let empty_msg = container(
            iced::widget::text("No windows open")
                .size(16)
                .color(style::TEXT_DIM_COLOR),
        )
        .padding(40)
        .style(empty_card_style);

        return center(empty_msg)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let list = window_list::window_list_view(&state.windows, state.selected_index);

    center(list)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn app_style(_state: &AppState, _theme: &iced::Theme) -> iced::theme::Style {
    iced::theme::Style {
        background_color: style::BG_COLOR,
        text_color: style::TEXT_COLOR,
    }
}

fn empty_card_style(theme: &iced::Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(iced::Background::Color(style::CARD_BG_COLOR)),
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: style::CARD_BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyprland::types::WindowEntry;

    fn make_windows(count: usize) -> Vec<WindowEntry> {
        (0..count)
            .map(|i| WindowEntry {
                address: format!("0x{i}"),
                class: format!("app-{i}"),
                title: format!("Window {i}"),
                workspace_id: (i as i32) + 1,
                workspace_name: format!("{}", i + 1),
                icon_path: None,
            })
            .collect()
    }

    #[test]
    fn test_cycle_next_wraps() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 2,
        };

        let _ = update(&mut state, Message::CycleNext);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_cycle_next_increments() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 0,
        };

        let _ = update(&mut state, Message::CycleNext);
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_cycle_prev_wraps() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 0,
        };

        let _ = update(&mut state, Message::CyclePrev);
        assert_eq!(state.selected_index, 2);
    }

    #[test]
    fn test_cycle_prev_decrements() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 2,
        };

        let _ = update(&mut state, Message::CyclePrev);
        assert_eq!(state.selected_index, 1);
    }

    #[test]
    fn test_cycle_empty_windows() {
        let mut state = AppState {
            windows: vec![],
            selected_index: 0,
        };

        let _ = update(&mut state, Message::CycleNext);
        assert_eq!(state.selected_index, 0);

        let _ = update(&mut state, Message::CyclePrev);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_namespace() {
        assert_eq!(namespace(), "hypr-switcher");
    }

    #[test]
    fn test_selected_address() {
        let state = AppState {
            windows: make_windows(3),
            selected_index: 1,
        };

        assert_eq!(selected_address(&state), Some("0x1".to_string()));
    }

    #[test]
    fn test_selected_address_empty() {
        let state = AppState {
            windows: vec![],
            selected_index: 0,
        };

        assert_eq!(selected_address(&state), None);
    }

    #[test]
    fn test_selected_address_out_of_bounds() {
        let state = AppState {
            windows: make_windows(2),
            selected_index: 5,
        };

        assert_eq!(selected_address(&state), None);
    }

    #[test]
    fn test_poll_ipc_next() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 0,
        };

        IPC_SIGNAL.store(1, Ordering::SeqCst);
        let _ = update(&mut state, Message::PollIpc);
        assert_eq!(state.selected_index, 1);
        assert_eq!(IPC_SIGNAL.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_poll_ipc_prev() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 1,
        };

        IPC_SIGNAL.store(2, Ordering::SeqCst);
        let _ = update(&mut state, Message::PollIpc);
        assert_eq!(state.selected_index, 0);
        assert_eq!(IPC_SIGNAL.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_poll_ipc_no_signal() {
        let mut state = AppState {
            windows: make_windows(3),
            selected_index: 1,
        };

        IPC_SIGNAL.store(0, Ordering::SeqCst);
        let _ = update(&mut state, Message::PollIpc);
        assert_eq!(state.selected_index, 1);
    }
}
