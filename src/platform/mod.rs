pub mod clipboard;
pub mod screenshot_analyzer;

pub use clipboard::{Clipboard, ClipboardError, SystemClipboard};
pub use screenshot_analyzer::{
    AnalysisError, BoardAnalysisResult, ImportError, ScreenshotAnalysisConfig, analyze_board_image,
    import_board_from_analysis,
};

#[cfg(not(target_arch = "wasm32"))]
pub trait ScreenshotRequester {
    fn request_interactive_capture(&self);
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "linux")]
pub mod screenshot_portal;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "linux")]
pub use screenshot_portal::{ScreenshotError, ScreenshotPortalService};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "windows")]
mod screenshot_windows;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(target_os = "windows")]
pub use screenshot_windows::{ScreenshotError, ScreenshotPortalService};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
mod screenshot_fallback;
#[cfg(not(target_arch = "wasm32"))]
#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub use screenshot_fallback::{ScreenshotError, ScreenshotPortalService};

#[derive(Debug, Clone)]
pub struct ScreenshotImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum PlatformEvent {
    ScreenshotReady(Result<ScreenshotImage, ScreenshotError>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreferredColorScheme {
    Light,
    Dark,
}

#[cfg(target_os = "linux")]
pub fn preferred_color_scheme() -> Option<PreferredColorScheme> {
    let output = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_gsettings_color_scheme(std::str::from_utf8(&output.stdout).ok()?)
}

#[cfg(not(target_os = "linux"))]
pub fn preferred_color_scheme() -> Option<PreferredColorScheme> {
    None
}

#[cfg(target_os = "linux")]
fn parse_gsettings_color_scheme(value: &str) -> Option<PreferredColorScheme> {
    match value.trim().trim_matches('\'') {
        "prefer-dark" => Some(PreferredColorScheme::Dark),
        "prefer-light" => Some(PreferredColorScheme::Light),
        _ => None,
    }
}

#[cfg(all(target_os = "windows", not(target_arch = "wasm32")))]
pub fn set_window_dark_mode(hwnd: *mut std::ffi::c_void, dark: bool) {
    #[link(name = "dwmapi")]
    unsafe extern "system" {
        fn DwmSetWindowAttribute(
            hwnd: *mut std::ffi::c_void,
            dw_attribute: u32,
            pv_attribute: *const std::ffi::c_void,
            cb_attribute: u32,
        ) -> i32;
    }
    const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
    let val: i32 = if dark { 1 } else { 0 };
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &val as *const i32 as *const std::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );

        const WM_NCACTIVATE: u32 = 0x0086;
        const RDW_FRAME: u32 = 0x0400;
        const RDW_INVALIDATE: u32 = 0x0001;
        const RDW_UPDATENOW: u32 = 0x0100;

        #[link(name = "user32")]
        unsafe extern "system" {
            fn SendMessageW(
                hwnd: *mut std::ffi::c_void,
                msg: u32,
                wparam: usize,
                lparam: isize,
            ) -> isize;
            fn RedrawWindow(
                hwnd: *mut std::ffi::c_void,
                rc: *const std::ffi::c_void,
                rgn: *mut std::ffi::c_void,
                flags: u32,
            ) -> i32;
        }

        SendMessageW(hwnd, WM_NCACTIVATE, 0, 0);
        SendMessageW(hwnd, WM_NCACTIVATE, 1, 0);
        RedrawWindow(
            hwnd,
            std::ptr::null(),
            std::ptr::null_mut(),
            RDW_FRAME | RDW_INVALIDATE | RDW_UPDATENOW,
        );
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::{PreferredColorScheme, parse_gsettings_color_scheme};

    #[test]
    fn parses_gsettings_color_scheme() {
        assert_eq!(
            parse_gsettings_color_scheme("'prefer-dark'\n"),
            Some(PreferredColorScheme::Dark)
        );
        assert_eq!(
            parse_gsettings_color_scheme("'prefer-light'\n"),
            Some(PreferredColorScheme::Light)
        );
        assert_eq!(parse_gsettings_color_scheme("'default'\n"), None);
    }
}
