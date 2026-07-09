//! Dynamic board layout from terminal size + scale setting.

use crate::piece::{BOARD_W, VISIBLE_H};
use crate::settings::{ScaleMode, Settings};
use crossterm::terminal;

/// Pixel-ish cell size in terminal columns / rows.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub term_w: u16,
    pub term_h: u16,
    /// Columns per mino (1, 2, or 4 for scales).
    pub cell_w: u16,
    /// Rows per mino (1 or 2).
    pub cell_h: u16,
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
}

impl Layout {
    pub fn compute(settings: &Settings) -> Self {
        let (term_w, term_h) = terminal::size().unwrap_or((80, 24));
        Self::compute_with_size(settings, term_w, term_h)
    }

    pub fn compute_with_size(settings: &Settings, term_w: u16, term_h: u16) -> Self {
        let factor = match settings.scale {
            ScaleMode::X1 => 1,
            ScaleMode::X2 => 2,
            ScaleMode::X3 => 3,
            ScaleMode::Auto => pick_auto_scale(term_w, term_h),
        };

        let (cell_w, cell_h) = match factor {
            1 => (1u16, 1u16),
            2 => (2, 1),
            _ => (2, 2), // 3x: double-height cells
        };

        let board_pixel_w = BOARD_W as u16 * cell_w;
        let board_pixel_h = VISIBLE_H as u16 * cell_h;

        // Side panels
        let hold_panel_w: u16 = 12;
        let next_panel_w: u16 = 12;
        let gap: u16 = 2;
        let chrome_top: u16 = 2;
        let chrome_bot: u16 = 2;

        let total_w = hold_panel_w + gap + board_pixel_w + 2 + gap + next_panel_w;
        let total_h = chrome_top + board_pixel_h + 2 + chrome_bot;

        let (origin_x, origin_y) = if settings.center {
            let ox = term_w.saturating_sub(total_w) / 2;
            let oy = term_h.saturating_sub(total_h) / 2;
            (ox.max(0), oy.max(0))
        } else {
            (1, 1)
        };

        let hold_x = origin_x;
        let hold_y = origin_y + chrome_top;
        let board_x = hold_x + hold_panel_w + gap;
        let board_y = origin_y + chrome_top;
        let next_x = board_x + board_pixel_w + 2 + gap;
        let next_y = board_y;
        let stats_x = hold_x;
        let stats_y = hold_y + 8;

        Self {
            term_w,
            term_h,
            cell_w,
            cell_h,
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
        }
    }
}

fn pick_auto_scale(term_w: u16, term_h: u16) -> u8 {
    // Prefer largest that fits
    for factor in [3u8, 2, 1] {
        let (cw, ch) = match factor {
            1 => (1u16, 1u16),
            2 => (2, 1),
            _ => (2, 2),
        };
        let bw = BOARD_W as u16 * cw + 2 + 12 + 12 + 6;
        let bh = VISIBLE_H as u16 * ch + 6;
        if bw + 2 <= term_w && bh + 2 <= term_h {
            return factor;
        }
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;

    #[test]
    fn auto_picks_something() {
        let mut s = Settings::default();
        s.scale = ScaleMode::Auto;
        let l = Layout::compute_with_size(&s, 120, 40);
        assert!(l.cell_w >= 1);
        assert!(l.board_pixel_w > 0);
    }
}
