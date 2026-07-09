//! Terminal rendering (TETR.IO-inspired layout: hold | board | next + stats).

use crate::board::Cell;
use crate::game::{ClearType, Game, GamePhase};
use crate::piece::{PieceKind, BOARD_W, VISIBLE_H};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType as TermClear},
};
use std::io::{stdout, Write};

const CELL_W: i32 = 2;

fn cell_block(color: Color) -> String {
    format!(
        "{}{}██{}",
        SetBackgroundColor(color),
        SetForegroundColor(color),
        ResetColor
    )
}

fn empty_cell(checker: bool) -> String {
    let bg = if checker {
        Color::Rgb {
            r: 0x1a,
            g: 0x1a,
            b: 0x2e,
        }
    } else {
        Color::Rgb {
            r: 0x12,
            g: 0x12,
            b: 0x20,
        }
    };
    format!(
        "{}{}  {}",
        SetBackgroundColor(bg),
        SetForegroundColor(bg),
        ResetColor
    )
}

fn ghost_block(color: Color) -> String {
    format!(
        "{}{}░░{}",
        SetForegroundColor(color),
        SetBackgroundColor(Color::Rgb {
            r: 0x12,
            g: 0x12,
            b: 0x20
        }),
        ResetColor
    )
}

fn draw_mini(kind: PieceKind, ox: u16, oy: u16) -> std::io::Result<()> {
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

    for (x, y) in cells {
        let px = ox + ((x - min_x + off_x) * CELL_W) as u16;
        let py = oy + (y - min_y + off_y) as u16;
        queue!(out, MoveTo(px, py), Print(cell_block(kind.color())))?;
    }
    Ok(())
}

