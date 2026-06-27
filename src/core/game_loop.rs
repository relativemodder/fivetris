use super::bag::QueueGenerator;
use super::board::Board;
use super::game_state::{
    BagMode, GameMode, GameState, GravityState, HoldState, QueueState, SpinState, Stats,
};
use super::highlight::HighlightBoard;
use super::history::{History, UndoSnapshot};
use super::hold::HoldError;
use super::modes::reset_for_mode;
use super::piece::{
    DEFAULT_BOARD_WIDTH, DEFAULT_HIDDEN_ROWS, DEFAULT_VISIBLE_HEIGHT, PieceState, Rotation,
    RotationCommand, Tetromino,
};
use super::rotation::{fits, freeze_active_piece, try_move, try_rotate};
use super::scoring::resolve_lock_and_clears;
use crate::config::load_static_queue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffect {
    Move,
    Rotate,
    HardDrop,
    Hold,
    Kick,
    Clear,
    Tetris,
    TSpin,
    BackToBack,
    Fall,
    Lose,
}

pub struct GameLoop {
    pub game: GameState,
    pub history: History,
    pub queue_generator: QueueGenerator,
    pub static_bag_enabled: bool,
    pub gravity_accum: u32,
    pub pending_sounds: Vec<SoundEffect>,
    turn_start_snapshot: Option<UndoSnapshot>,
}

impl GameLoop {
    fn create_queue_generator(
        mode: BagMode,
        seed: u16,
        static_bag_enabled: bool,
    ) -> QueueGenerator {
        if !static_bag_enabled {
            return QueueGenerator::new(mode, seed);
        }
        match load_static_queue() {
            Ok(Some(static_sequence)) if !static_sequence.is_empty() => {
                QueueGenerator::with_static_sequence(mode, seed, static_sequence)
            }
            _ => QueueGenerator::new(mode, seed),
        }
    }

    pub fn new(mode: GameMode, bag_mode: BagMode, seed: u16, infinite_hold: bool) -> Self {
        Self::new_with_board(
            mode,
            bag_mode,
            seed,
            infinite_hold,
            false,
            DEFAULT_BOARD_WIDTH,
            DEFAULT_VISIBLE_HEIGHT,
            DEFAULT_HIDDEN_ROWS,
        )
    }

    pub fn new_with_board(
        mode: GameMode,
        bag_mode: BagMode,
        seed: u16,
        infinite_hold: bool,
        static_bag_enabled: bool,
        board_width: usize,
        visible_height: usize,
        hidden_rows: usize,
    ) -> Self {
        let board = Board::new(board_width, visible_height, hidden_rows);
        let total = board.total_height();

        let mut game = GameState {
            mode,
            board,
            highlights: HighlightBoard::new(board_width, total),
            current: PieceState {
                kind: Tetromino::I,
                rotation: Rotation::Spawn,
                x: (board_width as i32) / 2 - 2,
                y: total as i32 - 2,
            },
            hold: HoldState {
                piece: None,
                swapped_this_turn: false,
                infinite_hold,
            },
            queue: QueueState {
                visible: Vec::new(),
                seed,
                mode: bag_mode,
            },
            stats: Stats {
                damage: 0,
                lines: 0,
                moves: 0,
                b2b: false,
                perfect_clear: false,
                combo: 0,
                lost: false,
                attack_text: None,
                b2b_text: None,
            },
            spin: SpinState {
                is_t_spin: false,
                is_mini: false,
                last_kick_index: None,
            },
            gravity: GravityState {
                gravity_hz: None,
                arr_ms: 0,
                das_ms: 120,
                sdd_ms: 0,
                sds_ms: 0,
                das_cancel: false,
            },
            ghost_piece: true,
            auto_color: false,
            auto_lock_on_ground: false,
            mirror_queue_with_field: false,
            highlight_clear: false,
            pc_leftover: 0,
            garbage_hole_pos: 0,
            garbage_hole_size: 0,
            garbage_hole_next_change: 0,
            lock_delay: super::game_state::LockDelayState {
                active: false,
                timer_ms: 0,
                moves: 0,
                max_moves: 15,
                delay_ms: 500,
            },
            clear_flash: super::game_state::ClearFlashState {
                active: false,
                lines: Vec::new(),
                timer_ms: 0,
                duration_ms: 300,
            },
        };

        reset_for_mode(&mut game, mode);

        game.current.y = (hidden_rows as i32) - 2;

        let generator =
            Self::create_queue_generator(game.queue.mode, game.queue.seed, static_bag_enabled);

        GameLoop {
            game,
            history: History::new(100),
            queue_generator: generator,
            static_bag_enabled,
            gravity_accum: 0,
            pending_sounds: Vec::new(),
            turn_start_snapshot: None,
        }
    }

