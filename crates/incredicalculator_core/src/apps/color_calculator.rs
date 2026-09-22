// use crate::audio_engine;
// use crate::input::IcKey;
// use crate::text::text_to_pos;
// use crate::{
//     app::IcApp,
//     platform::{rgb8_hex, IcPlatform},
//     text::{draw_text, draw_text_f},
// };
// use glam::IVec2;
// use rgb::Rgb;
// use alloc::format;

// struct LineBuffer<const N: usize> {
//     pub data: [u8; N],
//     pub len: usize,
//     pub cursor: usize,
// }

// impl<const N: usize> LineBuffer<N> {
//     pub const MAX_LEN: usize = N;

//     pub fn default() -> Self {
//         Self {
//             data: [0; N],
//             len: 0,
//             cursor: 0,
//         }
//     }

//     pub fn insert_char(&mut self, char_code: u8) {
//         if self.len < Self::MAX_LEN && self.cursor <= self.len {
//             for i in (self.cursor..self.len).rev() {
//                 self.data[i + 1] = self.data[i];
//             }
//             self.data[self.cursor] = char_code;
//             self.cursor += 1;
//             self.len += 1;
//         }
//     }

//     pub fn move_cursor(&mut self, right: bool) {
//         if right {
//             if self.cursor < self.len {
//                 self.cursor += 1;
//             }
//         } else if self.cursor > 0 {
//             self.cursor -= 1;
//         }
//     }

//     pub fn move_cursor_home(&mut self) {
//         self.cursor = 0;
//     }

//     pub fn move_cursor_end(&mut self) {
//         self.cursor = self.len;
//     }

//     pub fn backspace(&mut self) {
//         if self.cursor > 0 {
//             for i in self.cursor..self.len {
//                 self.data[i - 1] = self.data[i];
//             }
//             self.cursor -= 1;
//             self.len -= 1;
//             self.data[self.len] = 0;
//         }
//     }

//     pub fn as_str(&self) -> &str {
//         core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
//     }

//     pub fn set_str(&mut self, text: &str) {
//         let bytes = text.as_bytes();
//         let copy_len = bytes.len().min(N);
//         self.data[..copy_len].copy_from_slice(&bytes[..copy_len]);
//         self.len = copy_len;
//         self.cursor = copy_len;
//     }

//     pub fn clear(&mut self) {
//         self.data = [0; N];
//         self.len = 0;
//         self.cursor = 0;
//     }

//     pub fn parse_f32(&self) -> Option<f32> {
//         self.as_str().trim().parse::<f32>().ok()
//     }
// }

// struct InputBox {
//     buffer: LineBuffer<16>,
//     pos: IVec2,
//     size: IVec2,
//     label: &'static str,
// }

// impl InputBox {
//     fn new(pos: IVec2, size: IVec2, label: &'static str) -> Self {
//         Self {
//             buffer: LineBuffer::default(),
//             pos,
//             size,
//             label,
//         }
//     }

//     fn draw(&mut self, platform: &mut dyn IcPlatform, has_focus: bool) {
//         let bg_color = rgb8_hex(if has_focus { 0x2A2D3E } else { 0x161823 });
//         let border_color = rgb8_hex(if has_focus { 0x00E5FF } else { 0x4A4E69 });

//         platform.draw_rectangle(
//             self.pos,
//             self.pos + self.size,
//             border_color,
//             1,
//             Some(bg_color),
//         );

//         // Draw Label above or beside box
//         draw_text(
//             platform,
//             self.label,
//             self.pos.x as f32,
//             self.pos.y as f32 - 14.0,
//             1.5,
//             rgb8_hex(0x8D99AE),
//         );

//         // Text & Cursor
//         let text_x = (self.pos.x + 6) as f32;
//         let text_y = (self.pos.y + 6) as f32;
//         let scale = 2.0;

//         let display_text = self.buffer.as_str();
//         draw_text(
//             platform,
//             display_text,
//             text_x,
//             text_y,
//             scale,
//             rgb8_hex(if has_focus { 0xFFFFFF } else { 0xC0C0C0 }),
//         );

//         if has_focus {
//             let cursor_x = text_to_pos(display_text, text_x, scale, self.buffer.cursor);
//             platform.draw_line(
//                 IVec2::new(cursor_x as i32, text_y as i32 - 2),
//                 IVec2::new(cursor_x as i32, (text_y + 14.0) as i32),
//                 Rgb::new(0x00, 0xE5, 0xFF),
//                 2,
//             );
//         }
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// enum FocusField {
//     Hex,
//     RedFloat,
//     GreenFloat,
//     BlueFloat,
//     Hue,
//     Sat,
//     Val,
// }

