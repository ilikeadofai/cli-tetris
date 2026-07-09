# cli-tetris

Terminal Tetris in Rust, inspired by **TETR.IO** guideline rules — with a settings menu, dynamic sizing, and color themes.

## Features

- **Centered spawn** — pieces enter from the middle of the field
- **7-bag**, **SRS**, hold, next queue, ghost, lock delay
- Combo, B2B, T-spin detection, line-clear flash
- **Settings menu** (TETR.IO-style tabs): Handling · Video · Colors
- **Dynamic sizing** — Auto / 1× / 2× / 3× board scale, optional centering
- **Color themes** — Terminal (ANSI/TTY), Guideline, Catppuccin, Dracula, Nord, Gruvbox, Monochrome
- High score + config saved under your home directory

## Run

```bash
git clone https://github.com/ilikeadofai/cli-tetris.git
cd cli-tetris
cargo run --release
```

Needs a truecolor-capable terminal for most themes; **Terminal (ANSI)** follows your TTY palette.

## Controls

| Key | Action |
|-----|--------|
| `↑↓` / Enter | Title menu |
| `Space` | Quick start / hard drop |
| `S` | Settings (title or pause) |
| `←→` | Move / change setting |
| `↓` | Soft drop |
| `Z` / `X`/`↑` | Rotate CCW / CW |
| `A` | 180° |
| `C` | Hold |
| `P` | Pause |
| `R` | Restart / reset settings tab |
| `[` `]` | Settings tabs |
| `Esc` | Back / pause |
| `Q` | Quit (title) or return to title |

### Settings

| Tab | Options |
|-----|---------|
| **Handling** | DAS, ARR, SDF (soft-drop ms/cell) |
| **Video** | Scale (Auto/1x/2x/3x), ghost, next count, grid, center board |
| **Colors** | Theme presets + live swatch preview |

Config file: `~/.cli-tetris-config`  
High score: `~/.cli-tetris-hiscore`

## Project layout

```
src/
  main.rs      — screens + input
  game.rs      — gameplay
  settings.rs  — load/save settings
  theme.rs     — color palettes
  layout.rs    — dynamic board layout
  menu.rs      — title + settings navigation
  piece.rs / board.rs / bag.rs / render.rs / hiscore.rs
```
