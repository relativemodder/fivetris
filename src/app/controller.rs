use web_time::{Duration, Instant};

use crate::app::AppState;
use crate::app::actions::AppAction;
use crate::core::game_loop::SoundEffect;
use crate::core::GameMode;
use crate::core::input::{HorizontalDirection, InputRepeatState};
use crate::core::piece::RotationCommand;

pub struct GameController {
    repeat_state: InputRepeatState,
}

impl Default for GameController {
    fn default() -> Self {
        Self {
            repeat_state: InputRepeatState::default(),
        }
    }
}

impl GameController {
    pub fn clear_repeat_state(&mut self) {
        self.repeat_state.clear();
    }

    fn with_active_gameplay(state: &mut AppState, action: impl FnOnce(&mut AppState)) {
        if state.is_gameplay_active() {
            action(state);
        }
    }

    fn next_round_robin_mode(mode: GameMode) -> GameMode {
        match mode {
            GameMode::Training => GameMode::Cheese,
            GameMode::Cheese => GameMode::FourWide,
            GameMode::FourWide => GameMode::PerfectClear,
            GameMode::PerfectClear => GameMode::Master,
            GameMode::Master | GameMode::Edit => GameMode::Training,
        }
    }

    pub fn dispatch(&mut self, state: &mut AppState, action: AppAction) {
        match action {
            AppAction::CopyState
            | AppAction::PasteState
            | AppAction::RequestScreenshot
            | AppAction::ToggleGhost(_)
            | AppAction::SetVolume(_)
            | AppAction::SelectSkin(_)
            | AppAction::SelectTexture(_)
            | AppAction::ApplySettings
            | AppAction::ConfirmSettings
            | AppAction::CancelSettings
            | AppAction::BeginBrushStroke => {}
            AppAction::TogglePause => {
                state.paused = !state.paused;
                self.clear_repeat_state();
            }
            AppAction::Reset(mode) => {
                state.game_loop.reset(mode);
                state.paused = false;
                self.clear_repeat_state();
            }
            AppAction::ResetCurrent => {
                let mode = state.game_loop.game.mode;
                state.game_loop.reset(mode);
                state.paused = false;
                self.clear_repeat_state();
            }
            AppAction::CycleMode => {
                let mode = Self::next_round_robin_mode(state.game_loop.game.mode);
                state.game_loop.reset(mode);
                state.paused = false;
                self.clear_repeat_state();
            }
            AppAction::MoveLeftPress(now) => {
                if !state.is_gameplay_active() {
                    return;
                }
                let das_ms = state.game_loop.game.gravity.das_ms;
                let das_cancel = state.game_loop.game.gravity.das_cancel;
                if state.game_loop.try_move_piece(-1, 0) {
                    state.game_loop.pending_sounds.push(SoundEffect::Move);
                }
                self.repeat_state.press_horizontal(
                    HorizontalDirection::Left,
                    now,
                    das_ms,
                    das_cancel,
                );
            }
            AppAction::MoveLeftRelease => {
                let das_ms = state.game_loop.game.gravity.das_ms;
                self.repeat_state
                    .release_horizontal(HorizontalDirection::Left, das_ms);
            }
            AppAction::MoveRightPress(now) => {
                if !state.is_gameplay_active() {
                    return;
                }
                let das_ms = state.game_loop.game.gravity.das_ms;
                let das_cancel = state.game_loop.game.gravity.das_cancel;
                if state.game_loop.try_move_piece(1, 0) {
                    state.game_loop.pending_sounds.push(SoundEffect::Move);
                }
                self.repeat_state.press_horizontal(
                    HorizontalDirection::Right,
                    now,
                    das_ms,
                    das_cancel,
                );
            }
            AppAction::MoveRightRelease => {
                let das_ms = state.game_loop.game.gravity.das_ms;
                self.repeat_state
                    .release_horizontal(HorizontalDirection::Right, das_ms);
            }
            AppAction::SoftDropPress(now) => {
                if !state.is_gameplay_active() {
                    return;
                }
                let sdd_ms = state.game_loop.game.gravity.sdd_ms;
                if state.game_loop.try_move_piece(0, 1) {
                    state.game_loop.pending_sounds.push(SoundEffect::Move);
                }
                self.repeat_state.press_soft_drop(now, sdd_ms);
            }
            AppAction::SoftDropRelease => self.repeat_state.release_soft_drop(),
            AppAction::HardDrop => {
                Self::with_active_gameplay(state, |state| state.game_loop.hard_drop());
            }
            AppAction::RotateCcw => {
                Self::with_active_gameplay(state, |state| {
                    state.game_loop.try_rotate_piece(RotationCommand::Ccw);
                });
            }
            AppAction::RotateCw => {
                Self::with_active_gameplay(state, |state| {
                    state.game_loop.try_rotate_piece(RotationCommand::Cw);
                });
            }
            AppAction::Rotate180 => {
                Self::with_active_gameplay(state, |state| {
                    state.game_loop.try_rotate_piece(RotationCommand::Flip);
                });
            }
            AppAction::Hold => {
                Self::with_active_gameplay(state, |state| {
                    let _ = state.game_loop.hold_swap();
                });
            }
            AppAction::Undo => {
                Self::with_active_gameplay(state, |state| {
                    state.game_loop.undo();
                });
            }
            AppAction::Redo => {
                Self::with_active_gameplay(state, |state| {
                    state.game_loop.redo();
                });
            }
            AppAction::SetEditColor(_)
            | AppAction::ClearBoard
            | AppAction::ToggleHighlightMode
            | AppAction::ClearHighlight
            | AppAction::ToggleAutoColor(_)
            | AppAction::ToggleAutoLockOnGround(_)
            | AppAction::EditCell(_, _, _)
            | AppAction::EditHighlightCell(_, _, _)
            | AppAction::StartBagEdit
            | AppAction::ApplyBagEdit(_)
            | AppAction::CancelBagEdit
            | AppAction::StartHoldEdit(_)
            | AppAction::ApplyHoldEdit(_)
            | AppAction::CancelHoldEdit => {}
        }
    }

