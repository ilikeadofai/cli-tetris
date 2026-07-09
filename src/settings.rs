//! Persistent settings (TETR.IO-inspired handling, gameplay, video, colors).

use crate::theme::ThemeId;
use std::fs;
use std::path::PathBuf;

// ── enums ───────────────────────────────────────────────────────────

/// Horizontal mino width only (always 1 terminal row tall).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    Auto,
    X1,
    X2,
    X3,
    X4,
}

impl ScaleMode {
    pub fn label(self) -> &'static str {
        match self {
            ScaleMode::Auto => "Auto",
            ScaleMode::X1 => "Compact (1)",
            ScaleMode::X2 => "Normal (2)",
            ScaleMode::X3 => "Large (3)",
            ScaleMode::X4 => "Huge (4)",
        }
    }
    pub fn next(self) -> Self {
        match self {
            ScaleMode::Auto => ScaleMode::X1,
            ScaleMode::X1 => ScaleMode::X2,
            ScaleMode::X2 => ScaleMode::X3,
            ScaleMode::X3 => ScaleMode::X4,
            ScaleMode::X4 => ScaleMode::Auto,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            ScaleMode::Auto => ScaleMode::X4,
            ScaleMode::X1 => ScaleMode::Auto,
            ScaleMode::X2 => ScaleMode::X1,
            ScaleMode::X3 => ScaleMode::X2,
            ScaleMode::X4 => ScaleMode::X3,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            ScaleMode::Auto => "auto",
            ScaleMode::X1 => "1",
            ScaleMode::X2 => "2",
            ScaleMode::X3 => "3",
            ScaleMode::X4 => "4",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(ScaleMode::Auto),
            "1" | "1x" | "compact" => Some(ScaleMode::X1),
            "2" | "2x" | "normal" => Some(ScaleMode::X2),
            "3" | "3x" | "large" => Some(ScaleMode::X3),
            "4" | "4x" | "huge" => Some(ScaleMode::X4),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridStyle {
    Checker,
    Flat,
    Off,
}

impl GridStyle {
    pub fn label(self) -> &'static str {
        match self {
            GridStyle::Checker => "Checker",
            GridStyle::Flat => "Flat",
            GridStyle::Off => "Off",
        }
    }
    pub fn next(self) -> Self {
        match self {
            GridStyle::Checker => GridStyle::Flat,
            GridStyle::Flat => GridStyle::Off,
            GridStyle::Off => GridStyle::Checker,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            GridStyle::Checker => GridStyle::Off,
            GridStyle::Flat => GridStyle::Checker,
            GridStyle::Off => GridStyle::Flat,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            GridStyle::Checker => "checker",
            GridStyle::Flat => "flat",
            GridStyle::Off => "off",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "checker" => Some(GridStyle::Checker),
            "flat" => Some(GridStyle::Flat),
            "off" => Some(GridStyle::Off),
            _ => None,
        }
    }
}

/// Gravity curve (TETR.IO-ish speed profiles).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GravityCurve {
    /// Modern guideline-ish table (default).
    Modern,
    /// Slower early, steeper late.
    Classic,
    /// Fixed delay regardless of level.
    Static,
}

impl GravityCurve {
    pub fn label(self) -> &'static str {
        match self {
            GravityCurve::Modern => "Modern",
            GravityCurve::Classic => "Classic",
            GravityCurve::Static => "Static",
        }
    }
    pub fn next(self) -> Self {
        match self {
            GravityCurve::Modern => GravityCurve::Classic,
            GravityCurve::Classic => GravityCurve::Static,
            GravityCurve::Static => GravityCurve::Modern,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            GravityCurve::Modern => GravityCurve::Static,
            GravityCurve::Classic => GravityCurve::Modern,
            GravityCurve::Static => GravityCurve::Classic,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            GravityCurve::Modern => "modern",
            GravityCurve::Classic => "classic",
            GravityCurve::Static => "static",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "modern" => Some(GravityCurve::Modern),
            "classic" => Some(GravityCurve::Classic),
            "static" | "fixed" => Some(GravityCurve::Static),
            _ => None,
        }
    }
}

/// Quick handling profiles (TETR.IO community-style).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandlingPreset {
    Custom,
    Default,
    Fast,
    Instant,
    Slow,
}

