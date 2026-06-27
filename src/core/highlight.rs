use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::board::{Board, replace_visible_rows_bottom_aligned, shift_cells_down};
use super::piece::{Rotation, Tetromino, cell_from_piece, piece_shape};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HighlightBoard {
    pub width: usize,
    pub total_height: usize,
    pub cells: Vec<u8>,
}

impl HighlightBoard {
    pub fn new(width: usize, total_height: usize) -> Self {
        Self {
            width,
            total_height,
            cells: vec![0u8; width * total_height],
        }
    }

    pub fn reset(&mut self) {
        self.clear_all();
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, value: u8) {
        self.cells[y * self.width + x] = value;
    }

    pub fn clear_cell(&mut self, x: usize, y: usize) {
        self.set(x, y, 0);
    }

    pub fn clear_all(&mut self) {
        for cell in &mut self.cells {
            *cell = 0;
        }
    }

    pub fn sync_to_board(&mut self, board: &Board) {
        let width = board.width;
        let total_height = board.total_height();

        if self.width != width || self.total_height != total_height {
            *self = Self::new(width, total_height);
        } else {
            self.reset();
        }
    }

    pub fn replace_visible_rows_bottom_aligned(&mut self, visible_rows: &[u8], row_count: usize) {
        replace_visible_rows_bottom_aligned(
            &mut self.cells,
            self.width,
            self.total_height,
            0,
            visible_rows,
            row_count,
            0u8,
        );
    }

    pub fn clear_line(&mut self, y: usize) {
        shift_cells_down(&mut self.cells, self.width, y, 0u8);
    }

    pub fn mirror(&mut self) {
        let w = self.width;
        let h = self.total_height;
        let mut new_cells = vec![0u8; w * h];

        for y in 0..h {
            for x in 0..w {
                new_cells[y * w + (w - 1 - x)] = self.cells[y * w + x];
            }
        }

        self.cells = new_cells;
    }
}

pub fn auto_color_board(board: &mut Board) {
    let total_height = board.total_height();
    let mut visited = vec![false; board.width * total_height];

    for y in 0..total_height {
        for x in 0..board.width {
            let index = y * board.width + x;
            if visited[index] || board.get(x, y) != super::Cell::Gray {
                continue;
            }

            let component = collect_gray_component(board, x, y, &mut visited);
            if component.len() != 4 {
                continue;
            }

            if let Some(piece) = piece_from_coords(&component) {
                let cell = cell_from_piece(piece);
                for &(cell_x, cell_y) in &component {
                    board.set(cell_x, cell_y, cell);
                }
            }
        }
    }
}

