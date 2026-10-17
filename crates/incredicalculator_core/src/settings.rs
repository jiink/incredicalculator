use crate::app::InputContext;
use crate::fonts::FontId;
use crate::input::IcKey;
use crate::platform::{rgb8_hex, IcPlatform};
use crate::text::draw_text_f;
use crate::ui::{MathInput, MathInputMode};
use glam::IVec2;
use rgb::RGB8;

const VISIBLE_ENTRIES: usize = 5;
const ENTRY_HEIGHT: i32 = 31;
const INPUT_CHAR_LIMIT: usize = 10;

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
    editing_value: bool,
    value_input: MathInput<INPUT_CHAR_LIMIT>,
}

impl SettingsMenu {
    pub const fn new() -> Self {
        Self {
            selected_entry: 0,
            first_visible_entry: 0,
            editing_value: false,
            value_input: MathInput::new(
                IVec2::new(200, 38),
                IVec2::new(115, 23),
                MathInputMode::Integer,
                4.0,
                5.0,
                RGB8::new(0x30, 0x30, 0x30),
                RGB8::new(0xff, 0xff, 0x00),
                None,
            ),
        }
    }

    pub fn open(&mut self) {
        self.selected_entry = 0;
        self.first_visible_entry = 0;
        self.editing_value = false;
        self.value_input.clear();
    }

    pub fn handle_key(&mut self, key: IcKey, ctx: &InputContext, platform: &mut dyn IcPlatform) {
        if self.editing_value {
            match key {
                IcKey::Func6 => {
                    if let Ok(value) = self.value_input.text().parse::<u32>() {
                        set_entry_value(SETTINGS[self.selected_entry], value, platform);
                        self.editing_value = false;
                        self.value_input.clear();
                    }
                }
                _ => {
                    self.value_input.handle_key(key, ctx);
                }
            }
            return;
        }
        match key {
            IcKey::Num8 if self.selected_entry > 0 => {
                self.selected_entry -= 1;
                self.update_scroll();
            }
            IcKey::Num2 if self.selected_entry + 1 < SETTINGS.len() => {
                self.selected_entry += 1;
                self.update_scroll();
            }
            IcKey::Func6 => {
                let value = entry_value(SETTINGS[self.selected_entry], platform);
                let text = alloc::format!("{}", value);
                self.value_input
                    .set_text(&text)
                    .expect("u32 setting value fits the integer input");
                self.editing_value = true;
            }
            _ => {}
        }
    }

    pub fn draw(&mut self, platform: &mut dyn IcPlatform) {
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
                if self.editing_value {
                    self.value_input.pos = IVec2::new(200, y + 4);
                    self.value_input.draw(platform, true);
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
                        rgb8_hex(0xffff00),
                        FontId::Futural,
                    );
                }
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
        let instructions = if self.editing_value {
            ""
        } else {
            "^/v: Navigate  =: Edit"
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
