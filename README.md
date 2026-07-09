# cli-tetris

Terminal Tetris in Rust, inspired by **TETR.IO** — guideline rules, settings menu, dynamic sizing, and color themes.

## Features

- Centered spawn, 7-bag, SRS, hold, next queue, ghost, lock delay
- Combo, B2B, T-spin detection, line-clear flash
- **Deep settings** (TETR.IO-style tabs): Handling · Gameplay · Video · Colors
- Handling presets (Default / Fast / Instant ARR / Slow)
- Dynamic scale (width-only so minos stay square)
- Themes: Terminal ANSI, Guideline, Catppuccin, Dracula, Nord, Gruvbox, Monochrome
- High score + config under `~`

## Run

```bash
git clone https://github.com/ilikeadofai/cli-tetris.git
cd cli-tetris
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| `↑↓` / Enter | Title menu |
| `Space` | Quick start / hard drop |
| `S` | Settings |
| `[` `]` | Settings tabs |
| `←→` | Move / change setting |
| `R` | Restart / reset settings tab |
| `P` | Pause |
| `Esc` | Back / pause |
| `Q` | Quit title / return to title |

## Settings

### Handling
| Option | Notes |
|--------|--------|
| **Preset** | Default, Fast, Instant ARR, Slow (or Custom) |
| **DAS** | Delayed auto-shift (ms) |
| **ARR** | Auto-repeat rate; `0` = instant slide |
| **SDF** | Soft-drop interval (ms) or **INF** |
| **SDF infinite** | Soft drop at max speed |
| **Soft drop points** | Score while soft-dropping |

### Gameplay
| Option | Notes |
|--------|--------|
| **Start level** | 1–20 |
| **Lines / level** | Level-up interval |
| **Lock delay** | Ground lock timer (ms) |
| **Lock reset max** | Move/rotate resets before force-lock |
| **Gravity curve** | Modern / Classic / Static |
| **Static gravity** | Cell delay when curve is Static |
| **Hold** | Enable hold piece |
| **180° rotate** | Enable `A` 180 spin |
| **Next queue** | 0–5 previews |
| **Line clear anim** | Flash duration (`0` = instant) |

### Video
Scale, ghost, grid, center board, action text, stats panel, PPS, time, mino bevel, footer help.

### Colors
Theme presets + live swatches. **Reset ALL** restores every setting.

Config: `~/.cli-tetris-config` · High score: `~/.cli-tetris-hiscore`

## Layout

```
src/
  main.rs game.rs settings.rs menu.rs theme.rs layout.rs
  piece.rs board.rs bag.rs render.rs hiscore.rs
```
