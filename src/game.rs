//! Game state: gravity, lock delay, hold, scoring, T-spin / B2B / combo (TETR.IO-inspired).

use crate::bag::SevenBag;
use crate::board::{Board, Cell};
use crate::hiscore;
use crate::piece::{wall_kicks, wall_kicks_180, Piece, PieceKind};
use crate::settings::{GravityCurve, Settings};

fn gravity_ms(level: u32, curve: GravityCurve, static_ms: u64) -> u64 {
    match curve {
        GravityCurve::Static => static_ms,
        GravityCurve::Modern => {
            let table: [u64; 20] = [
                800, 720, 630, 550, 470, 380, 300, 220, 140, 100, 80, 60, 40, 30, 20, 16, 12, 10,
                8, 5,
            ];
            let idx = (level.saturating_sub(1) as usize).min(table.len() - 1);
            table[idx]
        }
        GravityCurve::Classic => {
            // NES-ish slower start
            let table: [u64; 20] = [
                1000, 900, 800, 700, 600, 500, 400, 300, 220, 160, 120, 90, 70, 50, 40, 30, 25, 20,
                16, 12,
            ];
            let idx = (level.saturating_sub(1) as usize).min(table.len() - 1);
            table[idx]
        }
    }
}

const KEY_HOLD_TIMEOUT_MS: u64 = 70;

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
    last_rotation_upgrades_tspin: bool,
    last_tspin: TSpinKind,

    // From settings (runtime)
    pub das_ms: u64,
    pub arr_ms: u64,
    pub sdf_ms: u64,
    pub sdf_infinite: bool,
    pub soft_drop_points: bool,
    pub next_count: usize,
    pub lock_delay_ms: u64,
    pub lock_resets_max: u32,
    pub gravity: GravityCurve,
    pub static_gravity_ms: u64,
    pub lines_per_level: u32,
    pub hold_enabled: bool,
    pub allow_180: bool,
    pub line_clear_ms: u64,
    /// Base level at game start (for display / scoring floor).
    pub start_level: u32,
}

impl Game {
    pub fn new() -> Self {
        Self::with_settings(&Settings::default())
    }

    pub fn with_settings(settings: &Settings) -> Self {
        let mut bag = SevenBag::new();
        let kind = bag.next();
        let mut g = Self {
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
            last_rotation_upgrades_tspin: false,
            last_tspin: TSpinKind::None,
            das_ms: settings.das_ms,
            arr_ms: settings.arr_ms,
            sdf_ms: settings.sdf_ms,
            sdf_infinite: settings.sdf_infinite,
            soft_drop_points: settings.soft_drop_points,
            next_count: settings.next_count,
            lock_delay_ms: settings.lock_delay_ms,
            lock_resets_max: settings.lock_resets_max,
            gravity: settings.gravity,
            static_gravity_ms: settings.static_gravity_ms,
            lines_per_level: settings.lines_per_level,
            hold_enabled: settings.hold_enabled,
            allow_180: settings.allow_180,
            line_clear_ms: settings.line_clear_ms,
            start_level: settings.start_level,
        };
        g.apply_settings(settings);
        g.level = settings.start_level.max(1);
        g
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        self.das_ms = settings.das_ms;
        self.arr_ms = settings.arr_ms;
        self.sdf_ms = settings.sdf_ms.max(1); // avoid div0 when not infinite
        self.sdf_infinite = settings.sdf_infinite;
        self.soft_drop_points = settings.soft_drop_points;
        self.next_count = settings.next_count.min(5);
        self.lock_delay_ms = settings.lock_delay_ms;
        self.lock_resets_max = settings.lock_resets_max;
        self.gravity = settings.gravity;
        self.static_gravity_ms = settings.static_gravity_ms;
        self.lines_per_level = settings.lines_per_level.max(1);
        self.hold_enabled = settings.hold_enabled;
        self.allow_180 = settings.allow_180;
        self.line_clear_ms = settings.line_clear_ms;
        self.start_level = settings.start_level.max(1);
        // Don't clobber mid-game level when tweaking settings from pause —
        // only raise floor if below start_level.
        if self.level < self.start_level {
            self.level = self.start_level;
        }
    }

