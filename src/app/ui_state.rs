use crate::config::AppConfig;
use crate::core::Cell;

pub use super::actions::AppAction;

#[derive(Debug, Clone)]
pub struct UiState {
    pub board_cell_size: f32,
    pub preview_cell_size: f32,
    pub preview_slots: usize,
    pub settings_open: bool,
    pub was_paused_before_settings: bool,
    pub status_message: Option<String>,
    pub shift_was_down: bool,
    pub edit_color: Cell,
    pub highlight_mode: bool,
    pub pending_config: Option<AppConfig>,
    pub bag_edit_open: bool,
    pub hold_edit_open: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            board_cell_size: 28.0,
            preview_cell_size: 14.0,
            preview_slots: 5,
            settings_open: false,
            was_paused_before_settings: false,
            status_message: None,
            shift_was_down: false,
            edit_color: Cell::Gray,
            highlight_mode: false,
            pending_config: None,
            bag_edit_open: false,
            hold_edit_open: false,
        }
    }
}
