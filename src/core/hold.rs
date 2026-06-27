use super::game_state::GameState;
use super::piece::Rotation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HoldError {
    Locked,
}

pub fn hold_swap(game: &mut GameState) -> Result<(), HoldError> {
    if game.hold.swapped_this_turn && !game.hold.infinite_hold {
        return Err(HoldError::Locked);
    }

    let width = game.board.width;
    let hidden_rows = game.board.hidden_rows;

    game.current.x = (width as i32) / 2 - 2;
    game.current.y = hidden_rows as i32 - 2;
    game.current.rotation = Rotation::Spawn;

    let current_kind = game.current.kind;

    if let Some(held_piece) = game.hold.piece {
        game.hold.piece = Some(current_kind);
        game.current.kind = held_piece;
    } else {
        game.hold.piece = Some(current_kind);
        if !game.queue.visible.is_empty() {
            game.current.kind = game.queue.visible.remove(0);
        }
    }

    if !game.hold.infinite_hold {
        game.hold.swapped_this_turn = true;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::core::game_state::GameState;
    use crate::core::piece::Tetromino;
    use crate::core::test_support;

    fn make_game_state() -> GameState {
        let mut game = test_support::game_state();
        game.board = Board::new(10, 20, 4);
        game.highlights.sync_to_board(&game.board);
        game.current.y = 22;
        game.hold.infinite_hold = false;
        game.queue.visible = vec![Tetromino::I, Tetromino::J, Tetromino::S];
        game
    }

    #[test]
    fn hold_swap_into_empty_hold() {
        let mut game = make_game_state();
        assert_eq!(game.hold.piece, None);
        assert!(game.current.kind == Tetromino::T);

        let result = hold_swap(&mut game);
        assert!(result.is_ok());

        assert_eq!(game.hold.piece, Some(Tetromino::T));
        assert_eq!(game.current.kind, Tetromino::I);
    }

    #[test]
    fn hold_swap_returns_previous_held_piece() {
        let mut game = make_game_state();
        game.hold.piece = Some(Tetromino::L);
        game.current.kind = Tetromino::S;

        let result = hold_swap(&mut game);
        assert!(result.is_ok());

        assert_eq!(game.hold.piece, Some(Tetromino::S));
        assert_eq!(game.current.kind, Tetromino::L);
    }

    #[test]
    fn hold_swap_locked_when_swapped_this_turn() {
        let mut game = make_game_state();
        game.hold.swapped_this_turn = true;
        game.hold.infinite_hold = false;

        let result = hold_swap(&mut game);
        assert_eq!(result, Err(HoldError::Locked));
        assert_eq!(game.current.kind, Tetromino::T);
        assert_eq!(game.hold.piece, None);
    }

    #[test]
    fn hold_swap_allows_repeat_with_infinite_hold() {
        let mut game = make_game_state();
        game.hold.swapped_this_turn = true;
        game.hold.infinite_hold = true;
        game.hold.piece = Some(Tetromino::J);
        game.current.kind = Tetromino::L;

        let result = hold_swap(&mut game);
        assert!(result.is_ok());
        assert_eq!(game.hold.piece, Some(Tetromino::L));
        assert_eq!(game.current.kind, Tetromino::J);
        assert!(game.hold.swapped_this_turn);
    }

    #[test]
    fn hold_swap_sets_swapped_flag() {
        let mut game = make_game_state();
        game.hold.infinite_hold = false;
        game.hold.piece = Some(Tetromino::O);

        let result = hold_swap(&mut game);
        assert!(result.is_ok());
        assert!(game.hold.swapped_this_turn);
    }

    #[test]
    fn hold_swap_does_not_set_swapped_flag_with_infinite_hold() {
        let mut game = make_game_state();
        game.hold.infinite_hold = true;
        game.hold.piece = Some(Tetromino::O);

        let result = hold_swap(&mut game);
        assert!(result.is_ok());
        assert!(!game.hold.swapped_this_turn);
    }

    #[test]
    fn hold_swap_resets_piece_position() {
        let mut game = make_game_state();
        game.current.x = 0;
        game.current.y = 0;
        game.current.rotation = Rotation::Right;

        let result = hold_swap(&mut game);
        assert!(result.is_ok());

        assert_eq!(game.current.x, 3);
        assert_eq!(game.current.y, 2);
        assert_eq!(game.current.rotation, Rotation::Spawn);
    }

    #[test]
    fn hold_swap_pops_from_queue_when_hold_empty() {
        let mut game = make_game_state();
        game.queue.visible = vec![Tetromino::I, Tetromino::O, Tetromino::Z];
        game.current.kind = Tetromino::T;

        let result = hold_swap(&mut game);
        assert!(result.is_ok());

        assert_eq!(game.hold.piece, Some(Tetromino::T));
        assert_eq!(game.current.kind, Tetromino::I);
        assert_eq!(game.queue.visible, vec![Tetromino::O, Tetromino::Z]);
    }
}
