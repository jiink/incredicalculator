use crate::app::InputContext;
use crate::fonts::{self, FontId};
use crate::input::IcKey;
use crate::platform::{rgb8_hex, IcPlatform};
use crate::text::{TextStyle, draw_text, text_to_pos, text_vertical_bounds};
use crate::ui::undo::{TextSnapshot, UndoHistory};
use glam::{IVec2, Vec2};
use rgb::RGB8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathInputMode {
    Integer,
    Expression,
    Hexadecimal,
    /// Integer bitwise expressions in the variable `t`, typed with the
    /// programmer calculator's keys plus Super+0 for `t`.
    Bytebeat,
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
        core::str::from_utf8(self.as_bytes()).unwrap_or("")
    }

    fn as_bytes(&self) -> &[u8] {
        &self.data[..self.len]
    }

    fn snapshot(&self) -> TextSnapshot<N> {
        TextSnapshot::new(self.as_bytes(), self.cursor)
    }

    fn restore(&mut self, snapshot: &TextSnapshot<N>) {
        self.data = snapshot.data;
        self.len = snapshot.len;
        self.cursor = snapshot.cursor;
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

    #[allow(dead_code)]
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
    undo_history: UndoHistory<N>,
    pub pos: IVec2,
    pub size: IVec2,
    min_text_scale: f32,
    max_text_scale: f32,
    background: RGB8,
    focused_background: RGB8,
    border: Option<RGB8>,
}

