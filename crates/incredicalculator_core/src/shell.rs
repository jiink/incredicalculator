use crate::app::IcApp;
use crate::app::InputContext;
use crate::apps::AspectRatioCalculator;
use crate::apps::Calculator;
use crate::apps::SoundTest;
use crate::apps::{ RangeMapperCalculator, FaceCalculator };
use crate::audio_engine::AudioEngine;
use crate::input;
use crate::input::IcKey;
use crate::input::KeyState;
use crate::platform::{IcPlatform, CANVAS_WIDTH, CANVAS_HEIGHT};
use crate::platform::rgb8_hex;
use crate::text::*;
use crate::audio_engine;
use alloc::boxed::Box;
use alloc::sync::Arc;
use glam::IVec2;
use num_traits::FromPrimitive;
use rgb::Rgb;
use rgb::*;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(usize)]
pub enum Adjustable {
    Brightness,
    Volume
}

pub struct IcShell {
    apps: [Box<dyn IcApp>; 5], // INCREASE THIS SIZE WHEN ADDING NEW APPS
    active_app_idx: Option<usize>,
    last_active_app_idx: Option<usize>,
    key_states: [KeyState; IcKey::COUNT],
    super_interrupted: bool,
    adjusting_something: Option<Adjustable>,
    audio: AudioEngine
}

impl IcShell {
    pub fn new() -> Self {
        Self {
            apps: [
                Box::new(Calculator::new()),
                Box::new(AspectRatioCalculator::new()),
                Box::new(RangeMapperCalculator::new()),
                Box::new(FaceCalculator::new()),
                Box::new(SoundTest::new())
            ],
            active_app_idx: Some(0),
            last_active_app_idx: None,
            key_states: [KeyState::default(); IcKey::COUNT],
            super_interrupted: false,
            adjusting_something: None,
            audio: AudioEngine::new()
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

    fn draw_battery(&mut self, platform: &mut dyn IcPlatform) {
        let batt_percentage: i32 = platform.get_battery_soc();
        let batt_icon_pos = IVec2::new(282, 3);
        let batt_icon_w = 34;
        let batt_icon_h = 17;
        let fill_w = (batt_percentage * batt_icon_w) / 100;
        platform.draw_rectangle(
            batt_icon_pos,
            batt_icon_pos + IVec2::new(batt_icon_w, batt_icon_h),
            RGB8::new(0, 0, 0),
            0,
            Some(RGB::new(0x80, 0x80, 0x80)),
        );
        platform.draw_rectangle(
            batt_icon_pos,
            batt_icon_pos + IVec2::new(fill_w, batt_icon_h),
            RGB8::new(0, 0, 0),
            0,
            Some(RGB::new(0xff, 0xff, 0xff)),
        );
        platform.draw_string_f(
            format_args!("{}", batt_percentage),
            IVec2::new(290, 2),
            4,
            Rgb::new(0, 0, 0),
        );
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
            },
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
            6                         
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
                fill_radius
            );
        }

        let percentage = (value as i32 * 100) / 255;
        draw_text_f(
            platform, 
            format_args!("{:?}", adjustable), 
            start.x as f32 + 3.0,    
            start.y as f32 - 20.0,    
            2.5, 
            rgb8_hex(0xFFFFFF)        
        );
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
            if !self.super_interrupted && 
                key != IcKey::Super && 
                self.key_states[IcKey::Super as usize].is_down {
                self.super_interrupted = true;
            }
            let mut input_consumed_by_shell: bool = false;
            if ctx.is_down(IcKey::Super) {
                match key {
                    IcKey::Func6 => {
                        self.adjusting_something = Some(Adjustable::Brightness);
                        input_consumed_by_shell = true;
                    },
                    IcKey::Func5 => {
                        self.adjusting_something = Some(Adjustable::Volume);
                        input_consumed_by_shell = true;
                    },
                    _ => ()
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
                let selected_app_i = match key {
                    IcKey::Num0 => Some(0),
                    IcKey::Num1 => Some(1),
                    IcKey::Num2 => Some(2),
                    IcKey::Num3 => Some(3),
                    IcKey::Num4 => Some(4),
                    _ => None
                };
                if let Some(idx) = selected_app_i {
                    self.active_app_idx = Some(idx);
                    self.apps[idx].on_enter();
                }
            }
        }
        if self.key_states[IcKey::Super as usize].just_released && !self.super_interrupted {
            if self.active_app_idx.is_some() {
                self.last_active_app_idx = self.active_app_idx;
                self.active_app_idx = None;
            } else if let Some(prev) = self.last_active_app_idx {
                self.active_app_idx = Some(prev);
                self.apps[prev].on_enter();
            }
        }
        if let Some(appidx) = self.active_app_idx {
            self.apps[appidx].update(platform, &ctx, &mut self.audio);
        } else {
            platform.clear(rgb8_hex(0x7FFF8E));
            for i in 0..self.apps.len() {
                draw_text_f(
                    platform,
                    format_args!("#{}: {}", i, self.apps[i].name()),
                    4.0,
                    4.0 + (20 * i) as f32,
                    2.0,
                    rgb8_hex(0x000000),
                );
            }
        }
        if let Some(adjustable) = self.adjusting_something {
            let v =Self::get_adjustable(platform, adjustable);
            Self::draw_adjustable_readout(platform, adjustable, v);
        }
        self.draw_battery(platform);
    }
}
