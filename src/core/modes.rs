use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use super::bag::QueueGenerator;
use super::board::Board;
use super::game_state::{GameMode, GameState, GravityState, HoldState, SpinState, Stats};
use super::piece::{
    ALL_PIECES, Cell, PieceState, Rotation, Tetromino, mirrored_cell, mirrored_piece,
};

pub fn reset_for_mode(game: &mut GameState, mode: GameMode) {
    game.mode = mode;

    game.board.clear_all();

    game.highlights.sync_to_board(&game.board);

    game.hold = HoldState {
        infinite_hold: game.hold.infinite_hold,
        ..Default::default()
    };

    game.stats = Stats::default();

    game.spin = SpinState::default();

    game.lock_delay = super::game_state::LockDelayState::default();

    game.clear_flash = super::game_state::ClearFlashState::default();

    game.gravity = match mode {
        GameMode::Master => GravityState {
            gravity_hz: Some(1),
            ..Default::default()
        },
        _ => GravityState::default(),
    };

    let width = game.board.width;

    game.current = PieceState {
        kind: Tetromino::I,
        rotation: Rotation::Spawn,
        x: (width as i32) / 2 - 2,
        y: game.board.total_height() as i32 - 2,
    };

    match mode {
        GameMode::PerfectClear => {
            let mut generator = QueueGenerator::new(game.queue.mode, game.queue.seed);
            let leftover = game.pc_leftover.clamp(1, 7);
            let pc_pieces = generate_pc_bag(leftover, &mut generator);
            game.queue.seed = generator.seed;
            game.queue.visible = pc_pieces;
        }
        _ => {
            game.queue.visible.clear();
        }
    }

    match mode {
        GameMode::Cheese => {
            let mut rng = SmallRng::seed_from_u64(game.queue.seed as u64);
            spawn_garbage_board(&mut game.board, 4, &mut rng);
        }
        GameMode::FourWide => {
            let mut rng = SmallRng::seed_from_u64(game.queue.seed as u64);
            spawn_four_wide(&mut game.board, &mut rng);
        }
        GameMode::Edit => {}
        _ => {}
    }

    game.hold.swapped_this_turn = false;
}

pub fn mirror_field_and_queue(game: &mut GameState) {
    mirror_board(&mut game.board);
    game.highlights.mirror();

    for piece in game.queue.visible.iter_mut() {
        *piece = mirrored_piece(*piece);
    }

    if let Some(held) = game.hold.piece.as_mut() {
        *held = mirrored_piece(*held);
    }
}

fn mirror_board(board: &mut Board) {
    let w = board.width;
    let h = board.total_height();
    let mut new_cells = vec![Cell::Empty; w * h];

    for y in 0..h {
        for x in 0..w {
            let src = board.cells[y * w + x];
            new_cells[y * w + (w - 1 - x)] = mirrored_cell(src);
        }
    }

    board.cells = new_cells;
}

fn spawn_garbage_board(board: &mut Board, lines: usize, rng: &mut SmallRng) {
    let w = board.width;
    let total_h = board.total_height();

    for line in 0..lines.min(total_h) {
        let hole = rng.gen_range(0..w);
        let y = total_h - 1 - line;
        for x in 0..w {
            if x != hole {
                board.cells[y * w + x] = Cell::Gray;
            }
        }
    }
}

fn spawn_four_wide(board: &mut Board, rng: &mut SmallRng) {
    let w = board.width;
    let total_h = board.total_height();
    let h_center = (w as i32 / 2) - 2;

    let hc = h_center as usize;
    let dy = total_h - 1;

    for y in 0..total_h {
        for x in 0..w {
            if x < hc || x >= hc + 4 {
                board.cells[y * w + x] = Cell::Gray;
            }
        }
    }

    let pattern = rng.gen_range(0..6);
    match pattern {
        0 => {
            board.cells[dy * w + hc] = Cell::Gray;
            board.cells[dy * w + (hc + 1)] = Cell::Gray;
            board.cells[dy * w + (hc + 2)] = Cell::Gray;
        }
        1 => {
            board.cells[dy * w + (hc + 1)] = Cell::Gray;
            board.cells[dy * w + (hc + 2)] = Cell::Gray;
            board.cells[dy * w + (hc + 3)] = Cell::Gray;
        }
        2 => {
            board.cells[dy * w + hc] = Cell::Gray;
            board.cells[(dy - 1) * w + hc] = Cell::Gray;
            board.cells[(dy - 1) * w + (hc + 1)] = Cell::Gray;
        }
        3 => {
            board.cells[dy * w + (hc + 3)] = Cell::Gray;
            board.cells[(dy - 1) * w + (hc + 3)] = Cell::Gray;
            board.cells[(dy - 1) * w + (hc + 2)] = Cell::Gray;
        }
        4 => {
            board.cells[dy * w + hc] = Cell::Gray;
            board.cells[(dy - 1) * w + hc] = Cell::Gray;
            board.cells[dy * w + (hc + 1)] = Cell::Gray;
        }
        5 => {
            board.cells[dy * w + (hc + 3)] = Cell::Gray;
            board.cells[(dy - 1) * w + (hc + 3)] = Cell::Gray;
            board.cells[dy * w + (hc + 2)] = Cell::Gray;
        }
        _ => {}
    }
}

