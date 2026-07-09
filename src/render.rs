//! Terminal rendering: dynamic layout, themes, title + settings menus.

use crate::board::Cell;
use crate::game::{ClearType, Game, GamePhase};
use crate::layout::Layout;
use crate::menu::{AppScreen, MenuState, SettingsTab, TitleItem};
use crate::piece::{PieceKind, BOARD_W, VISIBLE_H};
use crate::settings::{GridStyle, Settings};
use crate::theme::Theme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType as TermClear},
};
use std::io::{stdout, Write};

const VISIBLE_START: i32 = 20;

fn cell_fill(color: Color, cell_w: u16, cell_h: u16) -> Vec<String> {
    let ch = match cell_w {
        1 => "█".to_string(),
        2 => "██".to_string(),
        _ => "████".to_string(),
    };
    let line = format!(
        "{}{}{}{}",
        SetBackgroundColor(color),
        SetForegroundColor(color),
        ch,
        ResetColor
    );
    vec![line; cell_h as usize]
}

fn empty_fill(theme: Theme, checker: bool, grid: GridStyle, cell_w: u16, cell_h: u16) -> Vec<String> {
    let bg = match grid {
        GridStyle::Off => theme.grid_b(),
        GridStyle::Flat => theme.grid_b(),
        GridStyle::Checker => {
            if checker {
                theme.grid_a()
            } else {
                theme.grid_b()
            }
        }
    };
    let ch = match cell_w {
        1 => " ".to_string(),
        2 => "  ".to_string(),
        _ => "    ".to_string(),
    };
    // For Off grid still need spaces; Flat same color both
    let line = format!(
        "{}{}{}{}",
        SetBackgroundColor(bg),
        SetForegroundColor(bg),
        ch,
        ResetColor
    );
    vec![line; cell_h as usize]
}

fn ghost_fill(color: Color, cell_w: u16, cell_h: u16, theme: Theme) -> Vec<String> {
    let ch = match cell_w {
        1 => "░".to_string(),
        2 => "░░".to_string(),
        _ => "░░░░".to_string(),
    };
    let line = format!(
        "{}{}{}{}",
        SetForegroundColor(color),
        SetBackgroundColor(theme.grid_b()),
        ch,
        ResetColor
    );
    vec![line; cell_h as usize]
}

fn draw_box(
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    border: Color,
) -> std::io::Result<()> {
    let mut out = stdout();
    queue!(out, SetForegroundColor(border))?;
    queue!(out, MoveTo(x, y), Print("┌"))?;
    for _ in 0..w.saturating_sub(2) {
        queue!(out, Print("─"))?;
    }
    queue!(out, Print("┐"))?;
    for row in 1..h.saturating_sub(1) {
        queue!(out, MoveTo(x, y + row), Print("│"))?;
        queue!(
            out,
            MoveTo(x + w.saturating_sub(1), y + row),
            Print("│")
        )?;
    }
    queue!(out, MoveTo(x, y + h.saturating_sub(1)), Print("└"))?;
    for _ in 0..w.saturating_sub(2) {
        queue!(out, Print("─"))?;
    }
    queue!(out, Print("┘"), ResetColor)?;
    Ok(())
}

fn draw_mini_piece(
    kind: PieceKind,
    ox: u16,
    oy: u16,
    theme: Theme,
    dimmed: bool,
) -> std::io::Result<()> {
    let mut out = stdout();
    for row in 0..3u16 {
        queue!(out, MoveTo(ox, oy + row), Print("        "))?;
    }
    let cells = kind.cells(0);
    let min_x = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let min_y = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let max_x = cells.iter().map(|c| c.0).max().unwrap_or(0);
    let max_y = cells.iter().map(|c| c.1).max().unwrap_or(0);
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;
    let off_x = (4 - w) / 2;
    let off_y = (2 - h) / 2;
    let color = if dimmed {
        theme.muted()
    } else {
        theme.piece(kind)
    };
    for (x, y) in cells {
        let px = ox + ((x - min_x + off_x) * 2) as u16;
        let py = oy + (y - min_y + off_y) as u16;
        let fill = cell_fill(color, 2, 1);
        queue!(out, MoveTo(px, py), Print(&fill[0]))?;
    }
    Ok(())
}