fn collect_gray_component(
    board: &Board,
    start_x: usize,
    start_y: usize,
    visited: &mut [bool],
) -> Vec<(usize, usize)> {
    let mut queue = VecDeque::from([(start_x, start_y)]);
    let mut cells = Vec::new();

    while let Some((x, y)) = queue.pop_front() {
        let index = y * board.width + x;
        if visited[index] {
            continue;
        }
        visited[index] = true;

        if board.get(x, y) != super::Cell::Gray {
            continue;
        }

        cells.push((x, y));

        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if x + 1 < board.width {
            queue.push_back((x + 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if y + 1 < board.total_height() {
            queue.push_back((x, y + 1));
        }
    }

    cells
}

pub fn piece_from_coords(coords: &[(usize, usize)]) -> Option<Tetromino> {
    if coords.len() != 4 {
        return None;
    }

    let min_x = coords.iter().map(|(x, _)| *x).min()? as i32;
    let min_y = coords.iter().map(|(_, y)| *y).min()? as i32;
    let mut normalized = coords
        .iter()
        .map(|(x, y)| (*x as i32 - min_x, *y as i32 - min_y))
        .collect::<Vec<_>>();
    normalized.sort_unstable();

    let pieces = [
        Tetromino::I,
        Tetromino::J,
        Tetromino::S,
        Tetromino::O,
        Tetromino::Z,
        Tetromino::L,
        Tetromino::T,
    ];

    for piece in pieces {
        for rotation in [
            Rotation::Spawn,
            Rotation::Left,
            Rotation::Reverse,
            Rotation::Right,
        ] {
            let mut shape_coords = Vec::with_capacity(4);
            let shape = piece_shape(piece, rotation);
            for x in 0..4 {
                for y in 0..4 {
                    if shape[x][y] {
                        shape_coords.push((x as i32, y as i32));
                    }
                }
            }

            let shape_min_x = shape_coords.iter().map(|(x, _)| *x).min()?;
            let shape_min_y = shape_coords.iter().map(|(_, y)| *y).min()?;
            for coordinate in &mut shape_coords {
                coordinate.0 -= shape_min_x;
                coordinate.1 -= shape_min_y;
            }
            shape_coords.sort_unstable();

            if shape_coords == normalized {
                return Some(piece);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::HighlightBoard;
    use crate::core::board::Board;
    use crate::core::highlight::{auto_color_board, piece_from_coords};
    use crate::core::piece::{Cell, Tetromino};

    #[test]
    fn reset_clears_cells() {
        let mut highlights = HighlightBoard::new(3, 4);
        highlights.set(1, 2, 9);
        highlights.reset();
        assert!(highlights.cells.iter().all(|cell| *cell == 0));
    }

    #[test]
    fn clear_cell_zeros_single_position() {
        let mut highlights = HighlightBoard::new(3, 4);
        highlights.set(1, 2, 9);
        highlights.clear_cell(1, 2);
        assert_eq!(highlights.get(1, 2), 0);
    }

    #[test]
    fn replace_visible_rows_bottom_aligned_clears_and_aligns_rows() {
        let mut highlights = HighlightBoard::new(3, 5);
        highlights.set(0, 0, 7);

        highlights.replace_visible_rows_bottom_aligned(&[1, 2, 3, 4, 5, 6], 2);

        assert_eq!(highlights.get(0, 0), 0);
        assert_eq!(highlights.get(0, 3), 1);
        assert_eq!(highlights.get(1, 3), 2);
        assert_eq!(highlights.get(2, 3), 3);
        assert_eq!(highlights.get(0, 4), 4);
        assert_eq!(highlights.get(1, 4), 5);
        assert_eq!(highlights.get(2, 4), 6);
    }

    #[test]
    fn sync_to_board_resizes_or_clears_as_needed() {
        let board = Board::new(4, 3, 1);
        let mut highlights = HighlightBoard::new(3, 4);
        highlights.set(1, 1, 8);

        highlights.sync_to_board(&board);

        assert_eq!(highlights.width, 4);
        assert_eq!(highlights.total_height, 4);
        assert!(highlights.cells.iter().all(|cell| *cell == 0));
    }

    #[test]
    fn mirror_flips_each_row_by_width() {
        let mut highlights = HighlightBoard::new(4, 2);
        highlights.set(0, 0, 1);
        highlights.set(3, 0, 2);
        highlights.set(1, 1, 3);

        highlights.mirror();

        assert_eq!(highlights.get(3, 0), 1);
        assert_eq!(highlights.get(0, 0), 2);
        assert_eq!(highlights.get(2, 1), 3);
    }

    #[test]
    fn piece_from_coords_recognizes_t_piece() {
        let coords = vec![(1, 1), (0, 1), (2, 1), (1, 0)];
        assert_eq!(piece_from_coords(&coords), Some(Tetromino::T));
    }

    #[test]
    fn auto_color_board_recolors_exact_gray_tetrominoes() {
        let mut board = Board::new(10, 20, 4);
        board.set(4, 10, Cell::Gray);
        board.set(3, 10, Cell::Gray);
        board.set(5, 10, Cell::Gray);
        board.set(4, 9, Cell::Gray);

        auto_color_board(&mut board);

        assert_eq!(board.get(4, 10), Cell::T);
        assert_eq!(board.get(3, 10), Cell::T);
        assert_eq!(board.get(5, 10), Cell::T);
        assert_eq!(board.get(4, 9), Cell::T);
    }

    #[test]
    fn auto_color_board_ignores_non_tetromino_regions() {
        let mut board = Board::new(10, 20, 4);
        board.set(1, 1, Cell::Gray);
        board.set(2, 1, Cell::Gray);
        board.set(3, 1, Cell::Gray);

        auto_color_board(&mut board);

        assert_eq!(board.get(1, 1), Cell::Gray);
        assert_eq!(board.get(2, 1), Cell::Gray);
        assert_eq!(board.get(3, 1), Cell::Gray);
    }
}
