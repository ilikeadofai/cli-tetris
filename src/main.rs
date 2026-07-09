//! cli-tetris — terminal Tetris with TETR.IO-style guideline rules + settings.

mod bag;
mod board;
mod game;
mod hiscore;
mod layout;
mod menu;
mod piece;
mod render;
mod settings;
mod theme;

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use game::{Game, GamePhase};
use layout::Layout;
use menu::{AppScreen, MenuState, SettingsReturn, TitleItem};
use settings::Settings;
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
    let mut settings = Settings::load();
    let mut game = Game::with_settings(&settings);
    let mut menu = MenuState::default();
    let mut layout = Layout::compute(&settings);
    let mut last = Instant::now();
    let frame = Duration::from_millis(16);
    let mut need_clear = true;

    loop {
        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    layout = Layout::compute(&settings);
                    need_clear = true;
                }
                Event::Key(key) => {
                    let is_press =
                        key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat;
                    if !is_press {
                        continue;
                    }

                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
                    {
                        settings.save();
                        return Ok(());
                    }

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
                            | KeyCode::Char('s')
                            | KeyCode::Char('S')
                            | KeyCode::Char('q')
                            | KeyCode::Char('Q')
                            | KeyCode::Esc
                            | KeyCode::Enter
                            | KeyCode::Tab
                    );
                    // Allow repeat for left/right in settings (adjust values)
                    let allow_repeat_in_settings = menu.screen == AppScreen::Settings
                        && matches!(
                            key.code,
                            KeyCode::Left
                                | KeyCode::Right
                                | KeyCode::Char('+')
                                | KeyCode::Char('-')
                                | KeyCode::Char('=')
                        );
                    if discrete && key.kind == KeyEventKind::Repeat && !allow_repeat_in_settings {
                        continue;
                    }

                    let prev_screen = menu.screen;
                    match menu.screen {
                        AppScreen::Title => {
                            if handle_title_key(
                                key.code,
                                &mut menu,
                                &mut game,
                                &mut settings,
                            )? {
                                settings.save();
                                return Ok(());
                            }
                        }
                        AppScreen::Settings => {
                            let relayout = handle_settings_key(
                                key.code,
                                &mut menu,
                                &mut settings,
                                &mut game,
                                &mut layout,
                            );
                            if relayout {
                                need_clear = true;
                            }
                        }
                        AppScreen::Playing => {
                            if handle_play_key(
                                key.code,
                                &mut game,
                                &mut menu,
                                &mut settings,
                                &mut need_clear,
                            )? {
                                settings.save();
                                return Ok(());
                            }
                        }
                    }
                    if menu.screen != prev_screen {
                        need_clear = true;
                    }
                }
                _ => {}
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last).as_millis() as u64;
        if dt >= 1 {
            last = now;
            if menu.screen == AppScreen::Playing {
                game.tick(dt.min(100));
            }
        }

        if need_clear {
            render::clear_screen()?;
            need_clear = false;
        }
        render::draw(&game, &settings, &layout, &menu)?;
        std::thread::sleep(frame);
    }
}

fn handle_title_key(
    code: KeyCode,
    menu: &mut MenuState,
    game: &mut Game,
    settings: &mut Settings,
) -> Result<bool> {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(true),
        KeyCode::Up | KeyCode::Char('k') => menu.title_sel = menu.title_sel.prev(),
        KeyCode::Down | KeyCode::Char('j') => menu.title_sel = menu.title_sel.next(),
        KeyCode::Char(' ') => {
            game.apply_settings(settings);
            game.start();
            menu.screen = AppScreen::Playing;
        }
        KeyCode::Enter => match menu.title_sel {
            TitleItem::Start => {
                game.apply_settings(settings);
                game.start();
                menu.screen = AppScreen::Playing;
            }
            TitleItem::Settings => menu.open_settings(SettingsReturn::Title),
            TitleItem::Quit => return Ok(true),
        },
        KeyCode::Char('s') | KeyCode::Char('S') => {
            menu.open_settings(SettingsReturn::Title);
        }
        _ => {}
    }
    Ok(false)
}

fn handle_settings_key(
    code: KeyCode,
    menu: &mut MenuState,
    settings: &mut Settings,
    game: &mut Game,
    layout: &mut Layout,
) -> bool {
    let mut relayout = false;
    match code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            settings.clamp();
            settings.save();
            game.apply_settings(settings);
            *layout = Layout::compute(settings);
            match menu.settings_return {
                SettingsReturn::Title => menu.screen = AppScreen::Title,
                SettingsReturn::Pause => {
                    menu.screen = AppScreen::Playing;
                    // stay paused
                }
            }
            return true;
        }
        KeyCode::Tab => {
            menu.settings_tab = menu.settings_tab.next();
            menu.settings_row = 0;
        }
        KeyCode::BackTab => {
            menu.settings_tab = menu.settings_tab.prev();
            menu.settings_row = 0;
        }
        // Left/Right at tab level when holding... use Left/Right for values;
        // Shift+Left not easy — use [ ] for tabs
        KeyCode::Char('[') => {
            menu.settings_tab = menu.settings_tab.prev();
            menu.settings_row = 0;
        }
        KeyCode::Char(']') => {
            menu.settings_tab = menu.settings_tab.next();
            menu.settings_row = 0;
        }
        KeyCode::Up | KeyCode::Char('k') => menu.move_row(-1),
        KeyCode::Down | KeyCode::Char('j') => menu.move_row(1),
        KeyCode::Left | KeyCode::Char('-') => {
            menu.adjust(settings, -1);
            relayout = true;
        }
        KeyCode::Right | KeyCode::Char('+') | KeyCode::Char('=') => {
            menu.adjust(settings, 1);
            relayout = true;
        }
        KeyCode::Enter => {
            // Toggle / cycle current
            menu.adjust(settings, 1);
            relayout = true;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            menu.reset_current_tab(settings);
            relayout = true;
        }
        _ => {}
    }
    if relayout {
        *layout = Layout::compute(settings);
    }
    relayout
}

fn handle_play_key(
    code: KeyCode,
    game: &mut Game,
    menu: &mut MenuState,
    settings: &mut Settings,
    need_clear: &mut bool,
) -> Result<bool> {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            if game.phase == GamePhase::GameOver {
                return Ok(true);
            }
            // Q from play goes to title
            game.restart(settings);
            menu.screen = AppScreen::Title;
            menu.title_sel = TitleItem::Start;
            *need_clear = true;
        }
        KeyCode::Esc => {
            if game.phase == GamePhase::Playing {
                game.toggle_pause();
            } else if game.phase == GamePhase::Paused {
                game.toggle_pause();
            } else if game.phase == GamePhase::GameOver {
                game.restart(settings);
                menu.screen = AppScreen::Title;
                *need_clear = true;
            }
        }
        KeyCode::Char('p') | KeyCode::Char('P') => game.toggle_pause(),
        KeyCode::Char('s') | KeyCode::Char('S') => {
            if matches!(game.phase, GamePhase::Playing | GamePhase::Paused) {
                if game.phase == GamePhase::Playing {
                    game.toggle_pause();
                }
                menu.open_settings(SettingsReturn::Pause);
                *need_clear = true;
            }
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            game.restart(settings);
            game.start();
            *need_clear = true;
        }
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
    Ok(false)
}