// impl FocusField {
//     fn next(self) -> Self {
//         match self {
//             Self::Hex => Self::RedFloat,
//             Self::RedFloat => Self::GreenFloat,
//             Self::GreenFloat => Self::BlueFloat,
//             Self::BlueFloat => Self::Hue,
//             Self::Hue => Self::Sat,
//             Self::Sat => Self::Val,
//             Self::Val => Self::Hex,
//         }
//     }
// }

// #[derive(Clone, Copy, PartialEq, Eq)]
// enum KeyAction {
//     InsertChar(u8),
//     MoveLeft,
//     MoveRight,
//     Backspace,
//     Enter,
//     Clear,
//     Home,
//     End,
// }

// pub struct ColorCalculator {
//     focused: FocusField,

//     // Hex representation box
//     box_hex: InputBox,

//     // Float representations (0.0 - 1.0)
//     box_r: InputBox,
//     box_g: InputBox,
//     box_b: InputBox,

//     // HSV representations (H: 0..360, S: 0..100, V: 0..100)
//     box_h: InputBox,
//     box_s: InputBox,
//     box_v: InputBox,

//     // Calculated RGB state in [0.0, 1.0]
//     r: f32,
//     g: f32,
//     b: f32,
// }

// impl ColorCalculator {
//     pub fn new() -> Self {
//         let mut calc = Self {
//             focused: FocusField::Hex,

//             box_hex: InputBox::new(IVec2::new(20, 35), IVec2::new(120, 28), "HEX (0xRRGGBB)"),

//             box_r: InputBox::new(IVec2::new(20, 90), IVec2::new(80, 28), "Red (0..1)"),
//             box_g: InputBox::new(IVec2::new(110, 90), IVec2::new(80, 28), "Green (0..1)"),
//             box_b: InputBox::new(IVec2::new(200, 90), IVec2::new(80, 28), "Blue (0..1)"),

//             box_h: InputBox::new(IVec2::new(20, 145), IVec2::new(80, 28), "Hue (0..360)"),
//             box_s: InputBox::new(IVec2::new(110, 145), IVec2::new(80, 28), "Sat (0..100)"),
//             box_v: InputBox::new(IVec2::new(200, 145), IVec2::new(80, 28), "Val (0..100)"),

//             r: 0.2,
//             g: 0.6,
//             b: 1.0,
//         };

//         calc.sync_all_from_rgb();
//         calc
//     }

//     fn get_focused_box(&mut self) -> &mut InputBox {
//         match self.focused {
//             FocusField::Hex => &mut self.box_hex,
//             FocusField::RedFloat => &mut self.box_r,
//             FocusField::GreenFloat => &mut self.box_g,
//             FocusField::BlueFloat => &mut self.box_b,
//             FocusField::Hue => &mut self.box_h,
//             FocusField::Sat => &mut self.box_s,
//             FocusField::Val => &mut self.box_v,
//         }
//     }

//     // --- Color Conversion Helper Functions ---

//     fn sync_all_from_rgb(&mut self) {
//         let r_u8 = (self.r.clamp(0.0, 1.0) * 255.0).round() as u32;
//         let g_u8 = (self.g.clamp(0.0, 1.0) * 255.0).round() as u32;
//         let b_u8 = (self.b.clamp(0.0, 1.0) * 255.0).round() as u32;

//         // Sync Hex Box
//         if self.focused != FocusField::Hex {
//             let hex_val = (r_u8 << 16) | (g_u8 << 8) | b_u8;
//             self.box_hex.buffer.set_str(&format!("{:06X}", hex_val));
//         }

//         // Sync RGB Float Boxes
//         if self.focused != FocusField::RedFloat {
//             self.box_r.buffer.set_str(&format!("{:.2}", self.r));
//         }
//         if self.focused != FocusField::GreenFloat {
//             self.box_g.buffer.set_str(&format!("{:.2}", self.g));
//         }
//         if self.focused != FocusField::BlueFloat {
//             self.box_b.buffer.set_str(&format!("{:.2}", self.b));
//         }

