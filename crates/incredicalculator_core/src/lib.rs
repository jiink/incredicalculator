#![no_std]

use crate::text::draw_text;
mod text;

// IC stands for Incredicalculator

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
#[repr(usize)]
pub enum IcKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    NumA,
    NumB,
    NumC,
    NumD,
    NumE,
    NumF,
    Shift,
    Super,
    _Max
}

impl IcKey {
    pub const COUNT: usize = IcKey::_Max as usize;
    fn get_action(&self, is_shifted: bool, is_super: bool) -> Option<KeyAction> {
        if is_shifted {
            match self {
                Self::Num0 => Some(KeyAction::InsertChar(b'.')),
                Self::Num1 => Some(KeyAction::InsertChar(b'&')),
                Self::Num2 => Some(KeyAction::InsertChar(b'|')),
                Self::Num3 => Some(KeyAction::InsertChar(b'x')),
                Self::Num4 => Some(KeyAction::InsertChar(b'(')),
                Self::Num5 => Some(KeyAction::InsertChar(b')')),
                Self::Num6 => Some(KeyAction::InsertChar(b'%')),
                Self::Num7 => Some(KeyAction::InsertChar(b'<')),
                Self::Num8 => Some(KeyAction::InsertChar(b'>')),
                Self::Num9 => Some(KeyAction::Clear),
                Self::NumA => Some(KeyAction::InsertChar(b'^')),
                Self::NumB => Some(KeyAction::InsertChar(b'/')),
                Self::NumC => Some(KeyAction::InsertChar(b'*')),
                Self::NumD => Some(KeyAction::InsertChar(b'-')),
                Self::NumE => Some(KeyAction::InsertChar(b'+')),
                Self::NumF => Some(KeyAction::Enter),
                Self::Shift => None,
                Self::Super => None,
                Self::_Max => None
            }
        } else if is_super {
            match self {
                Self::Num0 => Some(KeyAction::Backspace),
                Self::Num1 => Some(KeyAction::End),
                Self::Num2 => Some(KeyAction::MoveDown),
                Self::Num3 => None,
                Self::Num4 => Some(KeyAction::MoveLeft),
                Self::Num5 => None,
                Self::Num6 => Some(KeyAction::MoveRight),
                Self::Num7 => Some(KeyAction::Home),
                Self::Num8 => Some(KeyAction::MoveUp),
                Self::Num9 => None,
                Self::NumA => Some(KeyAction::Delete),
                Self::NumB => None,
                Self::NumC => None,
                Self::NumD => None,
                Self::NumE => None,
                Self::NumF => None,
                Self::Shift => None,
                Self::Super => None,
                Self::_Max => None
            }
        } else {
            match self {
                Self::Num0 => Some(KeyAction::InsertChar(b'0')),
                Self::Num1 => Some(KeyAction::InsertChar(b'1')),
                Self::Num2 => Some(KeyAction::InsertChar(b'2')),
                Self::Num3 => Some(KeyAction::InsertChar(b'3')),
                Self::Num4 => Some(KeyAction::InsertChar(b'4')),
                Self::Num5 => Some(KeyAction::InsertChar(b'5')),
                Self::Num6 => Some(KeyAction::InsertChar(b'6')),
                Self::Num7 => Some(KeyAction::InsertChar(b'7')),
                Self::Num8 => Some(KeyAction::InsertChar(b'8')),
                Self::Num9 => Some(KeyAction::InsertChar(b'9')),
                Self::NumA => Some(KeyAction::InsertChar(b'A')),
                Self::NumB => Some(KeyAction::InsertChar(b'B')),
                Self::NumC => Some(KeyAction::InsertChar(b'C')),
                Self::NumD => Some(KeyAction::InsertChar(b'D')),
                Self::NumE => Some(KeyAction::InsertChar(b'E')),
                Self::NumF => Some(KeyAction::InsertChar(b'F')),
                Self::Shift => None,
                Self::Super => None,
                Self::_Max => None
            }
        }
    }
}

enum KeyAction {
    InsertChar(u8),
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Backspace,
    Delete,
    Enter,
    Clear,
    Home,
    End
}

pub trait IcPlatform {
    fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2:f32);
    fn clear_lines(&mut self);
}

#[derive(Clone, Copy)]
struct EqEntry {
    equation: [u8; Self::EQUATION_MAX_SIZE],
    equation_len: usize,
    result: [u8; Self::EQUATION_MAX_SIZE],
    result_len: usize
}

