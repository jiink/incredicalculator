use crate::input::{IcKey, KeyState};
use glam::IVec2;
use num_traits::{abs, clamp_max};
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
        self.input_box_width1.value <= 0
            || self.input_box_height1.value <= 0
            || self.input_box_width2.value <= 0
            || self.input_box_height2.value <= 0
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

    fn draw_tv_frame(
        &self,
        platform: &mut dyn crate::platform::IcPlatform,
        top_left: IVec2,
        bottom_right: IVec2,
    ) {
        // first the decorations around the rectangle is drawn and then the acutal screen rectangle you want
        let base_col = RGB8::new(0, 0, 0);
        let bezel_w = 3;
        platform.draw_rectangle(
            top_left - IVec2::new(bezel_w, bezel_w),
            bottom_right + IVec2::new(bezel_w, bezel_w),
            base_col,
            0,
            Some(base_col),
        );
        let center_x = (top_left.x + bottom_right.x) / 2;
        let screen_w = abs(top_left.x - bottom_right.x);
        let max_mount_w = 24;
        let mount_w = clamp_max(screen_w, max_mount_w);
        platform.draw_rectangle(
            IVec2::new(center_x - mount_w / 2, bottom_right.y + bezel_w + 1),
            IVec2::new(center_x + mount_w / 2, bottom_right.y + bezel_w + 2),
            base_col,
            0,
            Some(base_col),
        );
        let stem_w = 6;
        platform.draw_rectangle(
            IVec2::new(center_x - stem_w / 2, bottom_right.y + bezel_w + 3),
            IVec2::new(center_x + stem_w / 2, bottom_right.y + bezel_w + 6),
            base_col,
            0,
            Some(base_col),
        );
        let base_w = 36;
        platform.draw_rectangle(
            IVec2::new(center_x - base_w / 2, bottom_right.y + bezel_w + 7),
            IVec2::new(center_x + base_w / 2, bottom_right.y + bezel_w + 10),
            base_col,
            0,
            Some(base_col),
        );
        let max_antenna_base_w = 18;
        let antenna_base_w = clamp_max(screen_w, max_antenna_base_w);
        let antenna_l_pos = IVec2::new(center_x - 4, top_left.y - 15);
        let antenna_r_pos = IVec2::new(center_x + 10, top_left.y - 10);
        platform.draw_rectangle(
            IVec2::new(center_x - antenna_base_w / 2, top_left.y - bezel_w - 1),
            IVec2::new(center_x + antenna_base_w / 2, top_left.y - bezel_w - 2),
            base_col,
            0,
            Some(base_col),
        );
        platform.draw_line(
            IVec2::new(center_x - 1, top_left.y - 5),
            antenna_l_pos,
            base_col,
            2,
        );
        platform.draw_line(
            IVec2::new(center_x + 1, top_left.y - 5),
            antenna_r_pos,
            base_col,
            2,
        );
        let antenna_ball_size = 2;
        platform.draw_rectangle(
            antenna_l_pos - IVec2::new(antenna_ball_size, antenna_ball_size),
            antenna_l_pos + IVec2::new(antenna_ball_size, antenna_ball_size),
            base_col,
            0,
            Some(base_col),
        );
        platform.draw_rectangle(
            antenna_r_pos - IVec2::new(antenna_ball_size, antenna_ball_size),
            antenna_r_pos + IVec2::new(antenna_ball_size, antenna_ball_size),
            base_col,
            0,
            Some(base_col),
        );
        platform.draw_rectangle(
            top_left,
            bottom_right,
            base_col,
            0,
            Some(RGB8::new(0x51, 0x9A, 0x66)),
        );
    }

    fn draw_ratio_visualizer(&self, platform: &mut dyn crate::platform::IcPlatform) {
        if self.has_any_zeroes() {
            return;
        }
        let height: i32 = 60;
        let top_y = 159;
        let width_to_height =
            self.input_box_width1.value as f32 / self.input_box_height1.value as f32;
        let width = clamp_max((height as f32 * width_to_height) as i32, 320);
        let center_x = 320 / 2;
        let top_left = IVec2::new(center_x - width / 2, top_y);
        let bottom_right = IVec2::new(center_x + width / 2, top_y + height);
        self.draw_tv_frame(platform, top_left, bottom_right);
    }
}

impl IcApp for AspectRatioCalculator {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        let action = self.get_action(key, ctx.is_shifted(), ctx.is_super());
        match action {
            Some(KeyAction::InsertDigit(d)) => {
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
            "ASPECT RATIO CALCULATOR",
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
        platform.draw_line(
            IVec2::new(152, 83),
            IVec2::new(152 + 17, 83),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            IVec2::new(152, 93),
            IVec2::new(152 + 17, 93),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            IVec2::new(17, 89),
            IVec2::new(17 + 129, 89),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            IVec2::new(175, 89),
            IVec2::new(175 + 129, 89),
            RGB8::new(0, 0, 0),
            3,
        );
        self.draw_ratio_visualizer(platform);
    }
}
