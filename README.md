# cli-tetris

Terminal Tetris in Rust, inspired by **TETR.IO** guideline rules.

## Features

- **Centered spawn** — pieces enter from the middle of the 10-wide field
- **7-bag** randomizer
- **SRS** rotation + wall kicks
- **Hold** piece (once per drop)
- **Next** queue (5 pieces)
- **Ghost** piece
- Soft drop / hard drop
- Lock delay with limited resets
- Combo + Back-to-Back (B2B)
- T-spin detection (3-corner rule)
- Line-clear flash before stack settles
- Score, high score (saved to `~/.cli-tetris-hiscore`), time, PPS
- Start screen + pause + top-out summary

## Run

```bash
git clone https://github.com/ilikeadofai/cli-tetris.git
cd cli-tetris
cargo run --release
```

Needs a terminal that supports truecolor (most modern terminals) and [Rust](https://rustup.rs).

## Controls

| Key | Action |
|-----|--------|
| `Space` / `Enter` | Start (on title) |
| `←` `→` | Move |
| `↓` | Soft drop |
| `Space` | Hard drop (in game) |
| `Z` | Rotate CCW |
| `X` / `↑` | Rotate CW |
| `A` | 180° rotate |
| `C` | Hold |
| `P` | Pause |
| `R` | Restart |
| `Q` / `Esc` | Quit |

## Project layout

```
src/
  main.rs     — input loop + terminal setup
  game.rs     — gravity, lock, score, hold, DAS/ARR
  piece.rs    — shapes, SRS kicks, centered spawn
  board.rs    — playfield + line clear
  bag.rs      — 7-bag RNG
  hiscore.rs  — persistent high score
  render.rs   — crossterm UI
```
