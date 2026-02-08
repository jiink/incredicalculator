use alloc::boxed::Box;
use num_traits::FromPrimitive;
use rgb::Rgb;
use crate::app::IcApp;
use crate::app::InputContext;
use crate::input::KeyState;
use crate::input::IcKey;
use crate::apps::Calculator;
use crate::platform::IcPlatform;
use crate::text::*;

pub struct IcShell {
    apps: [Box<dyn IcApp>; 1], // INCREASE THIS SIZE WHEN ADDING NEW APPS
    active_app_idx: usize,
    key_states: [KeyState; IcKey::COUNT]
}

impl IcShell {
    pub fn new() -> Self {
        Self {
            apps: [
                Box::new(Calculator::new())
            ],
            active_app_idx: 0,
            key_states: [KeyState::default(); IcKey::COUNT],
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

    pub fn update(&mut self, platform: &mut dyn IcPlatform) {
        platform.clear_lines();
        for s in self.key_states.iter_mut() {
            s.just_pressed = s.is_down && !s.was_down;
            s.just_released = !s.is_down && s.was_down;
            s.was_down = s.is_down;
        }
        let ctx = InputContext {
            key_states: &self.key_states
        };
        for i in 0..IcKey::COUNT {
            if self.key_states[i].just_pressed {
                if let Some(key) = IcKey::from_usize(i) {
                    let mut input_consumed_by_shell: bool = false;
                    if ctx.is_down(IcKey::Super) {
                        match key {
                            IcKey::Func1 => { self.active_app_idx = 0; input_consumed_by_shell = true; }
                            IcKey::Func2 => { self.active_app_idx = 1; input_consumed_by_shell = true; }
                            _ => {}
                        }
                    } 
                    if input_consumed_by_shell {
                        continue;
                    }
                    self.apps[self.active_app_idx].on_key(key, &ctx);
                }
            }
        }
        self.apps[self.active_app_idx].update(platform, &ctx);
        draw_text(platform,
                "(shell)",
                0.0,
                0.0,
                3.0,
                Rgb {
                    r: 0xff,
                    g: 0xff,
                    b: 0x00,
                },);
    }
}