    pub fn push_snapshot(&mut self) {
        self.history.push(UndoSnapshot::new(
            &self.game,
            &self.queue_generator,
            self.gravity_accum,
        ));
    }

    fn push_turn_start_snapshot(&mut self) {
        if let Some(snapshot) = self.turn_start_snapshot.clone() {
            self.history.push(snapshot);
        }
    }

    fn sync_queue_seed(&mut self) {
        self.game.queue.seed = self.queue_generator.seed;
    }

    fn apply_snapshot(&mut self, snapshot: UndoSnapshot) {
        self.game = snapshot.game.clone();
        self.queue_generator = snapshot.queue_generator.clone();
        self.gravity_accum = snapshot.gravity_accum;
        self.turn_start_snapshot = Some(snapshot);
        self.pending_sounds.clear();
    }

    pub fn undo(&mut self) -> bool {
        if let Some(snapshot) = self.history.undo() {
            self.apply_snapshot(snapshot);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(snapshot) = self.history.redo() {
            self.apply_snapshot(snapshot);
            true
        } else {
            false
        }
    }

    pub fn spawn_next(&mut self) -> bool {
        if self.game.queue.visible.len() < 7 {
            self.queue_generator
                .fill_queue(&mut self.game.queue.visible);
            self.sync_queue_seed();
        }

        if self.game.queue.visible.is_empty() {
            return false;
        }

        let next = self.game.queue.visible.remove(0);
        let hidden = self.game.board.hidden_rows;
        self.game.current.kind = next;
        self.game.current.x = (self.game.board.width as i32) / 2 - 2;
        self.game.current.y = (hidden as i32) - 2;
        self.game.current.rotation = Rotation::Spawn;
        self.game.hold.swapped_this_turn = false;
        self.game.spin = SpinState::default();

        self.game.lock_delay.active = false;
        self.game.lock_delay.timer_ms = 0;
        self.game.lock_delay.moves = 0;

        if !fits(&self.game.board, self.game.current) {
            self.game.stats.lost = true;
            self.pending_sounds.push(SoundEffect::Lose);
            return false;
        }

        self.turn_start_snapshot = Some(UndoSnapshot::new(
            &self.game,
            &self.queue_generator,
            self.gravity_accum,
        ));

        true
    }

    pub fn try_move_piece(&mut self, dx: i32, dy: i32) -> bool {
        let success = try_move(&mut self.game, dx, dy);
        if success {
            self.pending_sounds.push(SoundEffect::Move);
            if self.game.lock_delay.active {
                self.game.lock_delay.timer_ms = 0;
                self.game.lock_delay.moves += 1;
                if self.game.lock_delay.moves >= self.game.lock_delay.max_moves {
                    self.push_turn_start_snapshot();
                    self.do_lock_and_spawn_inner();
                }
            }
        }
        success
    }

    pub fn try_rotate_piece(&mut self, command: RotationCommand) -> bool {
        let before = self.game.current;
        let success = try_rotate(&mut self.game, command);
        if success {
            let used_kick = self.game.spin.last_kick_index.is_some();
            self.pending_sounds.push(if used_kick {
                SoundEffect::Kick
            } else {
                SoundEffect::Rotate
            });
            if self.game.lock_delay.active {
                self.game.lock_delay.timer_ms = 0;
                self.game.lock_delay.moves += 1;
                if self.game.lock_delay.moves >= self.game.lock_delay.max_moves {
                    self.push_turn_start_snapshot();
                    self.do_lock_and_spawn_inner();
                }
            }
        }
        success
    }

    pub fn hold_swap(&mut self) -> Result<(), HoldError> {
        let result = super::hold::hold_swap(&mut self.game);
        if result.is_ok() {
            self.pending_sounds.push(SoundEffect::Hold);
            self.game.lock_delay.active = false;
            self.game.lock_delay.timer_ms = 0;
            self.game.lock_delay.moves = 0;
            if self.game.queue.visible.len() < 7 {
                self.queue_generator
                    .fill_queue(&mut self.game.queue.visible);
                self.sync_queue_seed();
            }
        }
        result
    }

    pub fn can_move_down(&self) -> bool {
        let next = PieceState {
            y: self.game.current.y + 1,
            ..self.game.current
        };
        fits(&self.game.board, next)
    }

    pub fn ghost_position(&self) -> PieceState {
        let mut ghost = self.game.current;
        loop {
            let next = PieceState {
                y: ghost.y + 1,
                ..ghost
            };
            if !fits(&self.game.board, next) {
                break;
            }
            ghost = next;
        }
        ghost
    }

    fn do_lock_and_spawn_inner(&mut self) {
        freeze_active_piece(&mut self.game);
        let outcome = resolve_lock_and_clears(&mut self.game);
        self.game.lock_delay.active = false;
        self.game.lock_delay.timer_ms = 0;
        self.game.lock_delay.moves = 0;

        if !self.game.stats.lost {
            if outcome.lines_cleared > 0 {
                if self.game.highlight_clear {
                    self.game.clear_flash.active = true;
                    self.game.clear_flash.lines = outcome.cleared_lines.clone();
                    self.game.clear_flash.timer_ms = 0;
                }

                match outcome.lines_cleared {
                    4 => self.pending_sounds.push(SoundEffect::Tetris),
                    _ => {
                        if outcome.spin_kind != super::game_state::SpinKind::None {
                            if outcome.b2b_after {
                                self.pending_sounds.push(SoundEffect::BackToBack);
                            }
                            self.pending_sounds.push(SoundEffect::TSpin);
                        } else {
                            self.pending_sounds.push(SoundEffect::Clear);
                        }
                    }
                }
            } else {
                self.pending_sounds.push(SoundEffect::Fall);
            }
            self.spawn_next();
        }
    }

    pub fn lock_and_spawn(&mut self) {
        self.push_turn_start_snapshot();
        self.do_lock_and_spawn_inner();
    }

    pub fn hard_drop(&mut self) {
        self.push_turn_start_snapshot();
        self.game.lock_delay.active = false;
        self.game.lock_delay.timer_ms = 0;
        self.game.lock_delay.moves = 0;
        self.pending_sounds.push(SoundEffect::HardDrop);
        while try_move(&mut self.game, 0, 1) {}
        self.do_lock_and_spawn_inner();
    }

    pub fn tick(&mut self, dt_ms: u32) {
        if self.game.stats.lost {
            return;
        }

        if self.game.clear_flash.active {
            self.game.clear_flash.timer_ms += dt_ms;
            if self.game.clear_flash.timer_ms >= self.game.clear_flash.duration_ms {
                self.game.clear_flash.active = false;
                self.game.clear_flash.lines.clear();
                self.game.clear_flash.timer_ms = 0;
            }
        }

        if let Some(hz) = self.game.gravity.gravity_hz {
            if hz > 0 {
                let interval = 1000 / hz;
                self.gravity_accum += dt_ms;
                while self.gravity_accum >= interval {
                    self.gravity_accum -= interval;
                    if !try_move(&mut self.game, 0, 1) {
                        self.gravity_accum = self.gravity_accum.min(interval - 1);
                        break;
                    }
                }
            }
        }

        if !self.can_move_down() {
            if !self.game.auto_lock_on_ground {
                return;
            }

            if !self.game.lock_delay.active {
                self.game.lock_delay.active = true;
                self.game.lock_delay.timer_ms = 0;
            }
            let next_timer_ms = self.game.lock_delay.timer_ms.saturating_add(dt_ms);
            if next_timer_ms >= self.game.lock_delay.delay_ms {
                self.push_turn_start_snapshot();
                self.do_lock_and_spawn_inner();
            } else {
                self.game.lock_delay.timer_ms = next_timer_ms;
            }
        } else {
            if self.game.lock_delay.active {
                self.game.lock_delay.active = false;
                self.game.lock_delay.timer_ms = 0;
                self.game.lock_delay.moves = 0;
            }
        }
    }

    pub fn reset(&mut self, mode: GameMode) {
        let infinite_hold = self.game.hold.infinite_hold;
        let bag_mode = self.game.queue.mode;
        let das_ms = self.game.gravity.das_ms;
        let arr_ms = self.game.gravity.arr_ms;
        let sdd_ms = self.game.gravity.sdd_ms;
        let sds_ms = self.game.gravity.sds_ms;
        let das_cancel = self.game.gravity.das_cancel;
        let ghost_piece = self.game.ghost_piece;
        let auto_color = self.game.auto_color;
        let auto_lock_on_ground = self.game.auto_lock_on_ground;
        let mirror_queue_with_field = self.game.mirror_queue_with_field;
        let highlight_clear = self.game.highlight_clear;
        reset_for_mode(&mut self.game, mode);
        self.game.hold.infinite_hold = infinite_hold;
        self.game.gravity.arr_ms = arr_ms;
        self.game.gravity.das_ms = das_ms;
        self.game.gravity.sdd_ms = sdd_ms;
        self.game.gravity.sds_ms = sds_ms;
        self.game.gravity.das_cancel = das_cancel;
        self.game.ghost_piece = ghost_piece;
        self.game.auto_color = auto_color;
        self.game.auto_lock_on_ground = auto_lock_on_ground;
        self.game.mirror_queue_with_field = mirror_queue_with_field;
        self.game.highlight_clear = highlight_clear;
        self.game.current.y = (self.game.board.hidden_rows as i32) - 2;
        self.history = History::new(100);
        self.queue_generator =
            Self::create_queue_generator(bag_mode, self.game.queue.seed, self.static_bag_enabled);
        self.gravity_accum = 0;
        self.turn_start_snapshot = None;
        self.pending_sounds.clear();
        self.spawn_next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::piece::{Cell, Tetromino};

    #[test]
    fn undo_restores_full_gameplay_state() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 17, true);
        game_loop.queue_generator = QueueGenerator::with_static_sequence(
            BagMode::Random,
            91,
            vec![Tetromino::Mono, Tetromino::J],
        );
        game_loop.gravity_accum = 73;
        game_loop.game.mode = GameMode::Master;
        game_loop.game.board.set(2, 6, Cell::Z);
        game_loop.game.highlights.set(5, 0, 4);
        game_loop.game.current.kind = Tetromino::L;
        game_loop.game.current.rotation = Rotation::Right;
        game_loop.game.current.x = 6;
        game_loop.game.current.y = 8;
        game_loop.game.hold.piece = Some(Tetromino::S);
        game_loop.game.hold.swapped_this_turn = true;
        game_loop.game.queue.visible = vec![Tetromino::O, Tetromino::I, Tetromino::T];
        game_loop.game.queue.seed = 1234;
        game_loop.game.stats.damage = 55;
        game_loop.game.stats.lines = 12;
        game_loop.game.stats.moves = 19;
        game_loop.game.stats.b2b = true;
        game_loop.game.stats.perfect_clear = true;
        game_loop.game.stats.combo = 5;
        game_loop.game.stats.lost = true;
        game_loop.game.stats.attack_text = Some("attack".to_string());
        game_loop.game.stats.b2b_text = Some("b2b".to_string());
        game_loop.game.spin.is_t_spin = true;
        game_loop.game.spin.is_mini = true;
        game_loop.game.spin.last_kick_index = Some(2);
        game_loop.game.gravity.gravity_hz = Some(15);
        game_loop.game.gravity.arr_ms = 1;
        game_loop.game.gravity.das_ms = 85;
        game_loop.game.gravity.sdd_ms = 2;
        game_loop.game.gravity.sds_ms = 3;
        game_loop.game.gravity.das_cancel = true;
        game_loop.game.ghost_piece = false;
        game_loop.game.auto_color = true;
        game_loop.game.mirror_queue_with_field = true;
        game_loop.game.highlight_clear = true;
        game_loop.game.pc_leftover = 4;
        game_loop.game.garbage_hole_pos = 3;
        game_loop.game.garbage_hole_size = 2;
        game_loop.game.garbage_hole_next_change = 8;
        game_loop.game.lock_delay.active = true;
        game_loop.game.lock_delay.timer_ms = 222;
        game_loop.game.lock_delay.moves = 7;
        game_loop.game.lock_delay.max_moves = 19;
        game_loop.game.lock_delay.delay_ms = 600;
        game_loop.game.clear_flash.active = true;
        game_loop.game.clear_flash.lines = vec![17, 18];
        game_loop.game.clear_flash.timer_ms = 99;
        game_loop.game.clear_flash.duration_ms = 333;
        game_loop.pending_sounds.push(SoundEffect::Move);

        let expected_game = game_loop.game.clone();
        let expected_generator = game_loop.queue_generator.clone();
        let expected_gravity_accum = game_loop.gravity_accum;

        game_loop.push_snapshot();

        game_loop.game.mode = GameMode::Cheese;
        game_loop.game.board.set(2, 6, Cell::Empty);
        game_loop.game.highlights.clear_cell(5, 0);
        game_loop.game.current.kind = Tetromino::I;
        game_loop.game.current.rotation = Rotation::Spawn;
        game_loop.game.current.x = 0;
        game_loop.game.current.y = 0;
        game_loop.game.hold.piece = None;
        game_loop.game.hold.swapped_this_turn = false;
        game_loop.game.queue.visible.clear();
        game_loop.game.queue.seed = 1;
        game_loop.game.stats.damage = 0;
        game_loop.game.stats.lines = 0;
        game_loop.game.stats.moves = 0;
        game_loop.game.stats.b2b = false;
        game_loop.game.stats.perfect_clear = false;
        game_loop.game.stats.combo = 0;
        game_loop.game.stats.lost = false;
        game_loop.game.stats.attack_text = None;
        game_loop.game.stats.b2b_text = None;
        game_loop.game.spin.is_t_spin = false;
        game_loop.game.spin.is_mini = false;
        game_loop.game.spin.last_kick_index = None;
        game_loop.game.gravity.gravity_hz = None;
        game_loop.game.gravity.arr_ms = 0;
        game_loop.game.gravity.das_ms = 120;
        game_loop.game.gravity.sdd_ms = 0;
        game_loop.game.gravity.sds_ms = 0;
        game_loop.game.gravity.das_cancel = false;
        game_loop.game.ghost_piece = true;
        game_loop.game.auto_color = false;
        game_loop.game.mirror_queue_with_field = false;
        game_loop.game.highlight_clear = false;
        game_loop.game.pc_leftover = 0;
        game_loop.game.garbage_hole_pos = 0;
        game_loop.game.garbage_hole_size = 0;
        game_loop.game.garbage_hole_next_change = 0;
        game_loop.game.lock_delay.active = false;
        game_loop.game.lock_delay.timer_ms = 0;
        game_loop.game.lock_delay.moves = 0;
        game_loop.game.lock_delay.max_moves = 15;
        game_loop.game.lock_delay.delay_ms = 500;
        game_loop.game.clear_flash.active = false;
        game_loop.game.clear_flash.lines.clear();
        game_loop.game.clear_flash.timer_ms = 0;
        game_loop.game.clear_flash.duration_ms = 300;
        game_loop.queue_generator = QueueGenerator::new(BagMode::SevenBag, 2);
        game_loop.gravity_accum = 0;
        game_loop.pending_sounds.push(SoundEffect::Clear);

        assert!(game_loop.undo());
        assert_eq!(game_loop.game, expected_game);
        assert_eq!(game_loop.queue_generator, expected_generator);
        assert_eq!(game_loop.gravity_accum, expected_gravity_accum);
        assert!(game_loop.pending_sounds.is_empty());
    }