pub fn clear_screen() -> std::io::Result<()> {
    let mut out = stdout();
    queue!(out, Clear(TermClear::All))?;
    out.flush()?;
    Ok(())
}

pub fn draw(
    game: &Game,
    settings: &Settings,
    layout: &Layout,
    menu: &MenuState,
) -> std::io::Result<()> {
    let theme = Theme::new(settings.theme);
    match menu.screen {
        AppScreen::Settings => draw_settings(settings, menu, theme, layout),
        AppScreen::Title => {
            draw_game_frame(game, settings, layout, theme, false)?;
            draw_title_overlay(game, menu, theme, layout)
        }
        AppScreen::Playing => draw_game_frame(game, settings, layout, theme, true),
    }
}

fn draw_game_frame(
    game: &Game,
    settings: &Settings,
    layout: &Layout,
    theme: Theme,
    show_overlays: bool,
) -> std::io::Result<()> {
    let mut out = stdout();
    let l = layout;

    // Title bar
    queue!(
        out,
        MoveTo(l.board_x, l.board_y.saturating_sub(2)),
        SetForegroundColor(theme.accent()),
        Print("cli-tetris"),
        ResetColor,
        SetForegroundColor(theme.muted()),
        Print("  ·  TETR.IO-style"),
        ResetColor
    )?;

    // HOLD
    queue!(
        out,
        MoveTo(l.hold_x, l.hold_y),
        SetForegroundColor(theme.text()),
        Print("HOLD"),
        ResetColor
    )?;
    draw_box(l.hold_x, l.hold_y + 1, 10, 6, theme.border())?;
    for row in 0..4u16 {
        queue!(
            out,
            MoveTo(l.hold_x + 1, l.hold_y + 2 + row),
            Print("        ")
        )?;
    }
    if let Some(h) = game.hold {
        draw_mini_piece(h, l.hold_x + 1, l.hold_y + 2, theme, game.hold_used)?;
    }

    // Board border
    let box_w = l.board_pixel_w + 2;
    let box_h = l.board_pixel_h + 2;
    draw_box(
        l.board_x.saturating_sub(1),
        l.board_y,
        box_w,
        box_h,
        theme.border(),
    )?;

    // Always draw stack; active piece only when playing / paused
    let show_piece = matches!(game.phase, GamePhase::Playing | GamePhase::Paused);
    let ghost_y = game.ghost_y();
    let mut ghost = game.current;
    ghost.y = ghost_y;
    let active_cells = game.current.cells();
    let ghost_cells = ghost.cells();

    for row in 0..VISIBLE_H {
        let by = VISIBLE_START + row;
        let row_flash = game.flashing_rows.contains(&by);
        for sub in 0..l.cell_h {
            let py = l.board_y + 1 + (row as u16) * l.cell_h + sub;
            queue!(out, MoveTo(l.board_x, py))?;
            for col in 0..BOARD_W {
                let checker = (row + col) % 2 == 0;
                let lines = if row_flash {
                    cell_fill(theme.flash(), l.cell_w, 1)
                } else {
                    let is_active = show_piece
                        && active_cells.iter().any(|&(x, y)| x == col && y == by);
                    let is_ghost = show_piece
                        && settings.ghost
                        && game.phase == GamePhase::Playing
                        && ghost_cells.iter().any(|&(x, y)| x == col && y == by);
                    if is_active {
                        cell_fill(theme.piece(game.current.kind), l.cell_w, 1)
                    } else if is_ghost {
                        ghost_fill(theme.ghost(game.current.kind), l.cell_w, 1, theme)
                    } else {
                        match game.board.get(col, by) {
                            Cell::Empty => {
                                empty_fill(theme, checker, settings.grid, l.cell_w, 1)
                            }
                            Cell::Filled(k) => cell_fill(theme.piece(k), l.cell_w, 1),
                        }
                    }
                };
                queue!(out, Print(&lines[0]))?;
            }
        }
    }
    // NEXT
    queue!(
        out,
        MoveTo(l.next_x, l.next_y),
        SetForegroundColor(theme.text()),
        Print("NEXT"),
        ResetColor
    )?;
    let next_box_h = 2 + (settings.next_count as u16).max(1) * 3;
    draw_box(l.next_x, l.next_y + 1, 10, next_box_h, theme.border())?;
    for (i, kind) in game.next_queue().into_iter().enumerate() {
        draw_mini_piece(
            kind,
            l.next_x + 1,
            l.next_y + 2 + (i as u16) * 3,
            theme,
            false,
        )?;
    }

    // Stats
    let stats: [(&str, String); 6] = [
        ("SCORE", format!("{}", game.score)),
        ("HIGH", format!("{}", game.high_score)),
        ("LINES", format!("{}", game.lines)),
        ("LEVEL", format!("{}", game.level)),
        ("TIME", game.time_label()),
        ("PPS", format!("{:.2}", game.pps())),
    ];
    for (i, (label, value)) in stats.iter().enumerate() {
        let row = (i as u16) * 2;
        queue!(
            out,
            MoveTo(l.stats_x, l.stats_y + row),
            SetForegroundColor(theme.muted()),
            Print(format!("{:<5}", label)),
            ResetColor,
            SetForegroundColor(theme.text()),
            Print(format!("{:>8}", value)),
            ResetColor
        )?;
    }

    let combo_y = l.stats_y + 13;
    queue!(
        out,
        MoveTo(l.stats_x, combo_y),
        Print("              "),
        MoveTo(l.stats_x, combo_y + 1),
        Print("              ")
    )?;
    if game.combo > 0 {
        queue!(
            out,
            MoveTo(l.stats_x, combo_y),
            SetForegroundColor(theme.combo()),
            Print(format!("COMBO x{}", game.combo)),
            ResetColor
        )?;
    }
    if game.b2b > 0 {
        queue!(
            out,
            MoveTo(l.stats_x, combo_y + 1),
            SetForegroundColor(theme.b2b()),
            Print(format!("B2B x{}", game.b2b)),
            ResetColor
        )?;
    }

    if game.clear_flash_ms > 0 && game.last_clear != ClearType::None {
        let label = game.last_clear.label();
        let cx = l.board_x + l.board_pixel_w / 2 - (label.len() as u16) / 2;
        let cy = l.board_y + l.board_pixel_h / 2;
        queue!(
            out,
            MoveTo(cx, cy),
            SetForegroundColor(theme.highlight()),
            Print(label),
            ResetColor
        )?;
    }

    if show_overlays {
        match game.phase {
            GamePhase::Paused => {
                center_on_board(l, 0, "PAUSED", theme.warn())?;
                center_on_board(l, 2, "P resume · S settings", theme.muted())?;
            }
            GamePhase::GameOver => {
                center_on_board(l, 0, "TOP OUT", theme.danger())?;
                center_on_board(l, 2, &format!("Score {}", game.score), theme.text())?;
                let hs = if game.score >= game.high_score && game.score > 0 {
                    "NEW HIGH SCORE!".to_string()
                } else {
                    format!("Best {}", game.high_score)
                };
                center_on_board(l, 3, &hs, theme.combo())?;
                center_on_board(l, 5, "R restart · Q quit", theme.muted())?;
            }
            _ => {}
        }
    }

    // Footer
    let foot_y = l.board_y + l.board_pixel_h + 3;
    queue!(
        out,
        MoveTo(l.hold_x.min(2), foot_y),
        SetForegroundColor(theme.muted()),
        Print("←→ move  ↓ soft  Space hard  Z/X rot  C hold  P pause  S settings  R restart  Q quit"),
        ResetColor
    )?;

    out.flush()?;
    Ok(())
}

