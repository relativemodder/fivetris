use std::fmt;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;

pub struct SystemClipboard;

pub trait Clipboard {
    fn set_text(&self, text: String) -> Result<(), ClipboardError>;
    fn get_text(&self) -> Result<String, ClipboardError>;
}

#[derive(Debug)]
pub enum ClipboardError {
    Unavailable(String),
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "clipboard error: {message}"),
        }
    }
}

impl std::error::Error for ClipboardError {}

#[cfg(not(target_arch = "wasm32"))]
impl Clipboard for SystemClipboard {
    fn set_text(&self, text: String) -> Result<(), ClipboardError> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))?;
        clipboard
            .set_text(text)
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))
    }

    fn get_text(&self) -> Result<String, ClipboardError> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))?;
        clipboard
            .get_text()
            .map_err(|error| ClipboardError::Unavailable(error.to_string()))
    }
}

#[cfg(target_arch = "wasm32")]
impl Clipboard for SystemClipboard {
    fn set_text(&self, text: String) -> Result<(), ClipboardError> {
        static STORED: Mutex<Option<String>> = Mutex::new(None);
        *STORED.lock().map_err(|e| ClipboardError::Unavailable(e.to_string()))? = Some(text);
        Ok(())
    }

    fn get_text(&self) -> Result<String, ClipboardError> {
        static STORED: Mutex<Option<String>> = Mutex::new(None);
        STORED
            .lock()
            .map_err(|e| ClipboardError::Unavailable(e.to_string()))?
            .clone()
            .ok_or_else(|| ClipboardError::Unavailable("clipboard is empty".to_string()))
    }
}

impl SystemClipboard {
    pub fn write_text(text: &str) -> Result<(), ClipboardError> {
        <Self as Clipboard>::set_text(&Self, text.to_string())
    }

    pub fn read_text() -> Result<String, ClipboardError> {
        <Self as Clipboard>::get_text(&Self)
    }
}
