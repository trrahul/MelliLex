use tauri::{
    menu::MenuEvent,
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    AppHandle, Runtime,
};

use crate::window_controls::{hide_main_window, show_main_window};

const MENU_SHOW: &str = "tray.show";
const MENU_HIDE: &str = "tray.hide";
const MENU_QUIT: &str = "tray.quit";

pub struct TrayMenuEventHandler;

impl TrayMenuEventHandler {
    pub fn handle<R: Runtime>(app: &AppHandle<R>, event: MenuEvent) {
        log::debug!("[TrayMenuEventHandler] Menu event: {}", event.id.as_ref());

        match event.id.as_ref() {
            MENU_SHOW => {
                if let Err(e) = show_main_window(app) {
                    log::error!("[TrayMenuEventHandler] Failed to show window: {}", e);
                }
            }
            MENU_HIDE => {
                if let Err(e) = hide_main_window(app) {
                    log::error!("[TrayMenuEventHandler] Failed to hide window: {}", e);
                }
            }
            MENU_QUIT => {
                log::info!("[TrayMenuEventHandler] Quit requested, exiting application");
                app.exit(0);
            }
            _ => {
                log::warn!(
                    "[TrayMenuEventHandler] Unknown menu event: {}",
                    event.id.as_ref()
                );
            }
        }
    }
}

pub struct TrayIconEventHandler;

impl TrayIconEventHandler {
    pub fn handle<R: Runtime>(app: AppHandle<R>, event: TrayIconEvent) {
        match event {
            TrayIconEvent::Click {
                button,
                button_state,
                ..
            } => {
                if matches!(button, MouseButton::Left)
                    && matches!(button_state, MouseButtonState::Up)
                {
                    log::debug!("[TrayIconEventHandler] Left-click detected, showing window");
                    if let Err(e) = show_main_window(&app) {
                        log::error!(
                            "[TrayIconEventHandler] Failed to show window on click: {}",
                            e
                        );
                    }
                }
            }
            _ => {
                log::trace!("[TrayIconEventHandler] Unhandled tray event: {:?}", event);
            }
        }
    }
}
