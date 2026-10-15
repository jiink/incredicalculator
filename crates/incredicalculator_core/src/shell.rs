use crate::app::IcApp;
use crate::app::InputContext;
use crate::apps::AspectRatioCalculator;
use crate::apps::ColorCalculator;
use crate::apps::SoundTest;
use crate::apps::{Calculator, EngineMode};
use crate::apps::{FaceCalculator, RangeMapperCalculator};
use crate::audio_engine::AudioEngine;
use crate::fonts::FontId;
use crate::graphics;
use crate::graphics::{draw_vitmap, Vitmap};
use crate::input::IcKey;
use crate::input::KeyState;
use crate::platform::rgb8_hex;
use crate::platform::{IcPlatform, CANVAS_HEIGHT, CANVAS_WIDTH};
use crate::settings::SettingsMenu;
use crate::text::*;
use alloc::boxed::Box;
use glam::{IVec2, Vec2};
use num_traits::clamp;
use num_traits::FromPrimitive;
use rgb::Rgb;
use rgb::*;

const APPS_PER_PAGE: usize = 6;

#[derive(Clone, Copy)]
enum SelectorAction {
    Launch(usize),
    PreviousPage,
    NextPage,
    OpenSettings,
    None,
}

#[derive(Clone, Copy)]
struct SelectorCard<'a> {
    name: &'a str,
    button: IcKey,
    vitmap: &'static Vitmap,
    color: RGB8,
    action: SelectorAction,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(usize)]
pub enum Adjustable {
    Brightness,
    Volume,
}

pub struct IcShell {
    apps: [Box<dyn IcApp>; 7], // INCREASE THIS SIZE WHEN ADDING NEW APPS
    active_app_idx: Option<usize>,
    last_active_app_idx: Option<usize>,
    selector_page: usize,
    selector_hints_until: Option<u64>,
    settings_open: bool,
    settings_menu: SettingsMenu,
    key_states: [KeyState; IcKey::COUNT],
    super_interrupted: bool,
    adjusting_something: Option<Adjustable>,
    audio: AudioEngine,
}

impl IcShell {
    pub fn new() -> Self {
        Self {
            apps: [
                Box::new(Calculator::new(EngineMode::Programmer)),
                Box::new(Calculator::new(EngineMode::Scientific)),
                Box::new(AspectRatioCalculator::new()),
                Box::new(RangeMapperCalculator::new()),
                Box::new(ColorCalculator::new()),
                Box::new(SoundTest::new()),
                Box::new(FaceCalculator::new()),
            ],
            active_app_idx: Some(0),
            last_active_app_idx: None,
            selector_page: 0,
            selector_hints_until: None,
            settings_open: false,
            settings_menu: SettingsMenu::new(),
            key_states: [KeyState::default(); IcKey::COUNT],
            super_interrupted: false,
            adjusting_something: None,
            audio: AudioEngine::new(),
        }
    }

    pub fn audio_mut(&mut self) -> &mut AudioEngine {
        &mut self.audio
    }

