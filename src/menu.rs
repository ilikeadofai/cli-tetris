//! Title and settings menu navigation (TETR.IO-style tabs).

use crate::settings::Settings;

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
    Video,
    Colors,
}

impl SettingsTab {
    pub const ALL: [SettingsTab; 3] =
        [SettingsTab::Handling, SettingsTab::Video, SettingsTab::Colors];

    pub fn label(self) -> &'static str {
        match self {
            SettingsTab::Handling => "Handling",
            SettingsTab::Video => "Video",
            SettingsTab::Colors => "Colors",
        }
    }

    pub fn next(self) -> Self {
        match self {
            SettingsTab::Handling => SettingsTab::Video,
            SettingsTab::Video => SettingsTab::Colors,
            SettingsTab::Colors => SettingsTab::Handling,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingsTab::Handling => SettingsTab::Colors,
            SettingsTab::Video => SettingsTab::Handling,
            SettingsTab::Colors => SettingsTab::Video,
        }
    }

    pub fn row_count(self) -> usize {
        match self {
            SettingsTab::Handling => 3, // DAS ARR SDF
            SettingsTab::Video => 5,    // scale ghost next grid center
            SettingsTab::Colors => 2,   // theme + reset all
        }
    }
}

/// Where we return after closing settings.
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
    /// Snapshot before opening settings (for cancel — we always apply on Esc).
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

    pub fn adjust(&mut self, settings: &mut Settings, dir: i32) {
        self.dirty = true;
        match self.settings_tab {
            SettingsTab::Handling => match self.settings_row {
                0 => {
                    // DAS step 5
                    let step = 5i64 * dir as i64;
                    settings.das_ms = (settings.das_ms as i64 + step).clamp(50, 333) as u64;
                }
                1 => {
                    let step = 1i64 * dir as i64;
                    settings.arr_ms = (settings.arr_ms as i64 + step).clamp(0, 100) as u64;
                }
                2 => {
                    let step = 1i64 * dir as i64;
                    settings.sdf_ms = (settings.sdf_ms as i64 + step).clamp(5, 100) as u64;
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
                    let n = settings.next_count as i32 + dir;
                    settings.next_count = n.clamp(1, 5) as usize;
                }
                3 => {
                    settings.grid = if dir > 0 {
                        settings.grid.next()
                    } else {
                        settings.grid.prev()
                    };
                }
                4 => settings.center = !settings.center,
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
                1 => {
                    // Reset all
                    *settings = Settings::default();
                }
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
                settings.das_ms = d.das_ms;
                settings.arr_ms = d.arr_ms;
                settings.sdf_ms = d.sdf_ms;
            }
            SettingsTab::Video => {
                settings.scale = d.scale;
                settings.ghost = d.ghost;
                settings.next_count = d.next_count;
                settings.grid = d.grid;
                settings.center = d.center;
            }
            SettingsTab::Colors => {
                settings.theme = d.theme;
            }
        }
    }
}


