use glam::IVec2;
use rgb::*;

pub struct Shape {
    pub start: IVec2,
    pub end: IVec2,
    pub color: RGB8,
}

pub trait IcPlatform {
    fn draw_shape(&mut self, shape: Shape);
    fn clear_lines(&mut self);
}