//         // Sync HSV Boxes
//         let (h, s, v) = rgb_to_hsv(self.r, self.g, self.b);
//         if self.focused != FocusField::Hue {
//             self.box_h.buffer.set_str(&format!("{:.0}", h));
//         }
//         if self.focused != FocusField::Sat {
//             self.box_s.buffer.set_str(&format!("{:.0}", s));
//         }
//         if self.focused != FocusField::Val {
//             self.box_v.buffer.set_str(&format!("{:.0}", v));
//         }
//     }

//     fn recalculate_from_focused(&mut self) {
//         match self.focused {
//             FocusField::Hex => {
//                 let s = self.box_hex.buffer.as_str().trim_start_matches("0x");
//                 if let Ok(val) = u32::from_str_radix(s, 16) {
//                     self.r = ((val >> 16) & 0xFF) as f32 / 255.0;
//                     self.g = ((val >> 8) & 0xFF) as f32 / 255.0;
//                     self.b = (val & 0xFF) as f32 / 255.0;
//                 }
//             }
//             FocusField::RedFloat | FocusField::GreenFloat | FocusField::BlueFloat => {
//                 if let Some(v) = self.box_r.buffer.parse_f32() {
//                     self.r = v.clamp(0.0, 1.0);
//                 }
//                 if let Some(v) = self.box_g.buffer.parse_f32() {
//                     self.g = v.clamp(0.0, 1.0);
//                 }
//                 if let Some(v) = self.box_b.buffer.parse_f32() {
//                     self.b = v.clamp(0.0, 1.0);
//                 }
//             }
//             FocusField::Hue | FocusField::Sat | FocusField::Val => {
//                 let h = self.box_h.buffer.parse_f32().unwrap_or(0.0).clamp(0.0, 360.0);
//                 let s = self.box_s.buffer.parse_f32().unwrap_or(0.0).clamp(0.0, 100.0);
//                 let v = self.box_v.buffer.parse_f32().unwrap_or(0.0).clamp(0.0, 100.0);
//                 let (r, g, b) = hsv_to_rgb(h, s, v);
//                 self.r = r;
//                 self.g = g;
//                 self.b = b;
//             }
//         }
//         self.sync_all_from_rgb();
//     }

//     fn get_action(&self, key: IcKey, is_shifted: bool, is_super: bool) -> Option<KeyAction> {
//         if is_super {
//             match key {
//                 IcKey::Num1 => Some(KeyAction::End),
//                 IcKey::Num4 => Some(KeyAction::MoveLeft),
//                 IcKey::Num6 => Some(KeyAction::MoveRight),
//                 IcKey::Num7 => Some(KeyAction::Home),
//                 IcKey::Num9 => Some(KeyAction::Clear),
//                 _ => None,
//             }
//         } else if is_shifted {
//             match key {
//                 IcKey::Num6 => Some(KeyAction::InsertChar(b'.')),
//                 _ => None,
//             }
//         } else {
//             match key {
//                 IcKey::Num0 => Some(KeyAction::InsertChar(b'0')),
//                 IcKey::Num1 => Some(KeyAction::InsertChar(b'1')),
//                 IcKey::Num2 => Some(KeyAction::InsertChar(b'2')),
//                 IcKey::Num3 => Some(KeyAction::InsertChar(b'3')),
//                 IcKey::Num4 => Some(KeyAction::InsertChar(b'4')),
//                 IcKey::Num5 => Some(KeyAction::InsertChar(b'5')),
//                 IcKey::Num6 => Some(KeyAction::InsertChar(b'6')),
//                 IcKey::Num7 => Some(KeyAction::InsertChar(b'7')),
//                 IcKey::Num8 => Some(KeyAction::InsertChar(b'8')),
//                 IcKey::Num9 => Some(KeyAction::InsertChar(b'9')),
//                 IcKey::Func1 => Some(KeyAction::Backspace),
//                 IcKey::Func6 => Some(KeyAction::Enter),
//                 _ => None,
//             }
//         }
//     }
// }

// // Math helpers for HSV <-> RGB conversions
// fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
//     let max = r.max(g).max(b);
//     let min = r.min(g).min(b);
//     let delta = max - min;

//     let h = if delta == 0.0 {
//         0.0
//     } else if max == r {
//         60.0 * (((g - b) / delta) % 6.0)
//     } else if max == g {
//         60.0 * (((b - r) / delta) + 2.0)
//     } else {
//         60.0 * (((r - g) / delta) + 4.0)
//     };

//     let h = if h < 0.0 { h + 360.0 } else { h };
//     let s = if max == 0.0 { 0.0 } else { (delta / max) * 100.0 };
//     let v = max * 100.0;

