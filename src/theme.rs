//! Color themes: TTY ANSI defaults + truecolor presets.

use crate::piece::PieceKind;
use crossterm::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeId {
    Terminal,
    Guideline,
    Catppuccin,
    Dracula,
    Nord,
    Gruvbox,
    Monochrome,
}

impl ThemeId {
    pub const ALL: [ThemeId; 7] = [
        ThemeId::Terminal,
        ThemeId::Guideline,
        ThemeId::Catppuccin,
        ThemeId::Dracula,
        ThemeId::Nord,
        ThemeId::Gruvbox,
        ThemeId::Monochrome,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemeId::Terminal => "Terminal (ANSI)",
            ThemeId::Guideline => "Guideline",
            ThemeId::Catppuccin => "Catppuccin Mocha",
            ThemeId::Dracula => "Dracula",
            ThemeId::Nord => "Nord",
            ThemeId::Gruvbox => "Gruvbox",
            ThemeId::Monochrome => "Monochrome",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "terminal" | "ansi" | "tty" => Some(ThemeId::Terminal),
            "guideline" | "default" => Some(ThemeId::Guideline),
            "catppuccin" | "mocha" => Some(ThemeId::Catppuccin),
            "dracula" => Some(ThemeId::Dracula),
            "nord" => Some(ThemeId::Nord),
            "gruvbox" => Some(ThemeId::Gruvbox),
            "mono" | "monochrome" => Some(ThemeId::Monochrome),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ThemeId::Terminal => "terminal",
            ThemeId::Guideline => "guideline",
            ThemeId::Catppuccin => "catppuccin",
            ThemeId::Dracula => "dracula",
            ThemeId::Nord => "nord",
            ThemeId::Gruvbox => "gruvbox",
            ThemeId::Monochrome => "monochrome",
        }
    }

    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|&t| t == self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub id: ThemeId,
}

impl Theme {
    pub fn new(id: ThemeId) -> Self {
        Self { id }
    }

    pub fn piece(self, kind: PieceKind) -> Color {
        match self.id {
            ThemeId::Terminal => match kind {
                PieceKind::I => Color::Cyan,
                PieceKind::O => Color::Yellow,
                PieceKind::T => Color::Magenta,
                PieceKind::S => Color::Green,
                PieceKind::Z => Color::Red,
                PieceKind::J => Color::Blue,
                PieceKind::L => Color::DarkYellow,
            },
            ThemeId::Guideline => match kind {
                PieceKind::I => rgb(0x00, 0xF0, 0xF0),
                PieceKind::O => rgb(0xF0, 0xF0, 0x00),
                PieceKind::T => rgb(0xA0, 0x00, 0xF0),
                PieceKind::S => rgb(0x00, 0xF0, 0x00),
                PieceKind::Z => rgb(0xF0, 0x00, 0x00),
                PieceKind::J => rgb(0x00, 0x00, 0xF0),
                PieceKind::L => rgb(0xF0, 0xA0, 0x00),
            },
            ThemeId::Catppuccin => match kind {
                PieceKind::I => rgb(0x89, 0xdC, 0xeb),
                PieceKind::O => rgb(0xf9, 0xe2, 0xaf),
                PieceKind::T => rgb(0xcB, 0xa6, 0xf7),
                PieceKind::S => rgb(0xa6, 0xe3, 0xa1),
                PieceKind::Z => rgb(0xf3, 0x8b, 0xa8),
                PieceKind::J => rgb(0x89, 0xb4, 0xfa),
                PieceKind::L => rgb(0xfa, 0xb3, 0x87),
            },
            ThemeId::Dracula => match kind {
                PieceKind::I => rgb(0x8b, 0xe9, 0xfd),
                PieceKind::O => rgb(0xf1, 0xfa, 0x8c),
                PieceKind::T => rgb(0xbd, 0x93, 0xf9),
                PieceKind::S => rgb(0x50, 0xfa, 0x7b),
                PieceKind::Z => rgb(0xff, 0x55, 0x55),
                PieceKind::J => rgb(0x62, 0x72, 0xa4),
                PieceKind::L => rgb(0xff, 0xb8, 0x6c),
            },
            ThemeId::Nord => match kind {
                PieceKind::I => rgb(0x88, 0xc0, 0xd0),
                PieceKind::O => rgb(0xeb, 0xcb, 0x8b),
                PieceKind::T => rgb(0xb4, 0x8e, 0xad),
                PieceKind::S => rgb(0xa3, 0xbe, 0x8c),
                PieceKind::Z => rgb(0xbf, 0x61, 0x6a),
                PieceKind::J => rgb(0x5e, 0x81, 0xac),
                PieceKind::L => rgb(0xd0, 0x87, 0x70),
            },
            ThemeId::Gruvbox => match kind {
                PieceKind::I => rgb(0x83, 0xa5, 0x98),
                PieceKind::O => rgb(0xfa, 0xbd, 0x2f),
                PieceKind::T => rgb(0xd3, 0x86, 0x9b),
                PieceKind::S => rgb(0xb8, 0xbb, 0x26),
                PieceKind::Z => rgb(0xfb, 0x49, 0x34),
                PieceKind::J => rgb(0x45, 0x85, 0x88),
                PieceKind::L => rgb(0xfe, 0x80, 0x19),
            },
            ThemeId::Monochrome => match kind {
                PieceKind::I => Color::White,
                PieceKind::O => Color::Grey,
                PieceKind::T => Color::White,
                PieceKind::S => Color::Grey,
                PieceKind::Z => Color::DarkGrey,
                PieceKind::J => Color::White,
                PieceKind::L => Color::Grey,
            },
        }
    }

