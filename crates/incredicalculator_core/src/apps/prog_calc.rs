use crate::app::IcApp;
use crate::app::InputContext;
use crate::input::{KeyState, IcKey};
use crate::platform::IcPlatform;
use crate::platform::Shape;
use alloc::{format, string::String};
use core::{num::ParseIntError, result};
use crate::text::{draw_text, draw_text_f, text_to_pos};
use glam::IVec2;
use rgb::*;

#[derive(Clone, Copy)]
struct EqEntry {
    equation: [u8; Self::EQUATION_MAX_SIZE],
    equation_len: usize,
    result: [u8; Self::EQUATION_MAX_SIZE],
    result_len: usize,
}

impl EqEntry {
    pub const EQUATION_MAX_SIZE: usize = 24;
    pub fn default() -> EqEntry {
        EqEntry {
            equation: [0; Self::EQUATION_MAX_SIZE],
            equation_len: 0,
            result: [0; Self::EQUATION_MAX_SIZE],
            result_len: 0,
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
    End,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusUi {
    Equation,
    BinaryWidget,
}




enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

const EQ_HISTORY_MAX: usize = 4;
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

pub struct ProgCalc {
    eq_history: [EqEntry; EQ_HISTORY_MAX],
    eq_history_len: usize,
    eq_history_write_idx: usize,
    current_eq: [u8; EqEntry::EQUATION_MAX_SIZE],
    pub current_eq_len: usize,
    current_result: [u8; EqEntry::EQUATION_MAX_SIZE],
    current_result_len: usize,
    cursor_pos: usize,
    history_selection_idx: Option<usize>, // none means youre editing the current equation
    focused_ui: FocusUi,
    binary_selection_idx: u8,
}

impl ProgCalc {
    pub fn new() -> ProgCalc {
        ProgCalc {
            eq_history: [EqEntry::default(); EQ_HISTORY_MAX],
            eq_history_len: 0,
            eq_history_write_idx: 0,
            current_eq: [0; EqEntry::EQUATION_MAX_SIZE],
            current_eq_len: 0,
            current_result: [0; EqEntry::EQUATION_MAX_SIZE],
            current_result_len: 0,
            cursor_pos: 0,
            history_selection_idx: None,
            focused_ui: FocusUi::Equation,
            binary_selection_idx: 0,
        }
    }

    fn get_action(key: IcKey, is_shifted: bool, is_super: bool) -> Option<KeyAction> {
        if is_shifted {
            match key {
                IcKey::Num0 => Some(KeyAction::InsertChar(b'A')),
                IcKey::Num1 => Some(KeyAction::InsertChar(b'B')),
                IcKey::Num2 => Some(KeyAction::InsertChar(b'C')),
                IcKey::Num3 => Some(KeyAction::InsertChar(b'D')),
                IcKey::Num4 => Some(KeyAction::InsertChar(b'E')),
                IcKey::Num5 => Some(KeyAction::InsertChar(b'F')),
                IcKey::Num6 => Some(KeyAction::InsertChar(b'%')),
                IcKey::Num7 => Some(KeyAction::InsertChar(b'<')),
                IcKey::Num8 => Some(KeyAction::InsertChar(b'>')),
                IcKey::Num9 => Some(KeyAction::Clear),
                IcKey::Func1 => Some(KeyAction::InsertChar(b'.')),
                IcKey::Func2 => Some(KeyAction::InsertChar(b'&')),
                IcKey::Func3 => Some(KeyAction::InsertChar(b'|')),
                IcKey::Func4 => Some(KeyAction::Backspace),
                IcKey::Func5 => Some(KeyAction::InsertChar(b'(')),
                IcKey::Func6 => Some(KeyAction::InsertChar(b')')),
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        } else if is_super {
            match key {
                IcKey::Num0 => Some(KeyAction::Backspace),
                IcKey::Num1 => Some(KeyAction::End),
                IcKey::Num2 => Some(KeyAction::MoveDown),
                IcKey::Num3 => Some(KeyAction::InsertChar(b'x')),
                IcKey::Num4 => Some(KeyAction::MoveLeft),
                IcKey::Num5 => None,
                IcKey::Num6 => Some(KeyAction::MoveRight),
                IcKey::Num7 => Some(KeyAction::Home),
                IcKey::Num8 => Some(KeyAction::MoveUp),
                IcKey::Num9 => None,
                IcKey::Func1 => Some(KeyAction::Delete),
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
                IcKey::Func1 => Some(KeyAction::InsertChar(b'^')),
                IcKey::Func2 => Some(KeyAction::InsertChar(b'/')),
                IcKey::Func3 => Some(KeyAction::InsertChar(b'*')),
                IcKey::Func4 => Some(KeyAction::InsertChar(b'-')),
                IcKey::Func5 => Some(KeyAction::InsertChar(b'+')),
                IcKey::Func6 => Some(KeyAction::Enter),
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        }
    }

    fn binary_widget_set_bit(&mut self, input_bit: bool) {
        let result_disp = core::str::from_utf8(&self.current_result[..self.current_result_len])
            .unwrap_or("Invalid UTF-8");
        let current_val = match result_disp.parse::<i32>() {
            Ok(s) => s,
            Err(_) => 0,
        };
        let mut temp_buf = [0u8; 20];
        let mut idx = 0;
        let mut new_val = match input_bit {
            true => current_val | (1 << self.binary_selection_idx),
            false => current_val & !(1 << self.binary_selection_idx),
        };
        if new_val == 0 {
            self.current_eq[0] = b'0';
            self.current_eq_len = 1;
            return;
        }
        while new_val > 0 {
            let digit = (new_val % 10) as u8;
            temp_buf[idx] = digit + b'0'; // 0-9 to '0'-'9'
            new_val /= 10;
            idx += 1;
        }
        for i in 0..idx {
            self.current_eq[i] = temp_buf[idx - 1 - i];
        }
        self.current_eq_len = idx;
    }

    fn ui_nav(&mut self, dir: NavDir) {
        match self.focused_ui {
            FocusUi::Equation => match dir {
                NavDir::Up => self.history_nav(true),
                NavDir::Down => {
                    if self.history_selection_idx.is_none() {
                        self.focused_ui = FocusUi::BinaryWidget;
                    } else {
                        self.history_nav(false);
                    }
                }
                NavDir::Left => self.move_cursor(false),
                NavDir::Right => self.move_cursor(true),
            },
            FocusUi::BinaryWidget => match dir {
                NavDir::Up => {
                    let in_top_row = self.binary_selection_idx > 15;
                    if in_top_row {
                        self.focused_ui = FocusUi::Equation;
                    } else {
                        self.binary_selection_idx += 16;
                    }
                }
                NavDir::Down => {
                    let in_bottom_row = self.binary_selection_idx <= 15;
                    if !in_bottom_row {
                        self.binary_selection_idx -= 16;
                    }
                }
                NavDir::Left => {
                    if self.binary_selection_idx < 31 {
                        self.binary_selection_idx += 1;
                    }
                }
                NavDir::Right => {
                    if self.binary_selection_idx > 0 {
                        self.binary_selection_idx -= 1;
                    }
                }
            },
        }
    }

    fn draw_rect(platform: &mut dyn IcPlatform, corner1: IVec2, corner2: IVec2, color: Rgb<u8>) {
        platform.draw_shape(Shape {
            start: corner1,
            end: IVec2 {
                x: corner1.x,
                y: corner2.y,
            },
            color: color,
        });
        platform.draw_shape(Shape {
            start: corner1,
            end: IVec2 {
                x: corner2.x,
                y: corner1.y,
            },
            color: color,
        });
        platform.draw_shape(Shape {
            start: IVec2 {
                x: corner2.x,
                y: corner1.y,
            },
            end: corner2,
            color: color,
        });
        platform.draw_shape(Shape {
            start: IVec2 {
                x: corner1.x,
                y: corner2.y,
            },
            end: corner2,
            color: color,
        });
    }

    fn insert_char_into_equation(&mut self, char_code: u8) {
        if self.cursor_pos < EqEntry::EQUATION_MAX_SIZE {
            for i in (self.cursor_pos..=(self.current_eq_len)).rev() {
                if i == EqEntry::EQUATION_MAX_SIZE - 1 {
                    break;
                }
                self.current_eq[i + 1] = self.current_eq[i];
            }
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
            if self.cursor_pos < self.current_eq_len {
                self.cursor_pos += 1;
            }
        } else {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
            }
        }
    }

    fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            for i in self.cursor_pos..self.current_eq_len {
                self.current_eq[i - 1] = self.current_eq[i];
            }
            self.cursor_pos -= 1;
            self.current_eq_len -= 1;
            self.current_eq[self.current_eq_len] = 0;
        }
    }

    fn backspace_del(&mut self) {
        // delete is the same thing as pressing right and then backspace
        self.move_cursor(true);
        self.backspace();
    }

    fn get_equation_answer(equation: &str) -> ([u8; EqEntry::EQUATION_MAX_SIZE], usize) {
        if equation.len() == 0 {
            return ([0; EqEntry::EQUATION_MAX_SIZE], 0);
        }
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
        (answer, answer_len)
    }

    fn run_equation(&mut self) {
        let (answer, answer_len) = Self::get_equation_answer(
            core::str::from_utf8(&self.current_eq[..self.current_eq_len])
                .unwrap_or("Invalid UTF-8"),
        );
        let new_hist_entry = EqEntry {
            equation: self.current_eq,
            equation_len: self.current_eq_len,
            result: answer,
            result_len: answer_len,
        };
        self.eq_history[self.eq_history_write_idx] = new_hist_entry;
        if self.eq_history_write_idx + 1 >= EQ_HISTORY_MAX {
            self.eq_history_write_idx = 0;
        } else {
            self.eq_history_write_idx += 1;
        }
        if self.eq_history_len + 1 <= EQ_HISTORY_MAX {
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
                    self.history_selection_idx = Some(self.eq_history_len - 1);
                }
            }
            Some(i) => {
                if up {
                    if i > 0 {
                        self.history_selection_idx = Some(i - 1);
                    }
                } else {
                    if (i + 1) < self.eq_history_len {
                        self.history_selection_idx = Some(i + 1);
                    } else {
                        self.history_selection_idx = None;
                    }
                }
            }
        }
    }

