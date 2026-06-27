use serde::{Deserialize, Serialize};

pub const MAX_BOARD_WIDTH: usize = 32;
pub const MAX_VISIBLE_HEIGHT: usize = 28;
pub const DEFAULT_BOARD_WIDTH: usize = 10;
pub const DEFAULT_VISIBLE_HEIGHT: usize = 20;
pub const DEFAULT_HIDDEN_ROWS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tetromino {
    I,
    J,
    S,
    O,
    Z,
    L,
    T,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rotation {
    Spawn,
    Left,
    Reverse,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationCommand {
    None,
    Ccw,
    Flip,
    Cw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    I,
    J,
    S,
    O,
    Z,
    L,
    T,
    Gray,
    Ghost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PieceState {
    pub kind: Tetromino,
    pub rotation: Rotation,
    pub x: i32,
    pub y: i32,
}

type Shape = [[bool; 4]; 4];

const SHAPES: [[Shape; 4]; 8] = [
    // I
    [
        [
            [false, true, false, false],
            [false, true, false, false],
            [false, true, false, false],
            [false, true, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, true, true],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, true, false],
            [false, false, true, false],
            [false, false, true, false],
            [false, false, true, false],
        ],
        [
            [false, false, false, false],
            [false, false, false, false],
            [true, true, true, true],
            [false, false, false, false],
        ],
    ],
    // J
    [
        [
            [true, true, false, false],
            [false, true, false, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, true, false],
            [true, true, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, false, false],
            [false, true, false, false],
            [false, true, true, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, true, false],
            [true, false, false, false],
            [false, false, false, false],
        ],
    ],
    // S
    [
        [
            [false, true, false, false],
            [true, true, false, false],
            [true, false, false, false],
            [false, false, false, false],
        ],
        [
            [true, true, false, false],
            [false, true, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, true, false],
            [false, true, true, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, false, false],
            [false, true, true, false],
            [false, false, false, false],
        ],
    ],
    // O
    [
        [
            [false, false, false, false],
            [true, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
    ],
    // Z
    [
        [
            [true, false, false, false],
            [true, true, false, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, true, false],
            [true, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, false, false],
            [false, true, true, false],
            [false, false, true, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [false, true, true, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
    ],
    // L
    [
        [
            [false, true, false, false],
            [false, true, false, false],
            [true, true, false, false],
            [false, false, false, false],
        ],
        [
            [true, false, false, false],
            [true, true, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, true, false],
            [false, true, false, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, true, false],
            [false, false, true, false],
            [false, false, false, false],
        ],
    ],
    // T
    [
        [
            [false, true, false, false],
            [true, true, false, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, false, false],
            [true, true, true, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, true, false, false],
            [false, true, true, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [true, true, true, false],
            [false, true, false, false],
            [false, false, false, false],
        ],
    ],
    // Mono
    [
        [
            [false, false, false, false],
            [false, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [false, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [false, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
        [
            [false, false, false, false],
            [false, true, false, false],
            [false, false, false, false],
            [false, false, false, false],
        ],
    ],
];

pub const ALL_PIECES: [Tetromino; 7] = [
    Tetromino::I,
    Tetromino::J,
    Tetromino::S,
    Tetromino::O,
    Tetromino::Z,
    Tetromino::L,
    Tetromino::T,
];

pub fn piece_shape(kind: Tetromino, rotation: Rotation) -> &'static Shape {
    &SHAPES[kind as usize][rotation as usize]
}

pub fn piece_name(kind: Tetromino) -> char {
    match kind {
        Tetromino::I => 'I',
        Tetromino::J => 'J',
        Tetromino::S => 'S',
        Tetromino::O => 'O',
        Tetromino::Z => 'Z',
        Tetromino::L => 'L',
        Tetromino::T => 'T',
        Tetromino::Mono => 'M',
    }
}

pub fn piece_from_name(ch: char) -> Option<Tetromino> {
    match ch {
        'I' => Some(Tetromino::I),
        'J' => Some(Tetromino::J),
        'S' => Some(Tetromino::S),
        'O' => Some(Tetromino::O),
        'Z' => Some(Tetromino::Z),
        'L' => Some(Tetromino::L),
        'T' => Some(Tetromino::T),
        'M' => Some(Tetromino::Mono),
        _ => None,
    }
}

pub fn cell_from_piece(kind: Tetromino) -> Cell {
    match kind {
        Tetromino::I => Cell::I,
        Tetromino::J => Cell::J,
        Tetromino::S => Cell::S,
        Tetromino::O => Cell::O,
        Tetromino::Z => Cell::Z,
        Tetromino::L => Cell::L,
        Tetromino::T => Cell::T,
        Tetromino::Mono => Cell::Gray,
    }
}

pub fn mirrored_piece(piece: Tetromino) -> Tetromino {
    match piece {
        Tetromino::J => Tetromino::L,
        Tetromino::L => Tetromino::J,
        Tetromino::S => Tetromino::Z,
        Tetromino::Z => Tetromino::S,
        other => other,
    }
}

pub fn mirrored_cell(cell: Cell) -> Cell {
    match cell {
        Cell::J => Cell::L,
        Cell::L => Cell::J,
        Cell::S => Cell::Z,
        Cell::Z => Cell::S,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_name_round_trip() {
        for ch in ['I', 'J', 'S', 'O', 'Z', 'L', 'T', 'M'] {
            let kind = piece_from_name(ch).unwrap();
            assert_eq!(piece_name(kind), ch);
        }
    }

    #[test]
    fn piece_from_name_invalid() {
        assert!(piece_from_name('X').is_none());
        assert!(piece_from_name('-').is_none());
        assert!(piece_from_name(' ').is_none());
    }

    #[test]
    fn piece_name_all_variants() {
        assert_eq!(piece_name(Tetromino::I), 'I');
        assert_eq!(piece_name(Tetromino::J), 'J');
        assert_eq!(piece_name(Tetromino::S), 'S');
        assert_eq!(piece_name(Tetromino::O), 'O');
        assert_eq!(piece_name(Tetromino::Z), 'Z');
        assert_eq!(piece_name(Tetromino::L), 'L');
        assert_eq!(piece_name(Tetromino::T), 'T');
        assert_eq!(piece_name(Tetromino::Mono), 'M');
    }

    #[test]
    fn cell_from_piece_mapping() {
        assert_eq!(cell_from_piece(Tetromino::I), Cell::I);
        assert_eq!(cell_from_piece(Tetromino::J), Cell::J);
        assert_eq!(cell_from_piece(Tetromino::S), Cell::S);
        assert_eq!(cell_from_piece(Tetromino::O), Cell::O);
        assert_eq!(cell_from_piece(Tetromino::Z), Cell::Z);
        assert_eq!(cell_from_piece(Tetromino::L), Cell::L);
        assert_eq!(cell_from_piece(Tetromino::T), Cell::T);
        assert_eq!(cell_from_piece(Tetromino::Mono), Cell::Gray);
    }

    #[test]
    fn each_standard_tetromino_has_exactly_four_cells() {
        let standard = [
            Tetromino::I,
            Tetromino::J,
            Tetromino::S,
            Tetromino::O,
            Tetromino::Z,
            Tetromino::L,
            Tetromino::T,
        ];
        let rotations = [
            Rotation::Spawn,
            Rotation::Left,
            Rotation::Reverse,
            Rotation::Right,
        ];
        for &kind in &standard {
            for &rot in &rotations {
                let shape = piece_shape(kind, rot);
                let count: usize = shape.iter().flatten().filter(|&&b| b).count();
                assert_eq!(count, 4, "{:?} at {:?} has {} cells", kind, rot, count);
            }
        }
    }

    #[test]
    fn mono_has_one_cell() {
        let rotations = [
            Rotation::Spawn,
            Rotation::Left,
            Rotation::Reverse,
            Rotation::Right,
        ];
        for &rot in &rotations {
            let shape = piece_shape(Tetromino::Mono, rot);
            let count: usize = shape.iter().flatten().filter(|&&b| b).count();
            assert_eq!(count, 1, "Mono at {:?} has {} cells", rot, count);
        }
    }

    #[test]
    fn mirrored_piece_j_l() {
        assert_eq!(mirrored_piece(Tetromino::J), Tetromino::L);
        assert_eq!(mirrored_piece(Tetromino::L), Tetromino::J);
    }

    #[test]
    fn mirrored_piece_s_z() {
        assert_eq!(mirrored_piece(Tetromino::S), Tetromino::Z);
        assert_eq!(mirrored_piece(Tetromino::Z), Tetromino::S);
    }

    #[test]
    fn mirrored_piece_identity() {
        assert_eq!(mirrored_piece(Tetromino::I), Tetromino::I);
        assert_eq!(mirrored_piece(Tetromino::O), Tetromino::O);
        assert_eq!(mirrored_piece(Tetromino::T), Tetromino::T);
        assert_eq!(mirrored_piece(Tetromino::Mono), Tetromino::Mono);
    }

    #[test]
    fn mirrored_cell_j_l() {
        assert_eq!(mirrored_cell(Cell::J), Cell::L);
        assert_eq!(mirrored_cell(Cell::L), Cell::J);
    }

    #[test]
    fn mirrored_cell_s_z() {
        assert_eq!(mirrored_cell(Cell::S), Cell::Z);
        assert_eq!(mirrored_cell(Cell::Z), Cell::S);
    }

    #[test]
    fn mirrored_cell_identity() {
        assert_eq!(mirrored_cell(Cell::I), Cell::I);
        assert_eq!(mirrored_cell(Cell::O), Cell::O);
        assert_eq!(mirrored_cell(Cell::T), Cell::T);
        assert_eq!(mirrored_cell(Cell::Empty), Cell::Empty);
        assert_eq!(mirrored_cell(Cell::Gray), Cell::Gray);
        assert_eq!(mirrored_cell(Cell::Ghost), Cell::Ghost);
    }
}