    pub fn next_queue(&self) -> Vec<PieceKind> {
        self.bag.peek(self.next_count)
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

    pub fn restart(&mut self, settings: &Settings) {
        let hs = self.high_score.max(hiscore::load());
        *self = Self::with_settings(settings);
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
        if self.phase != GamePhase::Playing || !self.allow_180 {
            return;
        }
        let to = (self.current.rot + 2) % 4;
        self.try_rotate_to(to, wall_kicks_180(self.current.kind));
    }

    pub fn hold(&mut self) {
        if self.phase != GamePhase::Playing || !self.hold_enabled || self.hold_used {
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
            if dy > 0 && self.soft_dropping && self.soft_drop_points {
                self.score += 1;
            }
            self.on_piece_moved();
            return true;
        }
        false
    }

    fn try_rotate(&mut self, cw: bool) -> bool {
        let from = self.current.rot;
        let to = if cw { (from + 1) % 4 } else { (from + 3) % 4 };
        self.try_rotate_to(to, wall_kicks(self.current.kind, from, to))
    }

    fn try_rotate_to(&mut self, to: u8, kicks: &[(i32, i32)]) -> bool {
        let is_180 = (to + 4 - self.current.rot % 4) % 4 == 2;
        for (i, &(dx, dy)) in kicks.iter().enumerate() {
            let mut next = self.current;
            next.rot = to;
            next.x += dx;
            next.y += dy;
            if self.board.fits(&next) {
                self.current = next;
                self.last_was_rotation = true;
                self.last_rotation_upgrades_tspin = !is_180 && i == 4;
                self.on_piece_moved();
                return true;
            }
        }
        false
    }

    fn on_piece_moved(&mut self) {
        if self.is_on_ground() {
            if self.lock_resets < self.lock_resets_max {
                self.lock_timer = Some(self.lock_delay_ms);
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
        let front =
            self.board.get(fa.0, fa.1) != Cell::Empty && self.board.get(fb.0, fb.1) != Cell::Empty;
        if front || self.last_rotation_upgrades_tspin {
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
        } else if self.line_clear_ms == 0 {
            self.pending_tspin = self.last_tspin;
            self.flashing_rows = rows;
            self.finish_clear_animation();
        } else {
            self.pending_tspin = self.last_tspin;
            self.flashing_rows = rows;
            self.flash_timer = self.line_clear_ms;
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
        self.clear_flash_ms = if self.line_clear_ms == 0 { 0 } else { 800 };
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
        let lpl = self.lines_per_level.max(1);
        self.level = self.start_level + self.lines / lpl;
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
            if self.das_timer >= self.das_ms {
                if self.arr_ms == 0 {
                    // Instant ARR: slide until blocked
                    while self.try_move(self.das_dir, 0) {}
                } else {
                    self.arr_timer = self.arr_timer.saturating_add(dt);
                    while self.arr_timer >= self.arr_ms {
                        self.arr_timer -= self.arr_ms;
                        if !self.try_move(self.das_dir, 0) {
                            break;
                        }
                    }
                }
            }
        }

        if self.soft_dropping {
            if self.sdf_infinite {
                while self.try_move(0, 1) {}
            } else {
                let step = self.sdf_ms.max(1);
                self.soft_timer = self.soft_timer.saturating_add(dt);
                while self.soft_timer >= step {
                    self.soft_timer -= step;
                    if !self.try_move(0, 1) {
                        break;
                    }
                }
            }
        } else {
            let g = gravity_ms(self.level, self.gravity, self.static_gravity_ms);
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
            let timer = self.lock_timer.get_or_insert(self.lock_delay_ms);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_180_rotation_reaches_opposite_state() {
        let settings = Settings {
            allow_180: true,
            ..Settings::default()
        };
        let mut game = Game::with_settings(&settings);
        game.phase = GamePhase::Playing;
        game.current = Piece {
            kind: PieceKind::T,
            x: 4,
            y: 25,
            rot: 3,
        };

        game.rotate_180();

        assert_eq!(game.current.rot, 1);
        assert_eq!((game.current.x, game.current.y), (4, 25));
    }

    #[test]
    fn late_180_kicks_do_not_upgrade_tspin_mini() {
        let settings = Settings {
            allow_180: true,
            ..Settings::default()
        };
        let mut game = Game::with_settings(&settings);
        game.phase = GamePhase::Playing;
        game.current = Piece {
            kind: PieceKind::T,
            x: 4,
            y: 25,
            rot: 0,
        };

        for (x, y) in [(4, 26), (5, 24), (6, 25)] {
            game.board.set(x, y, Cell::Filled(PieceKind::Z));
        }
        game.rotate_180();
        assert_eq!(
            (game.current.x, game.current.y, game.current.rot),
            (3, 25, 2)
        );

        for (x, y) in [(2, 24), (4, 24)] {
            game.board.set(x, y, Cell::Filled(PieceKind::Z));
        }

        assert_eq!(game.detect_tspin(), TSpinKind::Mini);
    }

    #[test]
    fn failed_180_rotation_is_atomic() {
        let settings = Settings {
            allow_180: true,
            ..Settings::default()
        };
        let mut game = Game::with_settings(&settings);
        game.phase = GamePhase::Playing;
        game.current = Piece {
            kind: PieceKind::L,
            x: 4,
            y: 25,
            rot: 0,
        };

        for (x, y) in [
            (3, 26),
            (6, 25),
            (4, 27),
            (3, 24),
            (6, 23),
            (3, 27),
            (2, 26),
            (7, 25),
            (1, 26),
        ] {
            game.board.set(x, y, Cell::Filled(PieceKind::Z));
        }

        let before = game.current;
        game.lock_timer = Some(137);
        game.lock_resets = 3;
        game.last_was_rotation = true;
        game.last_rotation_upgrades_tspin = true;
        game.rotate_180();

        assert_eq!(game.current.kind, before.kind);
        assert_eq!(game.current.x, before.x);
        assert_eq!(game.current.y, before.y);
        assert_eq!(game.current.rot, before.rot);
        assert_eq!(game.lock_timer, Some(137));
        assert_eq!(game.lock_resets, 3);
        assert!(game.last_was_rotation);
        assert!(game.last_rotation_upgrades_tspin);
    }
}
