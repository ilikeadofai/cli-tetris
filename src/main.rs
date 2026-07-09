//! cli-tetris — terminal Tetris with TETR.IO-style guideline rules.
//!
//! Controls:
//!   ← →     move
//!   ↓       soft drop
//!   Space   hard drop
//!   Z       rotate CCW
//!   X / ↑   rotate CW
//!   A       180° rotate
//!   C       hold
//!   P       pause
//!   R       restart
//!   Q/Esc   quit

mod bag;
mod board;
mod game;
mod piece;
mod render;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use game::{Game, GamePhase};
use std::io::{stdout, Result};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    let mut stdout = stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    let result = run_game();

    let _ = stdout.execute(Show);
    let _ = stdout.execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();

    result
}

fn run_game() -> Result<()> {
    let mut game = Game::new();
    let mut last = Instant::now();
    let frame = Duration::from_millis(16);

    render::draw(&game)?;

    loop {
        while event::poll(Duration::from_millis(0))? {
            let ev = event::read()?;
            let Event::Key(key) = ev else {
                continue;
            };

            let is_press = key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat;
            if !is_press {
                continue;
            }

            // Discrete actions: ignore terminal key-repeat
            let discrete = matches!(
                key.code,
                KeyCode::Char('z')
                    | KeyCode::Char('Z')
                    | KeyCode::Char('x')
                    | KeyCode::Char('X')
                    | KeyCode::Char('a')
                    | KeyCode::Char('A')
                    | KeyCode::Char('c')
                    | KeyCode::Char('C')
                    | KeyCode::Char(' ')
                    | KeyCode::Char('p')
                    | KeyCode::Char('P')
                    | KeyCode::Char('r')
                    | KeyCode::Char('R')
                    | KeyCode::Char('q')
                    | KeyCode::Char('Q')
                    | KeyCode::Esc
                    | KeyCode::Up
            );
            if discrete && key.kind == KeyEventKind::Repeat {
                continue;
            }

            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
            {
                return Ok(());
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('p') | KeyCode::Char('P') => game.toggle_pause(),
                KeyCode::Char('r') | KeyCode::Char('R') => game.restart(),
                KeyCode::Left => game.press_left(),
                KeyCode::Right => game.press_right(),
                KeyCode::Down => game.press_soft(),
                KeyCode::Char(' ') => game.hard_drop(),
                KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('X') => game.rotate_cw(),
                KeyCode::Char('z') | KeyCode::Char('Z') => game.rotate_ccw(),
                KeyCode::Char('a') | KeyCode::Char('A') => game.rotate_180(),
                KeyCode::Char('c') | KeyCode::Char('C') => game.hold(),
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_millis() as u64;
        if dt >= 1 {
            last = now;
            if game.phase == GamePhase::Playing {
                game.tick(dt.min(100));
            }
        }

        render::draw(&game)?;
        std::thread::sleep(frame);
    }
}
