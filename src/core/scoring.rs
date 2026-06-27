use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use super::board::Board;
use super::game_state::{ClearOutcome, GameMode, GameState, LineClearKind, SpinKind};
use super::piece::Cell;

fn grid_get_garbage_level(board: &Board) -> usize {
    let total = board.total_height();
    for y in 0..total {
        for x in 0..board.width {
            if board.get(x, y) == Cell::Gray {
                return total - y;
            }
        }
    }
    0
}

fn grid_add_garbage_line(board: &mut Board, hole_pos: usize) {
    board.push_line();
    let bottom = board.total_height() - 1;
    for x in 0..board.width {
        if x == hole_pos {
            board.set(x, bottom, Cell::Empty);
        } else {
            board.set(x, bottom, Cell::Gray);
        }
    }
}

pub fn spawn_garbage(game: &mut GameState) {
    let mut rng = SmallRng::seed_from_u64(game.queue.seed as u64);
    let garbage_amount = grid_get_garbage_level(&game.board);
    let half_y = (game.board.visible_height / 2) as usize;
    let hole_pos = &mut game.garbage_hole_pos;
    let hole_size = &mut game.garbage_hole_size;

    let mut next_change = garbage_amount + *hole_size;

    let mut i = garbage_amount;
    while i < half_y {
        if i == next_change {
            let hole_last_pos = *hole_pos;
            loop {
                *hole_pos = rng.gen_range(0..game.board.width);
                if *hole_pos != hole_last_pos {
                    break;
                }
            }
            let garbage_step: usize = rng.gen_range(1..=3);
            next_change += garbage_step;
        }
        grid_add_garbage_line(&mut game.board, *hole_pos);
        i += 1;
    }

    *hole_size = next_change.saturating_sub(i);
    game.queue.seed = Rng::r#gen(&mut rng);
}

pub fn add_wide(board: &mut Board, amount: usize) {
    let hole = board.width / 2 - 2;
    let rows = amount + 1;
    for y in 0..rows {
        for x in 0..board.width {
            if x < hole || x > hole + 3 {
                board.set(x, y, Cell::Gray);
            }
        }
    }
}

pub fn spawn_four_wide(game: &mut GameState) {
    add_wide(&mut game.board, 0);
}

fn build_attack_text(
    lines_cleared: u8,
    spin_kind: SpinKind,
    perfect_clear: bool,
) -> Option<String> {
    if perfect_clear {
        return Some("PERFECT CLEAR".to_string());
    }
    match spin_kind {
        SpinKind::None => match lines_cleared {
            0 => None,
            1 => Some("SINGLE".to_string()),
            2 => Some("DOUBLE".to_string()),
            3 => Some("TRIPLE".to_string()),
            4 => Some("TETRIS".to_string()),
            _ => None,
        },
        SpinKind::Mini => Some("T-SPIN MINI".to_string()),
        SpinKind::TSpin => match lines_cleared {
            0 => Some("T-SPIN".to_string()),
            1 => Some("T-SPIN SINGLE".to_string()),
            2 => Some("T-SPIN DOUBLE".to_string()),
            3 => Some("T-SPIN TRIPLE".to_string()),
            _ => Some("T-SPIN".to_string()),
        },
    }
}

