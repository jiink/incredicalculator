use crate::app::InputContext;
use crate::fonts::FontId;
use crate::input::{IcKey, KeyMode};
use crate::platform::{rgb8_hex, IcPlatform};
use crate::text::draw_text_f;
use crate::ui::{MathInput, MathInputMode};
use glam::{IVec2, Vec2};
use rgb::RGB8;

const VISIBLE_ENTRIES: usize = 5;
const ENTRY_HEIGHT: i32 = 31;
const INPUT_CHAR_LIMIT: usize = 10;

#[derive(Clone, Copy)]
enum IntegerSettingKind {
    ScreenTimeout,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum CheckboxSettingKind {
    StickyKeys,
}

#[derive(Clone, Copy)]
enum ChoiceSettingKind {
    KeyMode,
}

#[derive(Clone, Copy)]
struct SettingEntry {
    label: &'static str,
    display: SettingDisplay,
}

#[derive(Clone, Copy)]
enum SettingDisplay {
    Integer {
        kind: IntegerSettingKind,
        suffix: &'static str,
        zero_suffix: Option<&'static str>,
    },
    #[allow(dead_code)]
    Checkbox {
        kind: CheckboxSettingKind,
    },
    Choice {
        kind: ChoiceSettingKind,
    },
}

enum EntryValue {
    Integer { value: u32, suffix: &'static str },
    Checkbox { checked: bool },
    Choice { label: &'static str },
}

const SETTINGS: [SettingEntry; 2] = [
    SettingEntry {
        label: "Screen timeout",
        display: SettingDisplay::Integer {
            kind: IntegerSettingKind::ScreenTimeout,
            suffix: " sec",
            zero_suffix: Some(" (Never)"),
        },
    },
    SettingEntry {
        label: "Keys",
        display: SettingDisplay::Choice {
            kind: ChoiceSettingKind::KeyMode,
        },
    },
];

pub struct SettingsMenu {
    selected_entry: usize,
    first_visible_entry: usize,
    editing_value: bool,
    sticky_keys_enabled: bool,
    key_mode: KeyMode,
    value_input: MathInput<INPUT_CHAR_LIMIT>,
}

impl SettingsMenu {
    pub const fn new() -> Self {
        Self {
            selected_entry: 0,
            first_visible_entry: 0,
            editing_value: false,
            sticky_keys_enabled: false,
            key_mode: KeyMode::Normal,
            value_input: MathInput::new(
                IVec2::new(200, 38),
                IVec2::new(112, 23),
                MathInputMode::Integer,
                4.0,
                5.0,
                RGB8::new(0x30, 0x30, 0x30),
                RGB8::new(0xff, 0xff, 0x00),
                None,
            ),
        }
    }

    pub fn key_mode(&self) -> KeyMode {
        self.key_mode
    }

    pub fn open(&mut self) {
        self.selected_entry = 0;
        self.first_visible_entry = 0;
        self.editing_value = false;
        self.value_input.clear();
    }

    pub fn handle_key(
        &mut self,
        key: IcKey,
        ctx: &InputContext,
        platform: &mut dyn IcPlatform,
    ) -> bool {
        if self.editing_value {
            match key {
                IcKey::Func6 => {
                    if let Ok(value) = self.value_input.text().parse::<u32>() {
                        if let SettingDisplay::Integer { kind, .. } =
                            SETTINGS[self.selected_entry].display
                        {
                            set_entry_value(kind, value, platform);
                        }
                        self.editing_value = false;
                        self.value_input.clear();
                    }
                }
                _ => {
                    self.value_input.handle_key(key, ctx);
                }
            }
            return false;
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
                let entry = SETTINGS[self.selected_entry];
                match entry.display {
                    SettingDisplay::Integer { kind, .. } => {
                        let text = alloc::format!("{}", entry_value(kind, platform));
                        self.value_input
                            .set_text(&text)
                            .expect("u32 setting value fits the integer input");
                        self.editing_value = true;
                    }
                    SettingDisplay::Checkbox { kind } => match kind {
                        CheckboxSettingKind::StickyKeys => {
                            self.sticky_keys_enabled = !self.sticky_keys_enabled;
                        }
                    },
                    SettingDisplay::Choice { kind } => match kind {
                        ChoiceSettingKind::KeyMode => self.key_mode = self.key_mode.next(),
                    },
                }
            }
            IcKey::Func1 => return true,
            _ => {}
        }
        false
    }

    pub fn draw(&mut self, platform: &mut dyn IcPlatform) {
        platform.clear(rgb8_hex(0x000000));
        draw_text_f(
            platform,
            format_args!("Settings"),
            5.0,
            12.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );

        let visible_count = SETTINGS.len().min(VISIBLE_ENTRIES);
        for visible_index in 0..visible_count {
            let entry_index = self.first_visible_entry + visible_index;
            let entry = SETTINGS[entry_index];
            let y = 26 + visible_index as i32 * ENTRY_HEIGHT;
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

            if selected && self.editing_value {
                self.value_input.pos = IVec2::new(200, y + 2);
                self.value_input.draw(platform, true);
            } else {
                let value = self.resolve_entry_value(entry.display, platform);
                let color = rgb8_hex(if selected { 0xffff00 } else { 0xaaaaaa });
                draw_entry_value(platform, value, y, color);
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
            "^/v: Navigate  =: Edit/Cycle"
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

    fn resolve_entry_value(
        &self,
        display: SettingDisplay,
        platform: &dyn IcPlatform,
    ) -> EntryValue {
        match display {
            SettingDisplay::Integer {
                kind,
                suffix,
                zero_suffix,
            } => {
                let value = entry_value(kind, platform);
                EntryValue::Integer {
                    value,
                    suffix: if value == 0 {
                        zero_suffix.unwrap_or(suffix)
                    } else {
                        suffix
                    },
                }
            }
            SettingDisplay::Checkbox { kind } => EntryValue::Checkbox {
                checked: match kind {
                    CheckboxSettingKind::StickyKeys => self.sticky_keys_enabled,
                },
            },
            SettingDisplay::Choice { kind } => EntryValue::Choice {
                label: match kind {
                    ChoiceSettingKind::KeyMode => self.key_mode.label(),
                },
            },
        }
    }

    fn update_scroll(&mut self) {
        if self.selected_entry < self.first_visible_entry {
            self.first_visible_entry = self.selected_entry;
        } else if self.selected_entry >= self.first_visible_entry + VISIBLE_ENTRIES {
            self.first_visible_entry = self.selected_entry + 1 - VISIBLE_ENTRIES;
        }
    }
}

fn draw_entry_value(platform: &mut dyn IcPlatform, value: EntryValue, y: i32, color: RGB8) {
    match value {
        EntryValue::Integer { value, suffix } => {
            draw_text_f(
                platform,
                format_args!("{}{}", value, suffix),
                205.0,
                (y + 19) as f32,
                5.0,
                2.0,
                color,
                FontId::Futural,
            );
        }
        EntryValue::Choice { label } => {
            draw_text_f(
                platform,
                format_args!("{}", label),
                205.0,
                (y + 19) as f32,
                5.0,
                2.0,
                color,
                FontId::Futural,
            );
        }
        EntryValue::Checkbox { checked } => {
            let top_left = IVec2::new(294, y + 7);
            let bottom_right = top_left + IVec2::new(16, 16);
            platform.draw_rectangle(top_left, bottom_right, color, 1, None);
            if checked {
                platform.draw_line(
                    Vec2::new(297.0, (y + 15) as f32),
                    Vec2::new(301.0, (y + 19) as f32),
                    color,
                    2,
                );
                platform.draw_line(
                    Vec2::new(301.0, (y + 19) as f32),
                    Vec2::new(308.0, (y + 10) as f32),
                    color,
                    2,
                );
            }
        }
    }
}

fn entry_value(kind: IntegerSettingKind, platform: &dyn IcPlatform) -> u32 {
    match kind {
        IntegerSettingKind::ScreenTimeout => platform.get_screen_timeout_seconds(),
    }
}

fn set_entry_value(kind: IntegerSettingKind, value: u32, platform: &mut dyn IcPlatform) {
    match kind {
        IntegerSettingKind::ScreenTimeout => platform.set_screen_timeout_seconds(value),
    }
}
