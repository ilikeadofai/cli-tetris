//! Playfield: locked minos, collision, line clears.

use crate::piece::{Piece, PieceKind, BOARD_H, BOARD_W};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Filled(PieceKind),
}

#[derive(Clone, Debug)]
pub struct Board {
    /// Row-major, index = y * BOARD_W + x. y=0 is top of buffer.
    cells: Vec<Cell>,
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::Empty; (BOARD_W * BOARD_H) as usize],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Cell {
        if x < 0 || x >= BOARD_W || y < 0 || y >= BOARD_H {
            return Cell::Filled(PieceKind::I); // wall
        }
        self.cells[(y * BOARD_W + x) as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, cell: Cell) {
        if x >= 0 && x < BOARD_W && y >= 0 && y < BOARD_H {
            self.cells[(y * BOARD_W + x) as usize] = cell;
        }
    }

    pub fn fits(&self, piece: &Piece) -> bool {
        for (x, y) in piece.cells() {
            if x < 0 || x >= BOARD_W || y < 0 || y >= BOARD_H {
                return false;
            }
            if self.get(x, y) != Cell::Empty {
                return false;
            }
        }
        true
    }

    pub fn lock(&mut self, piece: &Piece) {
        for (x, y) in piece.cells() {
            self.set(x, y, Cell::Filled(piece.kind));
        }
    }

    /// Clear full lines. Returns number of lines cleared and which rows (from top).
    pub fn clear_lines(&mut self) -> u32 {
        let mut write_y = BOARD_H - 1;
        let mut cleared = 0u32;

        for read_y in (0..BOARD_H).rev() {
            let full = (0..BOARD_W).all(|x| self.get(x, read_y) != Cell::Empty);
            if full {
                cleared += 1;
                continue;
            }
            if write_y != read_y {
                for x in 0..BOARD_W {
                    let c = self.get(x, read_y);
                    self.set(x, write_y, c);
                }
            }
            write_y -= 1;
        }

        // Fill emptied top rows
        while write_y >= 0 {
            for x in 0..BOARD_W {
                self.set(x, write_y, Cell::Empty);
            }
            write_y -= 1;
        }

        cleared
    }

    /// Ghost piece Y: lowest y where piece still fits when moved down from current.
    pub fn ghost_y(&self, piece: &Piece) -> i32 {
        let mut ghost = *piece;
        while self.fits(&ghost) {
            ghost.y += 1;
        }
        ghost.y - 1
    }

    pub fn is_block_out(&self, piece: &Piece) -> bool {
        !self.fits(piece)
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::PieceKind;

    #[test]
    fn clear_full_row() {
        let mut b = Board::new();
        let y = BOARD_H - 1;
        for x in 0..BOARD_W {
            b.set(x, y, Cell::Filled(PieceKind::I));
        }
        assert_eq!(b.clear_lines(), 1);
        assert_eq!(b.get(0, y), Cell::Empty);
    }

    #[test]
    fn piece_fits_on_empty() {
        let b = Board::new();
        let p = Piece::new(PieceKind::T);
        assert!(b.fits(&p));
    }
}
