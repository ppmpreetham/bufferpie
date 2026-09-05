//! platform-specific window tweaks for the overlay pie menu

#[cfg(windows)]
mod imp {
    use gpui::Window;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_NOACTIVATE: isize = 0x0800_0000;
    const HWND_TOPMOST: isize = -1;
    const SWP_NOSIZE: u32 = 0x0001;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOACTIVATE: u32 = 0x0010;

    unsafe extern "system" {
        fn GetWindowLongPtrW(hwnd: isize, nindex: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: isize, nindex: i32, dwnewlong: isize) -> isize;
        fn SetWindowPos(
            hwnd: isize,
            hwndinsertafter: isize,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
    }

    fn hwnd(window: &Window) -> Option<isize> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
            _ => None,
        }
    }

    pub fn make_no_activate(window: &Window) {
        let Some(hwnd) = hwnd(window) else { return };
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE);
        }
    }

    pub fn raise(window: &Window) {
        let Some(hwnd) = hwnd(window) else { return };
        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }
}

#[cfg(windows)]
pub use imp::*;

#[cfg(not(windows))]
mod imp {
    use gpui::Window;

    pub fn make_no_activate(_: &Window) {}
    pub fn raise(_: &Window) {}
}
#[cfg(not(windows))]
pub use imp::*;