impl HandlingPreset {
    pub fn label(self) -> &'static str {
        match self {
            HandlingPreset::Custom => "Custom",
            HandlingPreset::Default => "Default",
            HandlingPreset::Fast => "Fast",
            HandlingPreset::Instant => "Instant ARR",
            HandlingPreset::Slow => "Slow",
        }
    }
    pub fn next(self) -> Self {
        match self {
            HandlingPreset::Custom => HandlingPreset::Default,
            HandlingPreset::Default => HandlingPreset::Fast,
            HandlingPreset::Fast => HandlingPreset::Instant,
            HandlingPreset::Instant => HandlingPreset::Slow,
            HandlingPreset::Slow => HandlingPreset::Custom,
        }
    }
    pub fn prev(self) -> Self {
        match self {
            HandlingPreset::Custom => HandlingPreset::Slow,
            HandlingPreset::Default => HandlingPreset::Custom,
            HandlingPreset::Fast => HandlingPreset::Default,
            HandlingPreset::Instant => HandlingPreset::Fast,
            HandlingPreset::Slow => HandlingPreset::Instant,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            HandlingPreset::Custom => "custom",
            HandlingPreset::Default => "default",
            HandlingPreset::Fast => "fast",
            HandlingPreset::Instant => "instant",
            HandlingPreset::Slow => "slow",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "custom" => Some(HandlingPreset::Custom),
            "default" => Some(HandlingPreset::Default),
            "fast" => Some(HandlingPreset::Fast),
            "instant" => Some(HandlingPreset::Instant),
            "slow" => Some(HandlingPreset::Slow),
            _ => None,
        }
    }

    /// Apply preset numbers onto settings (except Custom).
    pub fn apply(self, s: &mut Settings) {
        match self {
            HandlingPreset::Custom => {}
            HandlingPreset::Default => {
                s.das_ms = 120;
                s.arr_ms = 20;
                s.sdf_ms = 25;
                s.sdf_infinite = false;
                s.soft_drop_points = true;
            }
            HandlingPreset::Fast => {
                s.das_ms = 83;
                s.arr_ms = 0;
                s.sdf_ms = 10;
                s.sdf_infinite = false;
                s.soft_drop_points = true;
            }
            HandlingPreset::Instant => {
                s.das_ms = 100;
                s.arr_ms = 0;
                s.sdf_ms = 0;
                s.sdf_infinite = true;
                s.soft_drop_points = true;
            }
            HandlingPreset::Slow => {
                s.das_ms = 200;
                s.arr_ms = 50;
                s.sdf_ms = 40;
                s.sdf_infinite = false;
                s.soft_drop_points = true;
            }
        }
        s.handling_preset = self;
    }
}

// ── Settings ────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    // Handling
    pub handling_preset: HandlingPreset,
    pub das_ms: u64,
    pub arr_ms: u64,
    /// Soft-drop interval ms; ignored when `sdf_infinite`.
    pub sdf_ms: u64,
    /// INF soft drop (drop as fast as the tick allows).
    pub sdf_infinite: bool,
    pub soft_drop_points: bool,

    // Gameplay
    pub start_level: u32,
    pub lines_per_level: u32,
    pub lock_delay_ms: u64,
    pub lock_resets_max: u32,
    pub gravity: GravityCurve,
    /// Static gravity cell delay when curve is Static.
    pub static_gravity_ms: u64,
    pub hold_enabled: bool,
    pub allow_180: bool,
    pub next_count: usize,
    pub line_clear_ms: u64,

    // Video
    pub scale: ScaleMode,
    pub ghost: bool,
    pub grid: GridStyle,
    pub center: bool,
    pub show_action_text: bool,
    pub show_stats: bool,
    pub show_pps: bool,
    pub show_time: bool,
    pub mino_bevel: bool,
    pub show_footer: bool,

    // Colors
    pub theme: ThemeId,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            handling_preset: HandlingPreset::Default,
            das_ms: 120,
            arr_ms: 20,
            sdf_ms: 25,
            sdf_infinite: false,
            soft_drop_points: true,

            start_level: 1,
            lines_per_level: 10,
            lock_delay_ms: 500,
            lock_resets_max: 15,
            gravity: GravityCurve::Modern,
            static_gravity_ms: 500,
            hold_enabled: true,
            allow_180: true,
            next_count: 5,
            line_clear_ms: 150,

            scale: ScaleMode::Auto,
            ghost: true,
            grid: GridStyle::Checker,
            center: true,
            show_action_text: true,
            show_stats: true,
            show_pps: true,
            show_time: true,
            mino_bevel: true,
            show_footer: true,

            theme: ThemeId::Guideline,
        }
    }
}

impl Settings {
    pub fn clamp(&mut self) {
        self.das_ms = self.das_ms.clamp(1, 500);
        self.arr_ms = self.arr_ms.clamp(0, 200);
        self.sdf_ms = self.sdf_ms.clamp(0, 200);
        self.start_level = self.start_level.clamp(1, 20);
        self.lines_per_level = self.lines_per_level.clamp(1, 30);
        self.lock_delay_ms = self.lock_delay_ms.clamp(50, 2000);
        self.lock_resets_max = self.lock_resets_max.clamp(0, 40);
        self.static_gravity_ms = self.static_gravity_ms.clamp(10, 2000);
        self.next_count = self.next_count.clamp(0, 5);
        self.line_clear_ms = self.line_clear_ms.clamp(0, 500);
    }

