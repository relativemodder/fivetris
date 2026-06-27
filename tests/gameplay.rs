use fivetris::app::AppState;
use fivetris::app::actions::AppAction;
use fivetris::app::controller::GameController;
use fivetris::app::ui_state::UiState;
use fivetris::core::game_loop::{GameLoop, SoundEffect};
use fivetris::core::{BagMode, Cell, GameMode, HoldError, QueueGenerator, Tetromino};

fn make_game_loop(infinite_hold: bool) -> GameLoop {
    GameLoop::new(GameMode::Training, BagMode::SevenBag, 0, infinite_hold)
}

fn make_state() -> AppState {
    AppState {
        game_loop: make_game_loop(true),
        ui_state: UiState::default(),
        paused: false,
    }
}

fn fill_row(board: &mut fivetris::core::Board, y: usize, cell: Cell) {
    for x in 0..board.width {
        board.set(x, y, cell);
    }
}

fn add_noise_cell(board: &mut fivetris::core::Board) {
    board.set(0, 0, Cell::Gray);
}

#[test]
fn spawn_next_consumes_queue_in_order_and_refills_threshold() {
    let mut game_loop = make_game_loop(true);
    game_loop.game.queue.visible.clear();
    game_loop.queue_generator =
        QueueGenerator::with_static_sequence(BagMode::Random, 5, vec![Tetromino::T, Tetromino::O]);

    assert!(game_loop.spawn_next());
    assert_eq!(game_loop.game.current.kind, Tetromino::T);
    assert_eq!(game_loop.game.queue.visible.len(), 8);
    assert_eq!(
        &game_loop.game.queue.visible[..7],
        &[
            Tetromino::O,
            Tetromino::L,
            Tetromino::S,
            Tetromino::Z,
            Tetromino::S,
            Tetromino::L,
            Tetromino::O,
        ]
    );

    assert!(game_loop.spawn_next());
    assert_eq!(game_loop.game.current.kind, Tetromino::O);
    assert_eq!(game_loop.game.queue.visible.len(), 7);
    assert_eq!(game_loop.game.queue.visible[0], Tetromino::L);
}

#[test]
fn hold_swap_consumes_next_piece_and_locks_second_swap_for_turn() {
    let mut game_loop = make_game_loop(false);
    game_loop.game.queue.visible.clear();
    game_loop.queue_generator = QueueGenerator::with_static_sequence(
        BagMode::Random,
        8,
        vec![Tetromino::J, Tetromino::L, Tetromino::S],
    );

    assert!(game_loop.spawn_next());
    assert_eq!(game_loop.game.current.kind, Tetromino::J);

    game_loop.hold_swap().expect("first hold should succeed");

    assert_eq!(game_loop.game.hold.piece, Some(Tetromino::J));
    assert_eq!(game_loop.game.current.kind, Tetromino::L);
    assert!(game_loop.game.hold.swapped_this_turn);

    let second_swap = game_loop.hold_swap();
    assert_eq!(second_swap, Err(HoldError::Locked));
    assert_eq!(game_loop.game.hold.piece, Some(Tetromino::J));
    assert_eq!(game_loop.game.current.kind, Tetromino::L);
}

#[test]
fn hard_drop_clears_line_sets_flash_and_spawns_next_piece() {
    let mut game_loop = make_game_loop(true);
    game_loop.game.queue.visible.clear();
    game_loop.queue_generator = QueueGenerator::with_static_sequence(
        BagMode::Random,
        13,
        vec![Tetromino::Mono, Tetromino::I],
    );

    assert!(game_loop.spawn_next());
    let bottom = game_loop.game.board.total_height() - 1;

    for x in 0..game_loop.game.board.width {
        if x != 4 {
            game_loop.game.board.set(x, bottom, Cell::Gray);
        }
    }
    game_loop.game.board.set(0, bottom - 1, Cell::Gray);
    game_loop.game.highlight_clear = true;
    game_loop.game.highlights.set(0, bottom - 1, 2);
    game_loop.game.highlights.set(4, bottom, 3);
    game_loop.game.current.kind = Tetromino::Mono;
    game_loop.game.current.x = 3;
    game_loop.game.current.y = 0;

    game_loop.hard_drop();

    assert_eq!(game_loop.game.stats.lines, 1);
    assert_eq!(game_loop.game.stats.combo, 1);
    assert_eq!(game_loop.game.stats.attack_text.as_deref(), Some("SINGLE"));
    assert!(game_loop.game.clear_flash.active);
    assert_eq!(game_loop.game.clear_flash.lines, vec![bottom]);
    assert_eq!(game_loop.game.highlights.get(4, bottom), 0);
    assert_eq!(game_loop.game.current.kind, Tetromino::I);
    assert_eq!(
        game_loop.pending_sounds,
        vec![SoundEffect::HardDrop, SoundEffect::Clear]
    );
}

