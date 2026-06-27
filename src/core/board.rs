use serde::{Deserialize, Serialize};

pub use super::highlight::HighlightBoard;
use super::piece::{Cell, PieceState, Tetromino, cell_from_piece, piece_shape};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    pub width: usize,
    pub visible_height: usize,
    pub hidden_rows: usize,
    pub cells: Vec<Cell>,
}

impl Board {
    pub fn new(width: usize, visible_height: usize, hidden_rows: usize) -> Self {
        let total = visible_height + hidden_rows;
        let cells = vec![Cell::Empty; width * total];
        Board {
            width,
            visible_height,
            hidden_rows,
            cells,
        }
    }

    pub fn total_height(&self) -> usize {
        self.visible_height + self.hidden_rows
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.total_height()
    }

    pub fn is_occupied_or_oob(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }
        self.cells[y as usize * self.width + x as usize] != Cell::Empty
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        self.cells[y * self.width + x] = cell;
    }

    pub fn clear_all(&mut self) {
        for c in self.cells.iter_mut() {
            *c = Cell::Empty;
        }
    }

    pub fn replace_visible_rows_bottom_aligned(&mut self, visible_rows: &[Cell], row_count: usize) {
        let width = self.width;
        let visible_height = self.visible_height;
        let hidden_rows = self.hidden_rows;
        replace_visible_rows_bottom_aligned(
            &mut self.cells,
            width,
            visible_height,
            hidden_rows,
            visible_rows,
            row_count,
            Cell::Empty,
        );
    }

    pub fn is_line_full(&self, y: usize) -> bool {
        let start = y * self.width;
        for x in 0..self.width {
            if self.cells[start + x] == Cell::Empty {
                return false;
            }
        }
        true
    }

    pub fn clear_line(&mut self, y: usize) {
        shift_cells_down(&mut self.cells, self.width, y, Cell::Empty);
    }

    pub fn push_line(&mut self) {
        let total = self.total_height();
        for x in 0..self.width {
            for row in 0..total - 1 {
                let dst = row * self.width + x;
                let src = (row + 1) * self.width + x;
                self.cells[dst] = self.cells[src];
            }
        }
        let bottom = total - 1;
        for x in 0..self.width {
            self.cells[bottom * self.width + x] = Cell::Empty;
        }
    }

    pub fn freeze_piece(&mut self, piece: PieceState, piece_kind: Tetromino) {
        let shape = piece_shape(piece.kind, piece.rotation);
        let cell = cell_from_piece(piece_kind);
        for x in 0..4i32 {
            for y in 0..4i32 {
                if shape[x as usize][y as usize] {
                    let bx = piece.x + x;
                    let by = piece.y + y;
                    if self.in_bounds(bx, by) {
                        self.set(bx as usize, by as usize, cell);
                    }
                }
            }
        }
    }
}

pub(crate) fn shift_cells_down<T: Copy>(cells: &mut [T], width: usize, y: usize, empty: T) {
    for x in 0..width {
        for row in (1..=y).rev() {
            let dst = row * width + x;
            let src = (row - 1) * width + x;
            cells[dst] = cells[src];
        }
    }
    for x in 0..width {
        cells[x] = empty;
    }
}

