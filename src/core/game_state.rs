use serde::{Deserialize, Serialize};

use super::board::{Board, HighlightBoard};
use super::piece::{PieceState, Tetromino};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineClearKind {
    None,
    Single,
    Double,
    Triple,
    Tetris,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpinKind {
    None,
    Mini,
    TSpin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearOutcome {
    pub lines_cleared: u8,
    pub line_clear_kind: LineClearKind,
    pub spin_kind: SpinKind,
    pub perfect_clear: bool,
    pub damage_delta: i32,
    pub combo_after: u32,
    pub b2b_after: bool,
    pub attack_text: Option<String>,
    pub b2b_text: Option<String>,
    pub cleared_lines: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BagMode {
    SevenBag,
    FourteenBag,
    Random,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    Training,
    Cheese,
    FourWide,
    PerfectClear,
    Master,
    Edit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoldState {
    pub piece: Option<Tetromino>,
    pub swapped_this_turn: bool,
    pub infinite_hold: bool,
}

impl Default for HoldState {
    fn default() -> Self {
        Self {
            piece: None,
            swapped_this_turn: false,
            infinite_hold: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueState {
    pub visible: Vec<Tetromino>,
    pub seed: u16,
    pub mode: BagMode,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            visible: Vec::new(),
            seed: 0,
            mode: BagMode::SevenBag,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub damage: u32,
    pub lines: u32,
    pub moves: u32,
    pub b2b: bool,
    pub perfect_clear: bool,
    pub combo: u32,
    pub lost: bool,
    pub attack_text: Option<String>,
    pub b2b_text: Option<String>,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            damage: 0,
            lines: 0,
            moves: 0,
            b2b: false,
            perfect_clear: false,
            combo: 0,
            lost: false,
            attack_text: None,
            b2b_text: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpinState {
    pub is_t_spin: bool,
    pub is_mini: bool,
    pub last_kick_index: Option<usize>,
}

impl Default for SpinState {
    fn default() -> Self {
        Self {
            is_t_spin: false,
            is_mini: false,
            last_kick_index: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GravityState {
    pub gravity_hz: Option<u32>,
    pub arr_ms: u32,
    pub das_ms: u32,
    pub sdd_ms: u32,
    pub sds_ms: u32,
    pub das_cancel: bool,
}

impl Default for GravityState {
    fn default() -> Self {
        Self {
            gravity_hz: None,
            arr_ms: 0,
            das_ms: 120,
            sdd_ms: 0,
            sds_ms: 0,
            das_cancel: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockDelayState {
    pub active: bool,
    pub timer_ms: u32,
    pub moves: u8,
    pub max_moves: u8,
    pub delay_ms: u32,
}

impl LockDelayState {
    pub fn reset(&mut self) {
        self.active = false;
        self.timer_ms = 0;
        self.moves = 0;
    }
}

impl Default for LockDelayState {
    fn default() -> Self {
        Self {
            active: false,
            timer_ms: 0,
            moves: 0,
            max_moves: 15,
            delay_ms: 500,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearFlashState {
    pub active: bool,
    pub lines: Vec<usize>,
    pub timer_ms: u32,
    pub duration_ms: u32,
}

impl Default for ClearFlashState {
    fn default() -> Self {
        Self {
            active: false,
            lines: Vec::new(),
            timer_ms: 0,
            duration_ms: 300,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    pub mode: GameMode,
    pub board: Board,
    pub highlights: HighlightBoard,
    pub current: PieceState,
    pub hold: HoldState,
    pub queue: QueueState,
    pub stats: Stats,
    pub spin: SpinState,
    pub gravity: GravityState,
    pub ghost_piece: bool,
    pub auto_color: bool,
    pub auto_lock_on_ground: bool,
    pub mirror_queue_with_field: bool,
    pub highlight_clear: bool,
    pub pc_leftover: usize,
    pub garbage_hole_pos: usize,
    pub garbage_hole_size: usize,
    pub garbage_hole_next_change: usize,
    pub lock_delay: LockDelayState,
    pub clear_flash: ClearFlashState,
}

impl GameState {
    pub fn is_lost(&self) -> bool {
        self.stats.lost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::piece::Tetromino;

    #[test]
    fn hold_state_default_construction() {
        let h = HoldState::default();
        assert!(h.piece.is_none());
        assert!(!h.swapped_this_turn);
        assert!(h.infinite_hold);
    }

    #[test]
    fn hold_state_with_piece() {
        let h = HoldState {
            piece: Some(Tetromino::T),
            swapped_this_turn: true,
            infinite_hold: false,
        };
        assert_eq!(h.piece, Some(Tetromino::T));
    }

    #[test]
    fn queue_state_construction() {
        let q = QueueState {
            visible: vec![Tetromino::I, Tetromino::J, Tetromino::T],
            seed: 42,
            ..Default::default()
        };
        assert_eq!(q.visible.len(), 3);
        assert_eq!(q.seed, 42);
    }

    #[test]
    fn stats_defaults() {
        let s = Stats::default();
        assert_eq!(s.damage, 0);
        assert!(!s.lost);
    }

    #[test]
    fn spin_state_default() {
        let s = SpinState::default();
        assert!(!s.is_t_spin);
        assert!(!s.is_mini);
        assert!(s.last_kick_index.is_none());
    }

    #[test]
    fn gravity_state_default() {
        let g = GravityState::default();
        assert!(g.gravity_hz.is_none());
        assert_eq!(g.das_ms, 120);
    }

    #[test]
    fn lock_delay_state_defaults() {
        let ld = LockDelayState::default();
        assert!(!ld.active);
        assert_eq!(ld.delay_ms, 500);
        assert_eq!(ld.max_moves, 15);
    }

    #[test]
    fn clear_flash_state_defaults() {
        let cf = ClearFlashState::default();
        assert!(!cf.active);
        assert!(cf.lines.is_empty());
        assert_eq!(cf.duration_ms, 300);
    }
}