//     (h, s, v)
// }

// fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
//     let s = s / 100.0;
//     let v = v / 100.0;
//     let c = v * s;
//     let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
//     let m = v - c;

//     let (r_prime, g_prime, b_prime) = match h as u32 {
//         0..=59 | 360 => (c, x, 0.0),
//         60..=119 => (x, c, 0.0),
//         120..=179 => (0.0, c, x),
//         180..=239 => (0.0, x, c),
//         240..=299 => (x, 0.0, c),
//         _ => (c, 0.0, x),
//     };

//     (r_prime + m, g_prime + m, b_prime + m)
// }

// impl IcApp for ColorCalculator {
//     fn name(&self) -> &str {
//         "Color Calculator"
//     }

//     fn requires_realtime_updates(&self) -> bool {
//         false
//     }

//     fn on_enter(&mut self) {}

//     fn on_key(&mut self, key: IcKey, ctx: &crate::app::InputContext) {
//         let action = self.get_action(key, ctx.is_shifted(), ctx.is_super());

//         match action {
//             Some(KeyAction::InsertChar(ch)) => {
//                 // Perform input validation based on focused box
//                 let is_hex_box = self.focused == FocusField::Hex;
//                 let is_valid_char = match ch {
//                     b'0'..=b'9' | b'.' => true,
//                     b'a'..=b'f' | b'A'..=b'F' | b'x' | b'X' if is_hex_box => true,
//                     _ => false,
//                 };

//                 if is_valid_char {
//                     self.get_focused_box().buffer.insert_char(ch);
//                     self.recalculate_from_focused();
//                 }
//             }
//             Some(KeyAction::MoveLeft) => {
//                 self.get_focused_box().buffer.move_cursor(false);
//             }
//             Some(KeyAction::MoveRight) => {
//                 self.get_focused_box().buffer.move_cursor(true);
//             }
//             Some(KeyAction::Backspace) => {
//                 self.get_focused_box().buffer.backspace();
//                 self.recalculate_from_focused();
//             }
//             Some(KeyAction::Clear) => {
//                 self.get_focused_box().buffer.clear();
//                 self.recalculate_from_focused();
//             }
//             Some(KeyAction::Enter) => {
//                 self.focused = self.focused.next();
//             }
//             Some(KeyAction::Home) => {
//                 self.get_focused_box().buffer.move_cursor_home();
//             }
//             Some(KeyAction::End) => {
//                 self.get_focused_box().buffer.move_cursor_end();
//             }
//             None => (),
//         }
//     }

//     fn update(
//         &mut self,
//         platform: &mut dyn IcPlatform,
//         _ctx: &crate::app::InputContext,
//         _audio: &mut audio_engine::AudioEngine,
//     ) {
//         // Clear background with dark charcoal theme
//         platform.clear(rgb8_hex(0x0F111A));

//         // App Title
//         draw_text(platform, "COLOR CONVERTER", 20.0, 10.0, 2.0, rgb8_hex(0x00E5FF));

//         // Draw Inputs
//         self.box_hex.draw(platform, self.focused == FocusField::Hex);

//         self.box_r.draw(platform, self.focused == FocusField::RedFloat);
//         self.box_g.draw(platform, self.focused == FocusField::GreenFloat);
//         self.box_b.draw(platform, self.focused == FocusField::BlueFloat);

//         self.box_h.draw(platform, self.focused == FocusField::Hue);
//         self.box_s.draw(platform, self.focused == FocusField::Sat);
//         self.box_v.draw(platform, self.focused == FocusField::Val);

//         // Draw Color Swatch Preview
//         let preview_pos = IVec2::new(160, 35);
//         let preview_size = IVec2::new(120, 28);
//         let active_color = Rgb::new(
//             (self.r.clamp(0.0, 1.0) * 255.0) as u8,
//             (self.g.clamp(0.0, 1.0) * 255.0) as u8,
//             (self.b.clamp(0.0, 1.0) * 255.0) as u8,
//         );

//         // Fill active color preview box with border
//         platform.draw_rectangle(
//             preview_pos,
//             preview_pos + preview_size,
//             rgb8_hex(0xFFFFFF),
//             1,
//             Some(active_color),
//         );

//         draw_text_f(
//             platform,
//             format_args!(
//                 "RGB: ({:.2}, {:.2}, {:.2})",
//                 self.r, self.g, self.b
//             ),
//             20.0,
//             190.0,
//             1.5,
//             rgb8_hex(0x8D99AE),
//         );
//     }
// }




