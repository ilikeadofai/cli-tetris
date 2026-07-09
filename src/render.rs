//! Terminal rendering: square minos, polished panels, title/settings UI.

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

// ── glyphs ──────────────────────────────────────────────────────────

/// Near-square mino: width scales, always 1 row (no vertical stretch).
fn mino_str(cell_w: u16, kind: MinoStyle) -> String {
    match (cell_w, kind) {
        (1, MinoStyle::Solid) => "█".into(),
        (1, MinoStyle::Ghost) => "░".into(),
        (1, MinoStyle::Empty) => " ".into(),
        (1, MinoStyle::Dot) => "·".into(),
        (2, MinoStyle::Solid) => "██".into(),
        (2, MinoStyle::Ghost) => "░░".into(),
        (2, MinoStyle::Empty) => "  ".into(),
        (2, MinoStyle::Dot) => "··".into(),
        (3, MinoStyle::Solid) => "███".into(),
        (3, MinoStyle::Ghost) => "░░░".into(),
        (3, MinoStyle::Empty) => "   ".into(),
        (3, MinoStyle::Dot) => "···".into(),
        (_, MinoStyle::Solid) => "████".into(),
        (_, MinoStyle::Ghost) => "░░░░".into(),
        (_, MinoStyle::Empty) => "    ".into(),
        (_, MinoStyle::Dot) => "····".into(),
    }
}

#[derive(Clone, Copy)]
enum MinoStyle {
    Solid,
    Ghost,
    Empty,
    Dot,
}

fn paint_bg(fg: Color, bg: Color, text: &str) -> String {
    format!(
        "{}{}{}{}",
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        text,
        ResetColor
    )
}

fn solid_cell(color: Color, cell_w: u16) -> String {
    // Bevel: first col slightly brighter when wide enough
    if cell_w >= 2 {
        let hi = match color {
            Color::Rgb { r, g, b } => Color::Rgb {
                r: r.saturating_add(36).min(255),
                g: g.saturating_add(36).min(255),
                b: b.saturating_add(36).min(255),
            },
            other => other,
        };
        let rest = "█".repeat((cell_w - 1) as usize);
        format!(
            "{}{}█{}{}{}{}",
            SetForegroundColor(hi),
            SetBackgroundColor(color),
            SetForegroundColor(color),
            rest,
            ResetColor,
            ""
        )
    } else {
        paint_bg(color, color, &mino_str(cell_w, MinoStyle::Solid))
    }
}

fn empty_cell(theme: Theme, checker: bool, grid: GridStyle, cell_w: u16) -> String {
    match grid {
        GridStyle::Off => paint_bg(theme.grid_b(), theme.grid_b(), &mino_str(cell_w, MinoStyle::Empty)),
        GridStyle::Flat => paint_bg(theme.grid_b(), theme.grid_b(), &mino_str(cell_w, MinoStyle::Empty)),
        GridStyle::Checker => {
            let bg = if checker {
                theme.grid_a()
            } else {
                theme.grid_b()
            };
            // Subtle dot on checker A cells for depth without noise
            if checker && cell_w >= 2 {
                paint_bg(theme.border(), bg, &mino_str(cell_w, MinoStyle::Dot))
            } else {
                paint_bg(bg, bg, &mino_str(cell_w, MinoStyle::Empty))
            }
        }
    }
}

fn ghost_cell(color: Color, theme: Theme, cell_w: u16) -> String {
    paint_bg(color, theme.grid_b(), &mino_str(cell_w, MinoStyle::Ghost))
}

// ── boxes / panels ──────────────────────────────────────────────────

fn fill_rect(x: u16, y: u16, w: u16, h: u16, bg: Color) -> std::io::Result<()> {
    let mut out = stdout();
    let blank = " ".repeat(w as usize);
    for row in 0..h {
        queue!(
            out,
            MoveTo(x, y + row),
            SetBackgroundColor(bg),
            Print(&blank),
            ResetColor
        )?;
    }
    Ok(())
}

