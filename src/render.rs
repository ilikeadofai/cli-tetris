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
const VISIBLE_START: i32 = 20;

const BOARD_X: u16 = 16;
const BOARD_Y: u16 = 2;

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

fn flash_block() -> String {
    let c = Color::Rgb {
        r: 0xff,
        g: 0xff,
        b: 0xff,
    };
    cell_block(c)
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

/// Full layout (static chrome). Call on first frame and after resize.
pub fn draw_chrome() -> std::io::Result<()> {
    let mut out = stdout();
    queue!(out, Clear(TermClear::All))?;

    let bw = (BOARD_W * CELL_W) as u16;
    let bh = VISIBLE_H as u16;

    queue!(
        out,
        MoveTo(BOARD_X, 0),
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

    queue!(
        out,
        MoveTo(2, BOARD_Y),
        SetForegroundColor(Color::White),
        Print("HOLD"),
        ResetColor
    )?;
    draw_box(2, BOARD_Y + 1, 10, 6)?;

    draw_box(BOARD_X - 1, BOARD_Y, bw + 2, bh + 2)?;

    let next_x = BOARD_X + bw + 3;
    queue!(
        out,
        MoveTo(next_x, BOARD_Y),
        SetForegroundColor(Color::White),
        Print("NEXT"),
        ResetColor
    )?;
    draw_box(next_x, BOARD_Y + 1, 10, 18)?;

    let foot_y = BOARD_Y + bh + 3;
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

/// Redraw dynamic regions only (board, hold, next, stats, overlays).
pub fn draw(game: &Game) -> std::io::Result<()> {
    let mut out = stdout();
    let bw = (BOARD_W * CELL_W) as u16;
    let bh = VISIBLE_H as u16;

    // HOLD box interior
    for row in 0..4u16 {
        queue!(out, MoveTo(3, BOARD_Y + 2 + row), Print("        "))?;
    }
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
            let py = BOARD_Y + 2 + (y - min_y + off_y) as u16;
            queue!(out, MoveTo(px, py), Print(cell_block(color)))?;
        }
    }

    // Board
    let show_active = matches!(game.phase, GamePhase::Playing | GamePhase::Paused);
    let ghost_y = game.ghost_y();
    let mut ghost = game.current;
    ghost.y = ghost_y;
    let active_cells = game.current.cells();
    let ghost_cells = ghost.cells();

    for row in 0..VISIBLE_H {
        let by = VISIBLE_START + row;
        let row_flash = game.flashing_rows.contains(&by);
        queue!(out, MoveTo(BOARD_X, BOARD_Y + 1 + row as u16))?;
        for col in 0..BOARD_W {
            let checker = (row + col) % 2 == 0;
            if row_flash {
                queue!(out, Print(flash_block()))?;
                continue;
            }
            let is_active =
                show_active && active_cells.iter().any(|&(x, y)| x == col && y == by);
            let is_ghost = show_active
                && game.phase == GamePhase::Playing
                && ghost_cells.iter().any(|&(x, y)| x == col && y == by);

            if is_active {
                queue!(out, Print(cell_block(game.current.kind.color())))?;
            } else if is_ghost {
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
    let next_x = BOARD_X + bw + 3;
    for (i, kind) in game.next_queue().into_iter().enumerate() {
        draw_mini(kind, next_x + 1, BOARD_Y + 2 + (i as u16) * 3)?;
    }

    // Stats (left column under hold)
    let stats_y = BOARD_Y + 9;
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
            MoveTo(2, stats_y + row),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("{:<5}", label)),
            ResetColor,
            SetForegroundColor(Color::White),
            Print(format!("{:>8}", value)),
            ResetColor
        )?;
    }

    // Clear combo labels area then rewrite
    queue!(
        out,
        MoveTo(2, stats_y + 13),
        Print("              "),
        MoveTo(2, stats_y + 14),
        Print("              ")
    )?;
    if game.combo > 0 {
        queue!(
            out,
            MoveTo(2, stats_y + 13),
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
            MoveTo(2, stats_y + 14),
            SetForegroundColor(Color::Rgb {
                r: 0xff,
                g: 0x66,
                b: 0xaa
            }),
            Print(format!("B2B x{}", game.b2b)),
            ResetColor
        )?;
    }

    // Clear-type banner
    if game.clear_flash_ms > 0 && game.last_clear != ClearType::None {
        let label = game.last_clear.label();
        let cx = BOARD_X + bw / 2 - (label.len() as u16) / 2;
        queue!(
            out,
            MoveTo(cx, BOARD_Y + bh / 2),
            SetForegroundColor(Color::Rgb {
                r: 0xff,
                g: 0xff,
                b: 0x88
            }),
            Print(label),
            ResetColor
        )?;
    }

    // Overlays
    match game.phase {
        GamePhase::Ready => {
            center_msg(bw, bh, 0, "cli-tetris", Color::Rgb {
                r: 0x00,
                g: 0xe5,
                b: 0xa8,
            })?;
            center_msg(bw, bh, 2, "Press SPACE to start", Color::White)?;
            center_msg(
                bw,
                bh,
                4,
                &format!("High score: {}", game.high_score),
                Color::DarkGrey,
            )?;
        }
        GamePhase::Paused => {
            center_msg(bw, bh, 0, "PAUSED", Color::Yellow)?;
            center_msg(bw, bh, 2, "P to resume", Color::DarkGrey)?;
        }
        GamePhase::GameOver => {
            center_msg(bw, bh, 0, "TOP OUT", Color::Red)?;
            center_msg(bw, bh, 2, &format!("Score {}", game.score), Color::White)?;
            let hs_msg = if game.score >= game.high_score && game.score > 0 {
                "NEW HIGH SCORE!".to_string()
            } else {
                format!("Best {}", game.high_score)
            };
            center_msg(bw, bh, 3, &hs_msg, Color::Rgb {
                r: 0xff,
                g: 0xcc,
                b: 0x00,
            })?;
            center_msg(bw, bh, 5, "R restart · Q quit", Color::DarkGrey)?;
        }
        GamePhase::Playing | GamePhase::Clearing => {}
    }

    out.flush()?;
    Ok(())
}

fn center_msg(bw: u16, bh: u16, row_off: u16, msg: &str, color: Color) -> std::io::Result<()> {
    let mut out = stdout();
    let cx = BOARD_X + bw / 2 - (msg.len() as u16) / 2;
    let cy = BOARD_Y + bh / 2 - 2 + row_off;
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
