use crate::input::{IcKey, KeyState};
use crate::text::text_to_pos;
use crate::{
    app::{ IcApp, InputContext },
    platform::{self, IcPlatform, rgb8_hex},
    text::{draw_text, draw_text_f},
};
use glam::IVec2;
use num_traits::{abs, clamp_max};
use rgb::{RGB8, Rgb};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoboMood {
    Default,
    Tired,
    Angry,
    Happy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RoboPosition {
    N, 
    NE,
    E, 
    SE,
    S, 
    SW,
    W, 
    NW,
    Center,
}

// Converted from https://github.com/FluxGarage/RoboEyes
pub struct RoboEyes {
    pub screen_size: IVec2,
    pub bg_color: RGB8,
    pub main_color: RGB8,

    last_frame_time: u64,
    frame_interval: u64, // ms

    rng_seed: u32,

    pub tired: bool,
    pub angry: bool,
    pub happy: bool,
    pub curious: bool,
    pub cyclops: bool,
    eye_l_open: bool,
    eye_r_open: bool,

    eye_l_width_default: i32,
    eye_l_height_default: i32,
    eye_l_width_current: i32,
    eye_l_height_current: i32,
    eye_l_width_next: i32,
    eye_l_height_next: i32,
    eye_l_border_radius_current: i32,
    eye_l_border_radius_next: i32,
    
    eye_r_width_default: i32,
    eye_r_height_default: i32,
    eye_r_width_current: i32,
    eye_r_height_current: i32,
    eye_r_width_next: i32,
    eye_r_height_next: i32,
    eye_r_border_radius_current: i32,
    eye_r_border_radius_next: i32,

    eye_l_pos: IVec2,
    eye_l_pos_next: IVec2,
    eye_r_pos: IVec2,
    eye_r_pos_next: IVec2,

    eyelids_tired_height: i32,
    eyelids_tired_height_next: i32,
    eyelids_angry_height: i32,
    eyelids_angry_height_next: i32,
    eyelids_happy_offset: i32,
    eyelids_happy_offset_next: i32,
    
    space_between_current: i32,
    space_between_next: i32,

    h_flicker: bool,
    h_flicker_amp: i32,
    h_flicker_alt: bool,
    
    v_flicker: bool,
    v_flicker_amp: i32,
    v_flicker_alt: bool,

    autoblinker: bool,
    blink_interval: u64, // seconds
    blink_variation: u64, // seconds
    blink_timer_ms: u64,

    idle: bool,
    idle_interval: u64,
    idle_variation: u64,
    idle_timer_ms: u64,

    confused: bool,
    confused_timer: u64,
    confused_duration: u64,
    confused_toggle: bool,

    laugh: bool,
    laugh_timer: u64,
    laugh_duration: u64,
    laugh_toggle: bool,

    sweat: bool,
    sweat_radius: u32,
    sweat_drops: [SweatDrop; 3],
}

#[derive(Clone, Copy)]
struct SweatDrop {
    x_initial: i32,
    x: i32,
    y: f32,
    y_max: i32,
    w: f32,
    h: f32,
}

impl Default for SweatDrop {
    fn default() -> Self {
        Self { x_initial: 0, x: 0, y: 4.0, y_max: 100, w: 2.0, h: 4.0 }
    }
}

impl RoboEyes {
    pub fn new(width: i32, height: i32) -> Self {
        let eye_w = 50;
        let eye_h = 100;
        let space = 20;
        let screen_w = width;
        let screen_h = height;

        let el_x = (screen_w - (eye_w + space + eye_w)) / 2;
        let el_y = (screen_h - eye_h) / 2;
        let er_x = el_x + eye_w + space;

        Self {
            screen_size: IVec2::new(width, height),
            bg_color: rgb8_hex(0x000000),
            main_color: rgb8_hex(0xFFDA02),
            
            last_frame_time: 0,
            frame_interval: 20,
            rng_seed: 12345,

            tired: false, angry: false, happy: false, curious: false, cyclops: false,
            eye_l_open: true, eye_r_open: true,

            eye_l_width_default: eye_w, eye_l_height_default: eye_h,
            eye_l_width_current: eye_w, eye_l_height_current: 1,
            eye_l_width_next: eye_w, eye_l_height_next: eye_h,
            eye_l_border_radius_current: 16, eye_l_border_radius_next: 8,

            eye_r_width_default: eye_w, eye_r_height_default: eye_h,
            eye_r_width_current: eye_w, eye_r_height_current: 1,
            eye_r_width_next: eye_w, eye_r_height_next: eye_h,
            eye_r_border_radius_current: 16, eye_r_border_radius_next: 8,

            eye_l_pos: IVec2::new(el_x, el_y),
            eye_l_pos_next: IVec2::new(el_x, el_y),
            eye_r_pos: IVec2::new(er_x, el_y),
            eye_r_pos_next: IVec2::new(er_x, el_y),

            eyelids_tired_height: 0, eyelids_tired_height_next: 0,
            eyelids_angry_height: 0, eyelids_angry_height_next: 0,
            eyelids_happy_offset: 0, eyelids_happy_offset_next: 0,
            space_between_current: space, space_between_next: space,

            h_flicker: false, h_flicker_amp: 2, h_flicker_alt: false,
            v_flicker: false, v_flicker_amp: 10, v_flicker_alt: false,

            autoblinker: true, blink_interval: 2, blink_variation: 4, blink_timer_ms: 0,
            idle: false, idle_interval: 3, idle_variation: 2, idle_timer_ms: 0,
            confused: false, confused_timer: 0, confused_duration: 500, confused_toggle: true,
            laugh: false, laugh_timer: 0, laugh_duration: 500, laugh_toggle: true,
            
            sweat: false, sweat_radius: 3,
            sweat_drops: [SweatDrop::default(); 3],
        }
    }

    fn random(&mut self, max: i32) -> i32 {
        if max <= 0 { return 0; }
        self.rng_seed = self.rng_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (self.rng_seed as i32).abs() % max
    }

    fn get_screen_constraint_x(&self) -> i32 {
        self.screen_size.x - self.eye_l_width_current - self.space_between_current - self.eye_r_width_current
    }

    fn get_screen_constraint_y(&self) -> i32 {
        self.screen_size.y - self.eye_l_height_default
    }

    pub fn set_mood(&mut self, mood: RoboMood) {
        self.tired = false; self.angry = false; self.happy = false;
        match mood {
            RoboMood::Tired => self.tired = true,
            RoboMood::Angry => self.angry = true,
            RoboMood::Happy => self.happy = true,
            _ => {}
        }
    }

    pub fn set_position(&mut self, pos: RoboPosition) {
        let max_x = self.get_screen_constraint_x();
        let max_y = self.get_screen_constraint_y();
        
        match pos {
            RoboPosition::N =>  { self.eye_l_pos_next = IVec2::new(max_x/2, 0); },
            RoboPosition::NE => { self.eye_l_pos_next = IVec2::new(max_x, 0); },
            RoboPosition::E =>  { self.eye_l_pos_next = IVec2::new(max_x, max_y/2); },
            RoboPosition::SE => { self.eye_l_pos_next = IVec2::new(max_x, max_y); },
            RoboPosition::S =>  { self.eye_l_pos_next = IVec2::new(max_x/2, max_y); },
            RoboPosition::SW => { self.eye_l_pos_next = IVec2::new(0, max_y); },
            RoboPosition::W =>  { self.eye_l_pos_next = IVec2::new(0, max_y/2); },
            RoboPosition::NW => { self.eye_l_pos_next = IVec2::new(0, 0); },
            RoboPosition::Center => { self.eye_l_pos_next = IVec2::new(max_x/2, max_y/2); },
        }
    }

    pub fn set_autoblinker(&mut self, active: bool, interval: u64, variation: u64) {
        self.autoblinker = active;
        self.blink_interval = interval;
        self.blink_variation = variation;
    }

    pub fn set_idle_mode(&mut self, active: bool, interval: u64, variation: u64) {
        self.idle = active;
        self.idle_interval = interval;
        self.idle_variation = variation;
    }

    pub fn anim_confused(&mut self) {
        self.confused = true;
        self.confused_toggle = true;
    }

    pub fn anim_laugh(&mut self) {
        self.laugh = true;
        self.laugh_toggle = true;
    }

    pub fn blink(&mut self) {
        self.eye_l_height_next = 1;
        self.eye_r_height_next = 1;
        self.eye_l_open = false;
        self.eye_r_open = false;
        self.eye_l_open = true;
        self.eye_r_open = true;
    }

    pub fn update(&mut self, platform: &mut dyn IcPlatform) {
        let now = platform.millis();
        
        // Frame limiting
        if now - self.last_frame_time < self.frame_interval {
            return;
        }
        self.last_frame_time = now;

        let mut eye_l_h_offset = 0;
        let mut eye_r_h_offset = 0;
        
        if self.curious {
            if self.eye_l_pos_next.x <= 30 { eye_l_h_offset = 24; }
            else if self.eye_l_pos_next.x >= (self.get_screen_constraint_x() - 30) && self.cyclops { eye_l_h_offset = 8; }
            
            if self.eye_r_pos_next.x >= self.screen_size.x - self.eye_r_width_current - 30 { eye_r_h_offset = 24; }
        }

        self.eye_l_height_current = (self.eye_l_height_current + self.eye_l_height_next + eye_l_h_offset) / 2;
        self.eye_l_pos.y += (self.eye_l_height_default - self.eye_l_height_current) / 2;
        self.eye_l_pos.y -= eye_l_h_offset / 2;

        self.eye_r_height_current = (self.eye_r_height_current + self.eye_r_height_next + eye_r_h_offset) / 2;
        self.eye_r_pos.y += (self.eye_r_height_default - self.eye_r_height_current) / 2;
        self.eye_r_pos.y -= eye_r_h_offset / 2;

        if self.eye_l_open && self.eye_l_height_current <= 1 + eye_l_h_offset {
            self.eye_l_height_next = self.eye_l_height_default;
        }
        if self.eye_r_open && self.eye_r_height_current <= 1 + eye_r_h_offset {
            self.eye_r_height_next = self.eye_r_height_default;
        }

        self.eye_l_width_current = (self.eye_l_width_current + self.eye_l_width_next) / 2;
        self.eye_r_width_current = (self.eye_r_width_current + self.eye_r_width_next) / 2;
        self.eye_l_border_radius_current = (self.eye_l_border_radius_current + self.eye_l_border_radius_next) / 2;
        self.eye_r_border_radius_current = (self.eye_r_border_radius_current + self.eye_r_border_radius_next) / 2;

        self.space_between_current = (self.space_between_current + self.space_between_next) / 2;
        self.eye_l_pos.x = (self.eye_l_pos.x + self.eye_l_pos_next.x) / 2;
        self.eye_l_pos.y = (self.eye_l_pos.y + self.eye_l_pos_next.y) / 2;
        
        self.eye_r_pos_next.x = self.eye_l_pos_next.x + self.eye_l_width_current + self.space_between_current;
        self.eye_r_pos_next.y = self.eye_l_pos_next.y;
        
        self.eye_r_pos.x = (self.eye_r_pos.x + self.eye_r_pos_next.x) / 2;
        self.eye_r_pos.y = (self.eye_r_pos.y + self.eye_r_pos_next.y) / 2;

        if self.autoblinker && now >= self.blink_timer_ms {
            self.blink();
            let var_ms = self.random((self.blink_variation as i32) * 1000);
            self.blink_timer_ms = now + (self.blink_interval * 1000) + var_ms as u64;
        }

        if self.laugh {
            if self.laugh_toggle {
                self.v_flicker = true;
                self.v_flicker_amp = 5;
                self.laugh_timer = now;
                self.laugh_toggle = false;
            } else if now >= self.laugh_timer + self.laugh_duration {
                self.v_flicker = false;
                self.laugh = false;
                self.laugh_toggle = true;
            }
        }

        if self.confused {
            if self.confused_toggle {
                self.h_flicker = true;
                self.h_flicker_amp = 20;
                self.confused_timer = now;
                self.confused_toggle = false;
            } else if now >= self.confused_timer + self.confused_duration {
                self.h_flicker = false;
                self.confused = false;
                self.confused_toggle = true;
            }
        }

        if self.idle && now >= self.idle_timer_ms {
            let max_x = self.get_screen_constraint_x();
            let max_y = self.get_screen_constraint_y();
            self.eye_l_pos_next.x = self.random(max_x);
            self.eye_l_pos_next.y = self.random(max_y);
            
            let var_ms = self.random((self.idle_variation as i32) * 1000);
            self.idle_timer_ms = now + (self.idle_interval * 1000) + var_ms as u64;
        }

        if self.h_flicker {
            if self.h_flicker_alt {
                self.eye_l_pos.x += self.h_flicker_amp;
                self.eye_r_pos.x += self.h_flicker_amp;
            } else {
                self.eye_l_pos.x -= self.h_flicker_amp;
                self.eye_r_pos.x -= self.h_flicker_amp;
            }
            self.h_flicker_alt = !self.h_flicker_alt;
        }

        if self.v_flicker {
            if self.v_flicker_alt {
                self.eye_l_pos.y += self.v_flicker_amp;
                self.eye_r_pos.y += self.v_flicker_amp;
            } else {
                self.eye_l_pos.y -= self.v_flicker_amp;
                self.eye_r_pos.y -= self.v_flicker_amp;
            }
            self.v_flicker_alt = !self.v_flicker_alt;
        }

        if self.cyclops {
            self.eye_r_width_current = 0;
            self.eye_r_height_current = 0;
            self.space_between_current = 0;
        }

        // drawing ------------------------------------------------
        platform.clear(self.bg_color);

        platform.draw_rectangle_rounded(
            self.eye_l_pos,
            self.eye_l_pos + IVec2::new(self.eye_l_width_current, self.eye_l_height_current),
            self.main_color,
            0,
            Some(self.main_color),
            self.eye_l_border_radius_current as u32,
        );

        if !self.cyclops {
            platform.draw_rectangle_rounded(
                self.eye_r_pos,
                self.eye_r_pos + IVec2::new(self.eye_r_width_current, self.eye_r_height_current),
                self.main_color,
                0,
                Some(self.main_color),
                self.eye_r_border_radius_current as u32,
            );
        }

        self.eyelids_tired_height_next = if self.tired { self.eye_l_height_current / 2 } else { 0 };
        self.eyelids_tired_height = (self.eyelids_tired_height + self.eyelids_tired_height_next) / 2;
        
        if self.eyelids_tired_height > 0 {
            let h = self.eyelids_tired_height;
            platform.draw_triangle(
                IVec2::new(self.eye_l_pos.x, self.eye_l_pos.y - 1),
                IVec2::new(self.eye_l_pos.x + self.eye_l_width_current, self.eye_l_pos.y - 1),
                IVec2::new(self.eye_l_pos.x, self.eye_l_pos.y + h - 1),
                self.bg_color, 0, Some(self.bg_color)
            );
            if !self.cyclops {
                platform.draw_triangle(
                    IVec2::new(self.eye_r_pos.x, self.eye_r_pos.y - 1),
                    IVec2::new(self.eye_r_pos.x + self.eye_r_width_current, self.eye_r_pos.y - 1),
                    IVec2::new(self.eye_r_pos.x + self.eye_r_width_current, self.eye_r_pos.y + h - 1),
                    self.bg_color, 0, Some(self.bg_color)
                );
            }
        }

        self.eyelids_angry_height_next = if self.angry { self.eye_l_height_current / 2 } else { 0 };
        self.eyelids_angry_height = (self.eyelids_angry_height + self.eyelids_angry_height_next) / 2;

        if self.eyelids_angry_height > 0 {
            let h = self.eyelids_angry_height;
            platform.draw_triangle(
                IVec2::new(self.eye_l_pos.x, self.eye_l_pos.y - 1),
                IVec2::new(self.eye_l_pos.x + self.eye_l_width_current, self.eye_l_pos.y - 1),
                IVec2::new(self.eye_l_pos.x + self.eye_l_width_current, self.eye_l_pos.y + h - 1),
                self.bg_color, 0, Some(self.bg_color)
            );
            if !self.cyclops {
                platform.draw_triangle(
                    IVec2::new(self.eye_r_pos.x, self.eye_r_pos.y - 1),
                    IVec2::new(self.eye_r_pos.x + self.eye_r_width_current, self.eye_r_pos.y - 1),
                    IVec2::new(self.eye_r_pos.x, self.eye_r_pos.y + h - 1),
                    self.bg_color, 0, Some(self.bg_color)
                );
            }
        }

        self.eyelids_happy_offset_next = if self.happy { self.eye_l_height_current / 2 } else { 0 };
        self.eyelids_happy_offset = (self.eyelids_happy_offset + self.eyelids_happy_offset_next) / 2;

        if self.eyelids_happy_offset > 0 {
            let off = self.eyelids_happy_offset;
            platform.draw_rectangle_rounded(
                IVec2::new(self.eye_l_pos.x - 1, (self.eye_l_pos.y + self.eye_l_height_current) - off + 1),
                IVec2::new(self.eye_l_pos.x - 1 + self.eye_l_width_current + 2, 
                           (self.eye_l_pos.y + self.eye_l_height_current) - off + 1 + self.eye_l_height_default),
                self.bg_color, 0, Some(self.bg_color), self.eye_l_border_radius_current as u32
            );

            if !self.cyclops {
                platform.draw_rectangle_rounded(
                    IVec2::new(self.eye_r_pos.x - 1, (self.eye_r_pos.y + self.eye_r_height_current) - off + 1),
                    IVec2::new(self.eye_r_pos.x - 1 + self.eye_r_width_current + 2, 
                               (self.eye_r_pos.y + self.eye_r_height_current) - off + 1 + self.eye_r_height_default),
                    self.bg_color, 0, Some(self.bg_color), self.eye_r_border_radius_current as u32
                );
            }
        }

        if self.sweat {
            if self.sweat_drops[0].y <= self.sweat_drops[0].y_max as f32 {
                self.sweat_drops[0].y += 0.5;
            } else {
                self.sweat_drops[0].x_initial = self.random(30);
                self.sweat_drops[0].y = 2.0;
                self.sweat_drops[0].y_max = self.random(10) + 10;
                self.sweat_drops[0].w = 1.0;
                self.sweat_drops[0].h = 2.0;
            }
            if self.sweat_drops[0].y <= (self.sweat_drops[0].y_max / 2) as f32 {
                self.sweat_drops[0].w += 0.5;
                self.sweat_drops[0].h += 0.5;
            } else {
                self.sweat_drops[0].w -= 0.1;
                self.sweat_drops[0].h -= 0.5;
            }
            self.sweat_drops[0].x = self.sweat_drops[0].x_initial - (self.sweat_drops[0].w / 2.0) as i32;
            
            platform.draw_rectangle_rounded(
                IVec2::new(self.sweat_drops[0].x, self.sweat_drops[0].y as i32),
                IVec2::new(self.sweat_drops[0].x + self.sweat_drops[0].w as i32, self.sweat_drops[0].y as i32 + self.sweat_drops[0].h as i32),
                self.main_color, 0, Some(self.main_color), self.sweat_radius
            );
        }
    }
}

pub struct FaceCalculator {
    eyes: RoboEyes,
}

impl FaceCalculator {
    pub fn new() -> Self {
        let mut eyes = RoboEyes::new(320, 240);
        eyes.set_autoblinker(true, 2, 3);
        eyes.set_idle_mode(true, 3, 2);
        Self { eyes }
    }
}

impl IcApp for FaceCalculator {
    fn name(&self) -> &str {
        "Face"
    }

    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: IcKey, _ctx: &InputContext) {
        match key {
            IcKey::Num1 => self.eyes.set_mood(RoboMood::Default),
            IcKey::Num2 => self.eyes.set_mood(RoboMood::Tired),
            IcKey::Num3 => self.eyes.set_mood(RoboMood::Angry),
            IcKey::Num4 => self.eyes.set_mood(RoboMood::Happy),
            
            IcKey::Num5 => self.eyes.anim_confused(),
            IcKey::Num6 => self.eyes.anim_laugh(),

            IcKey::Num7 => self.eyes.sweat = true,
            IcKey::Num8 => self.eyes.sweat = false,
            
            IcKey::Func1 => self.eyes.set_position(RoboPosition::NW),
            IcKey::Func2 => self.eyes.set_position(RoboPosition::N),
            IcKey::Func3 => self.eyes.set_position(RoboPosition::NE),
            IcKey::Func5 => self.eyes.set_position(RoboPosition::Center),
            
            _ => {}
        }
    }

    fn update(&mut self, platform: &mut dyn IcPlatform, _ctx: &InputContext) {
        self.eyes.update(platform);
    }
}