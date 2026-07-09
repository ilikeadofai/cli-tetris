# cli-tetris

Terminal Tetris in Rust, inspired by **TETR.IO** guideline rules.

## Features

- **7-bag** randomizer
- **SRS** rotation + wall kicks
- **Hold** piece (once per drop)
- **Next** queue (5 pieces)
- **Ghost** piece
- Soft drop / hard drop
- Lock delay with limited resets
- Combo + Back-to-Back (B2B)
- T-spin detection (3-corner rule)
- Guideline colors and scoring
- 40-row board (20 visible + buffer)

## Run

```bash
cd ~/Coding/cli-tetris
cargo run --release
```

Needs a terminal that supports truecolor (most modern terminals).

## Controls

| Key | Action |
|-----|--------|
| `←` `→` | Move |
| `↓` | Soft drop |
| `Space` | Hard drop |
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
  main.rs    — input loop + terminal setup
  game.rs    — gravity, lock, score, hold, DAS/ARR
  piece.rs   — shapes, SRS kicks
  board.rs   — playfield + line clear
  bag.rs     — 7-bag RNG
  render.rs  — crossterm UI
```