    /// Mark handling as custom after manual tweak.
    pub fn mark_handling_custom(&mut self) {
        self.handling_preset = HandlingPreset::Custom;
    }

    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".cli-tetris-config"))
    }

    fn parse_bool(v: &str) -> bool {
        v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("on")
    }

    pub fn load() -> Self {
        let mut s = Self::default();
        let Some(p) = Self::path() else {
            return s;
        };
        let Ok(text) = fs::read_to_string(p) else {
            return s;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let k = k.trim();
            let v = v.trim();
            match k {
                "preset" | "handling_preset" => {
                    if let Some(p) = HandlingPreset::from_str(v) {
                        s.handling_preset = p;
                    }
                }
                "das" => {
                    if let Ok(n) = v.parse() {
                        s.das_ms = n;
                    }
                }
                "arr" => {
                    if let Ok(n) = v.parse() {
                        s.arr_ms = n;
                    }
                }
                "sdf" => {
                    if let Ok(n) = v.parse() {
                        s.sdf_ms = n;
                    }
                }
                "sdf_inf" | "sdf_infinite" => s.sdf_infinite = Self::parse_bool(v),
                "soft_drop_points" => s.soft_drop_points = Self::parse_bool(v),
                "start_level" => {
                    if let Ok(n) = v.parse() {
                        s.start_level = n;
                    }
                }
                "lines_per_level" => {
                    if let Ok(n) = v.parse() {
                        s.lines_per_level = n;
                    }
                }
                "lock_delay" => {
                    if let Ok(n) = v.parse() {
                        s.lock_delay_ms = n;
                    }
                }
                "lock_resets" => {
                    if let Ok(n) = v.parse() {
                        s.lock_resets_max = n;
                    }
                }
                "gravity" => {
                    if let Some(g) = GravityCurve::from_str(v) {
                        s.gravity = g;
                    }
                }
                "static_gravity" => {
                    if let Ok(n) = v.parse() {
                        s.static_gravity_ms = n;
                    }
                }
                "hold" => s.hold_enabled = Self::parse_bool(v),
                "allow_180" => s.allow_180 = Self::parse_bool(v),
                "next" => {
                    if let Ok(n) = v.parse() {
                        s.next_count = n;
                    }
                }
                "line_clear_ms" => {
                    if let Ok(n) = v.parse() {
                        s.line_clear_ms = n;
                    }
                }
                "scale" => {
                    if let Some(m) = ScaleMode::from_str(v) {
                        s.scale = m;
                    }
                }
                "ghost" => s.ghost = Self::parse_bool(v),
                "grid" => {
                    if let Some(g) = GridStyle::from_str(v) {
                        s.grid = g;
                    }
                }
                "center" => s.center = Self::parse_bool(v),
                "action_text" => s.show_action_text = Self::parse_bool(v),
                "show_stats" => s.show_stats = Self::parse_bool(v),
                "show_pps" => s.show_pps = Self::parse_bool(v),
                "show_time" => s.show_time = Self::parse_bool(v),
                "mino_bevel" => s.mino_bevel = Self::parse_bool(v),
                "show_footer" => s.show_footer = Self::parse_bool(v),
                "theme" => {
                    if let Some(t) = ThemeId::from_str(v) {
                        s.theme = t;
                    }
                }
                _ => {}
            }
        }
        // If a named preset was saved, re-apply its numbers so preset stays consistent
        if s.handling_preset != HandlingPreset::Custom {
            let p = s.handling_preset;
            p.apply(&mut s);
        }
        s.clamp();
        s
    }

    pub fn save(&self) {
        let Some(p) = Self::path() else {
            return;
        };
        let b = |v: bool| if v { 1 } else { 0 };
        let text = format!(
            "# cli-tetris config\n\
# Handling\n\
handling_preset={}\n\
das={}\n\
arr={}\n\
sdf={}\n\
sdf_infinite={}\n\
soft_drop_points={}\n\
# Gameplay\n\
start_level={}\n\
lines_per_level={}\n\
lock_delay={}\n\
lock_resets={}\n\
gravity={}\n\
static_gravity={}\n\
hold={}\n\
allow_180={}\n\
next={}\n\
line_clear_ms={}\n\
# Video\n\
scale={}\n\
ghost={}\n\
grid={}\n\
center={}\n\
action_text={}\n\
show_stats={}\n\
show_pps={}\n\
show_time={}\n\
mino_bevel={}\n\
show_footer={}\n\
# Colors\n\
theme={}\n",
            self.handling_preset.as_str(),
            self.das_ms,
            self.arr_ms,
            self.sdf_ms,
            b(self.sdf_infinite),
            b(self.soft_drop_points),
            self.start_level,
            self.lines_per_level,
            self.lock_delay_ms,
            self.lock_resets_max,
            self.gravity.as_str(),
            self.static_gravity_ms,
            b(self.hold_enabled),
            b(self.allow_180),
            self.next_count,
            self.line_clear_ms,
            self.scale.as_str(),
            b(self.ghost),
            self.grid.as_str(),
            b(self.center),
            b(self.show_action_text),
            b(self.show_stats),
            b(self.show_pps),
            b(self.show_time),
            b(self.mino_bevel),
            b(self.show_footer),
            self.theme.as_str(),
        );
        let _ = fs::write(p, text);
    }
}

pub fn on_off(v: bool) -> &'static str {
    if v {
        "On"
    } else {
        "Off"
    }
}
