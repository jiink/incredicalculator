use crate::app::IcApp;
use crate::app::InputContext;
use crate::apps::AspectRatioCalculator;
use crate::apps::Calculator;
use crate::apps::{FaceCalculator, RangeMapperCalculator};
use crate::input;
use crate::input::IcKey;
use crate::input::KeyState;
use crate::platform::IcPlatform;
use crate::platform::rgb8_hex;
use crate::text::*;
use alloc::boxed::Box;
use glam::IVec2;
use num_traits::FromPrimitive;
use rgb::Rgb;
use rgb::*;

pub struct IcShell {
    apps: [Box<dyn IcApp>; 4], // INCREASE THIS SIZE WHEN ADDING NEW APPS
    active_app_idx: Option<usize>,
    last_active_app_idx: Option<usize>,
    key_states: [KeyState; IcKey::COUNT],
    super_interrupted: bool,
}

impl IcShell {
    pub fn new() -> Self {
        Self {
            apps: [
                Box::new(Calculator::new()),
                Box::new(AspectRatioCalculator::new()),
                Box::new(RangeMapperCalculator::new()),
                Box::new(FaceCalculator::new()),
            ],
            active_app_idx: Some(0),
            last_active_app_idx: None,
            key_states: [KeyState::default(); IcKey::COUNT],
            super_interrupted: false
        }
    }

    pub fn key_down(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            ()
        }
        self.key_states[key as usize].is_down = true;
    }

    pub fn key_up(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            ()
        }
        self.key_states[key as usize].is_down = false;
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
            if self.key_states[i].just_pressed {
                if let Some(key) = IcKey::from_usize(i) {
                    if self.super_interrupted == false && 
                        key != IcKey::Super && 
                        self.key_states[IcKey::Super as usize].is_down {
                        self.super_interrupted = true;
                    }
                    if self.active_app_idx.is_some() {
                        let mut input_consumed_by_shell: bool = false;
                        if ctx.is_down(IcKey::Super) {
                            match key {
                                _ => {}
                            }
                        }
                        if input_consumed_by_shell {
                            continue;
                        }
                        if let Some(appidx) = self.active_app_idx {
                            self.apps[appidx].on_key(key, &ctx);
                        }
                    } else {
                        match key {
                            IcKey::Num0 => {
                                self.active_app_idx = Some(0);
                            }
                            IcKey::Num1 => {
                                self.active_app_idx = Some(1);
                            }
                            IcKey::Num2 => {
                                self.active_app_idx = Some(2);
                            }
                            IcKey::Num3 => {
                                self.active_app_idx = Some(3);
                            }
                            _ => ()
                        }
                        if let Some(appidx) = self.active_app_idx {
                            self.apps[appidx].on_enter();
                        }
                    }
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
            self.apps[appidx].update(platform, &ctx);
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
        self.draw_battery(platform);
    }
}