    pub fn ghost(self, kind: PieceKind) -> Color {
        match self.id {
            ThemeId::Terminal => match kind {
                PieceKind::I => Color::DarkCyan,
                PieceKind::O => Color::DarkYellow,
                PieceKind::T => Color::DarkMagenta,
                PieceKind::S => Color::DarkGreen,
                PieceKind::Z => Color::DarkRed,
                PieceKind::J => Color::DarkBlue,
                PieceKind::L => Color::DarkYellow,
            },
            ThemeId::Monochrome => Color::DarkGrey,
            _ => dim(self.piece(kind), 0.35),
        }
    }

    pub fn grid_a(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Black,
            ThemeId::Guideline => rgb(0x1a, 0x1a, 0x2e),
            ThemeId::Catppuccin => rgb(0x1e, 0x1e, 0x2e),
            ThemeId::Dracula => rgb(0x21, 0x22, 0x2c),
            ThemeId::Nord => rgb(0x2e, 0x34, 0x40),
            ThemeId::Gruvbox => rgb(0x28, 0x28, 0x28),
            ThemeId::Monochrome => rgb(0x18, 0x18, 0x18),
        }
    }

    pub fn grid_b(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Black,
            ThemeId::Guideline => rgb(0x12, 0x12, 0x20),
            ThemeId::Catppuccin => rgb(0x18, 0x18, 0x25),
            ThemeId::Dracula => rgb(0x19, 0x1a, 0x21),
            ThemeId::Nord => rgb(0x3b, 0x42, 0x52),
            ThemeId::Gruvbox => rgb(0x1d, 0x20, 0x21),
            ThemeId::Monochrome => rgb(0x10, 0x10, 0x10),
        }
    }

    pub fn border(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::DarkGrey,
            ThemeId::Guideline => rgb(0x3d, 0x3d, 0x5c),
            ThemeId::Catppuccin => rgb(0x58, 0x5b, 0x70),
            ThemeId::Dracula => rgb(0x62, 0x72, 0xa4),
            ThemeId::Nord => rgb(0x4c, 0x56, 0x6a),
            ThemeId::Gruvbox => rgb(0x50, 0x49, 0x45),
            ThemeId::Monochrome => Color::DarkGrey,
        }
    }

    pub fn text(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::White,
            _ => Color::White,
        }
    }

    pub fn muted(self) -> Color {
        Color::DarkGrey
    }

    pub fn accent(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Green,
            ThemeId::Guideline => rgb(0x00, 0xe5, 0xa8),
            ThemeId::Catppuccin => rgb(0x94, 0xe2, 0xd5),
            ThemeId::Dracula => rgb(0x50, 0xfa, 0x7b),
            ThemeId::Nord => rgb(0x8f, 0xbc, 0xbb),
            ThemeId::Gruvbox => rgb(0xb8, 0xbb, 0x26),
            ThemeId::Monochrome => Color::White,
        }
    }

    pub fn warn(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Yellow,
            _ => Color::Yellow,
        }
    }

    pub fn danger(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Red,
            _ => Color::Red,
        }
    }

    pub fn flash(self) -> Color {
        Color::White
    }

    pub fn highlight(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Cyan,
            ThemeId::Guideline => rgb(0xff, 0xff, 0x88),
            ThemeId::Catppuccin => rgb(0xf9, 0xe2, 0xaf),
            ThemeId::Dracula => rgb(0xf1, 0xfa, 0x8c),
            ThemeId::Nord => rgb(0xeb, 0xcb, 0x8b),
            ThemeId::Gruvbox => rgb(0xfa, 0xbd, 0x2f),
            ThemeId::Monochrome => Color::White,
        }
    }

    pub fn combo(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Yellow,
            _ => rgb(0xff, 0xcc, 0x00),
        }
    }

    pub fn b2b(self) -> Color {
        match self.id {
            ThemeId::Terminal => Color::Magenta,
            _ => rgb(0xff, 0x66, 0xaa),
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb { r, g, b }
}

fn dim(c: Color, factor: f32) -> Color {
    match c {
        Color::Rgb { r, g, b } => Color::Rgb {
            r: (r as f32 * factor) as u8,
            g: (g as f32 * factor) as u8,
            b: (b as f32 * factor) as u8,
        },
        other => other,
    }
}
