use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use super::game_state::BagMode;
use super::piece::{Tetromino, ALL_PIECES};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueGenerator {
    pub mode: BagMode,
    pub seed: u16,
    pub static_sequence: Vec<Tetromino>,
}

impl QueueGenerator {
    pub fn new(mode: BagMode, seed: u16) -> Self {
        QueueGenerator {
            mode,
            seed,
            static_sequence: Vec::new(),
        }
    }

    pub fn with_static_sequence(mode: BagMode, seed: u16, static_sequence: Vec<Tetromino>) -> Self {
        let mut generator = Self::new(mode, seed);
        generator.set_static_sequence(static_sequence);
        generator
    }

    pub fn set_static_sequence(&mut self, static_sequence: Vec<Tetromino>) {
        self.static_sequence = static_sequence;
    }

    pub fn fill_queue(&mut self, queue: &mut Vec<Tetromino>) {
        if !self.static_sequence.is_empty() && queue.is_empty() {
            queue.extend(self.static_sequence.iter().copied());
        }

        let mut rng = SmallRng::seed_from_u64(self.seed as u64);

        while queue.len() < 7 {
            match self.mode {
                BagMode::SevenBag => {
                    let mut bag: Vec<Tetromino> = ALL_PIECES.to_vec();
                    bag.shuffle(&mut rng);
                    queue.extend(bag);
                }
                BagMode::FourteenBag => {
                    let mut bag: Vec<Tetromino> = ALL_PIECES.to_vec();
                    bag.extend(ALL_PIECES.iter());
                    bag.shuffle(&mut rng);
                    queue.extend(bag);
                }
                BagMode::Random => {
                    for _ in 0..7 {
                        let idx = rng.gen_range(0..7);
                        queue.push(ALL_PIECES[idx]);
                    }
                }
            }
        }

        self.seed = Rng::r#gen(&mut rng);
    }

    pub fn shuffle_bag_ranges(&mut self, queue: &mut [Tetromino]) {
        if queue.len() < 7 {
            return;
        }

        let bag_size = match self.mode {
            BagMode::SevenBag => 7,
            BagMode::FourteenBag => 14,
            BagMode::Random => return,
        };

        let mut rng = SmallRng::seed_from_u64(self.seed as u64);

        let mut start = 0;
        while start + bag_size <= queue.len() {
            let end = start + bag_size;
            let slice = &mut queue[start..end];
            slice.shuffle(&mut rng);
            start = end;
        }

        self.seed = Rng::r#gen(&mut rng);
    }

