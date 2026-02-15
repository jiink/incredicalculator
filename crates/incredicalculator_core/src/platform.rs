use glam::IVec2;
use rgb::*;
use core::fmt;

pub struct Shape {
    pub start: IVec2,
    pub end: IVec2,
    pub color: RGB8,
}

pub trait IcPlatform {
    fn draw_shape(&mut self, shape: Shape);
    fn clear_lines(&mut self);
    fn log(&mut self, arg: fmt::Arguments);
}

#[macro_export]
macro_rules! _platform_debug_log {
    ($platform:expr, $($arg:tt)*) => {
        $platform.log(format_args!($($arg)*))
    };
}
pub use _platform_debug_log as debug_log;