    fn history_nav_is_at_top(&self) -> bool {
        return self.history_selection_idx == Some(0);
    }

    fn history_nav_is_at_bottom(&self) -> bool {
        if self.eq_history_len == 0 {
            return false;
        }
        return self.history_selection_idx == Some(self.eq_history_len - 1);
    }

    fn copy_from_history(&mut self) {
        if self.history_selection_idx == None {
            return;
        }
        let idx = self.history_selection_idx.unwrap_or(0);
        let entry = self.eq_history[idx];
        self.current_eq = entry.equation;
        self.current_eq_len = entry.equation_len;
        self.history_selection_idx = None;
    }

    fn delete_current_history_entry(&mut self) {
        if self.history_selection_idx.is_none() {
            return;
        }
        if self.eq_history_len == 0 {
            self.history_selection_idx = None;
            return;
        }
        let prev_history_selection_idx = self.history_selection_idx;
        let n = EQ_HISTORY_MAX;
        let mut p = self.history_selection_idx.unwrap();
        let w = self.eq_history_write_idx;
        let most_recent = (w + n - 1) % n;
        while p != most_recent {
            let next = (p + 1) % n;
            self.eq_history[p] = self.eq_history[next];
            p = next;
        }
        self.eq_history[most_recent] = EqEntry::default();
        self.eq_history_write_idx = most_recent;
        if self.eq_history_len > 0 {
            self.eq_history_len -= 1;
        }
        if self.eq_history_len == 0 {
            self.history_selection_idx = None;
        } else {
            self.history_selection_idx = match prev_history_selection_idx {
                None => None,
                Some(i) => match i {
                    0 => Some(0),
                    x => Some(x - 1),
                },
            };
        }
    }

