//! Bridges winit's raw-window-handle 0.5 to the 0.6 API that `rfd` uses, so
//! file and message dialogs can be modal to the emulator window.

use rwh06::{DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle};
use winit::window::Window;

pub struct DialogParent<'a> {
    handle: Option<rwh06::RawWindowHandle>,
    _window: std::marker::PhantomData<&'a Window>,
}

impl<'a> DialogParent<'a> {
    pub fn new(window: &'a Window) -> Self {
        use rwh05::HasRawWindowHandle;

        let handle = match window.raw_window_handle() {
            rwh05::RawWindowHandle::Win32(h) => {
                std::num::NonZeroIsize::new(h.hwnd as isize).map(|hwnd| {
                    let mut handle = rwh06::Win32WindowHandle::new(hwnd);
                    handle.hinstance = std::num::NonZeroIsize::new(h.hinstance as isize);
                    rwh06::RawWindowHandle::Win32(handle)
                })
            }
            rwh05::RawWindowHandle::AppKit(h) => std::ptr::NonNull::new(h.ns_view).map(|view| {
                rwh06::RawWindowHandle::AppKit(rwh06::AppKitWindowHandle::new(view))
            }),
            _ => None,
        };

        Self {
            handle,
            _window: std::marker::PhantomData,
        }
    }
}

impl HasWindowHandle for DialogParent<'_> {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // SAFETY: the handle came from a window that outlives `self`.
        self.handle
            .map(|h| unsafe { WindowHandle::borrow_raw(h) })
            .ok_or(HandleError::Unavailable)
    }
}

impl HasDisplayHandle for DialogParent<'_> {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        // Only needed by rfd's Linux backends, where no parent is attached.
        Err(HandleError::Unavailable)
    }
}