fn generate_pc_bag(leftover: usize, generator: &mut QueueGenerator) -> Vec<Tetromino> {
    let n = leftover.max(1).min(7);

    let mut rng = SmallRng::seed_from_u64(generator.seed as u64);

    loop {
        let mut bag: Vec<Tetromino> = ALL_PIECES.to_vec();
        bag.shuffle(&mut rng);
        bag.truncate(n);

        if n == 4 {
            let has_j = bag.contains(&Tetromino::J);
            let has_l = bag.contains(&Tetromino::L);
            let has_t = bag.contains(&Tetromino::T);
            if has_j && has_l && has_t {
                generator.seed = Rng::r#gen(&mut rng);
                continue;
            }
        }

        generator.seed = Rng::r#gen(&mut rng);
        return bag;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::{Board, HighlightBoard};
    use crate::core::game_state::GameState;
    use crate::core::piece::Tetromino;
    use crate::core::test_support;

    fn make_game_state() -> GameState {
        let mut game = test_support::game_state();
        game.board = Board::new(10, 20, 4);
        game.highlights = HighlightBoard::new(10, 24);
        game.current.y = 22;
        game.hold.infinite_hold = false;
        game.queue.visible = vec![Tetromino::I, Tetromino::J, Tetromino::S];
        game
    }

    #[test]
    fn reset_training_clears_board_and_refills_queue() {
        let mut game = make_game_state();
        game.board.set(5, 5, Cell::T);
        game.highlights.set(5, 5, 7);
        game.hold.piece = Some(Tetromino::L);
        game.hold.swapped_this_turn = true;
        game.stats.lines = 10;
        game.stats.damage = 5;

        reset_for_mode(&mut game, GameMode::Training);

        for y in 0..game.board.total_height() {
            for x in 0..game.board.width {
                assert_eq!(game.board.get(x, y), Cell::Empty);
                assert_eq!(game.highlights.get(x, y), 0);
            }
        }

        assert!(game.queue.visible.is_empty());
        assert_eq!(game.hold.piece, None);
        assert!(!game.hold.swapped_this_turn);
        assert_eq!(game.stats.lines, 0);
        assert_eq!(game.stats.damage, 0);
        assert!(!game.stats.lost);
    }

    #[test]
    fn reset_master_enables_gravity() {
        let mut game = make_game_state();
        reset_for_mode(&mut game, GameMode::Master);
        assert_eq!(game.gravity.gravity_hz, Some(1));
    }

    #[test]
    fn reset_cheese_adds_garbage() {
        let mut game = make_game_state();
        reset_for_mode(&mut game, GameMode::Cheese);

        let total = game.board.total_height();
        let bottom = total - 1;
        let garbage_count: usize = (0..game.board.width)
            .map(|x| {
                if game.board.get(x, bottom) != Cell::Empty {
                    1
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(garbage_count, game.board.width - 1);
    }

    #[test]
    fn reset_four_wide_creates_well() {
        let mut game = make_game_state();
        reset_for_mode(&mut game, GameMode::FourWide);

        let w = game.board.width;
        let h = game.board.total_height();
        let h_center = w / 2 - 2;

        for y in 0..h {
            for x in 0..w {
                if x < h_center || x >= h_center + 4 {
                    assert_eq!(
                        game.board.get(x, y),
                        Cell::Gray,
                        "cell ({},{}) should be Gray",
                        x,
                        y
                    );
                }
            }
        }
    }

    #[test]
    fn mirror_field_and_queue_mirrors_highlights() {
        let mut game = make_game_state();
        game.highlights.set(0, 5, 4);
        game.highlights.set(9, 6, 8);

        mirror_field_and_queue(&mut game);

        assert_eq!(game.highlights.get(9, 5), 4);
        assert_eq!(game.highlights.get(0, 6), 8);
    }

    #[test]
    fn reset_pc_mode_uses_custom_bag() {
        let mut game = make_game_state();
        game.pc_leftover = 3;
        reset_for_mode(&mut game, GameMode::PerfectClear);

        assert_eq!(game.queue.visible.len(), 3);
        assert_eq!(game.current.kind, Tetromino::I);
    }

    #[test]
    fn mirror_field_and_queue_mirrors_board() {
        let mut game = make_game_state();
        game.board.set(0, 0, Cell::J);
        game.board.set(9, 0, Cell::L);
        game.queue.visible = vec![Tetromino::J, Tetromino::S, Tetromino::T];
        game.hold.piece = Some(Tetromino::Z);

        mirror_field_and_queue(&mut game);

        assert_eq!(game.board.get(9, 0), Cell::L);
        assert_eq!(game.board.get(0, 0), Cell::J);
        assert_eq!(game.queue.visible[0], Tetromino::L);
        assert_eq!(game.queue.visible[1], Tetromino::Z);
        assert_eq!(game.queue.visible[2], Tetromino::T);
        assert_eq!(game.hold.piece, Some(Tetromino::S));
    }

    #[test]
    fn reset_preserves_infinite_hold_setting() {
        let mut game = make_game_state();
        game.hold.infinite_hold = true;
        reset_for_mode(&mut game, GameMode::Training);
        assert!(game.hold.infinite_hold);
    }

    #[test]
    fn reset_pc_mode_with_leftover() {
        let mut game = make_game_state();
        game.pc_leftover = 4;
        reset_for_mode(&mut game, GameMode::PerfectClear);
        assert_eq!(game.queue.visible.len(), 4);
        assert!(!game.hold.swapped_this_turn);
    }

    #[test]
    fn reset_clears_spin_state() {
        let mut game = make_game_state();
        game.spin.is_t_spin = true;
        game.spin.is_mini = true;
        game.spin.last_kick_index = Some(2);

        reset_for_mode(&mut game, GameMode::Training);
        assert!(!game.spin.is_t_spin);
        assert!(!game.spin.is_mini);
        assert_eq!(game.spin.last_kick_index, None);
    }
}