pub fn resolve_lock_and_clears(game: &mut GameState) -> ClearOutcome {
    let is_t_spin = game.spin.is_t_spin;
    let is_mini = game.spin.is_mini;

    let spin_kind = if is_t_spin {
        if is_mini {
            SpinKind::Mini
        } else {
            SpinKind::TSpin
        }
    } else {
        SpinKind::None
    };

    let total = game.board.total_height();

    let mut cleared_lines = Vec::new();
    for y in 0..total {
        if game.board.is_line_full(y) {
            cleared_lines.push(y);
        }
    }

    let mut lines_cleared = 0u8;
    let mut y = 0;
    while y < total {
        if game.board.is_line_full(y) {
            game.board.clear_line(y);
            if game.highlight_clear {
                game.highlights.clear_line(y);
            }
            lines_cleared += 1;
        } else {
            y += 1;
        }
    }

    let line_clear_kind = match lines_cleared {
        0 => LineClearKind::None,
        1 => LineClearKind::Single,
        2 => LineClearKind::Double,
        3 => LineClearKind::Triple,
        _ => LineClearKind::Tetris,
    };

    let perfect_clear = game.board.cells.iter().all(|&c| c == Cell::Empty);

    let mut damage_delta = 0i32;

    if perfect_clear {
        damage_delta += 10;
        game.stats.perfect_clear = true;
    }

    if lines_cleared > 0 {
        game.stats.combo += 1;

        match lines_cleared {
            1 | 2 | 3 => {
                if is_t_spin {
                    damage_delta += (lines_cleared as i32) * 2;
                    if is_mini {
                        damage_delta -= 2;
                    }
                } else {
                    damage_delta += (lines_cleared as i32) - 1;
                }
            }
            4 => {
                damage_delta += 4;
            }
            _ => {}
        }
    } else {
        game.stats.combo = 0;
    }

    for threshold in [2, 4, 6, 8, 11] {
        if game.stats.combo > threshold {
            damage_delta += 1;
        }
    }

    let is_b2b_eligible = lines_cleared == 4 || (lines_cleared > 0 && is_t_spin);

    if is_b2b_eligible {
        if game.stats.b2b {
            damage_delta += 1;
        } else {
            game.stats.b2b = true;
        }
    } else if lines_cleared > 0 {
        game.stats.b2b = false;
    }

    game.stats.lines += lines_cleared as u32;
    game.stats.damage = game.stats.damage.wrapping_add(damage_delta as u32);

    let attack_text = build_attack_text(lines_cleared, spin_kind, perfect_clear);
    let b2b_text = if is_b2b_eligible && game.stats.b2b && damage_delta > 0 {
        Some("B2B".to_string())
    } else {
        None
    };

    game.stats.attack_text = attack_text.clone();
    game.stats.b2b_text = b2b_text.clone();

    match game.mode {
        GameMode::Cheese => {
            if lines_cleared == 0 {
                spawn_garbage(game);
            }
        }
        GameMode::FourWide => {
            if lines_cleared == 0 {
                game.stats.lost = true;
            } else {
                add_wide(&mut game.board, lines_cleared as usize);
            }
        }
        GameMode::Edit => {}
        _ => {}
    }

    ClearOutcome {
        lines_cleared,
        line_clear_kind,
        spin_kind,
        perfect_clear,
        damage_delta,
        combo_after: game.stats.combo,
        b2b_after: game.stats.b2b,
        attack_text,
        b2b_text,
        cleared_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::core::game_state::GameState;
    use crate::core::piece::{Cell, PieceState, Rotation, Tetromino};
    use crate::core::test_support;

    fn empty_board() -> Board {
        Board::new(10, 20, 4)
    }

    fn default_game() -> GameState {
        let mut game = test_support::game_state();
        game.board = empty_board();
        game.highlights.sync_to_board(&game.board);
        game
    }

    fn fill_row(board: &mut Board, y: usize, cell: Cell) {
        for x in 0..board.width {
            board.set(x, y, cell);
        }
    }

    fn add_noise_cell(board: &mut Board) {
        board.set(0, 0, Cell::Gray);
    }

    #[test]
    fn single_line_clear_detection() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 1);
        assert_eq!(outcome.line_clear_kind, LineClearKind::Single);
        assert_eq!(outcome.damage_delta, 0);
        assert_eq!(outcome.combo_after, 1);
    }

    #[test]
    fn double_line_clear_detection() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 2);
        assert_eq!(outcome.line_clear_kind, LineClearKind::Double);
        assert_eq!(outcome.damage_delta, 1);
    }

    #[test]
    fn triple_line_clear_detection() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 21, Cell::I);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 3);
        assert_eq!(outcome.line_clear_kind, LineClearKind::Triple);
        assert_eq!(outcome.damage_delta, 2);
    }

    #[test]
    fn tetris_line_clear_detection() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 20, Cell::I);
        fill_row(&mut game.board, 21, Cell::I);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 4);
        assert_eq!(outcome.line_clear_kind, LineClearKind::Tetris);
        assert_eq!(outcome.damage_delta, 4);
    }

    #[test]
    fn combo_increments_on_consecutive_clears() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);
        let r1 = resolve_lock_and_clears(&mut game);
        assert_eq!(r1.combo_after, 1);

        fill_row(&mut game.board, 23, Cell::I);
        let r2 = resolve_lock_and_clears(&mut game);
        assert_eq!(r2.combo_after, 2);

        fill_row(&mut game.board, 23, Cell::I);
        let r3 = resolve_lock_and_clears(&mut game);
        assert_eq!(r3.combo_after, 3);
        assert_eq!(r3.damage_delta, 1);
    }

    #[test]
    fn combo_resets_on_zero_clear() {
        let mut game = default_game();
        fill_row(&mut game.board, 23, Cell::I);
        resolve_lock_and_clears(&mut game);
        assert_eq!(game.stats.combo, 1);

        game.board.clear_all();
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 0);
        assert_eq!(outcome.combo_after, 0);
        assert_eq!(game.stats.combo, 0);
    }

    #[test]
    fn b2b_set_on_tetris() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 20, Cell::I);
        fill_row(&mut game.board, 21, Cell::I);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        let r1 = resolve_lock_and_clears(&mut game);
        assert!(r1.b2b_after);
        assert_eq!(r1.damage_delta, 4);

        fill_row(&mut game.board, 20, Cell::I);
        fill_row(&mut game.board, 21, Cell::I);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        let r2 = resolve_lock_and_clears(&mut game);
        assert!(r2.b2b_after);
        assert_eq!(r2.damage_delta, 5);
    }

    #[test]
    fn b2b_resets_on_non_b2b_clear() {
        let mut game = default_game();
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 20, Cell::I);
        fill_row(&mut game.board, 21, Cell::I);
        fill_row(&mut game.board, 22, Cell::I);
        fill_row(&mut game.board, 23, Cell::I);
        resolve_lock_and_clears(&mut game);
        assert!(game.stats.b2b);

        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);
        let r2 = resolve_lock_and_clears(&mut game);
        assert!(!r2.b2b_after);
        assert!(!game.stats.b2b);
    }

    #[test]
    fn perfect_clear_detection() {
        let mut game = default_game();
        for y in 0..24 {
            fill_row(&mut game.board, y, Cell::I);
        }
        let outcome = resolve_lock_and_clears(&mut game);
        assert!(outcome.perfect_clear);
        assert!(game.stats.perfect_clear);
        assert!(outcome.damage_delta >= 10);
    }

    #[test]
    fn garbage_line_has_exactly_one_hole() {
        let mut board = empty_board();
        let hole_pos = 3;
        grid_add_garbage_line(&mut board, hole_pos);
        let bottom = board.total_height() - 1;

        let mut holes = 0;
        let mut grays = 0;
        for x in 0..board.width {
            match board.get(x, bottom) {
                Cell::Empty => holes += 1,
                Cell::Gray => grays += 1,
                _ => {}
            }
        }
        assert_eq!(holes, 1);
        assert_eq!(grays, board.width - 1);
    }

    #[test]
    fn combo_bonus_staircase() {
        let mut game = default_game();

        for combo_target in 1..=5 {
            fill_row(&mut game.board, 23, Cell::I);
            let outcome = resolve_lock_and_clears(&mut game);
            assert_eq!(outcome.combo_after, combo_target);
        }

        assert_eq!(game.stats.combo, 5);
    }

    #[test]
    fn board_empty_after_line_clear_and_free_fall() {
        let mut game = default_game();
        fill_row(&mut game.board, 23, Cell::I);
        resolve_lock_and_clears(&mut game);
        for x in 0..game.board.width {
            assert_eq!(game.board.get(x, 23), Cell::Empty);
        }
    }

    #[test]
    fn freeze_piece_writes_cells() {
        let mut board = empty_board();
        let piece = PieceState {
            kind: Tetromino::T,
            rotation: Rotation::Spawn,
            x: 3,
            y: 10,
        };
        board.freeze_piece(piece, Tetromino::T);
        assert_eq!(board.get(3, 11), Cell::T);
        assert_eq!(board.get(4, 10), Cell::T);
        assert_eq!(board.get(4, 11), Cell::T);
        assert_eq!(board.get(5, 11), Cell::T);
    }

    #[test]
    fn t_spin_single_damage() {
        let mut game = default_game();
        game.spin.is_t_spin = true;
        game.spin.is_mini = false;
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 1);
        assert_eq!(outcome.spin_kind, SpinKind::TSpin);
        assert_eq!(outcome.damage_delta, 2);
    }

    #[test]
    fn t_spin_mini_damage() {
        let mut game = default_game();
        game.spin.is_t_spin = true;
        game.spin.is_mini = true;
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);
        let outcome = resolve_lock_and_clears(&mut game);
        assert_eq!(outcome.lines_cleared, 1);
        assert_eq!(outcome.spin_kind, SpinKind::Mini);
        assert_eq!(outcome.damage_delta, 0);
    }

    #[test]
    fn push_line_shifts_up() {
        let mut board = empty_board();
        board.set(0, 0, Cell::I);
        board.push_line();
        assert_eq!(board.get(0, 0), Cell::Empty);
        assert_eq!(board.get(0, 0), Cell::Empty);
        assert_eq!(board.get(0, board.total_height() - 1), Cell::Empty);
    }

    #[test]
    fn is_line_full_detects_full_and_partial() {
        let mut board = empty_board();
        assert!(!board.is_line_full(23));
        fill_row(&mut board, 23, Cell::I);
        assert!(board.is_line_full(23));
        board.set(5, 23, Cell::Empty);
        assert!(!board.is_line_full(23));
    }

    #[test]
    fn highlight_clear_shifts_overlay_with_line_clear() {
        let mut game = default_game();
        game.highlight_clear = true;
        game.highlights.set(0, 22, 7);
        game.highlights.set(1, 23, 9);
        add_noise_cell(&mut game.board);
        fill_row(&mut game.board, 23, Cell::I);

        resolve_lock_and_clears(&mut game);

        assert_eq!(game.highlights.get(0, 23), 7);
        assert_eq!(game.highlights.get(1, 23), 0);
        assert_eq!(game.highlights.get(0, 0), 0);
    }
}