////////////////////////////////////////////////////////////////////////////////////////////




use crate::audio_engine;
use crate::input::IcKey;
use crate::text::text_to_pos;
use crate::{
    app::IcApp,
    platform::{rgb8_hex, IcPlatform},
    text::{draw_text, draw_text_f},
};
use glam::IVec2;
use rgb::Rgb;
use alloc::format;
use crate::fonts::FontId;

// Based on the input/editing pattern used by the example Range Mapper app.
// The original uses fixed-size LineBuffers, focus cycling with Enter, and
// redraws the calculator every frame. fileciteturn0file0L13-L25
// Its keyboard mapping also uses Shift for punctuation and Super for editing
// shortcuts. fileciteturn0file0L203-L248

struct LineBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
    cursor: usize,
}

impl<const N: usize> LineBuffer<N> {
    const MAX_LEN: usize = N;

    fn default() -> Self {
        Self {
            data: [0; N],
            len: 0,
            cursor: 0,
        }
    }

    fn insert_char(&mut self, c: u8) {
        if self.len >= Self::MAX_LEN || self.cursor > self.len {
            return;
        }

        for i in (self.cursor..self.len).rev() {
            self.data[i + 1] = self.data[i];
        }

        self.data[self.cursor] = c;
        self.cursor += 1;
        self.len += 1;
    }

    fn move_cursor(&mut self, right: bool) {
        if right {
            self.cursor = (self.cursor + 1).min(self.len);
        } else {
            self.cursor = self.cursor.saturating_sub(1);
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor = self.len;
    }

    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }

        for i in self.cursor..self.len {
            self.data[i - 1] = self.data[i];
        }

        self.cursor -= 1;
        self.len -= 1;
        self.data[self.len] = 0;
    }

    fn clear(&mut self) {
        self.data = [0; N];
        self.len = 0;
        self.cursor = 0;
    }

    fn set_content(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let n = bytes.len().min(Self::MAX_LEN);
        self.data[..n].copy_from_slice(&bytes[..n]);
        self.len = n;
        self.cursor = n;
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }
}

struct InputBox {
    expression: LineBuffer<16>,
    pos: IVec2,
    size: IVec2,
}

impl InputBox {
    fn new(pos: IVec2, size: IVec2) -> Self {
        Self {
            expression: LineBuffer::default(),
            pos,
            size,
        }
    }

    fn draw(&mut self, platform: &mut dyn IcPlatform, focused: bool) {
        let border = rgb8_hex(if focused { 0xFFFFFF } else { 0x39465F });
        let fill = rgb8_hex(if focused { 0x26354D } else { 0x151D2B });

        platform.draw_rectangle(self.pos, self.pos + self.size, border, 0, Some(fill));

        let text = self.expression.as_str();
        let scale = if self.expression.len > 7 { 2.0 } else { 3.0 };
        let x = (self.pos.x + 5) as f32;
        let y = (self.pos.y + 5) as f32;

        draw_text(
            platform,
            text,
            x,
            y,
            scale,
            rgb8_hex(if focused { 0xFFFFFF } else { 0xB9C6D8 }),
            FontId::Futural
        );

        if focused {
            let cursor_x = text_to_pos(text, x, scale, self.expression.cursor, FontId::Futural);
            platform.draw_line(
                IVec2::new(cursor_x as i32, self.pos.y + 4),
                IVec2::new(cursor_x as i32, self.pos.y + self.size.y - 5),
                Rgb::new(0x58, 0xD9, 0xFF),
                2,
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Hex,
    R,
    G,
    B,
    H,
    S,
    V,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Hex => Focus::R,
            Focus::R => Focus::G,
            Focus::G => Focus::B,
            Focus::B => Focus::H,
            Focus::H => Focus::S,
            Focus::S => Focus::V,
            Focus::V => Focus::Hex,
        }
    }

    fn previous(self) -> Self {
        match self {
            Focus::Hex => Focus::V,
            Focus::R => Focus::Hex,
            Focus::G => Focus::R,
            Focus::B => Focus::G,
            Focus::H => Focus::B,
            Focus::S => Focus::H,
            Focus::V => Focus::S,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    Insert(u8),
    Left,
    Right,
    Up,
    Down,
    Backspace,
    Enter,
    Clear,
    Home,
    End,
}

#[derive(Clone, Copy)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
}

impl Color {
    fn from_hex(hex: &str) -> Option<Self> {
        let s = hex.strip_prefix("0x").or_else(|| hex.strip_prefix("0X")).unwrap_or(hex);
        if s.len() != 6 {
            return None;
        }

        fn nibble(c: u8) -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            }
        }

        let bytes = s.as_bytes();
        let pair = |a: u8, b: u8| -> Option<u8> {
            Some(nibble(a)? * 16 + nibble(b)?)
        };

        Some(Self {
            r: pair(bytes[0], bytes[1])? as f32 / 255.0,
            g: pair(bytes[2], bytes[3])? as f32 / 255.0,
            b: pair(bytes[4], bytes[5])? as f32 / 255.0,
        })
    }

