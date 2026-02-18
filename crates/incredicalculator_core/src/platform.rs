use glam::IVec2;
use rgb::*;
use core::fmt;


pub trait IcPlatform {
    fn draw_line(&mut self, start: IVec2, end: IVec2, color: RGB8, width: u32);
    fn draw_rectangle(&mut self, start: IVec2, end: IVec2, stroke_color: RGB8, stroke_width: u32, fill_color: Option<RGB8>);
    fn draw_string(&mut self, text: &str, pos: IVec2, size: u32, color: RGB8);
    fn clear(&mut self, color: RGB8);
    fn log(&mut self, arg: fmt::Arguments);
}

#[macro_export]
macro_rules! _platform_debug_log {
    ($platform:expr, $($arg:tt)*) => {
        $platform.log(format_args!($($arg)*))
    };
}
pub use _platform_debug_log as debug_log;