fn draw_box(x: u16, y: u16, w: u16, h: u16, border: Color, fill: Option<Color>) -> std::io::Result<()> {
    let mut out = stdout();
    if let Some(bg) = fill {
        for row in 1..h.saturating_sub(1) {
            let inner = " ".repeat(w.saturating_sub(2) as usize);
            queue!(
                out,
                MoveTo(x + 1, y + row),
                SetBackgroundColor(bg),
                Print(inner),
                ResetColor
            )?;
        }
    }
    queue!(out, SetForegroundColor(border))?;
    if let Some(bg) = fill {
        queue!(out, SetBackgroundColor(bg))?;
    }
    queue!(out, MoveTo(x, y), Print("╭"))?;
    for _ in 0..w.saturating_sub(2) {
        queue!(out, Print("─"))?;
    }
    queue!(out, Print("╮"))?;
    for row in 1..h.saturating_sub(1) {
        queue!(out, MoveTo(x, y + row), Print("│"))?;
        queue!(
            out,
            MoveTo(x + w.saturating_sub(1), y + row),
            Print("│")
        )?;
    }
    queue!(out, MoveTo(x, y + h.saturating_sub(1)), Print("╰"))?;
    for _ in 0..w.saturating_sub(2) {
        queue!(out, Print("─"))?;
    }
    queue!(out, Print("╯"), ResetColor)?;
    Ok(())
}

