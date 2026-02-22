use rgb::{RGB8, Rgb};

use crate::{app::IcApp, text::draw_text};

pub struct AspectRatioCalculator {
}

impl AspectRatioCalculator {
    pub fn new() -> AspectRatioCalculator {
        AspectRatioCalculator {}
    }
}

impl IcApp for AspectRatioCalculator {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        ()
    }

    fn update(&mut self, platform: &mut dyn crate::platform::IcPlatform, ctx: &crate::app::InputContext) {
        platform.clear(RGB8::new(0xff, 0xb3, 0x3f));
        draw_text(platform, "aspect ratio calculator", 10.0, 10.0, 2.0, RGB8::new(0, 0, 0));
    }
}