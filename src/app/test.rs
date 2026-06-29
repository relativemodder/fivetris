use crate::app::ui_state::UiState;
use crate::app::AppState;
use crate::core::game_loop::GameLoop;
use crate::core::{BagMode, GameMode};

#[cfg(test)]
pub(crate) fn test_state() -> AppState {
    let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 0, true);
    game_loop.spawn_next();
    AppState {
        game_loop,
        ui_state: UiState::default(),
        paused: false,
    }
}