    fn from_rgb(r: f32, g: f32, b: f32) -> Option<Self> {
        if r.is_finite() && g.is_finite() && b.is_finite()
            && r >= 0.0 && r <= 1.0
            && g >= 0.0 && g <= 1.0
            && b >= 0.0 && b <= 1.0
        {
            Some(Self { r, g, b })
        } else {
            None
        }
    }

    fn from_hsv(h: f32, s: f32, v: f32) -> Option<Self> {
        if !h.is_finite() || !s.is_finite() || !v.is_finite()
            || h < 0.0 || h > 360.0
            || s < 0.0 || s > 100.0
            || v < 0.0 || v > 100.0
        {
            return None;
        }

        let s = s / 100.0;
        let v = v / 100.0;

        if s == 0.0 {
            return Some(Self { r: v, g: v, b: v });
        }

        let hh = (h % 360.0) / 60.0;
        let i = hh.floor() as i32;
        let f = hh - i as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Some(Self { r, g, b })
    }

    fn hsv(self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let d = max - min;

        let h = if d == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * (((self.g - self.b) / d) % 6.0)
        } else if max == self.g {
            60.0 * (((self.b - self.r) / d) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / d) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };
        let s = if max == 0.0 { 0.0 } else { d / max };
        (h, s * 100.0, max * 100.0)
    }

    fn rgb8(self) -> u32 {
        let r = (self.r * 255.0 + 0.5) as u32;
        let g = (self.g * 255.0 + 0.5) as u32;
        let b = (self.b * 255.0 + 0.5) as u32;
        (r << 16) | (g << 8) | b
    }
}

pub struct ColorCalculator {
    focus: Focus,

    hex: InputBox,
    r: InputBox,
    g: InputBox,
    b: InputBox,
    h: InputBox,
    s: InputBox,
    v: InputBox,

    color: Color,
}

impl ColorCalculator {
    pub fn new() -> Self {
        let mut app = Self {
            focus: Focus::Hex,

            hex: InputBox::new(IVec2::new(76, 39), IVec2::new(166, 28)),
            r: InputBox::new(IVec2::new(55, 95), IVec2::new(73, 28)),
            g: InputBox::new(IVec2::new(132, 95), IVec2::new(73, 28)),
            b: InputBox::new(IVec2::new(209, 95), IVec2::new(73, 28)),

            h: InputBox::new(IVec2::new(55, 151), IVec2::new(73, 28)),
            s: InputBox::new(IVec2::new(132, 151), IVec2::new(73, 28)),
            v: InputBox::new(IVec2::new(209, 151), IVec2::new(73, 28)),

            color: Color { r: 0.0, g: 0.0, b: 0.0 },
        };

        app.sync_all();
        app
    }

    fn input_mut(&mut self, focus: Focus) -> &mut InputBox {
        match focus {
            Focus::Hex => &mut self.hex,
            Focus::R => &mut self.r,
            Focus::G => &mut self.g,
            Focus::B => &mut self.b,
            Focus::H => &mut self.h,
            Focus::S => &mut self.s,
            Focus::V => &mut self.v,
        }
    }

    fn parse_f32(input: &InputBox) -> Option<f32> {
        input.expression.as_str().parse::<f32>().ok()
    }

