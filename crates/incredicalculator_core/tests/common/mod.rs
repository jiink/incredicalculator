//! Shared by the integration tests.

use glam::{IVec2, Vec2};
use incredicalculator_core::platform::IcPlatform;
use rgb::RGB8;

/// A platform that draws nothing.
#[derive(Default)]
pub struct NullPlatform;

impl IcPlatform for NullPlatform {
    fn draw_line(&mut self, _: Vec2, _: Vec2, _: RGB8, _: u32) {}
    fn draw_polygon(&mut self, _: &[Vec2], _: RGB8) {}
    fn draw_rectangle(&mut self, _: IVec2, _: IVec2, _: RGB8, _: u32, _: Option<RGB8>) {}
    fn draw_rectangle_rounded(
        &mut self,
        _: IVec2,
        _: IVec2,
        _: RGB8,
        _: u32,
        _: Option<RGB8>,
        _: u32,
    ) {
    }
    fn draw_triangle(&mut self, _: IVec2, _: IVec2, _: IVec2, _: RGB8, _: u32, _: Option<RGB8>) {}
    fn clear(&mut self, _: RGB8) {}
    fn log(&mut self, _: core::fmt::Arguments) {}
    fn millis(&self) -> u64 {
        0
    }
    fn get_battery_soc(&self) -> Option<i32> {
        None
    }
    fn get_brightness(&self) -> u8 {
        255
    }
    fn set_brightness(&mut self, _: u8) {}
    fn get_volume(&self) -> u8 {
        0
    }
    fn set_volume(&mut self, _: u8) {}
    fn get_screen_timeout_seconds(&self) -> u32 {
        30
    }
    fn set_screen_timeout_seconds(&mut self, _: u32) {}
}
