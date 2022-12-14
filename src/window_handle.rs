use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::os::raw::c_void;

pub struct WindowHandle(pub *mut c_void);

unsafe impl HasRawWindowHandle for WindowHandle {
    #[cfg(target_os = "linux")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::XcbHandle::empty();
        handle.window = self.0 as u32;

        RawWindowHandle::Xcb(handle)
    }

    #[cfg(target_os = "windows")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::Win32Handle::empty();
        handle.hwnd = self.0;

        RawWindowHandle::Win32(handle)
    }

    #[cfg(target_os = "macos")]
    fn raw_window_handle(&self) -> RawWindowHandle {
        let mut handle = raw_window_handle::AppKitHandle::empty();
        handle.ns_view = self.0;

        RawWindowHandle::AppKit(handle)
    }
}
