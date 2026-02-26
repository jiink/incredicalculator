use crate::input::{IcKey, KeyState};
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
    fn backspace(&mut self) {
        self.value /= 10;
    }
    fn clear(&mut self) {
        self.value = 0;
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
    fn draw_highlight(&mut self, platform: &mut dyn crate::platform::IcPlatform) {
        platform.draw_rectangle(
            self.pos,
            self.pos + self.size,
            RGB8::new(0, 0, 0xff),
            3,
            None,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    InsertDigit(u8),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Backspace,
    Enter,
    Clear,
}

impl AspectRatioCalculator {
    pub fn new() -> AspectRatioCalculator {
        AspectRatioCalculator {
            focused_ui: FocusUi::Width1,
            input_box_width1: NumInputBox::new(IVec2::new(17, 39), IVec2::new(129, 33)),
            input_box_width2: NumInputBox::new(IVec2::new(174, 39), IVec2::new(129, 33)),
            input_box_height1: NumInputBox::new(IVec2::new(17, 107), IVec2::new(129, 33)),
            input_box_height2: NumInputBox::new(IVec2::new(174, 107), IVec2::new(129, 33)),
        }
    }

    fn get_action(&self, key: IcKey, is_shifted: bool, is_super: bool) -> Option<KeyAction> {
        if is_shifted {
            match key {
                IcKey::Num0 => None,
                IcKey::Num1 => None,
                IcKey::Num2 => None,
                IcKey::Num3 => None,
                IcKey::Num4 => None,
                IcKey::Num5 => None,
                IcKey::Num6 => None,
                IcKey::Num7 => None,
                IcKey::Num8 => None,
                IcKey::Num9 => None,
                IcKey::Func1 => None,
                IcKey::Func2 => None,
                IcKey::Func3 => None,
                IcKey::Func4 => None,
                IcKey::Func5 => None,
                IcKey::Func6 => None,
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        } else if is_super {
            match key {
                IcKey::Num0 => None,
                IcKey::Num1 => Some(KeyAction::MoveDown),
                IcKey::Num2 => Some(KeyAction::MoveDown),
                IcKey::Num3 => None,
                IcKey::Num4 => Some(KeyAction::MoveLeft),
                IcKey::Num5 => None,
                IcKey::Num6 => Some(KeyAction::MoveRight),
                IcKey::Num7 => Some(KeyAction::MoveUp),
                IcKey::Num8 => Some(KeyAction::MoveUp),
                IcKey::Num9 => Some(KeyAction::Clear),
                IcKey::Func1 => None,
                IcKey::Func2 => None,
                IcKey::Func3 => None,
                IcKey::Func4 => None,
                IcKey::Func5 => None,
                IcKey::Func6 => None,
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        } else {
            match key {
                IcKey::Num0 => Some(KeyAction::InsertDigit(0)),
                IcKey::Num1 => Some(KeyAction::InsertDigit(1)),
                IcKey::Num2 => Some(KeyAction::InsertDigit(2)),
                IcKey::Num3 => Some(KeyAction::InsertDigit(3)),
                IcKey::Num4 => Some(KeyAction::InsertDigit(4)),
                IcKey::Num5 => Some(KeyAction::InsertDigit(5)),
                IcKey::Num6 => Some(KeyAction::InsertDigit(6)),
                IcKey::Num7 => Some(KeyAction::InsertDigit(7)),
                IcKey::Num8 => Some(KeyAction::InsertDigit(8)),
                IcKey::Num9 => Some(KeyAction::InsertDigit(9)),
                IcKey::Func1 => Some(KeyAction::Backspace),
                IcKey::Func2 => None,
                IcKey::Func3 => None,
                IcKey::Func4 => None,
                IcKey::Func5 => None,
                IcKey::Func6 => Some(KeyAction::Enter),
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        }
    }

    fn get_focused_input_box(&mut self) -> &mut NumInputBox {
        match self.focused_ui {
            FocusUi::Width1 => &mut self.input_box_width1,
            FocusUi::Height1 => &mut self.input_box_height1,
            FocusUi::Width2 => &mut self.input_box_width2,
            FocusUi::Height2 => &mut self.input_box_height2,
        }
    }

    fn has_any_zeroes(&self) -> bool {
        self.input_box_width1.value <= 0 || 
        self.input_box_height1.value <= 0 ||
        self.input_box_width2.value <= 0 ||
        self.input_box_height2.value <= 0
    }

    fn update_math(&mut self) {
        // w1/h1 side is called ratio and w2/h2 side is called result
        let w1 = self.input_box_width1.value as f32;
        let h1 = self.input_box_height1.value as f32;
        let w2 = self.input_box_width2.value as f32;
        let h2 = self.input_box_height2.value as f32;
        if w1 == 0.0 || h1 == 0.0 {
            return;
        }
        match self.focused_ui {
            FocusUi::Width1 => {
                self.input_box_width2.value = ((w1 / h1) * h2).round() as i32;
            }
            FocusUi::Height1 => {
                self.input_box_height2.value = ((h1 / w1) * w2).round() as i32;
            }
            FocusUi::Width2 => {
                self.input_box_height2.value = ((h1 / w1) * w2).round() as i32;
            }
            FocusUi::Height2 => {
                self.input_box_width2.value = ((w1 / h1) * h2).round() as i32;
            }
        }
    }
}

impl IcApp for AspectRatioCalculator {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        let action = self.get_action(key, ctx.is_shifted(), ctx.is_super());
        match action {
            Some(KeyAction::InsertDigit(d)) => 
            {
                self.get_focused_input_box().append_digit(d as u32);
                self.update_math();
            }
            Some(KeyAction::MoveUp) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Width1,
                    FocusUi::Height1 => FocusUi::Width1,
                    FocusUi::Width2 => FocusUi::Width2,
                    FocusUi::Height2 => FocusUi::Width2,
                }
            }
            Some(KeyAction::MoveDown) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Height1,
                    FocusUi::Height1 => FocusUi::Height1,
                    FocusUi::Width2 => FocusUi::Height2,
                    FocusUi::Height2 => FocusUi::Height2,
                }
            }
            Some(KeyAction::MoveLeft) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Width1,
                    FocusUi::Height1 => FocusUi::Width2,
                    FocusUi::Width2 => FocusUi::Width1,
                    FocusUi::Height2 => FocusUi::Height1,
                }
            }
            Some(KeyAction::MoveRight) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Width2,
                    FocusUi::Height1 => FocusUi::Height2,
                    FocusUi::Width2 => FocusUi::Height1,
                    FocusUi::Height2 => FocusUi::Height2,
                }
            }
            Some(KeyAction::Backspace) => {
                self.get_focused_input_box().backspace();
                self.update_math();
            }
            Some(KeyAction::Clear) => {
                self.get_focused_input_box().clear();
                self.update_math();
            }
            Some(KeyAction::Enter) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Height1,
                    FocusUi::Height1 => FocusUi::Width2,
                    FocusUi::Width2 => FocusUi::Height2,
                    FocusUi::Height2 => FocusUi::Width1,
                }
            }
            None => (),
        }
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
        self.get_focused_input_box().draw_highlight(platform);
    }
}
