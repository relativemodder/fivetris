use std::fmt;
use std::path::PathBuf;

use ashpd::desktop::screenshot::Screenshot;
use crossbeam_channel::Sender;
use image::GenericImageView;
use tokio::runtime::{Handle, Runtime};

use super::{PlatformEvent, ScreenshotImage};

pub trait ScreenshotRequester {
    fn request_interactive_capture(&self);
}

#[derive(Debug, Clone)]
pub enum ScreenshotError {
    Portal(String),
    InvalidUri(String),
    Decode(String),
}

impl fmt::Display for ScreenshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Portal(message) => write!(f, "portal error: {message}"),
            Self::InvalidUri(message) => write!(f, "invalid screenshot URI: {message}"),
            Self::Decode(message) => write!(f, "screenshot decode error: {message}"),
        }
    }
}

impl std::error::Error for ScreenshotError {}

pub struct ScreenshotPortalService {
    tx: Sender<PlatformEvent>,
    handle: Option<Handle>,
    _runtime: Option<Runtime>,
}

impl ScreenshotPortalService {
    pub fn new(tx: Sender<PlatformEvent>) -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let handle = runtime.handle().clone();

        Ok(Self {
            tx,
            handle: Some(handle),
            _runtime: Some(runtime),
        })
    }

    pub fn unavailable(tx: Sender<PlatformEvent>) -> Self {
        Self {
            tx,
            handle: None,
            _runtime: None,
        }
    }

    pub fn is_available(&self) -> bool {
        self.handle.is_some()
    }
}

impl ScreenshotRequester for ScreenshotPortalService {
    fn request_interactive_capture(&self) {
        let Some(handle) = &self.handle else {
            let _ = self.tx.send(PlatformEvent::ScreenshotReady(Err(
                ScreenshotError::Portal("screenshot service unavailable".to_string()),
            )));
            return;
        };
        let tx = self.tx.clone();
        handle.spawn(async move {
            let result = capture_screenshot().await;
            let _ = tx.send(PlatformEvent::ScreenshotReady(result));
        });
    }
}

async fn capture_screenshot() -> Result<ScreenshotImage, ScreenshotError> {
    let response = Screenshot::request()
        .interactive(true)
        .modal(true)
        .send()
        .await
        .map_err(|error| ScreenshotError::Portal(error.to_string()))?
        .response()
        .map_err(|error| ScreenshotError::Portal(error.to_string()))?;

    let image_path = response
        .uri()
        .to_file_path()
        .map_err(|()| ScreenshotError::InvalidUri(response.uri().to_string()))?;

    load_screenshot_image(&image_path)
}

fn load_screenshot_image(path: &PathBuf) -> Result<ScreenshotImage, ScreenshotError> {
    let image = image::open(path).map_err(|error| ScreenshotError::Decode(error.to_string()))?;
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8().into_raw();

    Ok(ScreenshotImage {
        width,
        height,
        rgba,
    })
}
