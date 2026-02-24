use glam::IVec2;
use num_traits::clamp_max;
use rgb::{RGB8, Rgb};

use crate::{
    app::IcApp,
    platform::{self, IcPlatform},
    text::{draw_text, draw_text_f},
};

struct NumInputBox {
    value: i32,
    pos: IVec2,
    size: IVec2,
}

impl NumInputBox {
    fn new(pos: IVec2, size: IVec2) -> NumInputBox {
        NumInputBox {
            value: 0,
            pos: pos,
            size: size,
        }
    }
    fn append_digit(&mut self, digit: u32) {
        self.value = self.value * 10 + clamp_max(digit as i32, 9);
    }
    fn draw(&mut self, platform: &mut dyn crate::platform::IcPlatform) {
        platform.draw_rectangle(
            self.pos,
            self.pos + self.size,
            RGB8::new(0xAD, 0x4B, 0x27),
            2,
            Some(RGB8::new(0xff, 0xff, 0xff)),
        );
        let margin = 4;
        draw_text_f(
            platform,
            format_args!("{}", self.value),
            (self.pos.x + margin) as f32,
            (self.pos.y + margin) as f32,
            4.0,
            RGB8::new(0, 0, 0),
        );
    }
}

enum FocusUi {
    Width1,
    Height1,
    Width2,
    Height2,
}

pub struct AspectRatioCalculator {
    focused_ui: FocusUi,
    input_box_width1: NumInputBox,
    input_box_height1: NumInputBox,
    input_box_width2: NumInputBox,
    input_box_height2: NumInputBox,
}

impl AspectRatioCalculator {
    pub fn new() -> AspectRatioCalculator {
        AspectRatioCalculator {
            focused_ui: FocusUi::Width1,
            input_box_width1: NumInputBox::new(IVec2::new(17, 39), IVec2::new(129, 33)),
            input_box_height1: NumInputBox::new(IVec2::new(174, 39), IVec2::new(129, 33)),
            input_box_width2: NumInputBox::new(IVec2::new(17, 107), IVec2::new(129, 33)),
            input_box_height2: NumInputBox::new(IVec2::new(174, 107), IVec2::new(129, 33)),
        }
    }
}

impl IcApp for AspectRatioCalculator {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        ()
    }

    fn update(
        &mut self,
        platform: &mut dyn crate::platform::IcPlatform,
        ctx: &crate::app::InputContext,
    ) {
        platform.clear(RGB8::new(0xff, 0xb3, 0x3f));
        draw_text(
            platform,
            "aspect ratio calculator",
            10.0,
            10.0,
            2.0,
            RGB8::new(0, 0, 0),
        );
        self.input_box_width1.draw(platform);
        self.input_box_height1.draw(platform);
        self.input_box_width2.draw(platform);
        self.input_box_height2.draw(platform);
    }
}
