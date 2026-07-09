//! Tetromino shapes, SRS rotations, and wall-kick tables (guideline / TETR.IO style).

use crossterm::style::Color;

pub const BOARD_W: i32 = 10;
pub const BOARD_H: i32 = 40; // 20 visible + 20 buffer (TETR.IO-style)
pub const VISIBLE_H: i32 = 20;
/// Spawn row: just above the visible field (y grows downward).
pub const SPAWN_Y: i32 = 19;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl PieceKind {
    pub const ALL: [PieceKind; 7] = [
        PieceKind::I,
        PieceKind::O,
        PieceKind::T,
        PieceKind::S,
        PieceKind::Z,
        PieceKind::J,
        PieceKind::L,
    ];

    pub fn color(self) -> Color {
        match self {
            PieceKind::I => Color::Rgb {
                r: 0x00,
                g: 0xF0,
                b: 0xF0,
            },
            PieceKind::O => Color::Rgb {
                r: 0xF0,
                g: 0xF0,
                b: 0x00,
            },
            PieceKind::T => Color::Rgb {
                r: 0xA0,
                g: 0x00,
                b: 0xF0,
            },
            PieceKind::S => Color::Rgb {
                r: 0x00,
                g: 0xF0,
                b: 0x00,
            },
            PieceKind::Z => Color::Rgb {
                r: 0xF0,
                g: 0x00,
                b: 0x00,
            },
            PieceKind::J => Color::Rgb {
                r: 0x00,
                g: 0x00,
                b: 0xF0,
            },
            PieceKind::L => Color::Rgb {
                r: 0xF0,
                g: 0xA0,
                b: 0x00,
            },
        }
    }

    pub fn ghost_color(self) -> Color {
        match self {
            PieceKind::I => Color::Rgb {
                r: 0x00,
                g: 0x60,
                b: 0x60,
            },
            PieceKind::O => Color::Rgb {
                r: 0x60,
                g: 0x60,
                b: 0x00,
            },
            PieceKind::T => Color::Rgb {
                r: 0x40,
                g: 0x00,
                b: 0x60,
            },
            PieceKind::S => Color::Rgb {
                r: 0x00,
                g: 0x60,
                b: 0x00,
            },
            PieceKind::Z => Color::Rgb {
                r: 0x60,
                g: 0x00,
                b: 0x00,
            },
            PieceKind::J => Color::Rgb {
                r: 0x00,
                g: 0x00,
                b: 0x60,
            },
            PieceKind::L => Color::Rgb {
                r: 0x60,
                g: 0x40,
                b: 0x00,
            },
        }
    }

    /// Local minos for rotation 0 (guideline spawn). y+ is down (board coords).
    fn offsets0(self) -> [(i32, i32); 4] {
        match self {
            // .... / IIII  — origin between cells 2–3 of the bar
            PieceKind::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            PieceKind::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            // .T. / TTT
            PieceKind::T => [(-1, 0), (0, 0), (1, 0), (0, -1)],
            // .SS / SS.
            PieceKind::S => [(0, -1), (1, -1), (-1, 0), (0, 0)],
            // ZZ. / .ZZ
            PieceKind::Z => [(-1, -1), (0, -1), (0, 0), (1, 0)],
            // J.. / JJJ
            PieceKind::J => [(-1, -1), (-1, 0), (0, 0), (1, 0)],
            // ..L / LLL
            PieceKind::L => [(1, -1), (-1, 0), (0, 0), (1, 0)],
        }
    }

    /// CW rotate with y+ down: (x, y) → (−y, x)
    fn rotate_cw(pts: [(i32, i32); 4]) -> [(i32, i32); 4] {
        let mut out = [(0, 0); 4];
        for (i, (x, y)) in pts.iter().enumerate() {
            out[i] = (-y, *x);
        }
        out
    }

    pub fn cells(self, rot: u8) -> [(i32, i32); 4] {
        let mut pts = self.offsets0();
        for _ in 0..(rot % 4) {
            pts = Self::rotate_cw(pts);
        }
        pts
    }

    pub fn spawn_pos(self) -> (i32, i32) {
        match self {
            PieceKind::I => (3, SPAWN_Y),
            PieceKind::O => (4, SPAWN_Y - 1),
            _ => (3, SPAWN_Y),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub kind: PieceKind,
    pub x: i32,
    pub y: i32,
    pub rot: u8,
}

impl Piece {
    pub fn new(kind: PieceKind) -> Self {
        let (x, y) = kind.spawn_pos();
        Self {
            kind,
            x,
            y,
            rot: 0,
        }
    }

    pub fn cells(&self) -> [(i32, i32); 4] {
        let local = self.kind.cells(self.rot);
        let mut out = [(0, 0); 4];
        for (i, (lx, ly)) in local.iter().enumerate() {
            out[i] = (self.x + lx, self.y + ly);
        }
        out
    }
}

/// SRS wall kicks. Tables are converted so dy is board-down (SRS y+ up negated).
pub fn wall_kicks(kind: PieceKind, from: u8, to: u8) -> &'static [(i32, i32)] {
    if kind == PieceKind::O {
        return &[(0, 0)];
    }

    let from = from % 4;
    let to = to % 4;

    if kind == PieceKind::I {
        match (from, to) {
            (0, 1) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
            (1, 0) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
            (1, 2) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
            (2, 1) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
            (2, 3) => &[(0, 0), (2, 0), (-1, 0), (2, -1), (-1, 2)],
            (3, 2) => &[(0, 0), (-2, 0), (1, 0), (-2, 1), (1, -2)],
            (3, 0) => &[(0, 0), (1, 0), (-2, 0), (1, 2), (-2, -1)],
            (0, 3) => &[(0, 0), (-1, 0), (2, 0), (-1, -2), (2, 1)],
            _ => &[(0, 0)],
        }
    } else {
        // JLSTZ
        match (from, to) {
            (0, 1) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (1, 0) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            (1, 2) => &[(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            (2, 1) => &[(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
            (2, 3) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            (3, 2) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (3, 0) => &[(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
            (0, 3) => &[(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
            _ => &[(0, 0)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_cycles() {
        assert_eq!(PieceKind::T.cells(0), PieceKind::T.cells(4));
    }

    #[test]
    fn t_spawn_points_up() {
        // stem at y-1
        let c = PieceKind::T.cells(0);
        assert!(c.contains(&(0, -1)));
    }
}
