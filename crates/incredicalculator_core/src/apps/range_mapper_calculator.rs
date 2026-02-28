use crate::input::{IcKey, KeyState};
use crate::{
    app::IcApp,
    platform::{self, IcPlatform},
    text::{draw_text, draw_text_f},
};
use glam::IVec2;
use num_traits::{abs, clamp_max};
use rgb::{RGB8, Rgb};

// todo put LineBuffer in a common place for both this and calculator.rs to use
struct LineBuffer<const N: usize> {
    pub data: [u8; N],
    pub len: usize,
    pub cursor: usize,
}

impl<const N: usize> LineBuffer<N> {
    pub const MAX_LEN: usize = 24;
    pub fn default() -> Self {
        Self {
            data: [0; N],
            len: 0,
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, char_code: u8) {
        if self.cursor < Self::MAX_LEN {
            for i in (self.cursor..=(self.len)).rev() {
                if i == Self::MAX_LEN - 1 {
                    break;
                }
                self.data[i + 1] = self.data[i];
            }
            self.data[self.cursor] = char_code;
            self.cursor += 1;
            self.len += 1;
            if self.len > Self::MAX_LEN {
                self.len = Self::MAX_LEN;
            }
        }
    }

    pub fn move_cursor(&mut self, right: bool) {
        if right {
            if self.cursor < self.len {
                self.cursor += 1;
            }
        } else {
            if self.cursor > 0 {
                self.cursor -= 1;
            }
        }
    }

    pub fn move_cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor = self.len;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            for i in self.cursor..self.len {
                self.data[i - 1] = self.data[i];
            }
            self.cursor -= 1;
            self.len -= 1;
            self.data[self.len] = 0;
        }
    }

    pub fn backspace_del(&mut self) {
        // delete is the same thing as pressing right and then backspace
        self.move_cursor(true);
        self.backspace();
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("Invalid UTF-8")
    }

    pub fn set_content(&mut self, content: &[u8]) {
        let copy_len = content.len().min(N);
        self.data[..copy_len].copy_from_slice(&content[..copy_len]);
        self.len = copy_len;
        self.cursor = copy_len;
    }

    pub fn clear(&mut self) {
        self.data = [0; N];
        self.len = 0;
        self.cursor = 0;
    }

    pub fn evaluate(&self) -> f32 {
        match exp_rs::interp(self.as_str(), None) {
            Ok(v) => v as f32,
            Err(_) => 0.0,
        }
    }
}

struct ExpressionInputBox {
    expression: LineBuffer<48>,
    pos: IVec2,
    size: IVec2,
}