    fn parse_focused(&self) -> Option<Color> {
        match self.focus {
            Focus::Hex => Color::from_hex(self.hex.expression.as_str()),
            Focus::R => Color::from_rgb(
                Self::parse_f32(&self.r)?,
                self.color.g,
                self.color.b,
            ),
            Focus::G => Color::from_rgb(
                self.color.r,
                Self::parse_f32(&self.g)?,
                self.color.b,
            ),
            Focus::B => Color::from_rgb(
                self.color.r,
                self.color.g,
                Self::parse_f32(&self.b)?,
            ),
            Focus::H => {
                let (_, s, v) = self.color.hsv();
                Color::from_hsv(Self::parse_f32(&self.h)?, s, v)
            }
            Focus::S => {
                let (h, _, v) = self.color.hsv();
                Color::from_hsv(h, Self::parse_f32(&self.s)?, v)
            }
            Focus::V => {
                let (h, s, _) = self.color.hsv();
                Color::from_hsv(h, s, Self::parse_f32(&self.v)?)
            }
        }
    }

    fn update_from_focused(&mut self) {
        if let Some(color) = self.parse_focused() {
            self.color = color;
            self.sync_all_except(self.focus);
        }
    }

    fn sync_all(&mut self) {
        self.sync_all_except(Focus::Hex);
        self.hex.expression.set_content("0x000000");
    }

    fn sync_all_except(&mut self, except: Focus) {
        let (h, s, v) = self.color.hsv();

        if except != Focus::Hex {
            self.hex.expression.set_content(&format!("0x{:06X}", self.color.rgb8()));
        }
        if except != Focus::R {
            self.r.expression.set_content(&format!("{:.4}", self.color.r));
        }
        if except != Focus::G {
            self.g.expression.set_content(&format!("{:.4}", self.color.g));
        }
        if except != Focus::B {
            self.b.expression.set_content(&format!("{:.4}", self.color.b));
        }
        if except != Focus::H {
            self.h.expression.set_content(&format!("{:.1}", h));
        }
        if except != Focus::S {
            self.s.expression.set_content(&format!("{:.1}", s));
        }
        if except != Focus::V {
            self.v.expression.set_content(&format!("{:.1}", v));
        }
    }

    // Shift+1..5 type A..E in the hex field, while Shift+6 remains '.'
    // for the decimal fields. F is available as Shift+0 in hex mode.
    fn get_action(&self, key: IcKey, shifted: bool, super_key: bool) -> Option<KeyAction> {
        if super_key {
            return match key {
                IcKey::Num1 => Some(KeyAction::End),
                IcKey::Num2 => Some(KeyAction::Down),
                IcKey::Num4 => Some(KeyAction::Left),
                IcKey::Num6 => Some(KeyAction::Right),
                IcKey::Num7 => Some(KeyAction::Home),
                IcKey::Num8 => Some(KeyAction::Up),
                IcKey::Num9 => Some(KeyAction::Clear),
                _ => None,
            };
        }

        if shifted {
            if self.focus == Focus::Hex {
                return match key {
                    IcKey::Num0 => Some(KeyAction::Insert(b'F')),
                    IcKey::Num1 => Some(KeyAction::Insert(b'A')),
                    IcKey::Num2 => Some(KeyAction::Insert(b'B')),
                    IcKey::Num3 => Some(KeyAction::Insert(b'C')),
                    IcKey::Num4 => Some(KeyAction::Insert(b'D')),
                    IcKey::Num5 => Some(KeyAction::Insert(b'E')),
                    IcKey::Num6 => None,
                    _ => None,
                };
            }

            return match key {
                IcKey::Num6 => Some(KeyAction::Insert(b'.')),
                _ => None,
            };
        }

        match key {
            IcKey::Num0 => Some(KeyAction::Insert(b'0')),
            IcKey::Num1 => Some(KeyAction::Insert(b'1')),
            IcKey::Num2 => Some(KeyAction::Insert(b'2')),
            IcKey::Num3 => Some(KeyAction::Insert(b'3')),
            IcKey::Num4 => Some(KeyAction::Insert(b'4')),
            IcKey::Num5 => Some(KeyAction::Insert(b'5')),
            IcKey::Num6 => Some(KeyAction::Insert(b'6')),
            IcKey::Num7 => Some(KeyAction::Insert(b'7')),
            IcKey::Num8 => Some(KeyAction::Insert(b'8')),
            IcKey::Num9 => Some(KeyAction::Insert(b'9')),
            IcKey::Func1 => Some(KeyAction::Backspace),
            IcKey::Func6 => Some(KeyAction::Enter),
            _ => None,
        }
    }
}

impl IcApp for ColorCalculator {
    fn name(&self) -> &str {
        "Color Calculator"
    }

    fn requires_realtime_updates(&self) -> bool {
        false
    }

    fn on_enter(&mut self) {}