impl EqEntry {
    pub const EQUATION_MAX_SIZE: usize = 160;
    pub fn default() -> EqEntry {
        EqEntry {
            equation: [0; Self::EQUATION_MAX_SIZE],
            equation_len: 0,
            result: [0; Self::EQUATION_MAX_SIZE],
            result_len: 0
        }
    }
}

pub struct IcState {
    key_states: [KeyState; IcKey::COUNT],
    eq_history: [EqEntry; Self::EQ_HISTORY_MAX],
    eq_history_len: usize,
    eq_history_write_idx: usize,
    current_eq: [u8; EqEntry::EQUATION_MAX_SIZE],
    current_eq_len: usize,
    cursor_pos: usize,
    history_selection_idx: Option<usize> // none means youre editing the current equation
}

#[derive(Clone, Copy)]
pub struct KeyState {
    pub is_down: bool,
    pub was_down: bool,
    pub just_pressed: bool,
    pub just_released: bool
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState { is_down: false, was_down: false, just_pressed: false, just_released: false }
    }
}

impl IcState {
    pub const EQ_HISTORY_MAX: usize = 32;
    const WIDTH: u32 = 320;
    const HEIGHT: u32 = 240;

    pub fn new() -> IcState {
        IcState {
            key_states: [KeyState::default(); IcKey::COUNT],
            eq_history: [EqEntry::default(); Self::EQ_HISTORY_MAX],
            eq_history_len: 0,
            eq_history_write_idx: 0,
            current_eq: [0; EqEntry::EQUATION_MAX_SIZE],
            current_eq_len: 0,
            cursor_pos: 0,
            history_selection_idx: None
        }
    }

    pub fn update(&mut self, platform: &mut impl IcPlatform) {
        platform.clear_lines();
        for s in self.key_states.iter_mut() {
            s.just_pressed = s.is_down && !s.was_down;
            s.just_released = !s.is_down && s.was_down;
            s.was_down = s.is_down;
        }
        
        let is_shifted = self.key_states[IcKey::Shift as usize].is_down;
        let is_super = self.key_states[IcKey::Super as usize].is_down;
        for i in 0..(IcKey::COUNT) {
            if self.key_states[i].just_pressed {
                let key = unsafe { core::mem::transmute::<usize, IcKey>(i) };
                let action = key.get_action(is_shifted, is_super);
                match action {
                    Some(KeyAction::InsertChar(c)) => self.insert_char_into_equation(c),
                    Some(KeyAction::Backspace) =>  self.backspace(),
                    Some(KeyAction::Clear) => self.clear_eq(),
                    Some(KeyAction::Delete) => self.backspace_del(),
                    Some(KeyAction::Enter) => if self.history_selection_idx.is_none() { self.run_equation(); } else { self.copy_from_history(); }
                    Some(KeyAction::MoveUp) => self.history_nav(true),
                    Some(KeyAction::MoveDown) => self.history_nav(false),
                    Some(KeyAction::MoveLeft) => self.move_cursor(false),
                    Some(KeyAction::MoveRight) => self.move_cursor(true),
                    Some(KeyAction::Home) => self.move_cursor(false),
                    Some(KeyAction::End) => self.move_cursor(true),
                    None => ()
                }
            }
        }
        if self.key_states[IcKey::Super as usize].just_pressed {
            self.backspace();
        }
        let mut draw_row: u32 = 0;
        let row_height: u32 = 40;
        let margin: u32 = 2;
        let font_size: f32 = 2.0;
        for i in 0..(self.eq_history_len as u32) {
            let most_recent_phys_idx = (self.eq_history_write_idx + Self::EQ_HISTORY_MAX - 1) % Self::EQ_HISTORY_MAX;
            let phys_idx = (most_recent_phys_idx + Self::EQ_HISTORY_MAX - i as usize) % Self::EQ_HISTORY_MAX;
            let entry = &self.eq_history[phys_idx];
            let eq_disp = core::str::from_utf8(&entry.equation[..entry.equation_len]).unwrap_or("Invalid UTF-8");
            let y = 120 + margin - draw_row * row_height;
            draw_text(platform, eq_disp, margin as f32, y as f32, font_size);
            let ans_disp = core::str::from_utf8(&entry.result[..entry.result_len]).unwrap_or("Invalid UTF-8");
            let y2 = 140 + margin - draw_row * row_height;
            draw_text(platform, "=", margin as f32, y2 as f32, font_size);
            draw_text(platform, ans_disp, margin as f32 + 11.0, y2 as f32, font_size);
            platform.draw_line(margin as f32, y2 as f32 + 16.0, (Self::WIDTH - margin) as f32, y2 as f32 + 16.0);
            draw_row += 1;
        }
        let equation_disp = core::str::from_utf8(&self.current_eq[..self.current_eq_len]).unwrap_or("Invalid UTF-8");
        draw_text(platform, &equation_disp, margin as f32, 170.0, 4.0);
    }

