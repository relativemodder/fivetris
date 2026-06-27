use super::board::Board;
use super::game_state::GameState;
use super::piece::{
    PieceState, Rotation, RotationCommand, Tetromino, piece_shape,
};

type KickSlice = &'static [(i8, i8)];

const I_CCW: &[[(i8, i8); 4]; 4] = &[
    [(2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(-2, 0), (1, 0), (-2, 1), (1, -2)],
    [(1, 0), (-2, 0), (1, 2), (-2, 1)],
];

const I_CW: &[[(i8, i8); 4]; 4] = &[
    [(1, 0), (-2, 0), (1, 2), (-2, -1)],
    [(2, 0), (-1, 0), (2, -1), (-1, 2)],
    [(-1, 0), (2, 0), (-1, -2), (2, 1)],
    [(-2, 0), (1, 0), (-2, 1), (1, -2)],
];

const NON_I_CCW: &[[(i8, i8); 4]; 4] = &[
    [(1, 0), (1, 1), (0, -2), (1, -2)],
    [(1, 0), (1, -1), (0, 2), (1, 2)],
    [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
];

const NON_I_CW: &[[(i8, i8); 4]; 4] = &[
    [(-1, 0), (-1, 1), (0, -2), (-1, -2)],
    [(1, 0), (1, -1), (0, 2), (1, 2)],
    [(1, 0), (1, 1), (0, -2), (1, -2)],
    [(-1, 0), (-1, -1), (0, 2), (-1, 2)],
];

const NON_I_180: &[[(i8, i8); 5]; 4] = &[
    [(0, 1), (-1, 1), (1, 1), (-1, 0), (1, 0)],
    [(1, 0), (1, -2), (1, -1), (0, -2), (0, -1)],
    [(0, -1), (1, -1), (-1, -1), (1, 0), (-1, 0)],
    [(-1, 0), (-1, 2), (-1, -1), (0, -2), (0, -1)],
];

fn fits_collision_reason(board: &Board, piece: PieceState) -> Option<(i32, i32, &'static str)> {
    let shape = piece_shape(piece.kind, piece.rotation);
    for x in 0..4i32 {
        for y in 0..4i32 {
            if !shape[x as usize][y as usize] {
                continue;
            }

            let bx = piece.x + x;
            let by = piece.y + y;
            if !board.in_bounds(bx, by) {
                return Some((bx, by, "oob"));
            }
            if board.is_occupied_or_oob(bx, by) {
                return Some((bx, by, "occupied"));
            }
        }
    }
    None
}

fn kick_offsets(kind: Tetromino, to: Rotation, command: RotationCommand) -> Option<KickSlice> {
    let idx = to as usize;
    match (kind, command) {
        (Tetromino::O | Tetromino::Mono, _) => None,
        (Tetromino::I, RotationCommand::Ccw) => Some(&I_CCW[idx][..]),
        (Tetromino::I, RotationCommand::Cw) => Some(&I_CW[idx][..]),
        (Tetromino::I, _) => None,
        (_, RotationCommand::Ccw) => Some(&NON_I_CCW[idx][..]),
        (_, RotationCommand::Flip) => Some(&NON_I_180[idx][..]),
        (_, RotationCommand::Cw) => Some(&NON_I_CW[idx][..]),
        _ => None,
    }
}

fn target_rotation(current: Rotation, command: RotationCommand) -> Rotation {
    let v = (current as u8 + command as u8) & 3;
    match v {
        0 => Rotation::Spawn,
        1 => Rotation::Left,
        2 => Rotation::Reverse,
        3 => Rotation::Right,
        _ => unreachable!(),
    }
}

pub fn fits(board: &Board, piece: PieceState) -> bool {
    fits_collision_reason(board, piece).is_none()
}

pub fn try_move(game: &mut GameState, dx: i32, dy: i32) -> bool {
    let candidate = PieceState {
        kind: game.current.kind,
        rotation: game.current.rotation,
        x: game.current.x + dx,
        y: game.current.y + dy,
    };
    if fits(&game.board, candidate) {
        game.current.x = candidate.x;
        game.current.y = candidate.y;
        game.spin.last_kick_index = None;
        game.stats.moves += 1;
        true
    } else {
        false
    }
}

pub fn try_rotate(game: &mut GameState, command: RotationCommand) -> bool {
    if command == RotationCommand::None {
        return false;
    }

    let from_rot = game.current.rotation;
    let to_rot = target_rotation(from_rot, command);
    let direct_candidate = PieceState {
        kind: game.current.kind,
        rotation: to_rot,
        x: game.current.x,
        y: game.current.y,
    };
    if fits(&game.board, direct_candidate) {
        game.current.rotation = to_rot;
        game.spin.last_kick_index = None;
        game.spin.is_t_spin = detect_t_spin_inner(game.current, &game.board);
        game.spin.is_mini = detect_t_spin_mini_inner(
            game.spin.is_t_spin,
            game.spin.last_kick_index,
            game.current,
            &game.board,
        );
        game.stats.moves += 1;
        return true;
    }

    let piece_kind = game.current.kind;
    if let Some(offsets) = kick_offsets(piece_kind, to_rot, command) {
        for (i, &(dx, dy)) in offsets.iter().enumerate() {
            let candidate = PieceState {
                kind: piece_kind,
                rotation: to_rot,
                x: game.current.x + dx as i32,
                y: game.current.y + dy as i32,
            };
            if fits(&game.board, candidate) {
                game.current.rotation = to_rot;
                game.current.x = candidate.x;
                game.current.y = candidate.y;
                game.spin.last_kick_index = Some(i);
                game.spin.is_t_spin = detect_t_spin_inner(game.current, &game.board);
                game.spin.is_mini = detect_t_spin_mini_inner(
                    game.spin.is_t_spin,
                    game.spin.last_kick_index,
                    game.current,
                    &game.board,
                );
                game.stats.moves += 1;
                return true;
            }
        }
    }

    false
}

pub fn freeze_active_piece(game: &mut GameState) {
    game.board.freeze_piece(game.current, game.current.kind);
}

fn detect_t_spin_inner(piece: PieceState, board: &Board) -> bool {
    if piece.kind != Tetromino::T {
        return false;
    }
    let (x, y) = (piece.x, piece.y);
    let corners = [(x, y), (x + 2, y), (x, y + 2), (x + 2, y + 2)];
    let occupied = corners
        .iter()
        .filter(|&&(cx, cy)| board.is_occupied_or_oob(cx, cy))
        .count();
    occupied >= 3
}

pub fn detect_t_spin(game: &GameState) -> bool {
    detect_t_spin_inner(game.current, &game.board)
}

fn detect_t_spin_mini_inner(
    is_t_spin: bool,
    last_kick_index: Option<usize>,
    piece: PieceState,
    board: &Board,
) -> bool {
    if !is_t_spin {
        return false;
    }
    if last_kick_index == Some(3) {
        return false;
    }
    let (x, y) = (piece.x, piece.y);
    match piece.rotation {
        Rotation::Spawn => !(board.is_occupied_or_oob(x + 2, y) && board.is_occupied_or_oob(x, y)),
        Rotation::Left => !(board.is_occupied_or_oob(x, y) && board.is_occupied_or_oob(x, y + 2)),
        Rotation::Reverse => {
            !(board.is_occupied_or_oob(x, y + 2) && board.is_occupied_or_oob(x + 2, y + 2))
        }
        Rotation::Right => {
            !(board.is_occupied_or_oob(x + 2, y + 2) && board.is_occupied_or_oob(x + 2, y))
        }
    }
}

pub fn detect_t_spin_mini(game: &GameState) -> bool {
    detect_t_spin_mini_inner(
        game.spin.is_t_spin,
        game.spin.last_kick_index,
        game.current,
        &game.board,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::core::game_state::GameState;
    use crate::core::test_support;

    fn empty_board() -> Board {
        Board::new(10, 20, 4)
    }

    fn default_game() -> GameState {
        let mut game = test_support::game_state();
        game.board = empty_board();
        game.highlights.sync_to_board(&game.board);
        game.current.kind = Tetromino::T;
        game.current.rotation = Rotation::Spawn;
        game.current.x = 3;
        game.current.y = 0;
        game
    }

    #[test]
    fn fits_piece_in_open_space() {
        let b = empty_board();
        let p = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(fits(&b, p));
    }

    #[test]
    fn fits_fails_wall_left() {
        let b = empty_board();
        let p = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: -1,
            y: 10,
        };
        assert!(!fits(&b, p));
    }

    #[test]
    fn fits_fails_wall_right() {
        let b = empty_board();
        let p = PieceState {
            kind: Tetromino::O,
            rotation: Rotation::Spawn,
            x: 9,
            y: 10,
        };
        assert!(!fits(&b, p));
    }

    #[test]
    fn fits_fails_wall_bottom() {
        let b = empty_board();
        let p = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 3,
            y: 23,
        };
        assert!(!fits(&b, p));
    }

    #[test]
    fn fits_fails_overlap() {
        let mut b = empty_board();
        b.set(4, 10, crate::core::piece::Cell::I);
        let p = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 9,
        };
        assert!(!fits(&b, p));
    }

    #[test]
    fn fits_on_floor_passes() {
        let b = empty_board();
        let p = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 21,
        };
        assert!(fits(&b, p));
    }

    #[test]
    fn move_left_success() {
        let mut g = default_game();
        let old_x = g.current.x;
        assert!(try_move(&mut g, -1, 0));
        assert_eq!(g.current.x, old_x - 1);
        assert_eq!(g.current.y, 0);
        assert_eq!(g.spin.last_kick_index, None);
        assert_eq!(g.stats.moves, 1);
    }

    #[test]
    fn move_left_blocked() {
        let mut g = default_game();
        g.current.x = 0;
        g.stats.moves = 0;
        assert!(!try_move(&mut g, -1, 0));
        assert_eq!(g.current.x, 0);
        assert_eq!(g.stats.moves, 0);
    }

    #[test]
    fn move_down_success() {
        let mut g = default_game();
        g.current.y = 10;
        assert!(try_move(&mut g, 0, 1));
        assert_eq!(g.current.y, 11);
    }

    #[test]
    fn rotate_cw_direct() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current.rotation, Rotation::Right);
    }

    #[test]
    fn rotate_ccw_direct() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Ccw));
        assert_eq!(g.current.rotation, Rotation::Left);
    }

    #[test]
    fn rotate_180_direct() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Flip));
        assert_eq!(g.current.rotation, Rotation::Reverse);
    }

    #[test]
    fn rotate_cw_wall_kick_non_i() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: -2,
            y: 10,
        };
        assert!(!try_rotate(&mut g, RotationCommand::Cw));
    }

    #[test]
    fn rotate_cw_wall_kick_non_i_on_floor() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 8,
            y: 20,
        };
        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current.rotation, Rotation::Right);
        assert_eq!(g.current.x, 7);
        assert_eq!(g.current.y, 20);
        assert_eq!(g.spin.last_kick_index, Some(0));
    }

    #[test]
    fn rotate_i_cw_success() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current.rotation, Rotation::Right);
    }

    #[test]
    fn rotate_i_ccw_success() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Ccw));
        assert_eq!(g.current.rotation, Rotation::Left);
    }

    #[test]
    fn rotate_i_cw_wall_kick() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 8,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current.x, 6);
        assert_eq!(g.current.y, 10);
        assert_eq!(g.current.rotation, Rotation::Right);
        assert_eq!(g.spin.last_kick_index, Some(0));
    }

    #[test]
    fn rotate_180_wall_kick() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: -1,
            y: 10,
        };
        assert!(try_rotate(&mut g, RotationCommand::Flip));
        assert_eq!(g.current.x, 0);
        assert_eq!(g.current.y, 9);
        assert_eq!(g.current.rotation, Rotation::Reverse);
        assert_eq!(g.spin.last_kick_index, Some(1));
    }

    #[test]
    fn non_i_180_kick_table_matches_reference() {
        assert_eq!(NON_I_180[3], [(-1, 0), (-1, 2), (-1, -1), (0, -2), (0, -1)]);
    }

    #[test]
    fn i_ccw_kick_table_matches_reference() {
        assert_eq!(I_CCW[3], [(1, 0), (-2, 0), (1, 2), (-2, 1)]);
    }

    #[test]
    fn rotate_i_ccw_uses_target_rotation_kick_table() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 9,
            y: 10,
        };

        assert!(try_rotate(&mut g, RotationCommand::Ccw));
        assert_eq!(g.current.rotation, Rotation::Left);
        assert_eq!(g.current.x, 8);
        assert_eq!(g.current.y, 10);
        assert_eq!(g.spin.last_kick_index, Some(0));
    }

    #[test]
    fn direct_rotation_clears_stale_kick_index() {
        let mut g = default_game();
        g.spin.last_kick_index = Some(2);
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };

        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.spin.last_kick_index, None);
    }

    #[test]
    fn o_piece_rotates_in_place_without_kick() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::O,
            rotation: Rotation::Spawn,
            x: -1,
            y: 10,
        };

        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current.x, -1);
        assert_eq!(g.current.y, 10);
        assert_eq!(g.current.rotation, Rotation::Right);
        assert_eq!(g.spin.last_kick_index, None);
    }

    #[test]
    fn rotate_failed_does_not_mutate() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: -4,
            y: 10,
        };
        let before = g.current;
        assert!(!try_rotate(&mut g, RotationCommand::Cw));
        assert_eq!(g.current, before);
    }

    #[test]
    fn rotate_none_returns_false() {
        let mut g = default_game();
        assert!(!try_rotate(&mut g, RotationCommand::None));
    }

    #[test]
    fn rotate_no_kicks_for_i_180() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Spawn,
            x: -2,
            y: 10,
        };
        assert!(!try_rotate(&mut g, RotationCommand::Flip));
    }

    #[test]
    fn freeze_piece_applies_cells() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        freeze_active_piece(&mut g);
        assert_eq!(g.board.get(3, 11), crate::core::piece::Cell::T);
        assert_eq!(g.board.get(4, 10), crate::core::piece::Cell::T);
        assert_eq!(g.board.get(4, 11), crate::core::piece::Cell::T);
        assert_eq!(g.board.get(5, 11), crate::core::piece::Cell::T);
    }

    #[test]
    fn freeze_piece_skips_oob_cells() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: -1,
            y: 10,
        };
        freeze_active_piece(&mut g);
        assert_eq!(g.board.get(0, 11), crate::core::piece::Cell::T);
        assert_eq!(g.board.get(0, 10), crate::core::piece::Cell::T);
    }

    #[test]
    fn freeze_i_piece() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::I,
            rotation: Rotation::Left,
            x: 0,
            y: 10,
        };
        freeze_active_piece(&mut g);
        assert_eq!(g.board.get(1, 10), crate::core::piece::Cell::I);
        assert_eq!(g.board.get(1, 11), crate::core::piece::Cell::I);
        assert_eq!(g.board.get(1, 12), crate::core::piece::Cell::I);
        assert_eq!(g.board.get(1, 13), crate::core::piece::Cell::I);
    }

    fn setup_t_spin_scenario(occupied_corners: &[(usize, usize)]) -> GameState {
        let mut g = default_game();
        for &(x, y) in occupied_corners {
            g.board.set(x, y, crate::core::piece::Cell::Gray);
        }
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        g.spin.last_kick_index = Some(0);
        g
    }

    #[test]
    fn t_spin_detected_three_corners() {
        let g = setup_t_spin_scenario(&[(3, 10), (5, 10), (3, 12)]);
        assert!(detect_t_spin(&g));
    }

    #[test]
    fn t_spin_detected_four_corners() {
        let g = setup_t_spin_scenario(&[(3, 10), (5, 10), (3, 12), (5, 12)]);
        assert!(detect_t_spin(&g));
    }

    #[test]
    fn t_spin_not_detected_two_corners() {
        let g = setup_t_spin_scenario(&[(3, 10), (5, 10)]);
        assert!(!detect_t_spin(&g));
    }

    #[test]
    fn t_spin_non_t_piece() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::J,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        assert!(!detect_t_spin(&g));
    }

    #[test]
    fn t_spin_negative_oob_serves_as_occupied() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: -1,
            y: 0,
        };
        assert!(!detect_t_spin(&g));
    }

    #[test]
    fn t_spin_mini_when_back_corners_free() {
        let mut g = setup_t_spin_scenario(&[(3, 10), (5, 10), (3, 12)]);
        g.spin.is_t_spin = true;
        g.spin.last_kick_index = Some(0);
        assert!(!detect_t_spin_mini(&g));
    }

    #[test]
    fn t_spin_mini_when_one_back_corner_free() {
        let mut g = setup_t_spin_scenario(&[(5, 10), (3, 12)]);
        g.spin.is_t_spin = true;
        g.spin.last_kick_index = Some(0);
        assert!(detect_t_spin_mini(&g));
    }

    #[test]
    fn t_spin_mini_false_when_kick_index_three() {
        let mut g = setup_t_spin_scenario(&[(3, 10), (5, 10), (3, 12)]);
        g.spin.is_t_spin = true;
        g.spin.last_kick_index = Some(3);
        assert!(!detect_t_spin_mini(&g));
    }

    #[test]
    fn t_spin_mini_false_when_not_t_spin() {
        let g = setup_t_spin_scenario(&[(3, 10), (5, 10)]);
        assert!(!detect_t_spin_mini(&g));
    }

    #[test]
    fn t_spin_mini_false_when_kick_index_none() {
        let mut g = setup_t_spin_scenario(&[(3, 10), (5, 10), (3, 12)]);
        g.spin.is_t_spin = true;
        g.spin.last_kick_index = None;
        assert!(!detect_t_spin_mini(&g));
    }

    #[test]
    fn t_spin_mini_left_rotation() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Left,
            x: 3,
            y: 10,
        };
        g.board.set(3, 10, crate::core::piece::Cell::Gray);
        g.board.set(3, 12, crate::core::piece::Cell::Gray);
        g.board.set(5, 10, crate::core::piece::Cell::Gray);
        g.spin.is_t_spin = true;
        g.spin.last_kick_index = Some(0);
        assert!(!detect_t_spin_mini(&g));
    }

    #[test]
    fn rotation_sets_t_spin_state() {
        let mut g = default_game();
        g.current = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        g.board.set(3, 10, crate::core::piece::Cell::Gray);
        g.board.set(5, 10, crate::core::piece::Cell::Gray);
        g.board.set(3, 12, crate::core::piece::Cell::Gray);
        assert!(try_rotate(&mut g, RotationCommand::Cw));
        assert!(g.spin.is_t_spin);
    }
}