    pub fn advance_timers(&mut self, state: &mut AppState, now: Instant) {
        if !state.is_gameplay_active() {
            return;
        }

        let horizontal_arr_ms = state.game_loop.game.gravity.arr_ms;
        let soft_drop_interval_ms = state.game_loop.game.gravity.sds_ms;
        if let Some(direction) = self.repeat_state.active_horizontal {
            let key_state = self.repeat_state.horizontal_mut(direction);
            if key_state.pressed {
                let delta = match direction {
                    HorizontalDirection::Left => -1,
                    HorizontalDirection::Right => 1,
                };
                Self::advance_repeat(now, key_state, horizontal_arr_ms, |instant_mode| {
                    if instant_mode {
                        while state.game_loop.try_move_piece(delta, 0) {}
                        false
                    } else {
                        state.game_loop.try_move_piece(delta, 0)
                    }
                });
            }
        }

        let soft_drop_state = &mut self.repeat_state.soft_drop;
        if soft_drop_state.pressed {
            Self::advance_repeat(
                now,
                soft_drop_state,
                soft_drop_interval_ms,
                |instant_mode| {
                    if instant_mode {
                        while state.game_loop.try_move_piece(0, 1) {}
                        false
                    } else {
                        state.game_loop.try_move_piece(0, 1)
                    }
                },
            );
        }
    }

    fn advance_repeat(
        now: Instant,
        key_state: &mut crate::core::input::RepeatKeyState,
        interval_ms: u32,
        mut step: impl FnMut(bool) -> bool,
    ) {
        if interval_ms == 0 {
            if key_state.next_repeat_at < now {
                step(true);
                key_state.next_repeat_at = now;
            }
            return;
        }

        let interval = Duration::from_millis(u64::from(interval_ms));
        while key_state.next_repeat_at < now {
            if !step(false) {
                break;
            }
            key_state.next_repeat_at += interval;
        }
    }
}

#[cfg(test)]
mod tests {
    use web_time::{Duration, Instant};

    use super::GameController;
    use crate::app::actions::AppAction;
    use crate::app::test::test_state;
    use crate::core::GameMode;

    #[test]
    fn cycle_mode_uses_round_robin_order() {
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::Training),
            GameMode::Cheese
        );
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::Cheese),
            GameMode::FourWide
        );
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::FourWide),
            GameMode::PerfectClear
        );
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::PerfectClear),
            GameMode::Master
        );
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::Master),
            GameMode::Training
        );
        assert_eq!(
            GameController::next_round_robin_mode(GameMode::Edit),
            GameMode::Training
        );
    }

    #[test]
    fn horizontal_repeat_starts_after_das() {
        let mut controller = GameController::default();
        let mut state = test_state();
        state.game_loop.game.gravity.das_ms = 120;
        state.game_loop.game.gravity.arr_ms = 40;
        let start_x = state.game_loop.game.current.x;
        let now = Instant::now();

        controller.dispatch(&mut state, AppAction::MoveLeftPress(now));
        assert_eq!(state.game_loop.game.current.x, start_x - 1);

        controller.advance_timers(&mut state, now + Duration::from_millis(120));
        assert_eq!(state.game_loop.game.current.x, start_x - 1);

        controller.advance_timers(&mut state, now + Duration::from_millis(121));
        assert_eq!(state.game_loop.game.current.x, start_x - 2);

        controller.advance_timers(&mut state, now + Duration::from_millis(161));
        assert_eq!(state.game_loop.game.current.x, start_x - 3);
    }

    #[test]
    fn releasing_left_hands_repeat_to_held_right() {
        let mut controller = GameController::default();
        let mut state = test_state();
        state.game_loop.game.gravity.das_ms = 120;
        state.game_loop.game.gravity.arr_ms = 40;
        state.game_loop.game.gravity.das_cancel = true;
        let start_x = state.game_loop.game.current.x;
        let now = Instant::now();

        controller.dispatch(&mut state, AppAction::MoveLeftPress(now));
        controller.dispatch(
            &mut state,
            AppAction::MoveRightPress(now + Duration::from_millis(10)),
        );
        controller.dispatch(&mut state, AppAction::MoveLeftRelease);

        controller.advance_timers(&mut state, now + Duration::from_millis(131));
        assert_eq!(state.game_loop.game.current.x, start_x + 1);
    }

    #[test]
    fn zero_arr_shifts_until_blocked() {
        let mut controller = GameController::default();
        let mut state = test_state();
        state.game_loop.game.gravity.das_ms = 0;
        state.game_loop.game.gravity.arr_ms = 0;
        let now = Instant::now();

        controller.dispatch(&mut state, AppAction::MoveLeftPress(now));
        controller.advance_timers(&mut state, now + Duration::from_millis(1));

        assert_eq!(state.game_loop.game.current.x, 0);
    }
}