    // If we wanted to stick to heapless no_std, then we would use write!()
    // instead of format!() and make a struct with a buffer and cursor and
    // implement as_str and fmt::Write for that
    fn dec_str_to_hex_str(
        input: &str,
        include_prefix: bool,
        uppercase: bool,
        is_signed: bool,
    ) -> Result<String, ParseIntError> {
        let raw_bits: u32 = if is_signed {
            let val = input.parse::<i32>()?;
            val as u32
        } else {
            input.parse::<u32>()?
        };
        let hex_str = if uppercase {
            format!("{:08X}", raw_bits)
        } else {
            format!("{:08x}", raw_bits)
        };
        if include_prefix {
            Ok(format!("0x{}", hex_str))
        } else {
            Ok(hex_str)
        }
    }
}

impl IcApp for ProgCalc {
    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        let mut dirty: bool = false;
        for i in 0..(IcKey::COUNT) {
            if ctx.key_states[i].just_pressed {
                let key = unsafe { core::mem::transmute::<usize, IcKey>(i) };
                let action = Self::get_action(key, ctx.is_shifted(), ctx.is_super());
                match action {
                    Some(KeyAction::InsertChar(c)) => match self.focused_ui {
                        FocusUi::Equation => self.insert_char_into_equation(c),
                        FocusUi::BinaryWidget => self.binary_widget_set_bit(c != b'0'),
                    },
                    Some(KeyAction::Backspace) => {
                        if self.history_selection_idx.is_none() {
                            self.backspace()
                        } else {
                            self.delete_current_history_entry()
                        }
                    }
                    Some(KeyAction::Clear) => self.clear_eq(),
                    Some(KeyAction::Delete) => self.backspace_del(),
                    Some(KeyAction::Enter) => {
                        if self.history_selection_idx.is_none() {
                            self.run_equation();
                        } else {
                            self.copy_from_history();
                        }
                    }
                    Some(KeyAction::MoveUp) => self.ui_nav(NavDir::Up),
                    Some(KeyAction::MoveDown) => self.ui_nav(NavDir::Down),
                    Some(KeyAction::MoveLeft) => self.ui_nav(NavDir::Left),
                    Some(KeyAction::MoveRight) => self.ui_nav(NavDir::Right),
                    Some(KeyAction::Home) => self.move_cursor(false),
                    Some(KeyAction::End) => self.move_cursor(true),
                    None => (),
                }
                dirty = true;
            }
        }
        if dirty {
            (self.current_result, self.current_result_len) = Self::get_equation_answer(
                core::str::from_utf8(&self.current_eq[..self.current_eq_len])
                    .unwrap_or("Invalid UTF-8"),
            );
        }
    }
    
    fn update(&mut self, platform: &mut dyn IcPlatform, ctx: &InputContext){
        
        // draw_text_f(
        //     platform,
        //     format_args!("{}, b{}", self.focused_ui as u8, self.binary_selection_idx),
        //     0.0,
        //     0.0,
        //     2.0,
        //     Rgb {
        //         r: 0xff,
        //         g: 0xff,
        //         b: 0,
        //     },
        // );
        let mut draw_row: u32 = 0;
        let row_height: u32 = 40;
        let margin: u32 = 2;
        let font_size: f32 = 2.0;
        let max_entries_to_disp: u32 = 3;
        let num_entries_to_disp = core::cmp::min(self.eq_history_len as u32, max_entries_to_disp);
        for i in 0..num_entries_to_disp {
            let most_recent_phys_idx =
                (self.eq_history_write_idx + EQ_HISTORY_MAX - 1) % EQ_HISTORY_MAX;
            let phys_idx =
                (most_recent_phys_idx + EQ_HISTORY_MAX - i as usize) % EQ_HISTORY_MAX;
            let entry = &self.eq_history[phys_idx];
            let eq_disp = core::str::from_utf8(&entry.equation[..entry.equation_len])
                .unwrap_or("Invalid UTF-8");
            let base_y: u32 = 108;
            let y = base_y + margin - draw_row * row_height;
            draw_text(
                platform,
                eq_disp,
                margin as f32,
                y as f32,
                font_size,
                Rgb {
                    r: 0x99,
                    g: 0x99,
                    b: 0x99,
                },
            );
            if Some(phys_idx as usize) == self.history_selection_idx {
                draw_text(
                    platform,
                    "\x03",
                    (WIDTH - margin - 9) as f32,
                    y as f32,
                    font_size,
                    Rgb {
                        r: 0x99,
                        g: 0x99,
                        b: 0x99,
                    },
                );
            }
            let ans_disp =
                core::str::from_utf8(&entry.result[..entry.result_len]).unwrap_or("Invalid UTF-8");
            let line_height: u32 = 20;
            let y2 = base_y + line_height + margin - draw_row * row_height;
            draw_text(
                platform,
                "=",
                margin as f32,
                y2 as f32,
                font_size,
                Rgb {
                    r: 0x99,
                    g: 0x99,
                    b: 0x99,
                },
            );
            draw_text(
                platform,
                ans_disp,
                margin as f32 + 11.0,
                y2 as f32,
                font_size,
                Rgb {
                    r: 0xff,
                    g: 0xff,
                    b: 0x00,
                },
            );
            platform.draw_shape(Shape {
                start: IVec2 {
                    x: margin as i32,
                    y: y2 as i32 + 16,
                },
                end: IVec2 {
                    x: (WIDTH - margin) as i32,
                    y: y2 as i32 + 16,
                },
                color: Rgb {
                    r: 0x80,
                    g: 0x80,
                    b: 0x80,
                },
            });
            draw_row += 1;
        }
        let equation_disp = core::str::from_utf8(&self.current_eq[..self.current_eq_len])
            .unwrap_or("Invalid UTF-8");
        let eq_scale = match self.current_eq_len {
            x if x > 12 => 2.0,
            _ => 4.0,
        };
        let eq_y: f32 = 154.0;
        draw_text(
            platform,
            &equation_disp,
            margin as f32,
            eq_y,
            eq_scale,
            Rgb {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            },
        );
        let cursor_x_pos = text_to_pos(&equation_disp, margin as f32, eq_scale, self.cursor_pos);
        draw_text(
            platform,
            "|",
            cursor_x_pos as f32,
            eq_y,
            eq_scale,
            Rgb {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            },
        );
        let result_disp = core::str::from_utf8(&self.current_result[..self.current_result_len])
            .unwrap_or("Invalid UTF-8");
        let ans_scale = match self.current_result_len {
            x if x > 12 => 2.0,
            _ => 4.0,
        };
        draw_text(
            platform,
            "=",
            margin as f32,
            eq_y + 31.0,
            ans_scale,
            Rgb {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            },
        );
        draw_text(
            platform,
            &result_disp,
            (margin + 24) as f32,
            eq_y + 31.0,
            ans_scale,
            Rgb {
                r: 0xff,
                g: 0xff,
                b: 0xff,
            },
        );
        let (result_as_int, result_is_int) = match result_disp.parse::<i32>() {
            Ok(s) => (s, true),
            Err(_) => (0, false),
        };
        if result_is_int {
            // draw hex form of ans
            let (result_as_hex, hex_err) =
                match Self::dec_str_to_hex_str(&result_disp, true, true, true) {
                    Ok(s) => (s, false),
                    Err(_) => (String::new(), true),
                };
            if !hex_err {
                draw_text(
                    platform,
                    &result_as_hex,
                    margin as f32,
                    222.0,
                    2.0,
                    Rgb {
                        r: 0x00,
                        g: 0xff,
                        b: 0xff,
                    },
                );
            }
            // draw bin form of ans
            let bin_widget_bit1_x: i32 = 310;
            let bin_widget_bit1_y: i32 = 230;
            let bin_widget_element_w: i32 = 8;
            let bin_widget_element_margin: i32 = 3;
            for i in 0..32 {
                let bit_x: i32 = bin_widget_bit1_x
                    - ((i % 16) * (bin_widget_element_w + bin_widget_element_margin));
                let bit_y: i32 = if i < 16 {
                    bin_widget_bit1_y
                } else {
                    bin_widget_bit1_y - bin_widget_element_margin - bin_widget_element_w
                };
                let bit_val: bool = (result_as_int >> i) & 1 != 0;
                let color: Rgb<u8> = if i == self.binary_selection_idx as i32 && self.focused_ui == FocusUi::BinaryWidget {
                    Rgb {
                        r: 0x00,
                        g: 0xff,
                        b: 0x55,
                    }
                } else {
                    Rgb {
                        r: 0xff,
                        g: 0xff,
                        b: 0x00,
                    }
                };
                if bit_val {
                    platform.draw_shape(Shape {
                        start: IVec2 {
                            x: (bit_x + bin_widget_element_w / 2) as i32,
                            y: bit_y as i32,
                        },
                        end: IVec2 {
                            x: (bit_x + bin_widget_element_w / 2) as i32,
                            y: (bit_y + bin_widget_element_w) as i32,
                        },
                        color,
                    });
                } else {
                    Self::draw_rect(
                        platform,
                        IVec2 {
                            x: (bit_x + 1) as i32,
                            y: bit_y as i32,
                        },
                        IVec2 {
                            x: (bit_x + bin_widget_element_w - 1) as i32,
                            y: (bit_y + bin_widget_element_w) as i32,
                        },
                        color,
                    );
                }
            }
        }
    }

    
    
    fn on_enter(&mut self) {
        todo!()
    }
}