#[allow(dead_code)]
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
            undo_history: UndoHistory::new(),
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
            MathInputMode::Hexadecimal | MathInputMode::Bytebeat => {
                for byte in text.bytes() {
                    let ch = byte as char;
                    if !Self::mode_char_allowed(self.mode, ch) {
                        return Err(MathInputError::InvalidCharacter(ch));
                    }
                }
            }
        }

        self.buffer.set_bytes(text.as_bytes());
        self.undo_history.clear();
        Ok(())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.undo_history.clear();
    }

    /// Reverts the last edit made through `handle_key`. Returns false if
    /// there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.undo_history.pop() {
            Some(snapshot) => {
                self.buffer.restore(&snapshot);
                true
            }
            None => false,
        }
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

    /// Inserts all of `text`, or nothing if any of it doesn't fit or isn't
    /// allowed.
    pub fn insert_str(&mut self, text: &str) -> bool {
        if self.buffer.len + text.len() > N || !text.chars().all(|ch| self.char_allowed(ch)) {
            return false;
        }
        for byte in text.bytes() {
            self.buffer.insert_byte(byte);
        }
        true
    }

    pub fn insert_digit(&mut self, digit: u8) -> bool {
        if digit > 9 {
            return false;
        }

        self.insert_char((b'0' + digit) as char)
    }

    /// Navigation that leaves this input is returned for the app to handle.
    pub fn handle_key(&mut self, key: IcKey, ctx: &InputContext) -> MathInputEvent {
        if ctx.is_super() && !ctx.is_shifted() && key == IcKey::Num3 {
            return Self::changed_from(self.undo());
        }

        let before = self.buffer.snapshot();
        let event = self.apply_key(key, ctx);
        self.undo_history
            .record_if_changed(before, self.buffer.as_bytes());
        event
    }

    fn apply_key(&mut self, key: IcKey, ctx: &InputContext) -> MathInputEvent {
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
        let value = self.evaluate_f64()? as f32;
        if !value.is_finite() {
            return Err(MathInputError::NonFinite);
        }
        Ok(value)
    }

    /// Like [`evaluate`](Self::evaluate), at full precision.
    pub fn evaluate_f64(&self) -> Result<f64, MathInputError> {
        if self.is_empty() {
            return Err(MathInputError::Empty);
        }

        let value = match self.mode {
            MathInputMode::Integer => self
                .text()
                .parse::<i32>()
                .map(f64::from)
                .map_err(|_| MathInputError::InvalidValue)?,

            MathInputMode::Expression => exp_rs::interp(self.text(), None)
                .map(|v| v as f64)
                .map_err(|_| MathInputError::InvalidValue)?,

            MathInputMode::Hexadecimal => u32::from_str_radix(self.text(), 16)
                .map(f64::from)
                .map_err(|_| MathInputError::InvalidValue)?,

            MathInputMode::Bytebeat => bitwise_expr::evaluate(self.text())
                .map(|v| v as f64)
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

        let available_width = (self.size.x as f32 - Self::TEXT_MARGIN * 2.0).max(0.0);
        let available_height = (self.size.y as f32 - Self::TEXT_MARGIN * 2.0).max(0.0);
        let max_width = crate::text::text_width(text, self.max_text_scale, font_id);
        let width_scale = if max_width > available_width {
            self.max_text_scale * available_width / max_width
        } else {
            self.max_text_scale
        };
        let height_scale = text_vertical_bounds(text, font_id)
            .map(|(min_y, max_y)| {
                let font_scale = self.max_text_scale / fonts::get_font(font_id).units_per_em;
                let max_height = (max_y - min_y) * font_scale;
                if max_height > available_height {
                    self.max_text_scale * available_height / max_height
                } else {
                    self.max_text_scale
                }
            })
            .unwrap_or(self.max_text_scale);
        let width_scale = width_scale.min(self.max_text_scale);
        let height_scale = height_scale.min(self.max_text_scale);
        let scale = width_scale.min(height_scale);
        scale.max(self.min_text_scale.min(width_scale).min(height_scale))
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
        let text_center_y = self.pos.y as f32 + self.size.y as f32 * 0.5;
        let font_scale = text_scale / fonts::get_font(font_id).units_per_em;
        let text_bounds = text_vertical_bounds(display_text, font_id);
        let text_y = text_bounds
            .map(|(min_y, max_y)| text_center_y - (min_y + max_y) * 0.5 * font_scale)
            .unwrap_or(text_center_y);

        if !display_text.is_empty() {
            draw_text(
                platform,
                display_text,
                Vec2::new(text_x, text_y),
                TextStyle::new(
                    text_scale,
                    rgb8_hex(if has_focus { 0x000000 } else { 0xffffff }),
                )
                .font(font_id)
                .weight(0.4),
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

            let cursor_height = text_bounds
                .map(|(min_y, max_y)| (max_y - min_y) * font_scale)
                .unwrap_or(4.0 * text_scale)
                .min((self.size.y as f32 - Self::TEXT_MARGIN * 2.0).max(0.0));

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
        if self.mode == MathInputMode::Bytebeat {
            return self.handle_shifted_bytebeat_key(key);
        }
        let ch = match key {
            IcKey::Num0 if self.mode == MathInputMode::Hexadecimal => Some('A'),
            IcKey::Num1 if self.mode == MathInputMode::Hexadecimal => Some('B'),
            IcKey::Num2 if self.mode == MathInputMode::Hexadecimal => Some('C'),
            IcKey::Num3 if self.mode == MathInputMode::Hexadecimal => Some('D'),
            IcKey::Num4 if self.mode == MathInputMode::Hexadecimal => Some('E'),
            IcKey::Num5 if self.mode == MathInputMode::Hexadecimal => Some('F'),
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

    /// Matches the programmer calculator's shifted keys.
    fn handle_shifted_bytebeat_key(&mut self, key: IcKey) -> MathInputEvent {
        let token = match key {
            IcKey::Num0 => "A",
            IcKey::Num1 => "B",
            IcKey::Num2 => "C",
            IcKey::Num3 => "D",
            IcKey::Num4 => "E",
            IcKey::Num5 => "F",
            IcKey::Num6 => "~",
            IcKey::Num7 => "(",
            IcKey::Num8 => ")",
            IcKey::Num9 => "0x",
            IcKey::Func1 => "&",
            IcKey::Func2 => "|",
            IcKey::Func3 => "%",
            IcKey::Func4 => "<<",
            IcKey::Func5 => ">>",
            IcKey::Func6 => "^",
            IcKey::Shift | IcKey::Super | IcKey::_Max => return MathInputEvent::Ignored,
        };
        Self::changed_from(self.insert_str(token))
    }

    fn handle_super_key(&mut self, key: IcKey) -> MathInputEvent {
        match key {
            IcKey::Num0 if self.mode == MathInputMode::Bytebeat => {
                Self::changed_from(self.insert_char('t'))
            }
            IcKey::Num0 => {
                self.insert_char('.');
                MathInputEvent::Changed
            },
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
            MathInputMode::Hexadecimal | MathInputMode::Bytebeat => {
                Self::mode_char_allowed(self.mode, ch)
            }
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
            MathInputMode::Hexadecimal => ch.is_ascii_hexdigit(),
            MathInputMode::Bytebeat => ch.is_ascii_hexdigit() || "tx()~&|%<>^/*-+".contains(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{KeyMode, KeyState};

    fn input() -> MathInput<8> {
        MathInput::new(
            IVec2::ZERO,
            IVec2::new(40, 20),
            MathInputMode::Expression,
            1.0,
            1.0,
            RGB8::default(),
            RGB8::default(),
            None,
        )
    }

    fn press_with<const N: usize>(
        input: &mut MathInput<N>,
        modifiers: &[IcKey],
        key: IcKey,
    ) -> MathInputEvent {
        let mut states = [KeyState::default(); IcKey::COUNT];
        for &modifier in modifiers {
            states[modifier as usize].is_down = true;
        }
        let ctx = InputContext {
            key_states: &states,
            key_mode: KeyMode::Normal,
            shift_engaged: false,
            shift_disengaged: false,
        };
        input.handle_key(key, &ctx)
    }

    fn undo(input: &mut MathInput<8>) -> MathInputEvent {
        press_with(input, &[IcKey::Super], IcKey::Num3)
    }

    #[test]
    fn super_num3_undoes_edits_in_reverse_order() {
        let mut input = input();
        press_with(&mut input, &[], IcKey::Num1);
        press_with(&mut input, &[], IcKey::Num2);
        press_with(&mut input, &[IcKey::Shift], IcKey::Num7);
        press_with(&mut input, &[], IcKey::Func1);
        press_with(&mut input, &[IcKey::Super], IcKey::Num4);
        press_with(&mut input, &[IcKey::Super], IcKey::Num9);
        assert_eq!(input.text(), "");

        assert_eq!(undo(&mut input), MathInputEvent::Changed);
        assert_eq!(input.text(), "12");
        assert_eq!(input.cursor(), 1);
        undo(&mut input);
        assert_eq!(input.text(), "12(");
        assert_eq!(input.cursor(), 3);
        undo(&mut input);
        assert_eq!(input.text(), "12");
        undo(&mut input);
        assert_eq!(input.text(), "1");
        undo(&mut input);
        assert_eq!(input.text(), "");
        assert_eq!(undo(&mut input), MathInputEvent::Handled);
    }

    #[test]
    fn rejected_keys_are_not_undo_steps() {
        let mut input = input();
        press_with(&mut input, &[], IcKey::Num5);
        // backspace at the start changes nothing
        press_with(&mut input, &[IcKey::Super], IcKey::Num7);
        press_with(&mut input, &[], IcKey::Func1);
        undo(&mut input);
        assert_eq!(input.text(), "");
    }

    #[test]
    fn programmatic_set_text_and_clear_reset_history() {
        let mut input = input();
        press_with(&mut input, &[], IcKey::Num1);
        input.set_text("42").unwrap();
        assert_eq!(undo(&mut input), MathInputEvent::Handled);
        assert_eq!(input.text(), "42");

        press_with(&mut input, &[], IcKey::Num3);
        input.clear();
        assert_eq!(undo(&mut input), MathInputEvent::Handled);
        assert_eq!(input.text(), "");
    }

    #[test]
    fn bytebeat_mode_uses_programmer_keys() {
        let mut input: MathInput<16> = MathInput::new(
            IVec2::ZERO,
            IVec2::new(40, 20),
            MathInputMode::Bytebeat,
            1.0,
            1.0,
            RGB8::default(),
            RGB8::default(),
            None,
        );
        press_with(&mut input, &[IcKey::Super], IcKey::Num0);
        press_with(&mut input, &[IcKey::Shift], IcKey::Func5);
        press_with(&mut input, &[], IcKey::Num7);
        press_with(&mut input, &[IcKey::Shift], IcKey::Func2);
        press_with(&mut input, &[IcKey::Shift], IcKey::Num9);
        press_with(&mut input, &[IcKey::Shift], IcKey::Num5);
        press_with(&mut input, &[], IcKey::Func3);
        press_with(&mut input, &[IcKey::Super], IcKey::Num0);
        assert_eq!(
            press_with(&mut input, &[IcKey::Shift], IcKey::Func6),
            MathInputEvent::Changed
        );
        assert_eq!(input.text(), "t>>7|0xF*t^");
        assert_eq!(
            press_with(&mut input, &[], IcKey::Func6),
            MathInputEvent::Submitted
        );

        // A two-character token is all or nothing when space runs out.
        input.set_text("t>>7|0xF*t^1234").unwrap();
        press_with(&mut input, &[IcKey::Shift], IcKey::Func4);
        assert_eq!(input.text(), "t>>7|0xF*t^1234");
    }
}
