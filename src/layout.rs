//! Dynamic board layout — horizontal scale only (keeps minos roughly square).
//!
//! Terminal character cells are taller than wide (~2:1). A mino that is
//! `cell_w` columns × 1 row looks near-square when `cell_w == 2`.
//! Double-height cells stretch pieces vertically and are intentionally avoided.

use crate::piece::{BOARD_W, VISIBLE_H};
use crate::settings::{ScaleMode, Settings};
use crossterm::terminal;

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub term_w: u16,
    pub term_h: u16,
    /// Columns per mino (1–4). Always one terminal row tall.
    pub cell_w: u16,
    pub board_x: u16,
    pub board_y: u16,
    pub hold_x: u16,
    pub hold_y: u16,
    pub next_x: u16,
    pub next_y: u16,
    pub stats_x: u16,
    pub stats_y: u16,
    pub board_pixel_w: u16,
    pub board_pixel_h: u16,
    pub title_x: u16,
    pub foot_y: u16,
}

impl Layout {
    pub fn compute(settings: &Settings) -> Self {
        let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
        Self::compute_with_size(settings, term_w, term_h)
    }

    pub fn compute_with_size(settings: &Settings, term_w: u16, term_h: u16) -> Self {
        let cell_w = match settings.scale {
            ScaleMode::X1 => 1,
            ScaleMode::X2 => 2,
            ScaleMode::X3 => 3,
            ScaleMode::X4 => 4,
            ScaleMode::Auto => pick_auto_cell_w(term_w, term_h),
        };

        // Always 1 row per mino — no vertical stretch.
        let board_pixel_w = BOARD_W as u16 * cell_w;
        let board_pixel_h = VISIBLE_H as u16;

        let hold_panel_w: u16 = 14;
        let next_panel_w: u16 = 14;
        let gap: u16 = 3;
        let chrome_top: u16 = 1;
        let chrome_bot: u16 = 2;

        let total_w = hold_panel_w + gap + board_pixel_w + 2 + gap + next_panel_w;
        let total_h = chrome_top + 1 + board_pixel_h + 2 + chrome_bot + 1;

        let (origin_x, origin_y) = if settings.center {
            (
                term_w.saturating_sub(total_w) / 2,
                term_h.saturating_sub(total_h) / 2,
            )
        } else {
            (1, 0)
        };

        let hold_x = origin_x;
        let hold_y = origin_y + chrome_top + 1;
        let board_x = hold_x + hold_panel_w + gap;
        let board_y = hold_y;
        let next_x = board_x + board_pixel_w + 2 + gap;
        let next_y = board_y;
        let stats_x = hold_x;
        // Stats sit under hold panel
        let stats_y = hold_y + 8;
        let title_x = board_x;
        let foot_y = (board_y + board_pixel_h + 3).min(term_h.saturating_sub(1));

        Self {
            term_w,
            term_h,
            cell_w,
            board_x,
            board_y,
            hold_x,
            hold_y,
            next_x,
            next_y,
            stats_x,
            stats_y,
            board_pixel_w,
            board_pixel_h,
            title_x,
            foot_y,
        }
    }
}

/// Largest cell width that fits; prefer square-ish 2+ over skinny 1 when possible.
fn pick_auto_cell_w(term_w: u16, term_h: u16) -> u16 {
    let need_h = VISIBLE_H as u16 + 8; // board + title + footer margin
    if term_h < need_h {
        // Still try to fit board alone
        if term_h < VISIBLE_H as u16 + 4 {
            return 1;
        }
    }

    for cw in [4u16, 3, 2, 1] {
        let need_w = BOARD_W as u16 * cw + 2 + 14 + 14 + 8;
        let need_h = VISIBLE_H as u16 + 8;
        if need_w <= term_w && need_h <= term_h {
            return cw;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn never_uses_multi_row_cells() {
        let mut s = Settings::default();
        s.scale = ScaleMode::Auto;
        let l = Layout::compute_with_size(&s, 200, 60);
        // height is always VISIBLE_H rows of minos
        assert_eq!(l.board_pixel_h, VISIBLE_H as u16);
    }

    #[test]
    fn auto_picks_something() {
        let mut s = Settings::default();
        s.scale = ScaleMode::Auto;
        let l = Layout::compute_with_size(&s, 120, 40);
        assert!((1..=4).contains(&l.cell_w));
    }

    #[test]
    fn large_terminal_prefers_at_least_2() {
        let mut s = Settings::default();
        s.scale = ScaleMode::Auto;
        let l = Layout::compute_with_size(&s, 120, 40);
        assert!(l.cell_w >= 2, "expected wide scale, got {}", l.cell_w);
    }
}
