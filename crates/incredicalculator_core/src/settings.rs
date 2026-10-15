use crate::fonts::FontId;
use crate::input::IcKey;
use crate::platform::{rgb8_hex, IcPlatform};
use crate::text::draw_text_f;
use glam::IVec2;

const VISIBLE_ENTRIES: usize = 5;
const ENTRY_HEIGHT: i32 = 31;

#[derive(Clone, Copy)]
enum SettingKind {
    ScreenTimeout,
}

#[derive(Clone, Copy)]
struct SettingEntry {
    label: &'static str,
    kind: SettingKind,
}

const SETTINGS: [SettingEntry; 2] = [
    SettingEntry {
        label: "Screen timeout",
        kind: SettingKind::ScreenTimeout,
    },
    SettingEntry {
        label: "Screen timeout",
        kind: SettingKind::ScreenTimeout,
    },
];

pub struct SettingsMenu {
    selected_entry: usize,
    first_visible_entry: usize,
    editing_value: Option<u32>,
}

impl SettingsMenu {
    pub const fn new() -> Self {
        Self {
            selected_entry: 0,
            first_visible_entry: 0,
            editing_value: None,
        }
    }

    pub fn open(&mut self) {
        self.selected_entry = 0;
        self.first_visible_entry = 0;
        self.editing_value = None;
    }

    pub fn handle_key(&mut self, key: IcKey, platform: &mut dyn IcPlatform) -> bool {
        if let Some(value) = self.editing_value {
            if let Some(digit) = key_digit(key) {
                self.editing_value = Some(value.saturating_mul(10).saturating_add(digit as u32));
            } else {
                match key {
                    IcKey::Func2 => {
                        set_entry_value(SETTINGS[self.selected_entry], value, platform);
                        self.editing_value = None;
                    }
                    IcKey::Func1 => self.editing_value = None,
                    _ => {}
                }
            }
            return false;
        }

        match key {
            IcKey::Func4 if self.selected_entry > 0 => {
                self.selected_entry -= 1;
                self.update_scroll();
            }
            IcKey::Func5 if self.selected_entry + 1 < SETTINGS.len() => {
                self.selected_entry += 1;
                self.update_scroll();
            }
            IcKey::Func1 => return true,
            key if key_digit(key).is_some() => {
                self.editing_value = key_digit(key).map(u32::from);
            }
            _ => {}
        }
        false
    }

    pub fn draw(&self, platform: &mut dyn IcPlatform) {
        platform.clear(rgb8_hex(0x000000));
        draw_text_f(
            platform,
            format_args!("Settings"),
            5.0,
            16.0,
            8.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );

        let visible_count = SETTINGS.len().min(VISIBLE_ENTRIES);
        for visible_index in 0..visible_count {
            let entry_index = self.first_visible_entry + visible_index;
            let entry = SETTINGS[entry_index];
            let y = 34 + visible_index as i32 * ENTRY_HEIGHT;
            let selected = entry_index == self.selected_entry;
            platform.draw_rectangle_rounded(
                IVec2::new(4, y),
                IVec2::new(316, y + ENTRY_HEIGHT - 3),
                rgb8_hex(if selected { 0xffffff } else { 0x666666 }),
                1,
                Some(rgb8_hex(if selected { 0x303030 } else { 0x111111 })),
                3,
            );
            draw_text_f(
                platform,
                format_args!("{}", entry.label),
                11.0,
                (y + 19) as f32,
                5.0,
                2.0,
                rgb8_hex(0xffffff),
                FontId::Futural,
            );

            if selected {
                let value = self
                    .editing_value
                    .unwrap_or_else(|| entry_value(entry, platform));
                let suffix = if self.editing_value.is_some() {
                    "_"
                } else if value == 0 {
                    " (Never)"
                } else {
                    " sec"
                };
                draw_text_f(
                    platform,
                    format_args!("{}{}", value, suffix),
                    205.0,
                    (y + 19) as f32,
                    5.0,
                    2.0,
                    rgb8_hex(0xffff00),
                    FontId::Futural,
                );
            } else {
                let value = entry_value(entry, platform);
                let suffix = if value == 0 { " (Never)" } else { " sec" };
                draw_text_f(
                    platform,
                    format_args!("{}{}", value, suffix),
                    205.0,
                    (y + 19) as f32,
                    5.0,
                    2.0,
                    rgb8_hex(0xaaaaaa),
                    FontId::Futural,
                );
            }
        }

        draw_text_f(
            platform,
            format_args!("{} / {}", self.selected_entry + 1, SETTINGS.len()),
            280.0,
            204.0,
            4.0,
            2.0,
            rgb8_hex(0xaaaaaa),
            FontId::Futural,
        );
        let instructions = if self.editing_value.is_some() {
            "0-9: Enter  FUNC2: Save  FUNC1: Cancel"
        } else {
            "FUNC4/5: Navigate  0-9: Edit  FUNC1: Back"
        };
        draw_text_f(
            platform,
            format_args!("{}", instructions),
            5.0,
            230.0,
            4.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
    }

    fn update_scroll(&mut self) {
        if self.selected_entry < self.first_visible_entry {
            self.first_visible_entry = self.selected_entry;
        } else if self.selected_entry >= self.first_visible_entry + VISIBLE_ENTRIES {
            self.first_visible_entry = self.selected_entry + 1 - VISIBLE_ENTRIES;
        }
    }
}

fn entry_value(entry: SettingEntry, platform: &dyn IcPlatform) -> u32 {
    match entry.kind {
        SettingKind::ScreenTimeout => platform.get_screen_timeout_seconds(),
    }
}

fn set_entry_value(entry: SettingEntry, value: u32, platform: &mut dyn IcPlatform) {
    match entry.kind {
        SettingKind::ScreenTimeout => platform.set_screen_timeout_seconds(value),
    }
}

fn key_digit(key: IcKey) -> Option<u8> {
    match key {
        IcKey::Num0 => Some(0),
        IcKey::Num1 => Some(1),
        IcKey::Num2 => Some(2),
        IcKey::Num3 => Some(3),
        IcKey::Num4 => Some(4),
        IcKey::Num5 => Some(5),
        IcKey::Num6 => Some(6),
        IcKey::Num7 => Some(7),
        IcKey::Num8 => Some(8),
        IcKey::Num9 => Some(9),
        _ => None,
    }
}
