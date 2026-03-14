use tauri::{
    menu::{Menu, MenuItem},
    AppHandle, Runtime,
};

const MENU_SHOW: &str = "tray.show";
const MENU_HIDE: &str = "tray.hide";
const MENU_QUIT: &str = "tray.quit";

pub struct TrayMenuBuilder;

impl TrayMenuBuilder {
    pub fn build<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
        log::debug!("[TrayMenuBuilder] Creating menu items");

        let show_item = MenuItem::with_id(app, MENU_SHOW, "Show Window", true, None::<&str>)?;
        let hide_item = MenuItem::with_id(app, MENU_HIDE, "Hide Window", true, None::<&str>)?;
        let quit_item = MenuItem::with_id(app, MENU_QUIT, "Quit MelliLex", true, None::<&str>)?;

        let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;
        log::debug!("[TrayMenuBuilder] Menu created with 3 items");

        Ok(menu)
    }
}
