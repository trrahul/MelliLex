use tauri::{tray::TrayIconBuilder, AppHandle, Runtime};

use crate::tray_events::{TrayIconEventHandler, TrayMenuEventHandler};
use crate::tray_menu::TrayMenuBuilder;

pub fn init<R: Runtime>(app: AppHandle<R>) -> tauri::Result<()> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        log::info!("[Tray] Initializing system tray");

        let menu = TrayMenuBuilder::build(&app)?;

        let icon = app
            .default_window_icon()
            .ok_or_else(|| {
                log::error!("[Tray] No default window icon found");
                tauri::Error::InvalidIcon(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No default window icon configured",
                ))
            })?
            .clone();

        log::debug!("[Tray] Icon loaded successfully");

        let _tray = TrayIconBuilder::new()
            .icon(icon)
            .menu(&menu)
            .show_menu_on_left_click(false)
            .tooltip("MelliLex Dictionary")
            .on_menu_event(TrayMenuEventHandler::handle)
            .on_tray_icon_event(|icon, event| {
                let app_handle = icon.app_handle();
                TrayIconEventHandler::handle(app_handle.clone(), event);
            })
            .build(&app)?;

        log::info!("[Tray] System tray initialized successfully");
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        log::debug!("[Tray] Skipping tray initialization on mobile platform");
    }

    Ok(())
}
