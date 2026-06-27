pub mod bag;
pub mod board;
pub mod codec;
pub mod game_loop;
pub mod game_state;
pub mod highlight;
pub mod history;
pub mod hold;
pub mod input;
pub mod modes;
pub mod piece;
pub mod rotation;
pub mod scoring;

#[cfg(test)]
pub mod test_support;

pub use bag::QueueGenerator;
pub use board::Board;
pub use codec::{
    CodecError, decode_json_state, decode_legacy_clipboard, encode_json_state,
};
pub use game_state::{
    BagMode, ClearFlashState, ClearOutcome, GameMode, GameState, GravityState, HoldState,
    LineClearKind, LockDelayState, QueueState, SpinKind, SpinState, Stats,
};
pub use highlight::{HighlightBoard, auto_color_board, piece_from_coords};
pub use history::{History, UndoSnapshot};
pub use hold::{HoldError, hold_swap};
pub use input::InputRepeatState;
pub use modes::{mirror_field_and_queue, reset_for_mode};
pub use piece::{
    Cell, DEFAULT_BOARD_WIDTH, DEFAULT_HIDDEN_ROWS, DEFAULT_VISIBLE_HEIGHT, MAX_BOARD_WIDTH,
    MAX_VISIBLE_HEIGHT, PieceState, Rotation, RotationCommand, Tetromino, cell_from_piece,
    mirrored_cell, mirrored_piece, piece_from_name, piece_name, piece_shape,
};
pub use rotation::{
    detect_t_spin, detect_t_spin_mini, fits, freeze_active_piece, try_move, try_rotate,
};
pub use scoring::resolve_lock_and_clears;
