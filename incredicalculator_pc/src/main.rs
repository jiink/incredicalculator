use ::core::fmt;
use std::collections::HashMap;
use std::time::Instant;

use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb565,
    prelude::{Primitive, RgbColor},
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder},
};
use embedded_graphics_framebuf::FrameBuf;
use raylib::{
    ffi::{SetTextureFilter, RL_TEXTURE_FILTER_LINEAR},
    prelude::*,
};

use incredicalculator_core::input::IcKey;
use incredicalculator_core::platform::IcPlatform;
use incredicalculator_core::shell::IcShell;
use glam::IVec2;

// Rodio: the only audio dependency now.
// In Cargo.toml add: rodio = { version = "0.21", features = ["playback"] }
use rodio::source::{SineWave, Source};

// ---------------------------------------------------------------------------
// Virtual keyboard
// ---------------------------------------------------------------------------

struct VirtualKey {
    key: IcKey,
    x: u32,
    y: u32,
    pressed: bool,
    hovered: bool,
    label: &'static str,
    shlabel: &'static str,
    sulabel: &'static str,
    sticky: bool,
}

// ---------------------------------------------------------------------------
// Render-target dimensions
// ---------------------------------------------------------------------------

const RENDER_W: u32 = 320;
const RENDER_H: u32 = 240;

// ---------------------------------------------------------------------------
// Platform implementation
// ---------------------------------------------------------------------------

pub struct IcRaylibPlatform {
    pub canvas_data: [Rgb565; (RENDER_W * RENDER_H) as usize],
    start_time: Instant,
}

impl IcRaylibPlatform {
    pub fn new() -> Self {
        Self {
            canvas_data: [Rgb565::BLACK; (RENDER_W * RENDER_H) as usize],
            start_time: Instant::now(),
        }
    }
}

impl IcPlatform for IcRaylibPlatform {
    fn clear(&mut self, color: rgb::RGB8) {
        self.canvas_data.fill(rgbu8_to_rgb565(color));
    }