    pub fn key_down(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            return;
        }
        self.key_states[key as usize].is_down = true;
    }

    pub fn key_up(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            return;
        }
        self.key_states[key as usize].is_down = false;
    }

    pub fn fill_audio(&mut self, out_pcm: &mut [i16]) {
        for sample in out_pcm {
            let s = self.audio.next_sample();
            *sample = s;
        }
    }

    pub fn has_active_audio(&self) -> bool {
        self.audio.is_active()
    }

    pub fn requires_realtime_updates(&self) -> bool {
        self.active_app_idx
            .map(|idx| self.apps[idx].requires_realtime_updates())
            .unwrap_or(false)
    }

    fn draw_battery(&mut self, platform: &mut dyn IcPlatform) {
        let soc = platform.get_battery_soc();
        let batt_percentage = soc.map(|val| clamp(val, 0, 100));
        let batt_icon_pos = IVec2::new(282, 3);
        let batt_icon_w = 34;
        let batt_icon_h = 17;
        let fill_w = (batt_percentage.unwrap_or(100) * batt_icon_w) / 100;
        platform.draw_rectangle(
            batt_icon_pos,
            batt_icon_pos + IVec2::new(batt_icon_w, batt_icon_h),
            RGB8::new(0, 0, 0),
            0,
            Some(RGB::new(0x80, 0x80, 0x80)),
        );
        if fill_w > 0 {
            platform.draw_rectangle(
                batt_icon_pos,
                batt_icon_pos + IVec2::new(fill_w, batt_icon_h),
                RGB8::new(0, 0, 0),
                0,
                Some(RGB::new(0xff, 0xff, 0xff)),
            );
        }
        match batt_percentage {
            Some(pct) => {
                let x_pos = match pct {
                    100.. => 284.0,
                    10.. => 290.0,
                    _ => 295.0,
                };
                draw_text_f(
                    platform,
                    format_args!("{}", pct),
                    x_pos,
                    12.0,
                    5.0,
                    2.0,
                    Rgb::new(0, 0, 0),
                    FontId::Futural,
                );
            }
            None => {
                draw_text_f(
                    platform,
                    format_args!("N/A"),
                    284.0,
                    12.0,
                    5.0,
                    2.0,
                    Rgb::new(0, 0, 0),
                    FontId::Futural,
                );
            }
        }
    }

    fn adjust_adjustable(platform: &mut dyn IcPlatform, adjustable: Adjustable, amount: i32) {
        if amount == 0 {
            return;
        }
        let og_val_32 = Self::get_adjustable(platform, adjustable) as i32;
        let new_val_32 = og_val_32.saturating_add(amount);
        let new_val = new_val_32.clamp(0, 255) as u8;
        match adjustable {
            Adjustable::Brightness => {
                platform.set_brightness(new_val);
            }
            Adjustable::Volume => {
                platform.set_volume(new_val);
            }
        }
    }

    fn get_adjustable(platform: &dyn IcPlatform, adjustable: Adjustable) -> u8 {
        match adjustable {
            Adjustable::Brightness => platform.get_brightness(),
            Adjustable::Volume => platform.get_volume(),
        }
    }

    fn draw_adjustable_readout(platform: &mut dyn IcPlatform, adjustable: Adjustable, value: u8) {
        let center_x = CANVAS_WIDTH as i32 / 2;
        let center_y = CANVAS_HEIGHT as i32 / 2;

        let bar_w = 194;
        let bar_h = 24;
        let stroke_w = 4;

        let start = IVec2::new(center_x - bar_w / 2, center_y - bar_h / 2);
        let end = IVec2::new(center_x + bar_w / 2, center_y + bar_h / 2);

        platform.draw_rectangle_rounded(
            start,
            end,
            rgb8_hex(0x000000),
            stroke_w as u32,
            Some(rgb8_hex(0x222222)),
            6,
        );

        let inner_start_x = start.x + stroke_w;
        let inner_start_y = start.y + stroke_w;
        let max_inner_w = bar_w - (stroke_w * 2);
        let inner_h = bar_h - (stroke_w * 2);

        let fill_w = (value as i32 * max_inner_w) / 255;

        if fill_w > 0 {
            let fill_radius = 2.min(fill_w as u32 / 2);

            platform.draw_rectangle_rounded(
                IVec2::new(inner_start_x, inner_start_y),
                IVec2::new(inner_start_x + fill_w, inner_start_y + inner_h),
                rgb8_hex(0x000000),
                0,
                Some(rgb8_hex(0x6ABE30)),
                fill_radius,
            );
        }

        let _percentage = (value as i32 * 100) / 255;
        draw_text_f(
            platform,
            format_args!("{:?}", adjustable),
            start.x as f32 + 3.0,
            start.y as f32 - 20.0,
            5.0,
            2.0,
            rgb8_hex(0xFFFFFF),
            FontId::Futural,
        );
    }

    fn selector_cards(&self) -> [SelectorCard<'_>; 9] {
        let mut cards = [
            SelectorCard {
                name: "",
                button: IcKey::Num7,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num8,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num9,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num4,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num5,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num6,
                vitmap: &graphics::SAMPLE_GRAPHIC,
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "Previous",
                button: IcKey::Num1,
                vitmap: &graphics::LEFT_ARROW,
                color: rgb8_hex(0xffffff),
                action: SelectorAction::PreviousPage,
            },
            SelectorCard {
                name: "Settings",
                button: IcKey::Num2,
                vitmap: &graphics::SETTINGS_ICON,
                color: rgb8_hex(0xffffff),
                action: SelectorAction::OpenSettings,
            },
            SelectorCard {
                name: "Next",
                button: IcKey::Num3,
                vitmap: &graphics::RIGHT_ARROW,
                color: rgb8_hex(0xffffff),
                action: SelectorAction::NextPage,
            },
        ];

        let first_app_idx = self.selector_page * APPS_PER_PAGE;
        for (slot, card) in cards.iter_mut().take(APPS_PER_PAGE).enumerate() {
            let app_idx = first_app_idx + slot;
            if let Some(app) = self.apps.get(app_idx) {
                card.name = app.name();
                card.vitmap = app.icon();
                card.color = app.color();
                card.action = SelectorAction::Launch(app_idx);
            }
        }

        cards
    }

    fn draw_app_selector(&self, platform: &mut dyn IcPlatform, show_key_hints: bool) {
        platform.clear(rgb8_hex(0x000000));
        let page_count = (self.apps.len() + APPS_PER_PAGE - 1) / APPS_PER_PAGE;
        draw_text_f(
            platform,
            format_args!("Apps - Page {} of {}", self.selector_page + 1, page_count),
            5.0,
            12.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );

        let cards = self.selector_cards();
        for (index, card) in cards.iter().enumerate() {
            let column = (index % 3) as i32;
            let row = (index / 3) as i32;
            let start = IVec2::new(4 + column * 106, 27 + row * 69);
            let end = start + IVec2::new(100, 64);
            let is_empty = matches!(card.action, SelectorAction::None) && index < APPS_PER_PAGE;
            if is_empty {
                continue;
            }
            platform.draw_rectangle_rounded(start, end, rgb8_hex(0xffffff), 0, Some(card.color), 5);
            draw_vitmap(
                platform,
                card.vitmap,
                0,
                Vec2::new(start.x as f32 + 50.0, start.y as f32 + 32.0),
                Vec2::splat(3.0),
            );
            // draw_text_f(
            //     platform,
            //     format_args!("{}", card.name),
            //     start.x as f32 + 4.0,
            //     start.y as f32 + 57.0,
            //     3.0,
            //     2.0,
            //     rgb8_hex(0xffffff),
            //     FontId::Futural,
            // );
            if show_key_hints {
                let hint_start = start;
                platform.draw_rectangle(
                    hint_start,
                    hint_start + IVec2::splat(30),
                    rgb8_hex(0x000000),
                    1,
                    Some(rgb8_hex(0xffff00)),
                );
                draw_text_f(
                    platform,
                    format_args!("{}", button_number(card.button)),
                    hint_start.x as f32 + 4.0,
                    hint_start.y as f32 + 17.0,
                    10.0,
                    4.0,
                    rgb8_hex(0x000000),
                    FontId::Futural,
                );
            }
        }
    }

    pub fn update(&mut self, platform: &mut dyn IcPlatform) {
        for s in self.key_states.iter_mut() {
            s.just_pressed = s.is_down && !s.was_down;
            s.just_released = !s.is_down && s.was_down;
            s.was_down = s.is_down;
        }
        let ctx = InputContext {
            key_states: &self.key_states,
        };
        if self.key_states[IcKey::Super as usize].just_pressed {
            self.super_interrupted = false;
        }
        for i in 0..IcKey::COUNT {
            if !self.key_states[i].just_pressed {
                continue;
            }
            let Some(key) = IcKey::from_usize(i) else {
                continue;
            };
            if !self.super_interrupted
                && key != IcKey::Super
                && self.key_states[IcKey::Super as usize].is_down
            {
                self.super_interrupted = true;
            }
            let mut input_consumed_by_shell: bool = false;
            if ctx.is_down(IcKey::Super) {
                match key {
                    IcKey::Func6 => {
                        self.adjusting_something = Some(Adjustable::Brightness);
                        input_consumed_by_shell = true;
                    }
                    IcKey::Func5 => {
                        self.adjusting_something = Some(Adjustable::Volume);
                        input_consumed_by_shell = true;
                    }
                    _ => (),
                }
            } else if let Some(adjustable) = self.adjusting_something {
                let adjust_amt = 32;
                input_consumed_by_shell = true;
                match key {
                    IcKey::Func4 => {
                        Self::adjust_adjustable(platform, adjustable, -adjust_amt);
                    }
                    IcKey::Func5 => {
                        Self::adjust_adjustable(platform, adjustable, adjust_amt);
                    }
                    _ => {
                        self.adjusting_something = None;
                    }
                }
            }
            if let Some(app_idx) = self.active_app_idx {
                if !input_consumed_by_shell {
                    self.apps[app_idx].on_key(key, &ctx);
                }
            } else {
                if self.settings_open {
                    if self.settings_menu.handle_key(key, platform) {
                        self.settings_open = false;
                    }
                } else {
                    let action = self
                        .selector_cards()
                        .iter()
                        .find(|card| card.button == key)
                        .map(|card| card.action);
                    match action {
                        Some(SelectorAction::Launch(idx)) => {
                            self.active_app_idx = Some(idx);
                            self.apps[idx].on_enter();
                        }
                        Some(SelectorAction::PreviousPage) if self.selector_page > 0 => {
                            self.selector_page -= 1;
                        }
                        Some(SelectorAction::NextPage)
                            if (self.selector_page + 1) * APPS_PER_PAGE < self.apps.len() =>
                        {
                            self.selector_page += 1;
                        }
                        Some(SelectorAction::OpenSettings) => {
                            self.settings_open = true;
                            self.settings_menu.open();
                        }
                        _ => {}
                    }
                }
            }
        }
        if self.key_states[IcKey::Super as usize].just_released && !self.super_interrupted {
            if self.active_app_idx.is_some() {
                self.last_active_app_idx = self.active_app_idx;
                self.active_app_idx = None;
                self.selector_hints_until = Some(platform.millis().saturating_add(1400));
                // release its sound instead of leaving a held note behind.
                self.audio.release_all();
            } else if self.settings_open {
                self.settings_open = false;
            } else if let Some(prev) = self.last_active_app_idx {
                self.active_app_idx = Some(prev);
                self.apps[prev].on_enter();
            }
        }
        self.audio.set_volume(platform.get_volume());
        if let Some(appidx) = self.active_app_idx {
            self.apps[appidx].update(platform, &ctx, &mut self.audio);
        } else if self.settings_open {
            self.settings_menu.draw(platform);
        } else {
            let show_key_hints = self
                .selector_hints_until
                .is_some_and(|expires_at| platform.millis() < expires_at);
            self.draw_app_selector(platform, show_key_hints);
        }
        if let Some(adjustable) = self.adjusting_something {
            let v = Self::get_adjustable(platform, adjustable);
            Self::draw_adjustable_readout(platform, adjustable, v);
        }
        self.draw_battery(platform);
    }
}

fn button_number(button: IcKey) -> u8 {
    match button {
        IcKey::Num1 => 1,
        IcKey::Num2 => 2,
        IcKey::Num3 => 3,
        IcKey::Num4 => 4,
        IcKey::Num5 => 5,
        IcKey::Num6 => 6,
        IcKey::Num7 => 7,
        IcKey::Num8 => 8,
        IcKey::Num9 => 9,
        _ => 0,
    }
}