fn draw_title_overlay(
    game: &Game,
    menu: &MenuState,
    theme: Theme,
    layout: &Layout,
) -> std::io::Result<()> {
    let mut out = stdout();
    let l = layout;
    center_on_board(l, 0, "cli-tetris", theme.accent())?;
    center_on_board(
        l,
        1,
        &format!("High score: {}", game.high_score),
        theme.muted(),
    )?;

    for (i, item) in TitleItem::ALL.iter().enumerate() {
        let selected = menu.title_sel == *item;
        let prefix = if selected { "> " } else { "  " };
        let label = format!("{}{}", prefix, item.label());
        let color = if selected {
            theme.accent()
        } else {
            theme.text()
        };
        center_on_board(l, 3 + i as u16, &label, color)?;
    }
    center_on_board(l, 7, "↑↓ select · Enter confirm · Space start", theme.muted())?;
    out.flush()?;
    Ok(())
}

fn draw_settings(
    settings: &Settings,
    menu: &MenuState,
    theme: Theme,
    layout: &Layout,
) -> std::io::Result<()> {
    let mut out = stdout();
    let l = layout;

    // Centered panel
    let panel_w: u16 = 52;
    let panel_h: u16 = 18;
    let px = l.term_w.saturating_sub(panel_w) / 2;
    let py = l.term_h.saturating_sub(panel_h) / 2;

    // Clear panel area with spaces
    for row in 0..panel_h {
        queue!(
            out,
            MoveTo(px, py + row),
            Print(" ".repeat(panel_w as usize))
        )?;
    }

    draw_box(px, py, panel_w, panel_h, theme.border())?;

    queue!(
        out,
        MoveTo(px + 2, py),
        SetForegroundColor(theme.accent()),
        Print(" SETTINGS "),
        ResetColor
    )?;

    // Tabs
    let mut tab_x = px + 2;
    for tab in SettingsTab::ALL {
        let selected = menu.settings_tab == tab;
        let label = if selected {
            format!("[{}]", tab.label())
        } else {
            format!(" {} ", tab.label())
        };
        let color = if selected {
            theme.accent()
        } else {
            theme.muted()
        };
        queue!(
            out,
            MoveTo(tab_x, py + 2),
            SetForegroundColor(color),
            Print(label),
            ResetColor
        )?;
        tab_x += 14;
    }

    // Rows
    let rows = settings_rows(settings, menu.settings_tab);
    for (i, (name, value)) in rows.iter().enumerate() {
        let selected = menu.settings_row == i;
        let marker = if selected { ">" } else { " " };
        let color = if selected {
            theme.highlight()
        } else {
            theme.text()
        };
        queue!(
            out,
            MoveTo(px + 3, py + 4 + i as u16),
            SetForegroundColor(color),
            Print(format!("{} {:<16} {:>18}", marker, name, value)),
            ResetColor
        )?;
    }

    // Theme preview swatches
    if menu.settings_tab == SettingsTab::Colors {
        let y = py + 4 + rows.len() as u16 + 1;
        queue!(
            out,
            MoveTo(px + 3, y),
            SetForegroundColor(theme.muted()),
            Print("Preview "),
            ResetColor
        )?;
        let mut sx = px + 11;
        for kind in PieceKind::ALL {
            let fill = cell_fill(theme.piece(kind), 2, 1);
            queue!(out, MoveTo(sx, y), Print(&fill[0]))?;
            sx += 3;
        }
    }

    queue!(
        out,
        MoveTo(px + 2, py + panel_h - 2),
        SetForegroundColor(theme.muted()),
        Print("←→ tab/value  ↑↓ row  R reset tab  Esc apply & back"),
        ResetColor
    )?;

    out.flush()?;
    Ok(())
}

