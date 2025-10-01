use std::{collections::HashMap};

use raylib::prelude::*;

use incredicalculator_core::{IcKey, IcPlatform, IcState};

struct Line {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32
}

pub struct IcRaylibPlatform {
    line_list: Vec<Line>
}

impl IcRaylibPlatform {
    pub fn new() -> IcRaylibPlatform {
        IcRaylibPlatform { line_list: Vec::<Line>::new() }
    }
}

impl IcPlatform for IcRaylibPlatform {
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2:f32) {
        self.line_list.push(Line { x1: x1, y1: y1, x2: x2, y2: y2 });
    }

    fn clear_lines(&mut self) {
        self.line_list.clear();
    }
}

fn main() {
    let mut icalc: IcState = IcState::new();
    let mut ic_rl_platform: IcRaylibPlatform = IcRaylibPlatform::new();
    let key_map: HashMap<KeyboardKey, IcKey> = { 
        let mut m: HashMap<KeyboardKey, IcKey> = HashMap::new();
        m.insert(KeyboardKey::KEY_Z, IcKey::NumE);
        m.insert(KeyboardKey::KEY_X, IcKey::NumF);
        m.insert(KeyboardKey::KEY_C, IcKey::Num0);
        m.insert(KeyboardKey::KEY_V, IcKey::Func1);
        m.insert(KeyboardKey::KEY_A, IcKey::NumD);
        m.insert(KeyboardKey::KEY_S, IcKey::Num1);
        m.insert(KeyboardKey::KEY_D, IcKey::Num2);
        m.insert(KeyboardKey::KEY_F, IcKey::Num3);
        m.insert(KeyboardKey::KEY_Q, IcKey::NumC);
        m.insert(KeyboardKey::KEY_W, IcKey::Num4);
        m.insert(KeyboardKey::KEY_E, IcKey::Num5);
        m.insert(KeyboardKey::KEY_R, IcKey::Num6);
        m.insert(KeyboardKey::KEY_ONE, IcKey::NumB);
        m.insert(KeyboardKey::KEY_TWO, IcKey::Num7);
        m.insert(KeyboardKey::KEY_THREE, IcKey::Num8);
        m.insert(KeyboardKey::KEY_FOUR, IcKey::Num9);
        m.insert(KeyboardKey::KEY_FIVE, IcKey::NumA);
        m.insert(KeyboardKey::KEY_SIX, IcKey::Func2);
        m.insert(KeyboardKey::KEY_SEVEN, IcKey::Func3);
        m.insert(KeyboardKey::KEY_EIGHT, IcKey::Func4);
        m
    };
    
    println!("Hello, world!");
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480).title("huh").vsync().build();
    rl_handle.set_target_fps(30);
    
    while !rl_handle.window_should_close() {
        while let Some(rl_key) = rl_handle.get_key_pressed() {
            if let Some(ic_key) = key_map.get(&rl_key) {
                icalc.key_press(*ic_key);
            }
        }
        icalc.update(&mut ic_rl_platform);

        let fps: u32 = rl_handle.get_fps();
        let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
        rl_draw_handle.clear_background(Color::GREEN);
        rl_draw_handle.draw_text(format!("What! {fps} FPS").as_str(),
             12, 12, 24, Color::WHITE);
        for l in ic_rl_platform.line_list.iter() {
            rl_draw_handle.draw_line_ex(
                Vector2::new(l.x1, l.y1), 
                Vector2::new(l.x2, l.y2),
                5.0, Color::WHITE);
        }
    }
}
