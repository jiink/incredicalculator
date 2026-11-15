//! A platform for unit tests.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use glam::{IVec2, Vec2};
use rgb::RGB8;

use crate::platform::IcPlatform;

/// Platform that draws nothing and keeps scripts in memory.
#[derive(Default)]
pub(crate) struct FakePlatform {
    scripts: Option<BTreeMap<String, String>>,
    version: u32,
}

impl FakePlatform {
    pub(crate) fn with_scripts(scripts: &[(&str, &str)]) -> Self {
        let scripts = scripts
            .iter()
            .map(|(name, source)| (name.to_string(), source.to_string()))
            .collect();
        Self {
            scripts: Some(scripts),
            ..Self::default()
        }
    }

    pub(crate) fn set_script(&mut self, name: &str, source: Option<&str>) {
        let scripts = self.scripts.as_mut().unwrap();
        match source {
            Some(source) => scripts.insert(name.into(), source.into()),
            None => scripts.remove(name),
        };
        self.version += 1;
    }
}

impl IcPlatform for FakePlatform {
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
    fn scripts_version(&self) -> u32 {
        self.version
    }
    fn list_scripts(&mut self) -> Option<Vec<String>> {
        Some(self.scripts.as_ref()?.keys().cloned().collect())
    }
    fn read_script(&mut self, file_name: &str) -> Option<String> {
        self.scripts.as_ref()?.get(file_name).cloned()
    }
}
