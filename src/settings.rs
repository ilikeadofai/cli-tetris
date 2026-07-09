//! Persistent settings (TETR.IO-inspired handling + video + colors).

use crate::theme::ThemeId;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleMode {
    Auto,
    X1,
    X2,
    X3,
}

impl ScaleMode {
    pub fn label(self) -> &'static str {
        match self {
            ScaleMode::Auto => "Auto",
            ScaleMode::X1 => "1x",
            ScaleMode::X2 => "2x",
            ScaleMode::X3 => "3x",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ScaleMode::Auto => ScaleMode::X1,
            ScaleMode::X1 => ScaleMode::X2,
            ScaleMode::X2 => ScaleMode::X3,
            ScaleMode::X3 => ScaleMode::Auto,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ScaleMode::Auto => ScaleMode::X3,
            ScaleMode::X1 => ScaleMode::Auto,
            ScaleMode::X2 => ScaleMode::X1,
            ScaleMode::X3 => ScaleMode::X2,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ScaleMode::Auto => "auto",
            ScaleMode::X1 => "1",
            ScaleMode::X2 => "2",
            ScaleMode::X3 => "3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Some(ScaleMode::Auto),
            "1" | "1x" => Some(ScaleMode::X1),
            "2" | "2x" => Some(ScaleMode::X2),
            "3" | "3x" => Some(ScaleMode::X3),
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

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    // Handling
    pub das_ms: u64,
    pub arr_ms: u64,
    pub sdf_ms: u64,
    // Video
    pub scale: ScaleMode,
    pub ghost: bool,
    pub next_count: usize,
    pub grid: GridStyle,
    pub center: bool,
    // Colors
    pub theme: ThemeId,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            das_ms: 120,
            arr_ms: 20,
            sdf_ms: 25,
            scale: ScaleMode::Auto,
            ghost: true,
            next_count: 5,
            grid: GridStyle::Checker,
            center: true,
            theme: ThemeId::Guideline,
        }
    }
}

impl Settings {
    pub fn clamp(&mut self) {
        self.das_ms = self.das_ms.clamp(50, 333);
        self.arr_ms = self.arr_ms.clamp(0, 100);
        self.sdf_ms = self.sdf_ms.clamp(5, 100);
        self.next_count = self.next_count.clamp(1, 5);
    }

    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".cli-tetris-config"))
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
                "scale" => {
                    if let Some(m) = ScaleMode::from_str(v) {
                        s.scale = m;
                    }
                }
                "ghost" => s.ghost = v == "1" || v.eq_ignore_ascii_case("true"),
                "next" => {
                    if let Ok(n) = v.parse() {
                        s.next_count = n;
                    }
                }
                "grid" => {
                    if let Some(g) = GridStyle::from_str(v) {
                        s.grid = g;
                    }
                }
                "center" => s.center = v == "1" || v.eq_ignore_ascii_case("true"),
                "theme" => {
                    if let Some(t) = ThemeId::from_str(v) {
                        s.theme = t;
                    }
                }
                _ => {}
            }
        }
        s.clamp();
        s
    }

    pub fn save(&self) {
        let Some(p) = Self::path() else {
            return;
        };
        let text = format!(
            "# cli-tetris config\n\
das={}\n\
arr={}\n\
sdf={}\n\
scale={}\n\
ghost={}\n\
next={}\n\
grid={}\n\
center={}\n\
theme={}\n",
            self.das_ms,
            self.arr_ms,
            self.sdf_ms,
            self.scale.as_str(),
            if self.ghost { 1 } else { 0 },
            self.next_count,
            self.grid.as_str(),
            if self.center { 1 } else { 0 },
            self.theme.as_str(),
        );
        let _ = fs::write(p, text);
    }
}