pub fn draw(game: &Game) -> std::io::Result<()> {
    let mut out = stdout();
    queue!(out, Clear(TermClear::All))?;

    let board_x: u16 = 16;
    let board_y: u16 = 2;
    let visible_start = 20i32;

    queue!(
        out,
        MoveTo(board_x, 0),
        SetForegroundColor(Color::Rgb {
            r: 0x00,
            g: 0xe5,
            b: 0xa8
        }),
        Print("cli-tetris"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print("  ·  TETR.IO-style guideline"),
        ResetColor
    )?;

    // HOLD
    queue!(
        out,
        MoveTo(2, board_y),
        SetForegroundColor(Color::White),
        Print("HOLD"),
        ResetColor
    )?;
    draw_box(2, board_y + 1, 10, 6)?;
    if let Some(h) = game.hold {
        let color = if game.hold_used {
            Color::DarkGrey
        } else {
            h.color()
        };
        let cells = h.cells(0);
        let min_x = cells.iter().map(|c| c.0).min().unwrap_or(0);
        let min_y = cells.iter().map(|c| c.1).min().unwrap_or(0);
        let max_x = cells.iter().map(|c| c.0).max().unwrap_or(0);
        let max_y = cells.iter().map(|c| c.1).max().unwrap_or(0);
        let w = max_x - min_x + 1;
        let hh = max_y - min_y + 1;
        let off_x = (4 - w) / 2;
        let off_y = (2 - hh) / 2;
        for (x, y) in cells {
            let px = 4 + ((x - min_x + off_x) * CELL_W) as u16;
            let py = board_y + 2 + (y - min_y + off_y) as u16;
            queue!(out, MoveTo(px, py), Print(cell_block(color)))?;
        }
    }

    let bw = (BOARD_W * CELL_W) as u16;
    let bh = VISIBLE_H as u16;
    draw_box(board_x - 1, board_y, bw + 2, bh + 2)?;

    let ghost_y = game.ghost_y();
    let mut ghost = game.current;
    ghost.y = ghost_y;
    let active_cells = game.current.cells();
    let ghost_cells = ghost.cells();

    for row in 0..VISIBLE_H {
        let by = visible_start + row;
        queue!(out, MoveTo(board_x, board_y + 1 + row as u16))?;
        for col in 0..BOARD_W {
            let checker = (row + col) % 2 == 0;
            let is_active = active_cells.iter().any(|&(x, y)| x == col && y == by);
            let is_ghost = ghost_cells.iter().any(|&(x, y)| x == col && y == by);

            if is_active {
                queue!(out, Print(cell_block(game.current.kind.color())))?;
            } else if is_ghost && game.phase == GamePhase::Playing {
                queue!(out, Print(ghost_block(game.current.kind.ghost_color())))?;
            } else {
                match game.board.get(col, by) {
                    Cell::Empty => queue!(out, Print(empty_cell(checker)))?,
                    Cell::Filled(k) => queue!(out, Print(cell_block(k.color())))?,
                }
            }
        }
    }

    // NEXT
    let next_x = board_x + bw + 3;
    queue!(
        out,
        MoveTo(next_x, board_y),
        SetForegroundColor(Color::White),
        Print("NEXT"),
        ResetColor
    )?;
    draw_box(next_x, board_y + 1, 10, 18)?;
    for (i, kind) in game.next_queue().into_iter().enumerate() {
        draw_mini(kind, next_x + 1, board_y + 2 + (i as u16) * 3)?;
    }

    // Stats
    let stats_y = board_y + 9;
    for (label, value, row) in [
        ("SCORE", format!("{}", game.score), 0u16),
        ("LINES", format!("{}", game.lines), 3),
        ("LEVEL", format!("{}", game.level), 6),
    ] {
        queue!(
            out,
            MoveTo(2, stats_y + row),
            SetForegroundColor(Color::DarkGrey),
            Print(label),
            ResetColor
        )?;
        queue!(
            out,
            MoveTo(2, stats_y + row + 1),
            SetForegroundColor(Color::White),
            Print(format!("{:>10}", value)),
            ResetColor
        )?;
    }

    if game.combo > 0 {
        queue!(
            out,
            MoveTo(2, stats_y + 9),
            SetForegroundColor(Color::Rgb {
                r: 0xff,
                g: 0xcc,
                b: 0x00
            }),
            Print(format!("COMBO x{}", game.combo)),
            ResetColor
        )?;
    }
    if game.b2b > 0 {
        queue!(
            out,
            MoveTo(2, stats_y + 10),
            SetForegroundColor(Color::Rgb {
                r: 0xff,
                g: 0x66,
                b: 0xaa
            }),
            Print(format!("B2B x{}", game.b2b)),
            ResetColor
        )?;
    }

    if game.clear_flash_ms > 0 && game.last_clear != ClearType::None {
        let label = game.last_clear.label();
        let cx = board_x + bw / 2 - (label.len() as u16) / 2;
        queue!(
            out,
            MoveTo(cx, board_y + bh / 2),
            SetForegroundColor(Color::Rgb {
                r: 0xff,
                g: 0xff,
                b: 0x88
            }),
            Print(label),
            ResetColor
        )?;
    }

    match game.phase {
        GamePhase::Paused => {
            center_msg(board_x, board_y, bw, bh, "PAUSED", Color::Yellow)?;
            center_msg(
                board_x,
                board_y + 2,
                bw,
                bh,
                "P to resume",
                Color::DarkGrey,
            )?;
        }
        GamePhase::GameOver => {
            center_msg(board_x, board_y, bw, bh, "TOP OUT", Color::Red)?;
            center_msg(
                board_x,
                board_y + 2,
                bw,
                bh,
                "R restart · Q quit",
                Color::DarkGrey,
            )?;
        }
        GamePhase::Playing => {}
    }

    let foot_y = board_y + bh + 3;
    queue!(
        out,
        MoveTo(2, foot_y),
        SetForegroundColor(Color::DarkGrey),
        Print("←→ move  ↓ soft  Space hard  Z/X/↑ rot  C hold  A 180  P pause  R restart  Q quit"),
        ResetColor
    )?;

    out.flush()?;
    Ok(())
}

fn center_msg(
    board_x: u16,
    board_y: u16,
    bw: u16,
    bh: u16,
    msg: &str,
    color: Color,
) -> std::io::Result<()> {
    let mut out = stdout();
    let cx = board_x + bw / 2 - (msg.len() as u16) / 2;
    let cy = board_y + bh / 2;
    queue!(
        out,
        MoveTo(cx, cy),
        SetForegroundColor(color),
        Print(msg),
        ResetColor
    )?;
    Ok(())
}

fn draw_box(x: u16, y: u16, w: u16, h: u16) -> std::io::Result<()> {
    let mut out = stdout();
    let border = Color::Rgb {
        r: 0x3d,
        g: 0x3d,
        b: 0x5c,
    };
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
