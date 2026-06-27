use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use super::bag::QueueGenerator;
use super::game_state::GameState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoSnapshot {
    pub game: GameState,
    pub queue_generator: QueueGenerator,
    pub gravity_accum: u32,
}

impl UndoSnapshot {
    pub fn new(game: &GameState, queue_generator: &QueueGenerator, gravity_accum: u32) -> Self {
        Self {
            game: game.clone(),
            queue_generator: queue_generator.clone(),
            gravity_accum,
        }
    }
}

pub struct History {
    pub snapshots: VecDeque<UndoSnapshot>,
    pub cursor: usize,
    pub capacity: usize,
}

impl History {
    pub fn new(capacity: usize) -> Self {
        History {
            snapshots: VecDeque::with_capacity(capacity),
            cursor: 0,
            capacity,
        }
    }

    pub fn push(&mut self, snapshot: UndoSnapshot) {
        while self.snapshots.len() > self.cursor {
            self.snapshots.pop_back();
        }
        if self.snapshots.len() >= self.capacity {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(snapshot);
        self.cursor = self.snapshots.len();
    }

    pub fn undo(&mut self) -> Option<UndoSnapshot> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        self.snapshots.get(self.cursor).cloned()
    }

    pub fn redo(&mut self) -> Option<UndoSnapshot> {
        if self.cursor >= self.snapshots.len() {
            return None;
        }
        let result = self.snapshots.get(self.cursor).cloned();
        self.cursor += 1;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::board::Board;
    use crate::core::piece::Cell;
    use crate::core::test_support;
    use crate::core::{
        BagMode, ClearFlashState, GameMode, GravityState, LockDelayState, PieceState, Rotation,
        SpinState, Stats, Tetromino,
    };

    fn dummy_snapshot() -> UndoSnapshot {
        let mut snapshot = test_support::undo_snapshot();
        snapshot.game.board = Board::new(10, 20, 4);
        snapshot.game.highlights.sync_to_board(&snapshot.game.board);
        snapshot.game.current.kind = Tetromino::T;
        snapshot.game.current.x = 4;
        snapshot.game.current.y = 2;
        snapshot.game.queue.visible = vec![Tetromino::T, Tetromino::I];
        snapshot.game.queue.seed = 42;
        snapshot.queue_generator = QueueGenerator::new(BagMode::SevenBag, 42);
        snapshot
    }

    fn full_snapshot() -> UndoSnapshot {
        let mut snapshot = dummy_snapshot();
        snapshot.game.mode = GameMode::Master;
        snapshot.game.board.set(1, 5, Cell::L);
        snapshot.game.highlights.set(3, 0, 9);
        snapshot.game.current = PieceState {
            kind: Tetromino::L,
            rotation: Rotation::Reverse,
            x: 7,
            y: 11,
        };
        snapshot.game.hold.piece = Some(Tetromino::O);
        snapshot.game.hold.swapped_this_turn = true;
        snapshot.game.queue.visible = vec![Tetromino::S, Tetromino::Z, Tetromino::I];
        snapshot.game.queue.seed = 777;
        snapshot.game.stats = Stats {
            damage: 42,
            lines: 9,
            moves: 15,
            b2b: true,
            perfect_clear: true,
            combo: 4,
            lost: true,
            attack_text: Some("Spike".to_string()),
            b2b_text: Some("B2B x3".to_string()),
        };
        snapshot.game.spin = SpinState {
            is_t_spin: true,
            is_mini: true,
            last_kick_index: Some(4),
        };
        snapshot.game.gravity = GravityState {
            gravity_hz: Some(20),
            arr_ms: 2,
            das_ms: 90,
            sdd_ms: 3,
            sds_ms: 4,
            das_cancel: true,
        };
        snapshot.game.ghost_piece = false;
        snapshot.game.auto_color = true;
        snapshot.game.mirror_queue_with_field = true;
        snapshot.game.highlight_clear = true;
        snapshot.game.pc_leftover = 6;
        snapshot.game.garbage_hole_pos = 2;
        snapshot.game.garbage_hole_size = 3;
        snapshot.game.garbage_hole_next_change = 5;
        snapshot.game.lock_delay = LockDelayState {
            active: true,
            timer_ms: 321,
            moves: 8,
            max_moves: 20,
            delay_ms: 650,
        };
        snapshot.game.clear_flash = ClearFlashState {
            active: true,
            lines: vec![18, 19],
            timer_ms: 111,
            duration_ms: 444,
        };
        snapshot.queue_generator = QueueGenerator::with_static_sequence(
            BagMode::Random,
            999,
            vec![Tetromino::Mono, Tetromino::J],
        );
        snapshot.gravity_accum = 87;
        snapshot
    }

    fn dummy_snapshot_with_values(damage: u32) -> UndoSnapshot {
        let mut snapshot = dummy_snapshot();
        snapshot.game.stats.damage = damage;
        snapshot
    }

    #[test]
    fn snapshot_new_captures_full_gameplay_state() {
        let snapshot = full_snapshot();
        let captured = UndoSnapshot::new(
            &snapshot.game,
            &snapshot.queue_generator,
            snapshot.gravity_accum,
        );

        assert_eq!(captured, snapshot);
    }

    #[test]
    fn history_undo_restores_full_gameplay_state() {
        let mut hist = History::new(100);
        let snap = full_snapshot();
        hist.push(snap.clone());

        let restored = hist.undo().unwrap();
        assert_eq!(restored, snap);
    }

    #[test]
    fn history_redo_restores_full_forward_state() {
        let mut hist = History::new(100);
        let first = full_snapshot();
        let mut second = full_snapshot();
        second.game.stats.damage = 88;
        second.game.current.x = 1;
        second.game.clear_flash.timer_ms = 222;
        second.queue_generator.seed = 444;
        second.gravity_accum = 12;

        hist.push(first);
        hist.push(second.clone());

        let _ = hist.undo();
        let restored = hist.redo().unwrap();
        assert_eq!(restored, second);
    }

    #[test]
    fn history_clears_redo_branch_on_push_after_undo() {
        let mut hist = History::new(100);
        hist.push(dummy_snapshot_with_values(10));
        hist.push(dummy_snapshot_with_values(20));

        let _ = hist.undo();
        hist.push(dummy_snapshot_with_values(30));

        assert!(hist.redo().is_none());

        let restored = hist.undo().unwrap();
        assert_eq!(restored.game.stats.damage, 30);

        let restored2 = hist.undo().unwrap();
        assert_eq!(restored2.game.stats.damage, 10);
    }

    #[test]
    fn history_empty_undo_returns_none() {
        let mut hist = History::new(100);
        assert!(hist.undo().is_none());
    }

    #[test]
    fn history_empty_redo_returns_none() {
        let mut hist = History::new(100);
        assert!(hist.redo().is_none());
    }

    #[test]
    fn history_capacity_enforced() {
        let mut hist = History::new(3);
        hist.push(dummy_snapshot_with_values(1));
        hist.push(dummy_snapshot_with_values(2));
        hist.push(dummy_snapshot_with_values(3));
        hist.push(dummy_snapshot_with_values(4));

        assert_eq!(hist.snapshots.len(), 3);
        assert_eq!(hist.snapshots[0].game.stats.damage, 2);
        assert_eq!(hist.snapshots[1].game.stats.damage, 3);
        assert_eq!(hist.snapshots[2].game.stats.damage, 4);
    }
}
