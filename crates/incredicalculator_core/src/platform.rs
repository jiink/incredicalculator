use glam::IVec2;
use rgb::*;
use core::fmt;

pub const CANVAS_WIDTH: u32 = 320;
pub const CANVAS_HEIGHT: u32 = 240;

pub trait IcPlatform {
    fn draw_line(&mut self, start: IVec2, end: IVec2, color: RGB8, width: u32);
    fn draw_rectangle(&mut self, start: IVec2, end: IVec2, stroke_color: RGB8, stroke_width: u32, fill_color: Option<RGB8>);
    fn draw_rectangle_rounded(
        &mut self,
        start: IVec2,
        end: IVec2,
        stroke_color: rgb::RGB8,
        stroke_width: u32,
        fill_color: Option<rgb::RGB8>,
        corner_radius: u32,
    );
    fn draw_triangle(&mut self, vertex1: IVec2, vertex2: IVec2, vertex3: IVec2, stroke_color: RGB8, stroke_width: u32, fill_color: Option<RGB8>);
    fn draw_string(&mut self, text: &str, pos: IVec2, size: u32, color: RGB8);
    fn draw_string_f(&mut self, arg: fmt::Arguments, pos: IVec2, size: u32, color: RGB8);
    fn clear(&mut self, color: RGB8);
    fn log(&mut self, arg: fmt::Arguments);
    fn millis(&self) -> u64;
    fn get_battery_soc(&self) -> i32;
}

#[macro_export]
macro_rules! _platform_debug_log {
    ($platform:expr, $($arg:tt)*) => {
        $platform.log(format_args!($($arg)*))
    };
}
pub use _platform_debug_log as debug_log;

pub const fn rgb8_hex(c: u32) -> RGB8 {
    let r = ((c >> 16) & 0xFF) as u8;
    let g = ((c >> 8) & 0xFF) as u8;
    let b = (c & 0xFF) as u8;
    RGB8::new(r, g, b)
}