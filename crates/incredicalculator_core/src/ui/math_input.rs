use crate::app::InputContext;
use crate::fonts::FontId;
use crate::input::IcKey;
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{draw_text, text_to_pos};
use glam::{IVec2, Vec2};
use rgb::RGB8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathInputMode {
    Integer,
    Expression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathInputDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Result of giving a key press to a MathInput.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathInputEvent {
    /// The key is not handled by this input.
    Ignored,
    /// The key was handled, but the contents did not change.
    Handled,
    /// The contents changed.
    Changed,
    /// Enter was pressed. An app can use this to move focus.
    Submitted,
    /// Navigation reached an input boundary or requested a neighboring field.
    Navigate(MathInputDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathInputError {
    Empty,
    InvalidCharacter(char),
    CapacityExceeded,
    InvalidValue, // The current text could not be parsed/evaluated as a number.
    NonFinite,
}

struct TextBuffer<const N: usize> {
    data: [u8; N],
    len: usize,
    cursor: usize,
}

impl<const N: usize> TextBuffer<N> {
    const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
            cursor: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap_or("")
    }

    fn insert_byte(&mut self, byte: u8) -> bool {
        if self.len >= N {
            return false;
        }

        for i in (self.cursor..self.len).rev() {
            self.data[i + 1] = self.data[i];
        }

        self.data[self.cursor] = byte;
        self.cursor += 1;
        self.len += 1;
        true
    }

    fn backspace(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        for i in self.cursor..self.len {
            self.data[i - 1] = self.data[i];
        }

        self.cursor -= 1;
        self.len -= 1;
        self.data[self.len] = 0;
        true
    }

    fn delete(&mut self) -> bool {
        if self.cursor >= self.len {
            return false;
        }

        for i in (self.cursor + 1)..self.len {
            self.data[i - 1] = self.data[i];
        }

        self.len -= 1;
        self.data[self.len] = 0;
        true
    }

    fn move_cursor_left(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor -= 1;
        true
    }

    fn move_cursor_right(&mut self) -> bool {
        if self.cursor >= self.len {
            return false;
        }

        self.cursor += 1;
        true
    }

    fn move_cursor_home(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }

        self.cursor = 0;
        true
    }

    fn move_cursor_end(&mut self) -> bool {
        if self.cursor == self.len {
            return false;
        }

        self.cursor = self.len;
        true
    }

    fn clear(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }

        self.data = [0; N];
        self.len = 0;
        self.cursor = 0;
        true
    }

    fn set_bytes(&mut self, bytes: &[u8]) {
        self.data = [0; N];
        self.data[..bytes.len()].copy_from_slice(bytes);
        self.len = bytes.len();
        self.cursor = self.len;
    }
}

/// a textbox element you can enter numbers and/or expressions into
pub struct MathInput<const N: usize> {
    mode: MathInputMode,
    buffer: TextBuffer<N>,
    pub pos: IVec2,
    pub size: IVec2,
    min_text_scale: f32,
    max_text_scale: f32,
    background: RGB8,
    focused_background: RGB8,
    border: Option<RGB8>,
}

impl<const N: usize> MathInput<N> {
    const TEXT_MARGIN: f32 = 4.0;

    pub const fn new(
        pos: IVec2,
        size: IVec2,
        mode: MathInputMode,
        min_text_scale: f32,
        max_text_scale: f32,
        background: RGB8,
        focused_background: RGB8,
        border: Option<RGB8>,
    ) -> Self {
        Self {
            mode,
            buffer: TextBuffer::new(),
            pos,
            size,
            min_text_scale,
            max_text_scale,
            background,
            focused_background,
            border,
        }
    }

    pub const fn mode(&self) -> MathInputMode {
        self.mode
    }

    pub fn text(&self) -> &str {
        self.buffer.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.len == 0
    }

    /// Index within `text()`.
    pub fn cursor(&self) -> usize {
        self.buffer.cursor
    }