fn label_chip(x: u16, y: u16, text: &str, theme: Theme) -> std::io::Result<()> {
    let mut out = stdout();
    queue!(
        out,
        MoveTo(x, y),
        SetForegroundColor(theme.panel()),
        SetBackgroundColor(theme.accent()),
        Print(format!(" {} ", text)),
        ResetColor
    )?;
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
        queue!(
            out,
            MoveTo(ox, oy + row),
            SetBackgroundColor(theme.panel()),
            Print("        "),
            ResetColor
        )?;
    }
    let cells = kind.cells(0);
    let min_x = cells.iter().map(|c| c.0).min().unwrap_or(0);
    let min_y = cells.iter().map(|c| c.1).min().unwrap_or(0);
    let max_x = cells.iter().map(|c| c.0).max().unwrap_or(0);
    let max_y = cells.iter().map(|c| c.1).max().unwrap_or(0);
    let w = max_x - min_x + 1;
    let h = max_y - min_y + 1;
    let off_x = (4 - w) / 2;
    let off_y = (2 - h).max(0) / 2;
    let color = if dimmed {
        theme.muted()
    } else {
        theme.piece(kind)
    };
    for (x, y) in cells {
        let px = ox + ((x - min_x + off_x) * 2) as u16;
        let py = oy + (y - min_y + off_y) as u16;
        queue!(out, MoveTo(px, py), Print(solid_cell(color, 2)))?;
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
            draw_title_modal(game, menu, theme, layout)
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
    let cw = l.cell_w;

    // Header
    queue!(
        out,
        MoveTo(l.title_x, l.board_y.saturating_sub(1)),
        SetForegroundColor(theme.accent()),
        Print("◆ cli-tetris"),
        ResetColor,
        SetForegroundColor(theme.muted()),
        Print(format!("  ·  {}col", cw)),
        ResetColor
    )?;

    // ── HOLD panel ──
    draw_box(l.hold_x, l.hold_y, 12, 6, theme.border(), Some(theme.panel()))?;
    label_chip(l.hold_x + 1, l.hold_y, "HOLD", theme)?;
    if let Some(h) = game.hold {
        draw_mini_piece(h, l.hold_x + 2, l.hold_y + 2, theme, game.hold_used)?;
    }

    // ── Board ──
    let box_w = l.board_pixel_w + 2;
    let box_h = l.board_pixel_h + 2;
    draw_box(
        l.board_x.saturating_sub(1),
        l.board_y,
        box_w,
        box_h,
        theme.border(),
        None,
    )?;

    let show_piece = matches!(game.phase, GamePhase::Playing | GamePhase::Paused);
    let ghost_y = game.ghost_y();
    let mut ghost = game.current;
    ghost.y = ghost_y;
    let active_cells = game.current.cells();
    let ghost_cells = ghost.cells();

    for row in 0..VISIBLE_H {
        let by = VISIBLE_START + row;
        let row_flash = game.flashing_rows.contains(&by);
        let py = l.board_y + 1 + row as u16;
        queue!(out, MoveTo(l.board_x, py))?;
        for col in 0..BOARD_W {
            let checker = (row + col) % 2 == 0;
            let cell = if row_flash {
                solid_cell(theme.flash(), cw)
            } else {
                let is_active =
                    show_piece && active_cells.iter().any(|&(x, y)| x == col && y == by);
                let is_ghost = show_piece
                    && settings.ghost
                    && game.phase == GamePhase::Playing
                    && ghost_cells.iter().any(|&(x, y)| x == col && y == by);
                if is_active {
                    solid_cell(theme.piece(game.current.kind), cw)
                } else if is_ghost {
                    ghost_cell(theme.ghost(game.current.kind), theme, cw)
                } else {
                    match game.board.get(col, by) {
                        Cell::Empty => empty_cell(theme, checker, settings.grid, cw),
                        Cell::Filled(k) => solid_cell(theme.piece(k), cw),
                    }
                }
            };
            queue!(out, Print(cell))?;
        }
    }

    // ── NEXT panel ──
    let next_inner = (settings.next_count as u16).max(1) * 3;
    let next_h = next_inner + 2;
    draw_box(
        l.next_x,
        l.next_y,
        12,
        next_h,
        theme.border(),
        Some(theme.panel()),
    )?;
    label_chip(l.next_x + 1, l.next_y, "NEXT", theme)?;
    for (i, kind) in game.next_queue().into_iter().enumerate() {
        draw_mini_piece(
            kind,
            l.next_x + 2,
            l.next_y + 1 + (i as u16) * 3,
            theme,
            false,
        )?;
    }

    // ── Stats card under hold ──
    let stats_h = 14u16;
    draw_box(
        l.stats_x,
        l.stats_y,
        12,
        stats_h,
        theme.border(),
        Some(theme.panel()),
    )?;
    label_chip(l.stats_x + 1, l.stats_y, "STATS", theme)?;

    let stats: [(&str, String); 6] = [
        ("score", format!("{}", game.score)),
        ("best", format!("{}", game.high_score)),
        ("lines", format!("{}", game.lines)),
        ("level", format!("{}", game.level)),
        ("time", game.time_label()),
        ("pps", format!("{:.2}", game.pps())),
    ];
    for (i, (label, value)) in stats.iter().enumerate() {
        let row = l.stats_y + 2 + i as u16;
        queue!(
            out,
            MoveTo(l.stats_x + 1, row),
            SetBackgroundColor(theme.panel()),
            SetForegroundColor(theme.muted()),
            Print(format!("{:<5}", label)),
            SetForegroundColor(theme.text()),
            Print(format!("{:>5}", value)),
            ResetColor
        )?;
    }

    // Combo / B2B badges
    let badge_y = l.stats_y + stats_h.saturating_sub(2);
    queue!(
        out,
        MoveTo(l.stats_x + 1, badge_y),
        SetBackgroundColor(theme.panel()),
        Print("          "),
        ResetColor
    )?;
    if game.combo > 0 {
        queue!(
            out,
            MoveTo(l.stats_x + 1, badge_y),
            SetBackgroundColor(theme.panel()),
            SetForegroundColor(theme.combo()),
            Print(format!("combo×{}", game.combo)),
            ResetColor
        )?;
    }
    if game.b2b > 0 {
        queue!(
            out,
            MoveTo(l.stats_x + 1, badge_y + 1),
            SetBackgroundColor(theme.panel()),
            SetForegroundColor(theme.b2b()),
            Print(format!("b2b×{}", game.b2b)),
            ResetColor
        )?;
    }

    // Clear-type banner
    if game.clear_flash_ms > 0 && game.last_clear != ClearType::None {
        let label = game.last_clear.label();
        center_on_board(l, 0, label, theme.highlight())?;
    }

    if show_overlays {
        match game.phase {
            GamePhase::Paused => {
                draw_modal_card(
                    l,
                    theme,
                    &[
                        ("PAUSED", theme.warn()),
                        ("", theme.muted()),
                        ("P  resume", theme.text()),
                        ("S  settings", theme.text()),
                    ],
                )?;
            }
            GamePhase::GameOver => {
                let hs = if game.score >= game.high_score && game.score > 0 {
                    "★ NEW HIGH SCORE".to_string()
                } else {
                    String::new()
                };
                draw_modal_card_owned(
                    l,
                    theme,
                    vec![
                        ("TOP OUT".into(), theme.danger()),
                        (String::new(), theme.muted()),
                        (format!("score  {}", game.score), theme.text()),
                        (format!("best   {}", game.high_score), theme.muted()),
                        (hs, theme.combo()),
                        (String::new(), theme.muted()),
                        ("R restart · Q title".into(), theme.muted()),
                    ],
                )?;
            }
            _ => {}
        }
    }

    // Footer
    queue!(
        out,
        MoveTo(0, l.foot_y),
        SetForegroundColor(theme.muted()),
        Print(format!(
            " ←→ move  ↓ soft  space hard  z/x rot  c hold  p pause  s settings  r restart  q title "
        )),
        ResetColor
    )?;

    out.flush()?;
    Ok(())
}

