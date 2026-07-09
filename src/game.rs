//! Game state: gravity, lock delay, hold, scoring, T-spin / B2B / combo (TETR.IO-inspired).

use crate::bag::SevenBag;
use crate::board::{Board, Cell};
use crate::hiscore;
use crate::piece::{wall_kicks, Piece, PieceKind};

fn gravity_ms(level: u32) -> u64 {
    let table: [u64; 15] = [
        800, 720, 630, 550, 470, 380, 300, 220, 140, 100, 80, 60, 40, 30, 20,
    ];
    let idx = (level.saturating_sub(1) as usize).min(table.len() - 1);
    table[idx]
}

const LOCK_DELAY_MS: u64 = 500;
const LOCK_RESET_MAX: u32 = 15;
const DAS_MS: u64 = 120;
const ARR_MS: u64 = 20;
const SOFT_DROP_MS: u64 = 25;
const KEY_HOLD_TIMEOUT_MS: u64 = 70;
const NEXT_COUNT: usize = 5;
const LINE_CLEAR_FLASH_MS: u64 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClearType {
    None,
    Single,
    Double,
    Triple,
    Tetris,
    TSpinMini,
    TSpinMiniSingle,
    TSpin,
    TSpinDouble,
    TSpinTriple,
}

impl ClearType {
    pub fn label(self) -> &'static str {
        match self {
            ClearType::None => "",
            ClearType::Single => "SINGLE",
            ClearType::Double => "DOUBLE",
            ClearType::Triple => "TRIPLE",
            ClearType::Tetris => "TETRIS",
            ClearType::TSpinMini => "T-SPIN MINI",
            ClearType::TSpinMiniSingle => "T-SPIN MINI SINGLE",
            ClearType::TSpin => "T-SPIN SINGLE",
            ClearType::TSpinDouble => "T-SPIN DOUBLE",
            ClearType::TSpinTriple => "T-SPIN TRIPLE",
        }
    }

    pub fn is_difficult(self) -> bool {
        matches!(
            self,
            ClearType::Tetris
                | ClearType::TSpin
                | ClearType::TSpinDouble
                | ClearType::TSpinTriple
                | ClearType::TSpinMiniSingle
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Ready,
    Playing,
    Clearing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TSpinKind {
    None,
    Mini,
    Full,
}

#[derive(Debug)]
pub struct Game {
    pub board: Board,
    pub current: Piece,
    pub hold: Option<PieceKind>,
    pub hold_used: bool,
    bag: SevenBag,
    pub score: u64,
    pub high_score: u64,
    pub lines: u32,
    pub level: u32,
    pub combo: i32,
    pub b2b: i32,
    pub last_clear: ClearType,
    pub clear_flash_ms: u64,
    pub phase: GamePhase,
    pub pieces_placed: u32,
    pub elapsed_ms: u64,
    /// Rows currently flashing before they are removed.
    pub flashing_rows: Vec<i32>,
    flash_timer: u64,
    pending_tspin: TSpinKind,

    gravity_accum: u64,
    lock_timer: Option<u64>,
    lock_resets: u32,
    soft_dropping: bool,

    left_held: bool,
    right_held: bool,
    left_age: u64,
    right_age: u64,
    soft_age: u64,
    das_dir: i32,
    das_timer: u64,
    arr_timer: u64,
    soft_timer: u64,

    last_was_rotation: bool,
    last_kick_index: usize,
    last_tspin: TSpinKind,
}

impl Game {
    pub fn new() -> Self {
        let mut bag = SevenBag::new();
        let kind = bag.next();
        Self {
            board: Board::new(),
            current: Piece::new(kind),
            hold: None,
            hold_used: false,
            bag,
            score: 0,
            high_score: hiscore::load(),
            lines: 0,
            level: 1,
            combo: -1,
            b2b: -1,
            last_clear: ClearType::None,
            clear_flash_ms: 0,
            phase: GamePhase::Ready,
            pieces_placed: 0,
            elapsed_ms: 0,
            flashing_rows: Vec::new(),
            flash_timer: 0,
            pending_tspin: TSpinKind::None,
            gravity_accum: 0,
            lock_timer: None,
            lock_resets: 0,
            soft_dropping: false,
            left_held: false,
            right_held: false,
            left_age: 0,
            right_age: 0,
            soft_age: 0,
            das_dir: 0,
            das_timer: 0,
            arr_timer: 0,
            soft_timer: 0,
            last_was_rotation: false,
            last_kick_index: 0,
            last_tspin: TSpinKind::None,
        }
    }

    pub fn next_queue(&self) -> Vec<PieceKind> {
        self.bag.peek(NEXT_COUNT)
    }

    pub fn ghost_y(&self) -> i32 {
        self.board.ghost_y(&self.current)
    }

    pub fn pps(&self) -> f64 {
        if self.elapsed_ms == 0 {
            return 0.0;
        }
        self.pieces_placed as f64 / (self.elapsed_ms as f64 / 1000.0)
    }

    pub fn time_label(&self) -> String {
        let total_secs = self.elapsed_ms / 1000;
        let m = total_secs / 60;
        let s = total_secs % 60;
        format!("{m:02}:{s:02}")
    }

    pub fn start(&mut self) {
        if self.phase == GamePhase::Ready {
            self.phase = GamePhase::Playing;
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.phase {
            GamePhase::Playing => self.phase = GamePhase::Paused,
            GamePhase::Paused => self.phase = GamePhase::Playing,
            _ => {}
        }
    }

    pub fn restart(&mut self) {
        let hs = self.high_score.max(hiscore::load());
        *self = Self::new();
        self.high_score = hs;
    }

    pub fn press_left(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let first = !self.left_held;
        self.left_held = true;
        self.left_age = 0;
        if first {
            self.try_move(-1, 0);
            self.das_dir = -1;
            self.das_timer = 0;
            self.arr_timer = 0;
        }
    }

    pub fn press_right(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let first = !self.right_held;
        self.right_held = true;
        self.right_age = 0;
        if first {
            self.try_move(1, 0);
            self.das_dir = 1;
            self.das_timer = 0;
            self.arr_timer = 0;
        }
    }

    pub fn press_soft(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let first = !self.soft_dropping;
        self.soft_dropping = true;
        self.soft_age = 0;
        if first {
            self.soft_timer = 0;
            let _ = self.try_move(0, 1);
        }
    }

    pub fn hard_drop(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let start_y = self.current.y;
        while self.try_move(0, 1) {}
        let dropped = (self.current.y - start_y) as u64;
        self.score += dropped * 2;
        self.lock_piece();
    }

    pub fn rotate_cw(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        self.try_rotate(true);
    }

    pub fn rotate_ccw(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        self.try_rotate(false);
    }

    pub fn rotate_180(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        if self.try_rotate(true) {
            let _ = self.try_rotate(true);
        }
    }

    pub fn hold(&mut self) {
        if self.phase != GamePhase::Playing || self.hold_used {
            return;
        }
        self.hold_used = true;
        self.last_was_rotation = false;
        self.lock_timer = None;
        self.lock_resets = 0;
        self.gravity_accum = 0;

        let current_kind = self.current.kind;
        match self.hold {
            None => {
                self.hold = Some(current_kind);
                self.spawn_next();
            }
            Some(held) => {
                self.hold = Some(current_kind);
                self.current = Piece::new(held);
                if self.board.is_block_out(&self.current) {
                    self.top_out();
                }
            }
        }
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        let mut next = self.current;
        next.x += dx;
        next.y += dy;
        if self.board.fits(&next) {
            self.current = next;
            if dx != 0 || dy != 0 {
                self.last_was_rotation = false;
            }
            if dy > 0 && self.soft_dropping {
                self.score += 1;
            }
            self.on_piece_moved();
            return true;
        }
        false
    }

    fn try_rotate(&mut self, cw: bool) -> bool {
        let from = self.current.rot;
        let to = if cw {
            (from + 1) % 4
        } else {
            (from + 3) % 4
        };
        let kicks = wall_kicks(self.current.kind, from, to);

        for (i, &(dx, dy)) in kicks.iter().enumerate() {
            let mut next = self.current;
            next.rot = to;
            next.x += dx;
            next.y += dy;
            if self.board.fits(&next) {
                self.current = next;
                self.last_was_rotation = true;
                self.last_kick_index = i;
                self.on_piece_moved();
                return true;
            }
        }
        false
    }

    fn on_piece_moved(&mut self) {
        if self.is_on_ground() {
            if self.lock_resets < LOCK_RESET_MAX {
                self.lock_timer = Some(LOCK_DELAY_MS);
                self.lock_resets += 1;
            }
        } else {
            self.lock_timer = None;
        }
    }

    fn is_on_ground(&self) -> bool {
        let mut below = self.current;
        below.y += 1;
        !self.board.fits(&below)
    }

    fn detect_tspin(&self) -> TSpinKind {
        if self.current.kind != PieceKind::T || !self.last_was_rotation {
            return TSpinKind::None;
        }
        let cx = self.current.x;
        let cy = self.current.y;
        let corners = [
            (cx - 1, cy - 1),
            (cx + 1, cy - 1),
            (cx - 1, cy + 1),
            (cx + 1, cy + 1),
        ];
        let filled = corners
            .iter()
            .filter(|&&(x, y)| self.board.get(x, y) != Cell::Empty)
            .count();
        if filled < 3 {
            return TSpinKind::None;
        }
        let (fa, fb) = match self.current.rot {
            0 => ((cx - 1, cy - 1), (cx + 1, cy - 1)),
            1 => ((cx + 1, cy - 1), (cx + 1, cy + 1)),
            2 => ((cx - 1, cy + 1), (cx + 1, cy + 1)),
            _ => ((cx - 1, cy - 1), (cx - 1, cy + 1)),
        };
        let front = self.board.get(fa.0, fa.1) != Cell::Empty
            && self.board.get(fb.0, fb.1) != Cell::Empty;
        if front || self.last_kick_index >= 4 {
            TSpinKind::Full
        } else {
            TSpinKind::Mini
        }
    }

    fn lock_piece(&mut self) {
        self.last_tspin = self.detect_tspin();
        self.board.lock(&self.current);
        self.pieces_placed += 1;

        let rows = self.board.full_rows();
        if rows.is_empty() {
            // Spin with no lines still scores
            let clear = self.classify_clear(0);
            self.apply_score(clear, 0);
            self.last_clear = clear;
            if clear != ClearType::None {
                self.clear_flash_ms = 600;
            }
            self.finish_lock_cycle();
        } else {
            self.pending_tspin = self.last_tspin;
            self.flashing_rows = rows;
            self.flash_timer = LINE_CLEAR_FLASH_MS;
            self.phase = GamePhase::Clearing;
            self.lock_timer = None;
            self.lock_resets = 0;
            self.gravity_accum = 0;
            self.last_was_rotation = false;
        }
    }

    fn finish_clear_animation(&mut self) {
        let n = self.flashing_rows.len() as u32;
        self.flashing_rows.clear();
        self.last_tspin = self.pending_tspin;
        let cleared = self.board.clear_lines();
        debug_assert_eq!(cleared, n);
        let clear = self.classify_clear(cleared);
        self.apply_score(clear, cleared);
        self.last_clear = clear;
        self.clear_flash_ms = 800;
        self.phase = GamePhase::Playing;
        self.finish_lock_cycle();
    }

    fn finish_lock_cycle(&mut self) {
        self.hold_used = false;
        self.lock_timer = None;
        self.lock_resets = 0;
        self.gravity_accum = 0;
        self.last_was_rotation = false;
        self.spawn_next();
    }

    fn classify_clear(&self, lines: u32) -> ClearType {
        match (self.last_tspin, lines) {
            (TSpinKind::None, 0) => ClearType::None,
            (TSpinKind::None, 1) => ClearType::Single,
            (TSpinKind::None, 2) => ClearType::Double,
            (TSpinKind::None, 3) => ClearType::Triple,
            (TSpinKind::None, 4..) => ClearType::Tetris,
            (TSpinKind::Mini, 0) => ClearType::TSpinMini,
            (TSpinKind::Mini, 1..) => ClearType::TSpinMiniSingle,
            (TSpinKind::Full, 0) => ClearType::TSpin,
            (TSpinKind::Full, 1) => ClearType::TSpin,
            (TSpinKind::Full, 2) => ClearType::TSpinDouble,
            (TSpinKind::Full, 3..) => ClearType::TSpinTriple,
        }
    }

    fn apply_score(&mut self, clear: ClearType, lines: u32) {
        if lines == 0 {
            if matches!(clear, ClearType::TSpin | ClearType::TSpinMini) {
                let base: u64 = match clear {
                    ClearType::TSpinMini => 100,
                    ClearType::TSpin => 400,
                    _ => 0,
                };
                self.score += base * self.level as u64;
            }
            self.combo = -1;
            self.touch_high_score();
            return;
        }

        self.combo += 1;

        let mut b2b_bonus = false;
        if clear.is_difficult() {
            if self.b2b >= 0 {
                b2b_bonus = true;
            }
            self.b2b += 1;
        } else {
            self.b2b = -1;
        }

        let base: u64 = match clear {
            ClearType::None => 0,
            ClearType::Single => 100,
            ClearType::Double => 300,
            ClearType::Triple => 500,
            ClearType::Tetris => 800,
            ClearType::TSpinMini => 100,
            ClearType::TSpinMiniSingle => 200,
            ClearType::TSpin => 800,
            ClearType::TSpinDouble => 1200,
            ClearType::TSpinTriple => 1600,
        };

        let mut pts = base * self.level as u64;
        if b2b_bonus {
            pts = pts * 3 / 2;
        }
        if self.combo > 0 {
            pts += 50 * self.combo as u64 * self.level as u64;
        }
        self.score += pts;

        self.lines += lines;
        self.level = 1 + self.lines / 10;
        self.touch_high_score();
    }

    fn touch_high_score(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
    }

    fn spawn_next(&mut self) {
        let kind = self.bag.next();
        self.current = Piece::new(kind);
        self.last_was_rotation = false;
        self.lock_timer = None;
        self.lock_resets = 0;
        if self.board.is_block_out(&self.current) {
            self.top_out();
        }
    }

    fn top_out(&mut self) {
        self.phase = GamePhase::GameOver;
        self.high_score = hiscore::update_if_better(self.score);
    }

    pub fn tick(&mut self, dt: u64) {
        if self.clear_flash_ms > 0 {
            self.clear_flash_ms = self.clear_flash_ms.saturating_sub(dt);
        }

        match self.phase {
            GamePhase::Ready | GamePhase::Paused | GamePhase::GameOver => return,
            GamePhase::Clearing => {
                self.flash_timer = self.flash_timer.saturating_sub(dt);
                if self.flash_timer == 0 {
                    self.finish_clear_animation();
                }
                return;
            }
            GamePhase::Playing => {}
        }

        self.elapsed_ms = self.elapsed_ms.saturating_add(dt);

        if self.left_held {
            self.left_age = self.left_age.saturating_add(dt);
            if self.left_age > KEY_HOLD_TIMEOUT_MS {
                self.left_held = false;
                if self.das_dir < 0 {
                    self.das_dir = if self.right_held { 1 } else { 0 };
                    self.das_timer = 0;
                    self.arr_timer = 0;
                }
            }
        }
        if self.right_held {
            self.right_age = self.right_age.saturating_add(dt);
            if self.right_age > KEY_HOLD_TIMEOUT_MS {
                self.right_held = false;
                if self.das_dir > 0 {
                    self.das_dir = if self.left_held { -1 } else { 0 };
                    self.das_timer = 0;
                    self.arr_timer = 0;
                }
            }
        }
        if self.soft_dropping {
            self.soft_age = self.soft_age.saturating_add(dt);
            if self.soft_age > KEY_HOLD_TIMEOUT_MS {
                self.soft_dropping = false;
            }
        }

        if self.das_dir != 0 && (self.left_held || self.right_held) {
            self.das_timer = self.das_timer.saturating_add(dt);
            if self.das_timer >= DAS_MS {
                self.arr_timer = self.arr_timer.saturating_add(dt);
                while self.arr_timer >= ARR_MS {
                    self.arr_timer -= ARR_MS;
                    if !self.try_move(self.das_dir, 0) {
                        break;
                    }
                }
            }
        }

        if self.soft_dropping {
            self.soft_timer = self.soft_timer.saturating_add(dt);
            while self.soft_timer >= SOFT_DROP_MS {
                self.soft_timer -= SOFT_DROP_MS;
                if !self.try_move(0, 1) {
                    break;
                }
            }
        } else {
            let g = gravity_ms(self.level);
            self.gravity_accum = self.gravity_accum.saturating_add(dt);
            while self.gravity_accum >= g {
                self.gravity_accum -= g;
                if !self.try_move(0, 1) {
                    break;
                }
            }
        }

        if self.phase != GamePhase::Playing {
            return;
        }

        if self.is_on_ground() {
            let timer = self.lock_timer.get_or_insert(LOCK_DELAY_MS);
            *timer = timer.saturating_sub(dt);
            if *timer == 0 {
                self.lock_piece();
            }
        } else {
            self.lock_timer = None;
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
