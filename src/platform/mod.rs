pub mod clipboard;
pub mod screenshot_analyzer;

pub use clipboard::{Clipboard, ClipboardError, SystemClipboard};
pub use screenshot_analyzer::{
    AnalysisError, BoardAnalysisResult, ImportError, ScreenshotAnalysisConfig, analyze_board_image,
    import_board_from_analysis,
};

#[cfg(target_os = "linux")]
pub mod screenshot_portal;
#[cfg(target_os = "linux")]
pub use screenshot_portal::{ScreenshotError, ScreenshotPortalService, ScreenshotRequester};

#[cfg(not(target_os = "linux"))]
mod screenshot_fallback;
#[cfg(not(target_os = "linux"))]
pub use screenshot_fallback::{ScreenshotError, ScreenshotPortalService, ScreenshotRequester};

#[derive(Debug, Clone)]
pub struct ScreenshotImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    ScreenshotReady(Result<ScreenshotImage, ScreenshotError>),
}