    /// Replaces the entire contents with `text`
    pub fn set_text(&mut self, text: &str) -> Result<(), MathInputError> {
        if !text.is_ascii() {
            for ch in text.chars() {
                if !ch.is_ascii() {
                    return Err(MathInputError::InvalidCharacter(ch));
                }
            }
        }

        if text.len() > N {
            return Err(MathInputError::CapacityExceeded);
        }

        match self.mode {
            MathInputMode::Integer => {
                for (i, &byte) in text.as_bytes().iter().enumerate() {
                    if byte == b'-' && i == 0 {
                        continue;
                    }
                    if !byte.is_ascii_digit() {
                        return Err(MathInputError::InvalidCharacter(byte as char));
                    }
                }
            }
            MathInputMode::Expression => {
                for byte in text.bytes() {
                    let ch = byte as char;
                    if !Self::mode_char_allowed(self.mode, ch) {
                        return Err(MathInputError::InvalidCharacter(ch));
                    }
                }
            }
        }

        self.buffer.set_bytes(text.as_bytes());
        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn home(&mut self) {
        self.buffer.move_cursor_home();
    }

    pub fn end(&mut self) {
        self.buffer.move_cursor_end();
    }

    pub fn move_left(&mut self) {
        self.buffer.move_cursor_left();
    }

    pub fn move_right(&mut self) {
        self.buffer.move_cursor_right();
    }

    pub fn delete(&mut self) {
        self.buffer.delete();
    }

    pub fn backspace(&mut self) {
        self.buffer.backspace();
    }

    pub fn insert_char(&mut self, ch: char) -> bool {
        if !self.char_allowed(ch) || !ch.is_ascii() {
            return false;
        }

        self.buffer.insert_byte(ch as u8)
    }

    pub fn insert_digit(&mut self, digit: u8) -> bool {
        if digit > 9 {
            return false;
        }

        self.insert_char((b'0' + digit) as char)
    }

    /// Navigation that leaves this input is returned for the app to handle.
    pub fn handle_key(&mut self, key: IcKey, ctx: &InputContext) -> MathInputEvent {
        if ctx.is_shifted() {
            return self.handle_shifted_key(key);
        }
        if ctx.is_super() {
            return self.handle_super_key(key);
        }
        match key {
            IcKey::Num0 => Self::changed_from(self.insert_digit(0)),
            IcKey::Num1 => Self::changed_from(self.insert_digit(1)),
            IcKey::Num2 => Self::changed_from(self.insert_digit(2)),
            IcKey::Num3 => Self::changed_from(self.insert_digit(3)),
            IcKey::Num4 => Self::changed_from(self.insert_digit(4)),
            IcKey::Num5 => Self::changed_from(self.insert_digit(5)),
            IcKey::Num6 => Self::changed_from(self.insert_digit(6)),
            IcKey::Num7 => Self::changed_from(self.insert_digit(7)),
            IcKey::Num8 => Self::changed_from(self.insert_digit(8)),
            IcKey::Num9 => Self::changed_from(self.insert_digit(9)),
            IcKey::Func1 => Self::changed_from(self.buffer.backspace()),
            IcKey::Func2 => Self::changed_from(self.insert_char('/')),
            IcKey::Func3 => Self::changed_from(self.insert_char('*')),
            IcKey::Func4 => Self::changed_from(self.insert_char('-')),
            IcKey::Func5 => Self::changed_from(self.insert_char('+')),
            IcKey::Func6 => MathInputEvent::Submitted,
            IcKey::Shift | IcKey::Super | IcKey::_Max => MathInputEvent::Ignored,
        }
    }

    pub fn evaluate(&self) -> Result<f32, MathInputError> {
        if self.is_empty() {
            return Err(MathInputError::Empty);
        }

        let value = match self.mode {
            MathInputMode::Integer => self
                .text()
                .parse::<i32>()
                .map(|v| v as f32)
                .map_err(|_| MathInputError::InvalidValue)?,

            MathInputMode::Expression => exp_rs::interp(self.text(), None)
                .map(|v| v as f32)
                .map_err(|_| MathInputError::InvalidValue)?,
        };

        if !value.is_finite() {
            return Err(MathInputError::NonFinite);
        }

        Ok(value)
    }

    /// Parses the current input as a literal integer.
    pub fn value_i32(&self) -> Result<i32, MathInputError> {
        if self.is_empty() {
            return Err(MathInputError::Empty);
        }

        self.text()
            .parse::<i32>()
            .map_err(|_| MathInputError::InvalidValue)
    }

    fn text_scale(&self, text: &str, font_id: FontId) -> f32 {
        if text.is_empty() {
            return self.max_text_scale;
        }
        let available_width = self.size.x as f32 - Self::TEXT_MARGIN * 2.0;
        let max_width = crate::text::text_width(text, self.max_text_scale, font_id);
        if max_width <= available_width {
            return self.max_text_scale;
        }
        let scale = self.max_text_scale * available_width / max_width;
        scale.clamp(self.min_text_scale, self.max_text_scale)
    }

    pub fn draw(&self, platform: &mut dyn IcPlatform, has_focus: bool) {
        let fill_color = if has_focus {
            self.focused_background
        } else {
            self.background
        };

        platform.draw_rectangle(
            self.pos,
            self.pos + self.size,
            self.border.unwrap_or(fill_color),
            if self.border.is_some() { 1 } else { 0 },
            Some(fill_color),
        );

        let margin = Self::TEXT_MARGIN;
        let display_text = self.text();
        let font_id = FontId::Futural;
        let text_scale = self.text_scale(display_text, font_id);
        let text_x = self.pos.x as f32 + margin;
        let text_y = self.pos.y as f32 + margin;

        if !display_text.is_empty() {
            draw_text(
                platform,
                display_text,
                text_x,
                text_y + 13.0,
                text_scale,
                2.0,
                rgb8_hex(if has_focus { 0x000000 } else { 0xffffff }),
                font_id,
            );
        }

        if has_focus {
            let cursor_x = text_to_pos(
                &display_text,
                text_x,
                text_scale,
                self.buffer.cursor,
                font_id,
            );

            let text_center_y = text_y + 12.0;
            let cursor_height = 4.0 * text_scale;

            platform.draw_line(
                Vec2::new(cursor_x, text_center_y - cursor_height * 0.5),
                Vec2::new(cursor_x, text_center_y + cursor_height * 0.5),
                rgb8_hex(0xff0044),
                2,
            );
        }
    }

    fn changed_from(changed: bool) -> MathInputEvent {
        if changed {
            MathInputEvent::Changed
        } else {
            MathInputEvent::Handled
        }
    }

    fn handled_cursor_move(_moved: bool) -> MathInputEvent {
        MathInputEvent::Handled
    }

    fn handle_shifted_key(&mut self, key: IcKey) -> MathInputEvent {
        let ch = match key {
            IcKey::Num6 if self.mode == MathInputMode::Expression => Some('.'),
            IcKey::Num7 if self.mode == MathInputMode::Expression => Some('('),
            IcKey::Num8 if self.mode == MathInputMode::Expression => Some(')'),
            IcKey::Func6 if self.mode == MathInputMode::Expression => Some('^'),
            _ => None,
        };

        match ch {
            Some(ch) => Self::changed_from(self.insert_char(ch)),
            None => MathInputEvent::Ignored,
        }
    }

    fn handle_super_key(&mut self, key: IcKey) -> MathInputEvent {
        match key {
            IcKey::Num1 => Self::handled_cursor_move(self.buffer.move_cursor_end()),
            IcKey::Num4 => {
                if self.buffer.move_cursor_left() {
                    MathInputEvent::Handled
                } else {
                    MathInputEvent::Navigate(MathInputDirection::Left)
                }
            }
            IcKey::Num6 => {
                if self.buffer.move_cursor_right() {
                    MathInputEvent::Handled
                } else {
                    MathInputEvent::Navigate(MathInputDirection::Right)
                }
            }
            IcKey::Num7 => Self::handled_cursor_move(self.buffer.move_cursor_home()),
            IcKey::Num9 => Self::changed_from(self.buffer.clear()),
            IcKey::Num2 => MathInputEvent::Navigate(MathInputDirection::Down),
            IcKey::Num8 => MathInputEvent::Navigate(MathInputDirection::Up),
            _ => MathInputEvent::Ignored,
        }
    }

    fn char_allowed(&self, ch: char) -> bool {
        match self.mode {
            MathInputMode::Integer => {
                ch.is_ascii_digit()
                    || (ch == '-'
                        && self.buffer.cursor == 0
                        && !self.text().as_bytes().contains(&b'-'))
            }
            MathInputMode::Expression => Self::mode_char_allowed(self.mode, ch),
        }
    }

    fn mode_char_allowed(mode: MathInputMode, ch: char) -> bool {
        match mode {
            MathInputMode::Integer => ch.is_ascii_digit() || ch == '-',
            MathInputMode::Expression => {
                matches!(
                    ch,
                    '0'..='9' | '.' | '(' | ')' | '^' | '/' | '*' | '-' | '+'
                )
            }
        }
    }
}
