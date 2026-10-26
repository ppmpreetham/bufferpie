use gpui_kit::Window;

#[cfg(windows)]
unsafe extern "system" {
    fn GetWindowLongPtrW(hwnd: isize, nindex: i32) -> isize;
    fn SetWindowLongPtrW(hwnd: isize, nindex: i32, dwnewlong: isize) -> isize;
    fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
}

#[cfg(windows)]
fn hwnd(window: &Window) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match HasWindowHandle::window_handle(window).ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

/// clicking the overlay must not steal focus from the app underneath
#[cfg(windows)]
pub fn make_no_activate(window: &Window) {
    const GWL_EXSTYLE: i32 = -20;
    const WS_EX_NOACTIVATE: isize = 0x0800_0000;
    if let Some(hwnd) = hwnd(window) {
        unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, GetWindowLongPtrW(hwnd, GWL_EXSTYLE) | WS_EX_NOACTIVATE) };
    }
}

/// keeps the overlay above every other window
#[cfg(windows)]
pub fn raise(window: &Window) {
    const HWND_TOPMOST: isize = -1;
    const SWP_NOSIZE_NOMOVE_NOACTIVATE: u32 = 0x0001 | 0x0002 | 0x0010;
    if let Some(hwnd) = hwnd(window) {
        unsafe { SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOSIZE_NOMOVE_NOACTIVATE) };
    }
}

#[cfg(not(windows))]
pub fn make_no_activate(_: &Window) {}

#[cfg(not(windows))]
pub fn raise(_: &Window) {}