fn settings_rows(settings: &Settings, tab: SettingsTab) -> Vec<(String, String)> {
    match tab {
        SettingsTab::Handling => vec![
            ("DAS".into(), format!("{} ms", settings.das_ms)),
            ("ARR".into(), format!("{} ms", settings.arr_ms)),
            ("SDF (soft drop)".into(), format!("{} ms", settings.sdf_ms)),
        ],
        SettingsTab::Video => vec![
            ("Scale".into(), settings.scale.label().into()),
            (
                "Ghost piece".into(),
                if settings.ghost { "On" } else { "Off" }.into(),
            ),
            ("Next count".into(), format!("{}", settings.next_count)),
            ("Grid".into(), settings.grid.label().into()),
            (
                "Center board".into(),
                if settings.center { "On" } else { "Off" }.into(),
            ),
        ],
        SettingsTab::Colors => vec![
            ("Theme".into(), settings.theme.label().into()),
            ("Reset all".into(), "Enter / ←→".into()),
        ],
    }
}

fn center_on_board(l: &Layout, row_off: u16, msg: &str, color: Color) -> std::io::Result<()> {
    let mut out = stdout();
    let cx = l.board_x + l.board_pixel_w / 2 - (msg.len() as u16) / 2;
    let cy = l.board_y + l.board_pixel_h / 2 - 2 + row_off;
    queue!(
        out,
        MoveTo(cx, cy),
        SetForegroundColor(color),
        Print(msg),
        ResetColor
    )?;
    Ok(())
}
