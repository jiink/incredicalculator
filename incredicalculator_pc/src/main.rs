use std::{collections::HashMap};

use raylib::{ffi::{SetTextureFilter, RL_TEXTURE_FILTER_LINEAR}, prelude::*};

use incredicalculator_core::{IcKey, IcPlatform, IcState};

struct VirtualKey {
    key: IcKey,
    x: u32,
    y: u32,
    pressed: bool,
    hovered: bool
}

struct Line {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32
}

// const RENDER_W: u32 = 320;
// const RENDER_H: u32 = 240;
const RENDER_W: u32 = 160;
const RENDER_H: u32 = 120;

fn world_to_px(x: f32, y: f32) -> (f32, f32) {
    (x * RENDER_W as f32, y * RENDER_W as f32)
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
        m.insert(KeyboardKey::KEY_SEVEN, IcKey::Shift);
        m.insert(KeyboardKey::KEY_EIGHT, IcKey::Super);
        m
    };
    let mut virtual_keys = [
        VirtualKey { key: IcKey::Super, x: 16 + 70 * 0, y: 147 + 70 * 0, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Func1, x: 16 + 70 * 1, y: 147 + 70 * 0, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Func2, x: 16 + 70 * 2, y: 147 + 70 * 0, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumA,  x: 16 + 70 * 3, y: 147 + 70 * 0, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num7,  x: 16 + 70 * 0, y: 147 + 70 * 1, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num8,  x: 16 + 70 * 1, y: 147 + 70 * 1, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num9,  x: 16 + 70 * 2, y: 147 + 70 * 1, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumB,  x: 16 + 70 * 3, y: 147 + 70 * 1, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num4,  x: 16 + 70 * 0, y: 147 + 70 * 2, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num5,  x: 16 + 70 * 1, y: 147 + 70 * 2, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num6,  x: 16 + 70 * 2, y: 147 + 70 * 2, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumC,  x: 16 + 70 * 3, y: 147 + 70 * 2, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num1,  x: 16 + 70 * 0, y: 147 + 70 * 3, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num2,  x: 16 + 70 * 1, y: 147 + 70 * 3, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num3,  x: 16 + 70 * 2, y: 147 + 70 * 3, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumD,  x: 16 + 70 * 3, y: 147 + 70 * 3, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Shift, x: 16 + 70 * 0, y: 147 + 70 * 4, pressed: false, hovered: false },
        VirtualKey { key: IcKey::Num0,  x: 16 + 70 * 1, y: 147 + 70 * 4, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumF,  x: 16 + 70 * 2, y: 147 + 70 * 4, pressed: false, hovered: false },
        VirtualKey { key: IcKey::NumE,  x: 16 + 70 * 3, y: 147 + 70 * 4, pressed: false, hovered: false },
    ];
    
    println!("Hello, world!");
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(300, 500).title("Incredicalculator PC").vsync().build();
    rl_handle.set_target_fps(30);
    let mut target_tex = match rl_handle.load_render_texture(&rl_thread, RENDER_W, RENDER_H) {
        Ok(tex) => tex,
        Err(e) => {
            eprintln!("Render texture fail: {}", e);
            return;
        }
    };
    unsafe {
        SetTextureFilter(target_tex.texture, RL_TEXTURE_FILTER_LINEAR as i32);
    }    
    while !rl_handle.window_should_close() {
        while let Some(rl_key) = rl_handle.get_key_pressed() {
            if let Some(ic_key) = key_map.get(&rl_key) {
                icalc.key_down(*ic_key);
            }
        }
        for rl_key in key_map.keys() {
            if rl_handle.is_key_released(*rl_key) {
                if let Some(ic_key) = key_map.get(&rl_key) {
                    icalc.key_up(*ic_key);
                }
            }
        }
        let virtual_key_size: i32 = 64;
        let mouse_pos = rl_handle.get_mouse_position();
        let mouse_down = rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        for vk in virtual_keys.iter_mut() {
            let key_rect = Rectangle::new(vk.x as f32, vk.y as f32, virtual_key_size as f32, virtual_key_size as f32);
            vk.hovered = key_rect.check_collision_point_rec(mouse_pos);
            if vk.hovered && mouse_down && !vk.pressed {
                vk.pressed = true;
                icalc.key_down(vk.key);
            } else if vk.pressed && !mouse_down {
                vk.pressed = false;
                icalc.key_up(vk.key);
            }
        }

        icalc.update(&mut ic_rl_platform);

        let fps: u32 = rl_handle.get_fps();

        {
            let mut d_tex = rl_handle.begin_texture_mode(&rl_thread, &mut target_tex);
            d_tex.clear_background(Color::GREEN);
            d_tex.clear_background(Color::GREEN);
            //d_tex.draw_text(format!("What! {fps} FPS").as_str(),
                //12, 12, 24, Color::WHITE);
            for l in ic_rl_platform.line_list.iter() {
                let (x1, y1) = world_to_px(l.x1, l.y1);
                let (x2, y2) = world_to_px(l.x2, l.y2);
                d_tex.draw_line_ex(
                    Vector2::new(x1, y1), 
                    Vector2::new(x2, y2),
                    1.0, Color::WHITE);
            }
        }

        let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
        rl_draw_handle.clear_background(Color::GRAY);
        for vk in virtual_keys.iter() {
            let c = if vk.pressed {
                Color::BLUE
            } else if vk.hovered {
                Color::WHITESMOKE
            } else {
                Color::LIGHTGRAY
            };
            rl_draw_handle.draw_rectangle(vk.x as i32, vk.y as i32, virtual_key_size, virtual_key_size, c);
            let lbl = match vk.key {
                IcKey::Num0 => "0",
                IcKey::Num1 => "1",
                IcKey::Num2 => "2",
                IcKey::Num3 => "3",
                IcKey::Num4 => "4",
                IcKey::Num5 => "5",
                IcKey::Num6 => "6",
                IcKey::Num7 => "7",
                IcKey::Num8 => "8",
                IcKey::Num9 => "9",
                IcKey::NumA => "A",
                IcKey::NumB => "B",
                IcKey::NumC => "C",
                IcKey::NumD => "D",
                IcKey::NumE => "E",
                IcKey::NumF => "F",
                IcKey::Shift => "^",
                IcKey::Super => "§",
                IcKey::Func1 => "F1",
                IcKey::Func2 => "F2",
                _ => "?"
            };
            rl_draw_handle.draw_text(&lbl, vk.x as i32 + 16, vk.y as i32 + 16, 20, Color::BLACK);
        }
        let source_rec = Rectangle::new(0.0, 0.0, target_tex.texture.width as f32, -target_tex.texture.height as f32);
        let dest_rec = Rectangle::new(72.0, 17.0, 160.0, 120.0);
        let origin = Vector2::new(0.0, 0.0);
        rl_draw_handle.draw_texture_pro(&target_tex, source_rec, dest_rec, origin, 0.0, Color::WHITE);
    }
}
