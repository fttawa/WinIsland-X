use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use crate::core::config::WINDOW_TITLE;

pub struct TrayManager {
    _tray: TrayIcon,
    toggle_item: MenuItem,
    quit_item: MenuItem,
}

impl TrayManager {
    pub fn new() -> Self {
        let menu = Menu::new();
        let toggle_item = MenuItem::new("Hide", true, None);
        let quit_item = MenuItem::new("Exit", true, None);
        let _ = menu.append(&toggle_item);
        let _ = menu.append(&quit_item);

        let tray = TrayIconBuilder::new()
            .with_tooltip(WINDOW_TITLE)
            .with_menu(Box::new(menu))
            .with_icon(Self::create_white_icon())
            .build()
            .unwrap();

        Self {
            _tray: tray,
            toggle_item,
            quit_item,
        }
    }

    pub fn handle_events(&self) -> Option<TrayAction> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.toggle_item.id() {
                return Some(TrayAction::ToggleVisibility);
            } else if event.id == self.quit_item.id() {
                return Some(TrayAction::Exit);
            }
        }
        None
    }

    pub fn update_item_text(&self, visible: bool) {
        if visible {
            self.toggle_item.set_text("Hide");
        } else {
            self.toggle_item.set_text("Show");
        }
    }

    fn create_white_icon() -> Icon {
        let mut rgba = vec![255u8; 32 * 32 * 4];
        Icon::from_rgba(rgba, 32, 32).unwrap()
    }
}

pub enum TrayAction {
    ToggleVisibility,
    Exit,
}