    pub fn key_down(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            ()
        }
        self.key_states[key as usize].is_down = true;
    }

    pub fn key_up(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            ()
        }
        self.key_states[key as usize].is_down = false;
    }

    fn insert_char_into_equation(&mut self, char_code: u8) {
        if self.cursor_pos < EqEntry::EQUATION_MAX_SIZE {
            self.current_eq[self.cursor_pos] = char_code;
            self.cursor_pos += 1;
            self.current_eq_len += 1;
            if self.current_eq_len > EqEntry::EQUATION_MAX_SIZE {
                self.current_eq_len = EqEntry::EQUATION_MAX_SIZE;
            }
        }
    }

    fn move_cursor(&mut self, right: bool) {
        if right {
            if self.cursor_pos < self.current_eq_len - 1 {
                self.cursor_pos += 1;
            }
        } else {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            }
        }
    }

    fn backspace(&mut self) {
        if self.cursor_pos > 0 && self.current_eq_len > 0 {
            
            for i in self.cursor_pos..(self.current_eq_len - 1) {
                self.current_eq[i] = self.current_eq[i+1];
            }
            self.cursor_pos -= 1;
            self.current_eq_len -= 1;
        }
    }

    fn backspace_del(&mut self) {
        // delete is the same thing as pressing right and then backspace
        self.move_cursor(true);
        self.backspace();
    }

    fn run_equation(&mut self) {
        let equation = core::str::from_utf8(&self.current_eq[..self.current_eq_len]).unwrap_or("Invalid UTF-8");
        let mut answer = [b'\0'; EqEntry::EQUATION_MAX_SIZE];
        let mut answer_len: usize = 0;
        match bitwise_expr::evaluate(equation) {
            Ok(result) => {
                let mut buf = itoa::Buffer::new();
                let b_slice = buf.format(result).as_bytes();
                answer_len = b_slice.len();
                answer[0..b_slice.len()].copy_from_slice(b_slice);
            }
            Err(msg) => {
                let msg_bytes = msg.as_bytes();
                let len_to_copy = core::cmp::min(answer.len(), msg_bytes.len());
                let dest_slice = &mut answer[..len_to_copy];
                dest_slice.copy_from_slice(&msg_bytes[..len_to_copy]);
                if len_to_copy < answer.len() {
                    answer[len_to_copy..].fill(0);
                    answer_len = len_to_copy;
                }
            }
        }
        let new_hist_entry = EqEntry {
            equation: self.current_eq,
            equation_len: self.current_eq_len,
            result: answer,
            result_len: answer_len
        };
        self.eq_history[self.eq_history_write_idx] = new_hist_entry;
        if self.eq_history_write_idx + 1 >= Self::EQ_HISTORY_MAX {
            self.eq_history_write_idx = 0;
        } else {
            self.eq_history_write_idx += 1;
        }
        if self.eq_history_len + 1 <= Self::EQ_HISTORY_MAX {
            self.eq_history_len += 1;
        }
        self.clear_eq();
    }

    fn clear_eq(&mut self) {
        self.current_eq = [0; EqEntry::EQUATION_MAX_SIZE];
        self.current_eq_len = 0;
        self.cursor_pos = 0;
    }

    fn history_nav(&mut self, up: bool) {
        if self.eq_history_len == 0 {
            self.history_selection_idx = None;
            return;
        }
        match self.history_selection_idx {
            None => {
                if up && self.eq_history_len > 0 {
                    self.history_selection_idx = Some(0);
                }
            },
            Some(i) => {
                if up {
                    if (i + 1) < self.eq_history_len {
                        self.history_selection_idx = Some(i + 1);
                    }
                } else {
                    if i <= 0 {
                        self.history_selection_idx = None;
                    } else {
                        self.history_selection_idx = Some(i - 1);
                    }
                }
            }
        }
    }

    fn copy_from_history(&mut self) {

    }
}
