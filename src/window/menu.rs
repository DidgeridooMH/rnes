//! Native menu bar (Windows and macOS). On other platforms the menu is a
//! no-op and the keyboard shortcuts are the only way to trigger actions.

use winit::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    OpenRom,
    CloseRom,
    Exit,
    KeyMapping,
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
pub struct AppMenu {
    _menu: muda::Menu,
    open: muda::MenuItem,
    close: muda::MenuItem,
    exit: muda::MenuItem,
    key_mapping: muda::MenuItem,
}

#[cfg(any(target_os = "windows", target_os = "macos"))]
impl AppMenu {
    pub fn new(window: &Window) -> Result<Self, String> {
        use muda::{
            accelerator::{Accelerator, Code, Modifiers},
            Menu, MenuItem, PredefinedMenuItem, Submenu,
        };

        // Shortcuts are handled by the window's keyboard input; the
        // accelerators here only label the menu items.
        #[cfg(target_os = "macos")]
        let modifier = Modifiers::META;
        #[cfg(not(target_os = "macos"))]
        let modifier = Modifiers::CONTROL;

        let open = MenuItem::new(
            "&Open ROM...",
            true,
            Some(Accelerator::new(modifier, Code::KeyO)),
        );
        let close = MenuItem::new("&Close ROM", false, None);
        let exit = MenuItem::new("E&xit", true, None);

        let file = Submenu::with_items(
            "&File",
            true,
            &[&open, &close, &PredefinedMenuItem::separator(), &exit],
        )
        .map_err(|e| e.to_string())?;

        let key_mapping = MenuItem::new(
            "&Key Mapping...",
            true,
            Some(Accelerator::new(modifier, Code::KeyK)),
        );
        let options =
            Submenu::with_items("&Options", true, &[&key_mapping]).map_err(|e| e.to_string())?;

        let menu = Menu::new();

        // On macOS the first submenu becomes the application menu.
        #[cfg(target_os = "macos")]
        {
            let app = Submenu::with_items("RNES", true, &[&PredefinedMenuItem::quit(None)])
                .map_err(|e| e.to_string())?;
            menu.append(&app).map_err(|e| e.to_string())?;
        }

        menu.append(&file).map_err(|e| e.to_string())?;
        menu.append(&options).map_err(|e| e.to_string())?;

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowExtWindows;
            // SAFETY: the HWND belongs to a live window owned by this thread.
            unsafe { menu.init_for_hwnd(window.hwnd()) }.map_err(|e| e.to_string())?;
        }
        #[cfg(target_os = "macos")]
        {
            let _ = window;
            menu.init_for_nsapp();
        }

        Ok(Self {
            _menu: menu,
            open,
            close,
            exit,
            key_mapping,
        })
    }

    /// Returns the next pending menu action, if any.
    pub fn poll(&self) -> Option<MenuAction> {
        while let Ok(event) = muda::MenuEvent::receiver().try_recv() {
            if event.id == *self.open.id() {
                return Some(MenuAction::OpenRom);
            } else if event.id == *self.close.id() {
                return Some(MenuAction::CloseRom);
            } else if event.id == *self.exit.id() {
                return Some(MenuAction::Exit);
            } else if event.id == *self.key_mapping.id() {
                return Some(MenuAction::KeyMapping);
            }
        }
        None
    }

    pub fn set_rom_loaded(&self, loaded: bool) {
        self.close.set_enabled(loaded);
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub struct AppMenu;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl AppMenu {
    pub fn new(_window: &Window) -> Result<Self, String> {
        Ok(Self)
    }

    pub fn poll(&self) -> Option<MenuAction> {
        None
    }

    pub fn set_rom_loaded(&self, _loaded: bool) {}
}
