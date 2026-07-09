//! Title and settings menu navigation (TETR.IO-style tabs).

use crate::settings::{on_off, GravityCurve, HandlingPreset, Settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppScreen {
    Title,
    Playing,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleItem {
    Start,
    Settings,
    Quit,
}

impl TitleItem {
    pub const ALL: [TitleItem; 3] = [TitleItem::Start, TitleItem::Settings, TitleItem::Quit];

    pub fn label(self) -> &'static str {
        match self {
            TitleItem::Start => "Start",
            TitleItem::Settings => "Settings",
            TitleItem::Quit => "Quit",
        }
    }

    pub fn next(self) -> Self {
        match self {
            TitleItem::Start => TitleItem::Settings,
            TitleItem::Settings => TitleItem::Quit,
            TitleItem::Quit => TitleItem::Start,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            TitleItem::Start => TitleItem::Quit,
            TitleItem::Settings => TitleItem::Start,
            TitleItem::Quit => TitleItem::Settings,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsTab {
    Handling,
    Gameplay,
    Video,
    Colors,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 4] = [
        SettingsTab::Handling,
        SettingsTab::Gameplay,
        SettingsTab::Video,
        SettingsTab::Colors,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Handling => "Handling",
            SettingsTab::Gameplay => "Gameplay",
            SettingsTab::Video => "Video",
            SettingsTab::Colors => "Colors",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SettingsTab::Handling => SettingsTab::Gameplay,
            SettingsTab::Gameplay => SettingsTab::Video,
            SettingsTab::Video => SettingsTab::Colors,
            SettingsTab::Colors => SettingsTab::Handling,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingsTab::Handling => SettingsTab::Colors,
            SettingsTab::Gameplay => SettingsTab::Handling,
            SettingsTab::Video => SettingsTab::Gameplay,
            SettingsTab::Colors => SettingsTab::Video,
        }
    }

    pub fn row_count(self) -> usize {
        match self {
            SettingsTab::Handling => 6,
            SettingsTab::Gameplay => 10,
            SettingsTab::Video => 10,
            SettingsTab::Colors => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsReturn {
    Title,
    Pause,
}

#[derive(Clone, Debug)]
pub struct MenuState {
    pub screen: AppScreen,
    pub title_sel: TitleItem,
    pub settings_tab: SettingsTab,
    pub settings_row: usize,
    pub settings_return: SettingsReturn,
    pub dirty: bool,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            screen: AppScreen::Title,
            title_sel: TitleItem::Start,
            settings_tab: SettingsTab::Handling,
            settings_row: 0,
            settings_return: SettingsReturn::Title,
            dirty: false,
        }
    }
}

impl MenuState {
    pub fn open_settings(&mut self, ret: SettingsReturn) {
        self.screen = AppScreen::Settings;
        self.settings_return = ret;
        self.settings_tab = SettingsTab::Handling;
        self.settings_row = 0;
        self.dirty = false;
    }

    pub fn move_row(&mut self, delta: i32) {
        let n = self.settings_tab.row_count() as i32;
        let mut r = self.settings_row as i32 + delta;
        if r < 0 {
            r = n - 1;
        } else if r >= n {
            r = 0;
        }
        self.settings_row = r as usize;
    }

    /// First visible row index for scrollable lists.
    pub fn scroll_offset(&self, visible: usize) -> usize {
        let n = self.settings_tab.row_count();
        if n <= visible {
            return 0;
        }
        let sel = self.settings_row;
        if sel < visible / 2 {
            0
        } else if sel + visible / 2 >= n {
            n - visible
        } else {
            sel - visible / 2
        }
    }

    pub fn adjust(&mut self, settings: &mut Settings, dir: i32) {
        self.dirty = true;
        match self.settings_tab {
            SettingsTab::Handling => match self.settings_row {
                0 => {
                    let p = if dir > 0 {
                        settings.handling_preset.next()
                    } else {
                        settings.handling_preset.prev()
                    };
                    // Skip Custom when cycling with arrows from a named preset
                    let p = if p == HandlingPreset::Custom {
                        if dir > 0 {
                            HandlingPreset::Default
                        } else {
                            HandlingPreset::Slow
                        }
                    } else {
                        p
                    };
                    p.apply(settings);
                }
                1 => {
                    settings.das_ms =
                        (settings.das_ms as i64 + 5 * dir as i64).clamp(1, 500) as u64;
                    settings.mark_handling_custom();
                }
                2 => {
                    settings.arr_ms = (settings.arr_ms as i64 + dir as i64).clamp(0, 200) as u64;
                    settings.mark_handling_custom();
                }
                3 => {
                    if settings.sdf_infinite {
                        // turning off infinite → start at 25
                        if dir < 0 {
                            settings.sdf_infinite = false;
                            settings.sdf_ms = 25;
                        }
                    } else {
                        let next = settings.sdf_ms as i64 + dir as i64;
                        if next < 0 || (next == 0 && dir < 0) {
                            settings.sdf_infinite = true;
                            settings.sdf_ms = 0;
                        } else {
                            settings.sdf_ms = next.clamp(1, 200) as u64;
                        }
                    }
                    settings.mark_handling_custom();
                }
                4 => {
                    settings.sdf_infinite = !settings.sdf_infinite;
                    if !settings.sdf_infinite && settings.sdf_ms == 0 {
                        settings.sdf_ms = 25;
                    }
                    settings.mark_handling_custom();
                }
                5 => {
                    settings.soft_drop_points = !settings.soft_drop_points;
                    settings.mark_handling_custom();
                }
                _ => {}
            },
            SettingsTab::Gameplay => match self.settings_row {
                0 => {
                    settings.start_level = (settings.start_level as i32 + dir).clamp(1, 20) as u32;
                }
                1 => {
                    settings.lines_per_level =
                        (settings.lines_per_level as i32 + dir).clamp(1, 30) as u32;
                }
                2 => {
                    settings.lock_delay_ms =
                        (settings.lock_delay_ms as i64 + 25 * dir as i64).clamp(50, 2000) as u64;
                }
                3 => {
                    settings.lock_resets_max =
                        (settings.lock_resets_max as i32 + dir).clamp(0, 40) as u32;
                }
                4 => {
                    settings.gravity = if dir > 0 {
                        settings.gravity.next()
                    } else {
                        settings.gravity.prev()
                    };
                }
                5 => {
                    settings.static_gravity_ms = (settings.static_gravity_ms as i64
                        + 25 * dir as i64)
                        .clamp(10, 2000) as u64;
                }
                6 => settings.hold_enabled = !settings.hold_enabled,
                7 => settings.allow_180 = !settings.allow_180,
                8 => {
                    settings.next_count = (settings.next_count as i32 + dir).clamp(0, 5) as usize;
                }
                9 => {
                    settings.line_clear_ms =
                        (settings.line_clear_ms as i64 + 25 * dir as i64).clamp(0, 500) as u64;
                }
                _ => {}
            },
            SettingsTab::Video => match self.settings_row {
                0 => {
                    settings.scale = if dir > 0 {
                        settings.scale.next()
                    } else {
                        settings.scale.prev()
                    };
                }
                1 => settings.ghost = !settings.ghost,
                2 => {
                    settings.grid = if dir > 0 {
                        settings.grid.next()
                    } else {
                        settings.grid.prev()
                    };
                }
                3 => settings.center = !settings.center,
                4 => settings.show_action_text = !settings.show_action_text,
                5 => settings.show_stats = !settings.show_stats,
                6 => settings.show_pps = !settings.show_pps,
                7 => settings.show_time = !settings.show_time,
                8 => settings.mino_bevel = !settings.mino_bevel,
                9 => settings.show_footer = !settings.show_footer,
                _ => {}
            },
            SettingsTab::Colors => match self.settings_row {
                0 => {
                    settings.theme = if dir > 0 {
                        settings.theme.next()
                    } else {
                        settings.theme.prev()
                    };
                }
                1 => *settings = Settings::default(),
                _ => {}
            },
        }
        settings.clamp();
    }

    pub fn reset_current_tab(&mut self, settings: &mut Settings) {
        self.dirty = true;
        let d = Settings::default();
        match self.settings_tab {
            SettingsTab::Handling => {
                HandlingPreset::Default.apply(settings);
            }
            SettingsTab::Gameplay => {
                settings.start_level = d.start_level;
                settings.lines_per_level = d.lines_per_level;
                settings.lock_delay_ms = d.lock_delay_ms;
                settings.lock_resets_max = d.lock_resets_max;
                settings.gravity = d.gravity;
                settings.static_gravity_ms = d.static_gravity_ms;
                settings.hold_enabled = d.hold_enabled;
                settings.allow_180 = d.allow_180;
                settings.next_count = d.next_count;
                settings.line_clear_ms = d.line_clear_ms;
            }
            SettingsTab::Video => {
                settings.scale = d.scale;
                settings.ghost = d.ghost;
                settings.grid = d.grid;
                settings.center = d.center;
                settings.show_action_text = d.show_action_text;
                settings.show_stats = d.show_stats;
                settings.show_pps = d.show_pps;
                settings.show_time = d.show_time;
                settings.mino_bevel = d.mino_bevel;
                settings.show_footer = d.show_footer;
            }
            SettingsTab::Colors => {
                settings.theme = d.theme;
            }
        }
    }
}

/// Human-readable rows for the current settings tab.
pub fn settings_rows(settings: &Settings, tab: SettingsTab) -> Vec<(String, String)> {
    match tab {
        SettingsTab::Handling => vec![
            ("Preset".into(), settings.handling_preset.label().into()),
            ("DAS".into(), format!("{} ms", settings.das_ms)),
            ("ARR".into(), format!("{} ms", settings.arr_ms)),
            (
                "SDF".into(),
                if settings.sdf_infinite {
                    "INF".into()
                } else {
                    format!("{} ms", settings.sdf_ms)
                },
            ),
            ("SDF infinite".into(), on_off(settings.sdf_infinite).into()),
            (
                "Soft drop points".into(),
                on_off(settings.soft_drop_points).into(),
            ),
        ],
        SettingsTab::Gameplay => vec![
            ("Start level".into(), format!("{}", settings.start_level)),
            (
                "Lines / level".into(),
                format!("{}", settings.lines_per_level),
            ),
            (
                "Lock delay".into(),
                format!("{} ms", settings.lock_delay_ms),
            ),
            (
                "Lock reset max".into(),
                format!("{}", settings.lock_resets_max),
            ),
            ("Gravity curve".into(), settings.gravity.label().into()),
            (
                "Static gravity".into(),
                format!("{} ms", settings.static_gravity_ms),
            ),
            ("Hold".into(), on_off(settings.hold_enabled).into()),
            ("180° rotate".into(), on_off(settings.allow_180).into()),
            ("Next queue".into(), format!("{}", settings.next_count)),
            (
                "Line clear anim".into(),
                format!("{} ms", settings.line_clear_ms),
            ),
        ],
        SettingsTab::Video => vec![
            ("Scale".into(), settings.scale.label().into()),
            ("Ghost piece".into(), on_off(settings.ghost).into()),
            ("Grid".into(), settings.grid.label().into()),
            ("Center board".into(), on_off(settings.center).into()),
            (
                "Action text".into(),
                on_off(settings.show_action_text).into(),
            ),
            ("Stats panel".into(), on_off(settings.show_stats).into()),
            ("Show PPS".into(), on_off(settings.show_pps).into()),
            ("Show time".into(), on_off(settings.show_time).into()),
            ("Mino bevel".into(), on_off(settings.mino_bevel).into()),
            ("Footer help".into(), on_off(settings.show_footer).into()),
        ],
        SettingsTab::Colors => vec![
            ("Theme".into(), settings.theme.label().into()),
            ("Reset ALL settings".into(), "←→ / Enter".into()),
        ],
    }
}

// silence unused import if GravityCurve only used via settings
#[allow(dead_code)]
fn _g(_: GravityCurve) {}