impl ExpressionInputBox {
    fn new(pos: IVec2, size: IVec2) -> ExpressionInputBox {
        ExpressionInputBox {
            expression: LineBuffer::default(),
            pos,
            size,
        }
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
        let display_text = core::str::from_utf8(&self.expression.data[..self.expression.len])
            .unwrap_or("Invalid UTF-8");
        let text_scale = if self.expression.len > 5 { 2.0 } else { 4.0 };
        draw_text(
            platform,
            &display_text,
            (self.pos.x + margin) as f32,
            (self.pos.y + margin) as f32,
            text_scale,
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusUi {
    InValue,
    InMin,
    InMax,
    OutMin,
    OutMax,
}

pub struct RangeMapperCalculator {
    focused_ui: FocusUi,
    input_box_in_val: ExpressionInputBox,
    input_box_in_min: ExpressionInputBox,
    input_box_in_max: ExpressionInputBox,
    input_box_out_min: ExpressionInputBox,
    input_box_out_max: ExpressionInputBox,
    answer: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeyAction {
    InsertChar(u8),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Backspace,
    Enter,
    Clear,
}

impl RangeMapperCalculator {
    pub fn new() -> RangeMapperCalculator {
        RangeMapperCalculator {
            focused_ui: FocusUi::InValue,
            input_box_in_val: ExpressionInputBox::new(IVec2::new(50, 7), IVec2::new(122, 36)),
            input_box_in_min: ExpressionInputBox::new(IVec2::new(50, 56), IVec2::new(122, 36)),
            input_box_in_max: ExpressionInputBox::new(IVec2::new(188, 56), IVec2::new(122, 36)),
            input_box_out_min: ExpressionInputBox::new(IVec2::new(50, 105), IVec2::new(122, 36)),
            input_box_out_max: ExpressionInputBox::new(IVec2::new(188, 105), IVec2::new(122, 36)),
            answer: 0.0
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
                IcKey::Num6 => Some(KeyAction::InsertChar(b'.')),
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
                IcKey::Num0 => Some(KeyAction::InsertChar(b'0')),
                IcKey::Num1 => Some(KeyAction::InsertChar(b'1')),
                IcKey::Num2 => Some(KeyAction::InsertChar(b'2')),
                IcKey::Num3 => Some(KeyAction::InsertChar(b'3')),
                IcKey::Num4 => Some(KeyAction::InsertChar(b'4')),
                IcKey::Num5 => Some(KeyAction::InsertChar(b'5')),
                IcKey::Num6 => Some(KeyAction::InsertChar(b'6')),
                IcKey::Num7 => Some(KeyAction::InsertChar(b'7')),
                IcKey::Num8 => Some(KeyAction::InsertChar(b'8')),
                IcKey::Num9 => Some(KeyAction::InsertChar(b'9')),
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

    fn get_focused_input_box(&mut self) -> &mut ExpressionInputBox {
        match self.focused_ui {
            FocusUi::InValue => &mut self.input_box_in_val,
            FocusUi::InMin => &mut self.input_box_in_min,
            FocusUi::InMax => &mut self.input_box_in_max,
            FocusUi::OutMin => &mut self.input_box_out_min,
            FocusUi::OutMax => &mut self.input_box_out_max,
        }
    }

    fn has_valid_inputs(&self) -> bool {
        true
    }

    fn update_math(&mut self) {
        if !self.has_valid_inputs() {
            self.answer = 0.0;
        }
        let x = self.input_box_in_val.expression.evaluate();
        let a = self.input_box_in_min.expression.evaluate();
        let b = self.input_box_in_max.expression.evaluate();
        let c = self.input_box_out_min.expression.evaluate();
        let d = self.input_box_out_max.expression.evaluate();
        self.answer = c + ((x - a) * (d - c) / b - a);
    }
}

impl IcApp for RangeMapperCalculator {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: IcKey, ctx: &crate::app::InputContext) {
        let action = self.get_action(key, ctx.is_shifted(), ctx.is_super());
        match action {
            Some(KeyAction::InsertChar(d)) => {
                self.get_focused_input_box().expression.insert_char(d as u8);
                self.update_math();
            }
            Some(KeyAction::MoveUp) => (),
            Some(KeyAction::MoveDown) => (),
            Some(KeyAction::MoveLeft) => {
                self.get_focused_input_box().expression.move_cursor(false);
            }
            Some(KeyAction::MoveRight) => {
                self.get_focused_input_box().expression.move_cursor(true);
            }
            Some(KeyAction::Backspace) => {
                self.get_focused_input_box().expression.backspace();
                self.update_math();
            }
            Some(KeyAction::Clear) => {
                self.get_focused_input_box().expression.clear();
                self.update_math();
            }
            Some(KeyAction::Enter) => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::InValue => FocusUi::InMin,
                    FocusUi::InMin => FocusUi::InMax,
                    FocusUi::InMax => FocusUi::OutMin,
                    FocusUi::OutMin => FocusUi::OutMax,
                    FocusUi::OutMax => FocusUi::InValue,
                }
            }
            None => (),
        }
    }

    fn update(&mut self, platform: &mut dyn IcPlatform, ctx: &crate::app::InputContext) {
        platform.clear(RGB8::new(0x1C, 0x07, 0x70));
        self.input_box_in_val.draw(platform);
        self.input_box_in_min.draw(platform);
        self.input_box_in_max.draw(platform);
        self.input_box_out_min.draw(platform);
        self.input_box_out_max.draw(platform);
        self.get_focused_input_box().draw_highlight(platform);
        draw_text_f(
            platform,
            format_args!("{}", self.answer),
            49.0,
            155.0,
            4.0,
            RGB8::new(0xff, 0xff, 0xff),
        );
    }
}