    #[test]
    fn redo_restores_full_gameplay_state() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 17, true);

        game_loop.game.current.kind = Tetromino::J;
        game_loop.game.stats.damage = 10;
        game_loop.push_snapshot();

        game_loop.game.current.kind = Tetromino::T;
        game_loop.game.spin.last_kick_index = Some(3);
        game_loop.game.highlight_clear = true;
        game_loop.game.lock_delay.active = true;
        game_loop.game.lock_delay.timer_ms = 88;
        game_loop.game.clear_flash.active = true;
        game_loop.game.clear_flash.lines = vec![19];
        game_loop.game.gravity.gravity_hz = Some(30);
        game_loop.game.garbage_hole_next_change = 6;
        game_loop.queue_generator =
            QueueGenerator::with_static_sequence(BagMode::Random, 321, vec![Tetromino::L]);
        game_loop.gravity_accum = 44;
        let expected_game = game_loop.game.clone();
        let expected_generator = game_loop.queue_generator.clone();
        let expected_gravity_accum = game_loop.gravity_accum;
        game_loop.push_snapshot();

        assert!(game_loop.undo());

        game_loop.game.current.kind = Tetromino::I;
        game_loop.game.spin.last_kick_index = None;
        game_loop.game.highlight_clear = false;
        game_loop.game.lock_delay.active = false;
        game_loop.game.lock_delay.timer_ms = 0;
        game_loop.game.clear_flash.active = false;
        game_loop.game.clear_flash.lines.clear();
        game_loop.game.gravity.gravity_hz = None;
        game_loop.game.garbage_hole_next_change = 0;
        game_loop.queue_generator = QueueGenerator::new(BagMode::SevenBag, 1);
        game_loop.gravity_accum = 0;
        game_loop.pending_sounds.push(SoundEffect::Rotate);

        assert!(game_loop.redo());
        assert_eq!(game_loop.game, expected_game);
        assert_eq!(game_loop.queue_generator, expected_generator);
        assert_eq!(game_loop.gravity_accum, expected_gravity_accum);
        assert!(game_loop.pending_sounds.is_empty());
    }

    #[test]
    fn new_with_board_uses_requested_dimensions() {
        let game_loop = GameLoop::new_with_board(
            GameMode::Training,
            BagMode::SevenBag,
            7,
            true,
            false,
            12,
            22,
            5,
        );

        assert_eq!(game_loop.game.board.width, 12);
        assert_eq!(game_loop.game.board.visible_height, 22);
        assert_eq!(game_loop.game.board.hidden_rows, 5);
        assert_eq!(game_loop.game.highlights.total_height, 27);
        assert_eq!(game_loop.game.current.x, 4);
        assert_eq!(game_loop.game.current.y, 3);
    }

    #[test]
    fn reset_preserves_runtime_handling_settings() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 7, true);
        game_loop.game.gravity.arr_ms = 16;
        game_loop.game.gravity.das_ms = 120;
        game_loop.game.gravity.sdd_ms = 67;
        game_loop.game.gravity.sds_ms = 16;
        game_loop.game.gravity.das_cancel = true;
        game_loop.game.ghost_piece = false;
        game_loop.game.auto_color = true;
        game_loop.game.mirror_queue_with_field = true;
        game_loop.game.highlight_clear = true;

        game_loop.reset(GameMode::Master);

        assert_eq!(game_loop.game.gravity.gravity_hz, Some(1));
        assert_eq!(game_loop.game.gravity.arr_ms, 16);
        assert_eq!(game_loop.game.gravity.das_ms, 120);
        assert_eq!(game_loop.game.gravity.sdd_ms, 67);
        assert_eq!(game_loop.game.gravity.sds_ms, 16);
        assert!(game_loop.game.gravity.das_cancel);
        assert!(!game_loop.game.ghost_piece);
        assert!(game_loop.game.auto_color);
        assert!(game_loop.game.mirror_queue_with_field);
        assert!(game_loop.game.highlight_clear);
    }

    #[test]
    fn spawn_next_keeps_game_seed_in_sync_with_generator() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 7, true);
        game_loop.game.queue.visible.clear();

        assert!(game_loop.spawn_next());

        assert_eq!(game_loop.game.queue.seed, game_loop.queue_generator.seed);
    }

    #[test]
    fn hold_refill_keeps_game_seed_in_sync_with_generator() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 7, true);
        game_loop.game.queue.visible = vec![Tetromino::I];
        game_loop.game.current.kind = Tetromino::T;
        game_loop.game.hold.piece = None;

        game_loop.hold_swap().expect("hold should succeed");

        assert_eq!(game_loop.game.queue.seed, game_loop.queue_generator.seed);
    }

    #[test]
    fn reset_rebuilds_generator_from_current_queue_seed() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 7, true);
        game_loop.game.queue.visible.clear();
        assert!(game_loop.spawn_next());

        let synced_seed = game_loop.game.queue.seed;
        assert_eq!(synced_seed, game_loop.queue_generator.seed);

        game_loop.reset(GameMode::Cheese);

        assert_eq!(game_loop.queue_generator.seed, game_loop.game.queue.seed);
        assert_ne!(game_loop.queue_generator.seed, 7);
    }

    #[test]
    fn undo_after_auto_lock_clear_restores_turn_spawn_state() {
        let mut game_loop = GameLoop::new(GameMode::Training, BagMode::SevenBag, 7, true);
        let bottom = game_loop.game.board.total_height() - 1;
        let spawn_y = game_loop.game.board.hidden_rows as i32 - 2;

        for x in 0..game_loop.game.board.width {
            if x != 4 {
                game_loop.game.board.set(x, bottom, Cell::Gray);
            }
        }

        game_loop.game.current.kind = Tetromino::Mono;
        game_loop.game.current.rotation = Rotation::Spawn;
        game_loop.game.current.x = 3;
        game_loop.game.current.y = spawn_y;
        game_loop.turn_start_snapshot = Some(UndoSnapshot::new(
            &game_loop.game,
            &game_loop.queue_generator,
            game_loop.gravity_accum,
        ));
        game_loop.game.current.y = bottom as i32 - 1;
        game_loop.game.lock_delay.active = true;
        game_loop.game.lock_delay.timer_ms = 490;
        game_loop.game.lock_delay.delay_ms = 500;
        game_loop.game.lock_delay.moves = 0;
        game_loop.game.auto_lock_on_ground = true;
        game_loop.game.highlight_clear = true;
        game_loop.game.stats.lines = 0;

        game_loop.tick(10);

        assert_eq!(game_loop.game.stats.lines, 1);
        assert!(game_loop.game.clear_flash.active);

        assert!(game_loop.undo());
        assert_eq!(game_loop.game.stats.lines, 0);
        assert_eq!(game_loop.game.current.y, spawn_y);
        assert_eq!(game_loop.game.lock_delay.timer_ms, 0);
        assert!(!game_loop.game.lock_delay.active);

        game_loop.tick(0);

        assert_eq!(game_loop.game.stats.lines, 0);
        assert_eq!(game_loop.game.lock_delay.timer_ms, 0);
        assert!(!game_loop.game.lock_delay.active);
    }
}