#[test]
fn controller_reset_and_snapshot_navigation_restore_pushed_states() {
    let mut controller = GameController::default();
    let mut state = make_state();

    state.game_loop.queue_generator =
        QueueGenerator::with_static_sequence(BagMode::Random, 21, vec![Tetromino::I, Tetromino::O]);
    state.game_loop.game.queue.visible.clear();
    assert!(state.game_loop.spawn_next());
    let spawn_x = state.game_loop.game.current.x;

    state.game_loop.push_snapshot();
    assert!(state.game_loop.try_move_piece(1, 0));
    state.game_loop.push_snapshot();
    assert_eq!(state.game_loop.game.current.x, spawn_x + 1);

    assert!(state.game_loop.try_move_piece(1, 0));
    state.game_loop.push_snapshot();
    assert_eq!(state.game_loop.game.current.x, spawn_x + 2);

    controller.dispatch(&mut state, AppAction::Undo);
    assert_eq!(state.game_loop.game.current.x, spawn_x + 2);

    controller.dispatch(&mut state, AppAction::Undo);
    assert_eq!(state.game_loop.game.current.x, spawn_x + 1);

    controller.dispatch(&mut state, AppAction::Redo);
    assert_eq!(state.game_loop.game.current.x, spawn_x + 1);

    controller.dispatch(&mut state, AppAction::Redo);
    assert_eq!(state.game_loop.game.current.x, spawn_x + 2);

    state.paused = true;
    controller.dispatch(&mut state, AppAction::Reset(GameMode::Master));

    assert!(!state.paused);
    assert_eq!(state.game_loop.game.mode, GameMode::Master);
    assert_eq!(state.game_loop.game.gravity.gravity_hz, Some(1));
    assert_eq!(state.game_loop.game.current.y, 2);
    assert!(!state.game_loop.game.stats.lost);
    assert_eq!(state.game_loop.history.cursor, 0);
    assert!(state.game_loop.history.snapshots.is_empty());
    assert!(state.game_loop.game.queue.visible.len() >= 6);
}

#[test]
fn cheese_mode_garbage_progression_is_seeded_and_updates_seed() {
    let mut game_loop = GameLoop::new(GameMode::Cheese, BagMode::SevenBag, 77, true);
    let original_seed = game_loop.game.queue.seed;

    game_loop.lock_and_spawn();

    let gray_count = game_loop
        .game
        .board
        .cells
        .iter()
        .filter(|&&cell| cell == Cell::Gray)
        .count();
    assert!(gray_count > 0);
    assert_ne!(game_loop.game.queue.seed, original_seed);
    assert!(game_loop.game.garbage_hole_size <= game_loop.game.board.visible_height / 2);
}

#[test]
fn consecutive_tetrises_award_b2b_bonus() {
    let mut game_loop = make_game_loop(true);
    let bottom = game_loop.game.board.total_height() - 1;

    add_noise_cell(&mut game_loop.game.board);

    for offset in 0..4 {
        fill_row(&mut game_loop.game.board, bottom - offset, Cell::I);
    }
    let first = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(first.damage_delta, 4);
    assert!(game_loop.game.stats.b2b);

    for offset in 0..4 {
        fill_row(&mut game_loop.game.board, bottom - offset, Cell::I);
    }
    let second = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(second.damage_delta, 5);
    assert_eq!(second.b2b_text.as_deref(), Some("B2B"));
}

#[test]
fn combo_thresholds_add_expected_bonus_damage() {
    let mut game_loop = make_game_loop(true);
    let bottom = game_loop.game.board.total_height() - 1;

    add_noise_cell(&mut game_loop.game.board);

    fill_row(&mut game_loop.game.board, bottom, Cell::I);
    let first = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(first.damage_delta, 0);

    fill_row(&mut game_loop.game.board, bottom, Cell::I);
    let second = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(second.damage_delta, 0);

    fill_row(&mut game_loop.game.board, bottom, Cell::I);
    let third = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(third.combo_after, 3);
    assert_eq!(third.damage_delta, 1);

    fill_row(&mut game_loop.game.board, bottom, Cell::I);
    let fourth = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(fourth.combo_after, 4);
    assert_eq!(fourth.damage_delta, 1);

    fill_row(&mut game_loop.game.board, bottom, Cell::I);
    let fifth = fivetris::core::resolve_lock_and_clears(&mut game_loop.game);
    assert_eq!(fifth.combo_after, 5);
    assert_eq!(fifth.damage_delta, 2);
}