fn draw_title_modal(
    game: &Game,
    menu: &MenuState,
    theme: Theme,
    layout: &Layout,
) -> std::io::Result<()> {
    let mut items = vec![
        ("cli-tetris".into(), theme.accent()),
        (format!("best  {}", game.high_score), theme.muted()),
        (String::new(), theme.muted()),
    ];
    for item in TitleItem::ALL {
        let selected = menu.title_sel == item;
        let label = if selected {
            format!(" › {}  ", item.label())
        } else {
            format!("   {}  ", item.label())
        };
        let color = if selected {
            theme.highlight()
        } else {
            theme.text()
        };
        items.push((label, color));
    }
    items.push((String::new(), theme.muted()));
    items.push(("↑↓  enter · space start".into(), theme.muted()));
    draw_modal_card_owned(layout, theme, items)
}

fn draw_modal_card(
    l: &Layout,
    theme: Theme,
    lines: &[(&str, Color)],
) -> std::io::Result<()> {
    let owned: Vec<(String, Color)> = lines
        .iter()
        .map(|(s, c)| ((*s).to_string(), *c))
        .collect();
    draw_modal_card_owned(l, theme, owned)
}

fn draw_modal_card_owned(
    l: &Layout,
    theme: Theme,
    lines: Vec<(String, Color)>,
) -> std::io::Result<()> {
    let mut out = stdout();
    let content_w = lines
        .iter()
        .map(|(s, _)| s.chars().count())
        .max()
        .unwrap_or(10)
        .max(18) as u16
        + 4;
    let content_h = lines.len() as u16 + 2;
    let px = l.board_x + l.board_pixel_w.saturating_sub(content_w) / 2;
    let py = l.board_y + l.board_pixel_h.saturating_sub(content_h) / 2;

    fill_rect(px, py, content_w, content_h, theme.panel())?;
    draw_box(px, py, content_w, content_h, theme.border(), Some(theme.panel()))?;

    for (i, (text, color)) in lines.iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        let tx = px + (content_w.saturating_sub(text.chars().count() as u16)) / 2;
        let selected = text.contains('›');
        if selected {
            // highlight bar
            let bar = " ".repeat(content_w.saturating_sub(2) as usize);
            queue!(
                out,
                MoveTo(px + 1, py + 1 + i as u16),
                SetBackgroundColor(theme.select_bg()),
                Print(bar),
                ResetColor
            )?;
            queue!(
                out,
                MoveTo(tx, py + 1 + i as u16),
                SetBackgroundColor(theme.select_bg()),
                SetForegroundColor(*color),
                Print(text),
                ResetColor
            )?;
        } else {
            queue!(
                out,
                MoveTo(tx, py + 1 + i as u16),
                SetBackgroundColor(theme.panel()),
                SetForegroundColor(*color),
                Print(text),
                ResetColor
            )?;
        }
    }
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

    let panel_w: u16 = 56.min(l.term_w.saturating_sub(4));
    let panel_h: u16 = 20.min(l.term_h.saturating_sub(2));
    let px = l.term_w.saturating_sub(panel_w) / 2;
    let py = l.term_h.saturating_sub(panel_h) / 2;

    fill_rect(px, py, panel_w, panel_h, theme.panel())?;
    draw_box(px, py, panel_w, panel_h, theme.border(), Some(theme.panel()))?;
    label_chip(px + 2, py, "SETTINGS", theme)?;

    // Tabs with underline
    let mut tab_x = px + 2;
    for tab in SettingsTab::ALL {
        let selected = menu.settings_tab == tab;
        let label = format!(" {} ", tab.label());
        if selected {
            queue!(
                out,
                MoveTo(tab_x, py + 2),
                SetBackgroundColor(theme.select_bg()),
                SetForegroundColor(theme.accent()),
                Print(&label),
                ResetColor
            )?;
            // underline
            queue!(
                out,
                MoveTo(tab_x, py + 3),
                SetForegroundColor(theme.accent()),
                Print("─".repeat(label.len())),
                ResetColor
            )?;
        } else {
            queue!(
                out,
                MoveTo(tab_x, py + 2),
                SetBackgroundColor(theme.panel()),
                SetForegroundColor(theme.muted()),
                Print(&label),
                ResetColor
            )?;
            queue!(
                out,
                MoveTo(tab_x, py + 3),
                SetBackgroundColor(theme.panel()),
                Print(" ".repeat(label.len())),
                ResetColor
            )?;
        }
        tab_x += label.len() as u16 + 2;
    }

    let rows = settings_rows(settings, menu.settings_tab);
    let row_w = panel_w.saturating_sub(4) as usize;
    for (i, (name, value)) in rows.iter().enumerate() {
        let y = py + 5 + i as u16;
        let selected = menu.settings_row == i;
        let marker = if selected { "›" } else { " " };
        let mut line = format!("{marker} {name:<18} {value:>16}");
        // pad / truncate to panel width
        let count = line.chars().count();
        if count < row_w {
            line.push_str(&" ".repeat(row_w - count));
        } else if count > row_w {
            line = line.chars().take(row_w).collect();
        }
        if selected {
            queue!(
                out,
                MoveTo(px + 2, y),
                SetBackgroundColor(theme.select_bg()),
                SetForegroundColor(theme.highlight()),
                Print(line),
                ResetColor
            )?;
        } else {
            queue!(
                out,
                MoveTo(px + 2, y),
                SetBackgroundColor(theme.panel()),
                SetForegroundColor(theme.text()),
                Print(line),
                ResetColor
            )?;
        }
    }

    if menu.settings_tab == SettingsTab::Colors {
        let y = py + 5 + rows.len() as u16 + 1;
        queue!(
            out,
            MoveTo(px + 3, y),
            SetBackgroundColor(theme.panel()),
            SetForegroundColor(theme.muted()),
            Print("preview  "),
            ResetColor
        )?;
        let mut sx = px + 12;
        for kind in PieceKind::ALL {
            queue!(out, MoveTo(sx, y), Print(solid_cell(theme.piece(kind), 2)))?;
            sx += 3;
        }
    }

    // Hint bar
    queue!(
        out,
        MoveTo(px + 2, py + panel_h - 2),
        SetBackgroundColor(theme.panel()),
        SetForegroundColor(theme.muted()),
        Print("[ ] tabs   ↑↓ row   ←→ value   R reset tab   Esc save"),
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
            ("Reset all defaults".into(), "←→ apply".into()),
        ],
    }
}

fn center_on_board(l: &Layout, row_off: u16, msg: &str, color: Color) -> std::io::Result<()> {
    let mut out = stdout();
    let cx = l.board_x + l.board_pixel_w / 2 - (msg.chars().count() as u16) / 2;
    let cy = l.board_y + l.board_pixel_h / 2 + row_off;
    queue!(
        out,
        MoveTo(cx, cy),
        SetForegroundColor(color),
        Print(msg),
        ResetColor
    )?;
    Ok(())
}
