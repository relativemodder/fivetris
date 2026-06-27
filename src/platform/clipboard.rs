use std::fmt;

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

impl SystemClipboard {
    pub fn write_text(text: &str) -> Result<(), ClipboardError> {
        <Self as Clipboard>::set_text(&Self, text.to_string())
    }

    pub fn read_text() -> Result<String, ClipboardError> {
        <Self as Clipboard>::get_text(&Self)
    }
}