    fn on_key(&mut self, key: IcKey, ctx: &crate::app::InputContext) {
        match self.get_action(key, ctx.is_shifted(), ctx.is_super()) {
            Some(KeyAction::Insert(c)) => {
                self.input_mut(self.focus).expression.insert_char(c);
                self.update_from_focused();
            }

            Some(KeyAction::Left) => self.input_mut(self.focus).expression.move_cursor(false),
            Some(KeyAction::Right) => self.input_mut(self.focus).expression.move_cursor(true),

            Some(KeyAction::Up) => self.focus = self.focus.previous(),
            Some(KeyAction::Down) => self.focus = self.focus.next(),

            Some(KeyAction::Backspace) => {
                self.input_mut(self.focus).expression.backspace();
                self.update_from_focused();
            }

            Some(KeyAction::Clear) => {
                self.input_mut(self.focus).expression.clear();
                self.update_from_focused();
            }

            Some(KeyAction::Enter) => {
                self.focus = self.focus.next();
            }

            Some(KeyAction::Home) => self.input_mut(self.focus).expression.move_cursor_home(),
            Some(KeyAction::End) => self.input_mut(self.focus).expression.move_cursor_end(),
            None => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        _ctx: &crate::app::InputContext,
        _audio: &mut audio_engine::AudioEngine,
    ) {
        let rgb = self.color.rgb8();

        // Dark, spacious backdrop.
        platform.clear(rgb8_hex(0x0B1020));

        // Header.
        draw_text(
            platform,
            "COLOR CALCULATOR",
            12.0,
            8.0,
            2.0,
            rgb8_hex(0xFFFFFF),
            FontId::Futural
        );
        draw_text(
            platform,
            "HEX  /  RGB FLOAT  /  HSV",
            13.0,
            24.0,
            1.0,
            rgb8_hex(0x6F829E),
            FontId::Futural
        );

        // Large color swatch.
        platform.draw_rectangle(
            IVec2::new(10, 38),
            IVec2::new(310, 68),
            rgb8_hex(0x273148),
            0,
            Some(rgb8_hex(rgb)),
        );

        draw_text(
            platform,
            "0x",
            51.0,
            47.0,
            2.0,
            rgb8_hex(0x6F829E),
            FontId::Futural
        );
        self.hex.draw(platform, self.focus == Focus::Hex);

        // Section labels.
        draw_text(platform, "RGB  0.0 — 1.0", 12.0, 79.0, 1.0, rgb8_hex(0x6F829E), FontId::Futural);
        draw_text(platform, "R", 55.0, 87.0, 1.0, rgb8_hex(0xFF667A), FontId::Futural);
        draw_text(platform, "G", 132.0, 87.0, 1.0, rgb8_hex(0x62E88B), FontId::Futural);
        draw_text(platform, "B", 209.0, 87.0, 1.0, rgb8_hex(0x61A9FF), FontId::Futural);

        self.r.draw(platform, self.focus == Focus::R);
        self.g.draw(platform, self.focus == Focus::G);
        self.b.draw(platform, self.focus == Focus::B);

        draw_text(platform, "HSV  H 0—360   S/V 0—100", 12.0, 135.0, 1.0, rgb8_hex(0x6F829E), FontId::Futural);
        draw_text(platform, "H", 55.0, 143.0, 1.0, rgb8_hex(0xFFB84D), FontId::Futural);
        draw_text(platform, "S", 132.0, 143.0, 1.0, rgb8_hex(0xD28CFF), FontId::Futural);
        draw_text(platform, "V", 209.0, 143.0, 1.0, rgb8_hex(0xFFFFFF), FontId::Futural);

        self.h.draw(platform, self.focus == Focus::H);
        self.s.draw(platform, self.focus == Focus::S);
        self.v.draw(platform, self.focus == Focus::V);

        // Footer / current color readout.
        draw_text(platform, "CURRENT", 12.0, 190.0, 1.0, rgb8_hex(0x6F829E), FontId::Futural);
        draw_text_f(
            platform,
            format_args!("#{:06X}", rgb),
            12.0,
            201.0,
            2.0,
            rgb8_hex(0xFFFFFF),
            FontId::Futural
        );

        draw_text(
            platform,
            "ENTER next   UP/DOWN field   ←/→ cursor   ⌘9 clear",
            12.0,
            225.0,
            1.0,
            rgb8_hex(0x52627A),
            FontId::Futural
        );
    }
}
