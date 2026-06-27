use std::fmt;

use crossbeam_channel::Sender;

use super::PlatformEvent;

pub trait ScreenshotRequester {
    fn request_interactive_capture(&self);
}

#[derive(Debug, Clone)]
pub enum ScreenshotError {
    Portal(String),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Portal(message) => write!(f, "portal error: {message}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}

pub struct ScreenshotPortalService {
    tx: Sender<PlatformEvent>,
}

impl ScreenshotPortalService {
    pub fn new(tx: Sender<PlatformEvent>) -> Result<Self, std::io::Error> {
        Ok(Self { tx })
    }

    pub fn unavailable(tx: Sender<PlatformEvent>) -> Self {
        Self { tx }
    }

    pub fn is_available(&self) -> bool {
        false
    }
}

impl ScreenshotRequester for ScreenshotPortalService {
    fn request_interactive_capture(&self) {
        let _ = self.tx.send(PlatformEvent::ScreenshotReady(Err(
            ScreenshotError::Portal(
                "screenshot capture is not supported on this platform".to_string(),
            ),
        )));
    }
}