    fn draw_line(&mut self, start: IVec2, end: IVec2, color: rgb::RGB8, width: u32) {
        let mut fbuf = FrameBuf::new(&mut self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        embedded_graphics::primitives::Line::new(
            embedded_graphics::prelude::Point::new(start.x, start.y),
            embedded_graphics::prelude::Point::new(end.x, end.y),
        )
        .into_styled(PrimitiveStyle::with_stroke(rgbu8_to_rgb565(color), width))
        .draw(&mut fbuf)
        .unwrap();
    }

    fn log(&mut self, arg: fmt::Arguments) {
        println!("{}", arg);
    }

    fn draw_rectangle(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: rgb::RGB8,
        stroke_width: u32,
        fill_color: Option<rgb::RGB8>,
    ) {
        let mut fbuf = FrameBuf::new(&mut self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        embedded_graphics::primitives::Rectangle::with_corners(
            embedded_graphics::prelude::Point::new(start.x, start.y),
            embedded_graphics::prelude::Point::new(end.x, end.y),
        )
        .into_styled(style_builder.build())
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_rectangle_rounded(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: rgb::RGB8,
        stroke_width: u32,
        fill_color: Option<rgb::RGB8>,
        corner_radius: u32,
    ) {
        let mut fbuf = FrameBuf::new(&mut self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        embedded_graphics::primitives::RoundedRectangle::with_equal_corners(
            embedded_graphics::primitives::Rectangle::with_corners(
                embedded_graphics::prelude::Point::new(start.x, start.y),
                embedded_graphics::prelude::Point::new(end.x, end.y),
            ),
            embedded_graphics::prelude::Size::new(corner_radius, corner_radius),
        )
        .into_styled(style_builder.build())
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_triangle(
        &mut self,
        vertex1: IVec2,
        vertex2: IVec2,
        vertex3: IVec2,
        stroke_color: rgb::RGB8,
        stroke_width: u32,
        fill_color: Option<rgb::RGB8>,
    ) {
        let mut fbuf = FrameBuf::new(&mut self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let mut style_builder = PrimitiveStyleBuilder::new()
            .stroke_color(rgbu8_to_rgb565(stroke_color))
            .stroke_width(stroke_width)
            .stroke_alignment(embedded_graphics::primitives::StrokeAlignment::Center);
        if let Some(c) = fill_color {
            style_builder = style_builder.fill_color(rgbu8_to_rgb565(c));
        }
        embedded_graphics::primitives::Triangle::new(
            embedded_graphics::prelude::Point::new(vertex1.x, vertex1.y),
            embedded_graphics::prelude::Point::new(vertex2.x, vertex2.y),
            embedded_graphics::prelude::Point::new(vertex3.x, vertex3.y),
        )
        .into_styled(style_builder.build())
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_string(&mut self, text: &str, pos: IVec2, _size: u32, color: rgb::RGB8) {
        let mut fbuf = FrameBuf::new(&mut self.canvas_data, RENDER_W as usize, RENDER_H as usize);
        let char_style = embedded_graphics::mono_font::MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            rgbu8_to_rgb565(color),
        );
        let text_style = embedded_graphics::text::TextStyleBuilder::new()
            .alignment(embedded_graphics::text::Alignment::Left)
            .baseline(embedded_graphics::text::Baseline::Top)
            .build();
        embedded_graphics::text::Text::with_text_style(
            text,
            embedded_graphics::prelude::Point::new(pos.x, pos.y),
            char_style,
            text_style,
        )
        .draw(&mut fbuf)
        .unwrap();
    }

    fn draw_string_f(&mut self, arg: fmt::Arguments, pos: IVec2, size: u32, color: rgb::RGB8) {
        let mut buf = [0u8; 128];
        self.draw_string(format_no_std::show(&mut buf, arg).unwrap(), pos, size, color);
    }

    fn millis(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    fn get_battery_soc(&self) -> i32 {
        77
    }
}

// ---------------------------------------------------------------------------
// Colour helpers
// ---------------------------------------------------------------------------

fn rgb565_to_rl_color(c: Rgb565) -> Color {
    Color { r: c.r() << 3, g: c.g() << 2, b: c.b() << 3, a: 255 }
}

fn rgbu8_to_rgb565(c: rgb::Rgb<u8>) -> Rgb565 {
    Rgb565::new(c.r >> 3, c.g >> 2, c.b >> 3)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let mut icalc = IcShell::new();
    let mut ic_rl_platform = Box::new(IcRaylibPlatform::new());

    // -----------------------------------------------------------------------
    // AUDIO — rodio sine wave test
    //
    // `audio_handle` owns the OS audio device connection; dropping it stops
    // playback.  `audio_player` is the sequenced play queue connected to that
    // device.  Both must live until the end of main.
    // -----------------------------------------------------------------------
    let audio_handle = match rodio::DeviceSinkBuilder::open_default_sink() {
        Ok(h) => {
            println!("Audio: device opened OK");
            Some(h)
        }
        Err(e) => {
            eprintln!("Audio: could not open device ({e}); continuing without audio");
            None
        }
    };

    // `audio_player` is kept alive in an Option for the same reason.
    let audio_player = audio_handle.as_ref().map(|h| {
        let player = rodio::Player::connect_new(&h.mixer());
        // SineWave is an infinite iterator, so this plays forever until the
        // player (or handle) is dropped.
        player.append(SineWave::new(440.0).amplify(0.1));
        println!("Audio: 440 Hz sine wave started via rodio");
        player
    });

    // -----------------------------------------------------------------------
    // Keyboard mapping
    // -----------------------------------------------------------------------
    let key_map: HashMap<KeyboardKey, IcKey> = {
        let mut m = HashMap::new();
        m.insert(KeyboardKey::KEY_E, IcKey::Func6);
        m.insert(KeyboardKey::KEY_F, IcKey::Func5);
        m.insert(KeyboardKey::KEY_ZERO, IcKey::Num0);
        m.insert(KeyboardKey::KEY_D, IcKey::Func4);
        m.insert(KeyboardKey::KEY_ONE, IcKey::Num1);
        m.insert(KeyboardKey::KEY_TWO, IcKey::Num2);
        m.insert(KeyboardKey::KEY_THREE, IcKey::Num3);
        m.insert(KeyboardKey::KEY_C, IcKey::Func3);
        m.insert(KeyboardKey::KEY_FOUR, IcKey::Num4);
        m.insert(KeyboardKey::KEY_FIVE, IcKey::Num5);
        m.insert(KeyboardKey::KEY_SIX, IcKey::Num6);
        m.insert(KeyboardKey::KEY_B, IcKey::Func2);
        m.insert(KeyboardKey::KEY_SEVEN, IcKey::Num7);
        m.insert(KeyboardKey::KEY_EIGHT, IcKey::Num8);
        m.insert(KeyboardKey::KEY_NINE, IcKey::Num9);
        m.insert(KeyboardKey::KEY_A, IcKey::Func1);
        m.insert(KeyboardKey::KEY_LEFT_SHIFT, IcKey::Shift);
        m.insert(KeyboardKey::KEY_Z, IcKey::Super);
        m
    };

    // -----------------------------------------------------------------------
    // Virtual on-screen keyboard
    // -----------------------------------------------------------------------
    let mut virtual_keys = [
        VirtualKey { key: IcKey::Func1, x: 7 + 69 * 3, y: 9 + 69 * 0, pressed: false, hovered: false, label: "Bk", shlabel: "&",   sulabel: "F1", sticky: false },
        VirtualKey { key: IcKey::Func2, x: 7 + 69 * 3, y: 9 + 69 * 1, pressed: false, hovered: false, label: "/", shlabel: "|",   sulabel: "F2", sticky: false },
        VirtualKey { key: IcKey::Num7,  x: 7 + 69 * 0, y: 9 + 69 * 2, pressed: false, hovered: false, label: "7", shlabel: "(",   sulabel: "Hm", sticky: false },
        VirtualKey { key: IcKey::Num8,  x: 7 + 69 * 1, y: 9 + 69 * 2, pressed: false, hovered: false, label: "8", shlabel: ")",   sulabel: "^",  sticky: false },
        VirtualKey { key: IcKey::Num9,  x: 7 + 69 * 2, y: 9 + 69 * 2, pressed: false, hovered: false, label: "9", shlabel: "0x",  sulabel: "Clr",sticky: false },
        VirtualKey { key: IcKey::Func3, x: 7 + 69 * 3, y: 9 + 69 * 2, pressed: false, hovered: false, label: "*", shlabel: "%",   sulabel: "F3", sticky: false },
        VirtualKey { key: IcKey::Num4,  x: 7 + 69 * 0, y: 9 + 69 * 3, pressed: false, hovered: false, label: "4", shlabel: "E",   sulabel: "<",  sticky: false },
        VirtualKey { key: IcKey::Num5,  x: 7 + 69 * 1, y: 9 + 69 * 3, pressed: false, hovered: false, label: "5", shlabel: "F",   sulabel: "Sel",sticky: false },
        VirtualKey { key: IcKey::Num6,  x: 7 + 69 * 2, y: 9 + 69 * 3, pressed: false, hovered: false, label: "6", shlabel: ".",   sulabel: ">",  sticky: false },
        VirtualKey { key: IcKey::Func4, x: 7 + 69 * 3, y: 9 + 69 * 3, pressed: false, hovered: false, label: "-", shlabel: "<<",  sulabel: "F4", sticky: false },
        VirtualKey { key: IcKey::Num1,  x: 7 + 69 * 0, y: 9 + 69 * 4, pressed: false, hovered: false, label: "1", shlabel: "B",   sulabel: "End",sticky: false },
        VirtualKey { key: IcKey::Num2,  x: 7 + 69 * 1, y: 9 + 69 * 4, pressed: false, hovered: false, label: "2", shlabel: "C",   sulabel: "v",  sticky: false },
        VirtualKey { key: IcKey::Num3,  x: 7 + 69 * 2, y: 9 + 69 * 4, pressed: false, hovered: false, label: "3", shlabel: "D",   sulabel: "",   sticky: false },
        VirtualKey { key: IcKey::Func5, x: 7 + 69 * 3, y: 9 + 69 * 4, pressed: false, hovered: false, label: "+", shlabel: ">>",  sulabel: "F5", sticky: false },
        VirtualKey { key: IcKey::Num0,  x: 7 + 69 * 0, y: 9 + 69 * 5, pressed: false, hovered: false, label: "0", shlabel: "A",   sulabel: "",   sticky: false },
        VirtualKey { key: IcKey::Shift, x: 7 + 69 * 1, y: 9 + 69 * 5, pressed: false, hovered: false, label: "Shft", shlabel: "", sulabel: "",   sticky: true  },
        VirtualKey { key: IcKey::Super, x: 7 + 69 * 2, y: 9 + 69 * 5, pressed: false, hovered: false, label: "§",  shlabel: "",  sulabel: "",   sticky: true  },
        VirtualKey { key: IcKey::Func6, x: 7 + 69 * 3, y: 9 + 69 * 5, pressed: false, hovered: false, label: "=", shlabel: "^",   sulabel: "F6", sticky: false },
    ];

    // -----------------------------------------------------------------------
    // Raylib window
    // -----------------------------------------------------------------------
    println!("Hello, world!");
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(800, 600)
        .title("Incredicalculator PC")
        .vsync()
        .build();
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

    // -----------------------------------------------------------------------
    // Main loop
    // -----------------------------------------------------------------------
    while !rl_handle.window_should_close() {
        // --- Physical keyboard ---
        while let Some(rl_key) = rl_handle.get_key_pressed() {
            if let Some(&ic_key) = key_map.get(&rl_key) {
                icalc.key_down(ic_key);
            }
        }
        for rl_key in key_map.keys() {
            if rl_handle.is_key_released(*rl_key) {
                if let Some(&ic_key) = key_map.get(rl_key) {
                    icalc.key_up(ic_key);
                }
            }
        }

        // --- Virtual keyboard (mouse) ---
        let virtual_key_size: i32 = 64;
        let mouse_pos = rl_handle.get_mouse_position();
        let mouse_down = rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        let mouse_pressed_this_frame =
            rl_handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);

        for vk in virtual_keys.iter_mut() {
            let key_rect = Rectangle::new(
                vk.x as f32, vk.y as f32,
                virtual_key_size as f32, virtual_key_size as f32,
            );
            vk.hovered = key_rect.check_collision_point_rec(mouse_pos);

            if vk.sticky {
                if vk.hovered && mouse_pressed_this_frame {
                    vk.pressed = !vk.pressed;
                    if vk.pressed {
                        icalc.key_down(vk.key);
                    } else {
                        icalc.key_up(vk.key);
                    }
                }
            } else {
                if vk.hovered && mouse_down && !vk.pressed {
                    vk.pressed = true;
                    icalc.key_down(vk.key);
                } else if vk.pressed && !mouse_down {
                    vk.pressed = false;
                    icalc.key_up(vk.key);
                }
            }
        }

        // --- Core update ---
        icalc.update(ic_rl_platform.as_mut());

        // --- Upload pixel data to GPU ---
        let fps = rl_handle.get_fps();
        let mut raw_pixels: Vec<u8> =
            Vec::with_capacity((RENDER_W * RENDER_H * 4) as usize);
        for &pixel in ic_rl_platform.canvas_data.iter() {
            let c = rgb565_to_rl_color(pixel);
            raw_pixels.push(c.r);
            raw_pixels.push(c.g);
            raw_pixels.push(c.b);
            raw_pixels.push(c.a);
        }
        target_tex.update_texture(&raw_pixels).unwrap();

        // --- Draw ---
        let mut d = rl_handle.begin_drawing(&rl_thread);
        d.clear_background(Color::DARKOLIVEGREEN);
        d.draw_rectangle(0, 0, 286, 430, Color::GRAY);

        for vk in virtual_keys.iter() {
            let c = if vk.pressed {
                Color::BLUE
            } else if vk.hovered {
                Color::WHITESMOKE
            } else {
                Color::LIGHTGRAY
            };
            d.draw_rectangle(vk.x as i32, vk.y as i32, virtual_key_size, virtual_key_size, c);
            d.draw_text(vk.label,   vk.x as i32 + 16, vk.y as i32 + 16, 20, Color::BLACK);
            d.draw_text(vk.shlabel, vk.x as i32 + 46, vk.y as i32 + 46, 20, Color::BLUE);
            d.draw_text(vk.sulabel, vk.x as i32 +  6, vk.y as i32 + 46, 20, Color::RED);
        }

        let source_rec = Rectangle::new(
            0.0, 0.0,
            target_tex.texture.width as f32,
            target_tex.texture.height as f32,
        );
        let origin = Vector2::new(0.0, 0.0);

        // Small preview (fits in the calculator bezel)
        let dest_rec_small = Rectangle::new(23.0, 10.0, 160.0, 120.0);
        d.draw_texture_pro(&target_tex, source_rec, dest_rec_small, origin, 0.0, Color::WHITE);

        // Full-size view on the right
        let dest_rec_zoom = Rectangle::new(300.0, 10.0, RENDER_W as f32, RENDER_H as f32);
        d.draw_texture_pro(&target_tex, source_rec, dest_rec_zoom, origin, 0.0, Color::WHITE);

        d.draw_text(
            format!("What! {fps} FPS").as_str(),
            12, 435, 24, Color::WHITE,
        );
    }

    // Explicitly drop audio last so it doesn't cut out early if the compiler
    // decides to clean up in a surprising order.
    drop(audio_player);
    drop(audio_handle);
}