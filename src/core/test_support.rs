use super::board::Board;
use super::game_state::{
    BagMode, ClearFlashState, GameMode, GameState, GravityState, HoldState, LockDelayState,
    QueueState, SpinState, Stats,
};
use super::highlight::HighlightBoard;
use super::history::UndoSnapshot;
use super::piece::{PieceState, Rotation, Tetromino};

pub fn game_state() -> GameState {
    let board = Board::new(10, 20, 4);
    GameState {
        mode: GameMode::Training,
        highlights: HighlightBoard::new(10, board.total_height()),
        current: PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 0,
        },
        hold: HoldState::default(),
        queue: QueueState::default(),
        stats: Stats::default(),
        spin: SpinState::default(),
        gravity: GravityState::default(),
        ghost_piece: true,
        auto_color: false,
        auto_lock_on_ground: false,
        mirror_queue_with_field: false,
        highlight_clear: false,
        pc_leftover: 0,
        garbage_hole_pos: 0,
        garbage_hole_size: 0,
        garbage_hole_next_change: 0,
        lock_delay: LockDelayState::default(),
        clear_flash: ClearFlashState::default(),
        board,
    }
}

pub fn undo_snapshot() -> UndoSnapshot {
    let game = game_state();
    UndoSnapshot {
        game,
        queue_generator: super::bag::QueueGenerator::new(BagMode::SevenBag, 0),
        gravity_accum: 0,
    }
}