pub(crate) fn replace_visible_rows_bottom_aligned<T: Copy>(
    cells: &mut [T],
    width: usize,
    visible_height: usize,
    hidden_rows: usize,
    visible_rows: &[T],
    row_count: usize,
    empty: T,
) {
    let clamped_rows = row_count.min(visible_height);
    assert_eq!(visible_rows.len(), width * clamped_rows);

    for c in cells.iter_mut() {
        *c = empty;
    }

    let start_row = hidden_rows + (visible_height - clamped_rows);
    for row in 0..clamped_rows {
        let src_start = row * width;
        let dst_start = (start_row + row) * width;
        cells[dst_start..dst_start + width]
            .copy_from_slice(&visible_rows[src_start..src_start + width]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::piece::Cell;

    #[test]
    fn board_new_creates_empty_cells() {
        let b = Board::new(10, 20, 4);
        assert_eq!(b.width, 10);
        assert_eq!(b.visible_height, 20);
        assert_eq!(b.hidden_rows, 4);
        assert_eq!(b.total_height(), 24);
        assert_eq!(b.cells.len(), 240);
        assert!(b.cells.iter().all(|c| *c == Cell::Empty));
    }

    #[test]
    fn board_indexing() {
        let mut b = Board::new(10, 20, 4);
        b.set(3, 5, Cell::T);
        assert_eq!(b.get(3, 5), Cell::T);
        let idx = 5 * b.width + 3;
        assert_eq!(b.cells[idx], Cell::T);
    }

    #[test]
    fn board_in_bounds() {
        let b = Board::new(10, 20, 4);
        assert!(b.in_bounds(0, 0));
        assert!(b.in_bounds(9, 23));
        assert!(b.in_bounds(5, 10));
    }

    #[test]
    fn board_out_of_bounds() {
        let b = Board::new(10, 20, 4);
        assert!(!b.in_bounds(-1, 0));
        assert!(!b.in_bounds(0, -1));
        assert!(!b.in_bounds(10, 0));
        assert!(!b.in_bounds(0, 24));
        assert!(!b.in_bounds(10, 24));
        assert!(!b.in_bounds(-5, -5));
    }

    #[test]
    fn board_is_occupied_or_oob_oob() {
        let b = Board::new(10, 20, 4);
        assert!(b.is_occupied_or_oob(-1, 0));
        assert!(b.is_occupied_or_oob(10, 0));
        assert!(b.is_occupied_or_oob(0, 24));
    }

    #[test]
    fn board_is_occupied_or_oob_empty() {
        let b = Board::new(10, 20, 4);
        assert!(!b.is_occupied_or_oob(0, 0));
        assert!(!b.is_occupied_or_oob(9, 23));
    }

    #[test]
    fn board_is_occupied_or_oob_occupied() {
        let mut b = Board::new(10, 20, 4);
        b.set(4, 5, Cell::I);
        assert!(b.is_occupied_or_oob(4, 5));
        assert!(!b.is_occupied_or_oob(3, 5));
    }

    #[test]
    fn board_clear_all() {
        let mut b = Board::new(10, 20, 4);
        b.set(0, 0, Cell::O);
        b.set(9, 23, Cell::J);
        b.clear_all();
        assert!(b.cells.iter().all(|c| *c == Cell::Empty));
    }

    #[test]
    fn board_get_returns_correct_cell() {
        let mut b = Board::new(5, 10, 2);
        b.set(2, 7, Cell::Z);
        assert_eq!(b.get(2, 7), Cell::Z);
        assert_eq!(b.get(0, 0), Cell::Empty);
    }

    #[test]
    fn replace_visible_rows_bottom_aligned_clears_and_aligns_rows() {
        let mut b = Board::new(3, 4, 2);
        b.set(0, 0, Cell::I);
        b.set(1, 5, Cell::T);

        b.replace_visible_rows_bottom_aligned(
            &[Cell::J, Cell::Empty, Cell::L, Cell::Gray, Cell::S, Cell::Z],
            2,
        );

        for y in 0..4 {
            assert_eq!(b.get(0, y), Cell::Empty);
            assert_eq!(b.get(1, y), Cell::Empty);
            assert_eq!(b.get(2, y), Cell::Empty);
        }

        assert_eq!(b.get(0, 4), Cell::J);
        assert_eq!(b.get(1, 4), Cell::Empty);
        assert_eq!(b.get(2, 4), Cell::L);
        assert_eq!(b.get(0, 5), Cell::Gray);
        assert_eq!(b.get(1, 5), Cell::S);
        assert_eq!(b.get(2, 5), Cell::Z);
    }

    #[test]
    fn highlight_board_construction() {
        let h = HighlightBoard::new(10, 24);
        assert_eq!(h.cells.len(), 240);
    }

    #[test]
    fn highlight_board_clear_line_shifts_values_down() {
        let mut highlights = HighlightBoard::new(3, 4);
        highlights.set(0, 0, 1);
        highlights.set(1, 1, 2);
        highlights.set(2, 2, 3);

        highlights.clear_line(2);

        assert_eq!(highlights.get(0, 0), 0);
        assert_eq!(highlights.get(1, 1), 0);
        assert_eq!(highlights.get(0, 1), 1);
        assert_eq!(highlights.get(1, 2), 2);
        assert_eq!(highlights.get(2, 3), 0);
    }

    #[test]
    fn highlight_board_mirror_flips_columns() {
        let mut highlights = HighlightBoard::new(4, 2);
        highlights.set(0, 0, 1);
        highlights.set(3, 0, 2);
        highlights.set(1, 1, 3);

        highlights.mirror();

        assert_eq!(highlights.get(3, 0), 1);
        assert_eq!(highlights.get(0, 0), 2);
        assert_eq!(highlights.get(2, 1), 3);
    }
}