    pub fn mirror_queue(&self, queue: &mut [Tetromino]) {
        for piece in queue.iter_mut() {
            *piece = super::piece::mirrored_piece(*piece);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count_pieces(queue: &[Tetromino], kind: Tetromino) -> usize {
        queue.iter().filter(|&&p| p == kind).count()
    }

    #[test]
    fn seven_bag_fill_boundaries_each_contain_all_pieces() {
        let mut generator = QueueGenerator::new(BagMode::SevenBag, 42);
        let mut queue: Vec<Tetromino> = Vec::new();

        for fill_num in 0..4 {
            let len_before = queue.len();
            generator.fill_queue(&mut queue);
            let added: Vec<Tetromino> = queue[len_before..].to_vec();
            assert_eq!(
                added.len(),
                7,
                "fill {} added wrong number of pieces",
                fill_num
            );
            for &kind in &ALL_PIECES {
                assert_eq!(
                    added.iter().filter(|&&p| p == kind).count(),
                    1,
                    "fill {} missing piece {:?}: {:?}",
                    fill_num,
                    kind,
                    added
                );
            }
            for _ in 0..7 {
                queue.remove(0);
            }
        }
    }

    #[test]
    fn seven_bag_has_exactly_one_of_each() {
        let mut generator = QueueGenerator::new(BagMode::SevenBag, 42);
        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        let first_7: Vec<_> = queue.iter().take(7).collect();
        assert_eq!(first_7.len(), 7);
        for &kind in &ALL_PIECES {
            assert_eq!(count_pieces(&queue, kind), 1, "Missing piece {:?}", kind);
        }
    }

    #[test]
    fn seven_bag_deterministic() {
        let mut generator1 = QueueGenerator::new(BagMode::SevenBag, 123);
        let mut generator2 = QueueGenerator::new(BagMode::SevenBag, 123);
        let mut q1 = Vec::new();
        let mut q2 = Vec::new();
        generator1.fill_queue(&mut q1);
        generator2.fill_queue(&mut q2);
        assert_eq!(q1, q2);
    }

    #[test]
    fn fourteen_bag_has_two_of_each() {
        let mut generator = QueueGenerator::new(BagMode::FourteenBag, 42);
        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        let first_14: Vec<Tetromino> = queue.iter().take(14).copied().collect();
        assert_eq!(first_14.len(), 14);
        for &kind in &ALL_PIECES {
            assert_eq!(count_pieces(&first_14, kind), 2);
        }
    }

    #[test]
    fn random_mode_produces_valid_pieces() {
        let mut generator = QueueGenerator::new(BagMode::Random, 42);
        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        assert!(queue.len() >= 7);
        for &piece in &queue {
            assert!(ALL_PIECES.contains(&piece));
        }
    }

    #[test]
    fn fill_queue_always_produces_at_least_7() {
        let mut generator = QueueGenerator::new(BagMode::SevenBag, 0);
        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        assert!(queue.len() >= 7);
        let len_before = queue.len();
        generator.fill_queue(&mut queue);
        assert_eq!(queue.len(), len_before);
    }

    #[test]
    fn fill_queue_uses_seed_and_updates_it() {
        let seed = 42u16;
        let mut generator = QueueGenerator::new(BagMode::SevenBag, seed);
        let original_seed = generator.seed;
        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        assert_ne!(
            generator.seed, original_seed,
            "seed should be updated after fill"
        );
    }

    #[test]
    fn shuffle_bag_ranges_seven_bag() {
        let seed = 99u16;
        let mut generator = QueueGenerator::new(BagMode::SevenBag, seed);
        let mut queue = vec![
            Tetromino::I,
            Tetromino::I,
            Tetromino::I,
            Tetromino::I,
            Tetromino::I,
            Tetromino::I,
            Tetromino::I,
            Tetromino::J,
            Tetromino::J,
            Tetromino::J,
            Tetromino::J,
            Tetromino::J,
            Tetromino::J,
            Tetromino::J,
        ];
        generator.shuffle_bag_ranges(&mut queue);
        let bag1: Vec<_> = queue[..7].to_vec();
        let bag2: Vec<_> = queue[7..14].to_vec();
        assert_eq!(bag1.len(), 7);
        assert_eq!(bag2.len(), 7);
    }

    #[test]
    fn mirror_queue_maps_correctly() {
        let generator = QueueGenerator::new(BagMode::SevenBag, 0);
        let mut queue = vec![
            Tetromino::I,
            Tetromino::J,
            Tetromino::S,
            Tetromino::O,
            Tetromino::Z,
            Tetromino::L,
            Tetromino::T,
            Tetromino::Mono,
        ];
        generator.mirror_queue(&mut queue);
        assert_eq!(queue[0], Tetromino::I);
        assert_eq!(queue[1], Tetromino::L);
        assert_eq!(queue[2], Tetromino::Z);
        assert_eq!(queue[3], Tetromino::O);
        assert_eq!(queue[4], Tetromino::S);
        assert_eq!(queue[5], Tetromino::J);
        assert_eq!(queue[6], Tetromino::T);
        assert_eq!(queue[7], Tetromino::Mono);
    }

    #[test]
    fn static_bag_uses_sequence() {
        let mut generator = QueueGenerator::with_static_sequence(
            BagMode::SevenBag,
            0,
            vec![
                Tetromino::I,
                Tetromino::L,
                Tetromino::S,
                Tetromino::O,
                Tetromino::T,
                Tetromino::Z,
                Tetromino::J,
                Tetromino::I,
                Tetromino::I,
                Tetromino::I,
                Tetromino::I,
                Tetromino::I,
                Tetromino::I,
                Tetromino::I,
                Tetromino::O,
                Tetromino::T,
                Tetromino::O,
                Tetromino::T,
                Tetromino::O,
                Tetromino::T,
                Tetromino::O,
            ],
        );

        let mut queue = Vec::new();
        generator.fill_queue(&mut queue);
        assert!(queue.len() >= 7);
        assert_eq!(queue.len(), 21);
        assert_eq!(queue[0], Tetromino::I);
        assert_eq!(queue[1], Tetromino::L);
        assert_eq!(queue[2], Tetromino::S);
        assert_eq!(queue[3], Tetromino::O);
        assert_eq!(queue[4], Tetromino::T);
        assert_eq!(queue[5], Tetromino::Z);
        assert_eq!(queue[6], Tetromino::J);
        assert_eq!(queue[20], Tetromino::O);
    }

    #[test]
    fn empty_static_sequence_disables_static_bag() {
        let generator = QueueGenerator::with_static_sequence(BagMode::SevenBag, 0, Vec::new());

        assert!(generator.static_sequence.is_empty());
    }
}
