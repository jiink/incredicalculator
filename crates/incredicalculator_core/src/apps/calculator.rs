use crate::app::IcApp;
use crate::app::InputContext;
use crate::input::{IcKey, KeyState};
use crate::platform;
use crate::platform::IcPlatform;
use crate::platform::Shape;
use crate::text::{draw_text, draw_text_f, text_to_pos};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::{format, string::String};
use core::str::FromStr;
use core::{num::ParseIntError, result};
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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    Mode
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusUi {
    Equation,
    Widget,
}

enum NavDir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EngineMode {
    Programmer,
    Scientific
}

const EQ_HISTORY_MAX: usize = 4;
const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct LineBuffer<const N: usize> {
    pub data: [u8; N],
    pub len: usize,
    pub cursor: usize,
}

impl<const N: usize> LineBuffer<N> {
    pub fn default() -> Self {
        Self {
            data: [0; N],
            len: 0,
            cursor: 0,
        }
    }

    pub fn insert_char(&mut self, char_code: u8) {
        if self.cursor < EqEntry::EQUATION_MAX_SIZE {
            for i in (self.cursor..=(self.len)).rev() {
                if i == EqEntry::EQUATION_MAX_SIZE - 1 {
                    break;
                }
                self.data[i + 1] = self.data[i];
            }
            self.data[self.cursor] = char_code;
            self.cursor += 1;
            self.len += 1;
            if self.len > EqEntry::EQUATION_MAX_SIZE {
                self.len = EqEntry::EQUATION_MAX_SIZE;
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
}

trait CalcEngine {
    fn evaluate(&self, equation: &str) -> String;
    fn draw_widgets(&self, platform: &mut dyn IcPlatform, result_str: &str);
    // true means this CalcEngine consumed the input
    fn on_widget_key(
        &mut self,
        key: KeyAction,
        buffer: &mut LineBuffer<{ EqEntry::EQUATION_MAX_SIZE }>,
        current_result: &str,
    ) -> bool;
    fn has_widget(&self) -> bool;
}

pub struct ScientificEngine {}

impl ScientificEngine {
    pub fn default() -> Self {
        Self {}
    }
}

impl CalcEngine for ScientificEngine {
    fn evaluate(&self, equation: &str) -> String {
        match exp_rs::interp(equation, None) {
            Ok(v) => format!("{}", v),
            Err(msg) => msg.to_string(),
        }
    }

    fn draw_widgets(&self, platform: &mut dyn IcPlatform, _result_str: &str) {
        draw_text(
            platform,
            "Scientific",
            2.0,
            222.0,
            2.0,
            Rgb {
                r: 0x44,
                g: 0x44,
                b: 0x44,
            },
        );
    }

    fn on_widget_key(
        &mut self,
        _key: KeyAction,
        _buffer: &mut LineBuffer<{ EqEntry::EQUATION_MAX_SIZE }>,
        _current_result: &str,
    ) -> bool {
        false
    }

    fn has_widget(&self) -> bool {
        false
    }
}

pub struct ProgrammerEngine {
    binary_selection_idx: u8,
}

impl ProgrammerEngine {
    pub fn default() -> Self {
        Self {
            binary_selection_idx: 0,
        }
    }
    fn binary_widget_set_bit(
        &self,
        bit_idx: u8,
        buffer: &mut LineBuffer<24>,
        current_result: &str,
    ) {
        let current_val = match current_result.parse::<i32>() {
            Ok(s) => s,
            Err(_) => 0,
        };
        let new_val = current_val ^ (1 << bit_idx);
        let new_str = format!("{}", new_val);
        buffer.set_content(new_str.as_bytes());
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
}

impl CalcEngine for ProgrammerEngine {
    fn evaluate(&self, equation: &str) -> String {
        bitwise_expr::evaluate_str(equation)
    }
    fn has_widget(&self) -> bool {
        true
    }
    fn on_widget_key(
        &mut self,
        key: KeyAction,
        buffer: &mut LineBuffer<{ EqEntry::EQUATION_MAX_SIZE }>,
        current_result: &str,
    ) -> bool {
        match key {
            KeyAction::MoveUp => {
                let in_top_row = self.binary_selection_idx > 15;
                if in_top_row {
                    // go back to focus equation
                    return false;
                } else {
                    self.binary_selection_idx += 16;
                }
            }
            KeyAction::MoveDown => {
                let in_bottom_row = self.binary_selection_idx <= 15;
                if !in_bottom_row {
                    self.binary_selection_idx -= 16;
                }
            }
            KeyAction::MoveLeft => {
                if self.binary_selection_idx < 31 {
                    self.binary_selection_idx += 1;
                }
            }
            KeyAction::MoveRight => {
                if self.binary_selection_idx > 0 {
                    self.binary_selection_idx -= 1;
                }
            }
            KeyAction::InsertChar(c) => match c {
                b'0'..=b'9' => {
                    let set = c != b'0';
                    let current_val = current_result.parse::<i32>().unwrap_or(0);
                    let is_set = (current_val & (1 << self.binary_selection_idx)) != 0;
                    if set != is_set {
                        self.binary_widget_set_bit(
                            self.binary_selection_idx,
                            buffer,
                            current_result,
                        );
                    }
                }
                _ => {}
            },
            KeyAction::Enter => {
                self.binary_widget_set_bit(self.binary_selection_idx, buffer, current_result);
            }
            _ => return false,
        }
        true
    }

    fn draw_widgets(&self, platform: &mut dyn IcPlatform, result_str: &str) {
        let margin = 2;
        let (result_as_int, result_is_int) = match result_str.parse::<i32>() {
            Ok(s) => (s, true),
            Err(_) => (0, false),
        };
        if result_is_int {
            // draw hex form of ans
            let (result_as_hex, hex_err) =
                match Self::dec_str_to_hex_str(&result_str, true, true, true) {
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
                let color: Rgb<u8> = if i == self.binary_selection_idx as i32 {
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
        } else {
            draw_text(
                platform,
                "Programmer",
                margin as f32,
                222.0,
                2.0,
                Rgb {
                    r: 0x44,
                    g: 0x44,
                    b: 0x44,
                },
            );
        }
    }
}

pub struct Calculator {
    current_eq: LineBuffer<{ EqEntry::EQUATION_MAX_SIZE }>,
    eq_history: [EqEntry; EQ_HISTORY_MAX],
    eq_history_len: usize,
    eq_history_write_idx: usize,
    current_result: [u8; EqEntry::EQUATION_MAX_SIZE],
    current_result_len: usize,
    history_selection_idx: Option<usize>, // none means youre editing the current equation
    focused_ui: FocusUi,
    engine: Box<dyn CalcEngine>,
    engine_mode: EngineMode
}

impl Calculator {
    pub fn new() -> Calculator {
        Calculator {
            current_eq: LineBuffer::default(),
            eq_history: [EqEntry::default(); EQ_HISTORY_MAX],
            eq_history_len: 0,
            eq_history_write_idx: 0,
            current_result: [0; EqEntry::EQUATION_MAX_SIZE],
            current_result_len: 0,
            history_selection_idx: None,
            focused_ui: FocusUi::Equation,
            engine: Box::new(ProgrammerEngine::default()),
            engine_mode: EngineMode::Programmer
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
                IcKey::Num6 => Some(KeyAction::InsertChar(b'.')),
                IcKey::Num7 => Some(KeyAction::InsertChar(b'(')),
                IcKey::Num8 => Some(KeyAction::InsertChar(b')')),
                IcKey::Num9 => Some(KeyAction::InsertChar(b'x')),
                IcKey::Func1 => Some(KeyAction::InsertChar(b'&')),
                IcKey::Func2 => Some(KeyAction::InsertChar(b'|')),
                IcKey::Func3 => Some(KeyAction::InsertChar(b'%')),
                IcKey::Func4 => Some(KeyAction::InsertChar(b'<')),
                IcKey::Func5 => Some(KeyAction::InsertChar(b'>')),
                IcKey::Func6 => None,
                IcKey::Shift => None,
                IcKey::Super => None,
                IcKey::_Max => None,
            }
        } else if is_super {
            match key {
                IcKey::Num0 => None,
                IcKey::Num1 => Some(KeyAction::End),
                IcKey::Num2 => Some(KeyAction::MoveDown),
                IcKey::Num3 => Some(KeyAction::Clear),
                IcKey::Num4 => Some(KeyAction::MoveLeft),
                IcKey::Num5 => Some(KeyAction::Mode),
                IcKey::Num6 => Some(KeyAction::End),
                IcKey::Num7 => Some(KeyAction::Home),
                IcKey::Num8 => Some(KeyAction::MoveUp),
                IcKey::Num9 => Some(KeyAction::Backspace),
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

    fn ui_nav(&mut self, dir: NavDir) {
        match self.focused_ui {
            FocusUi::Equation => match dir {
                NavDir::Up => self.history_nav(true),
                NavDir::Down => {
                    if self.history_selection_idx.is_none() {
                        if self.engine.has_widget() {
                            self.focused_ui = FocusUi::Widget;
                        }
                    } else {
                        self.history_nav(false);
                    }
                }
                NavDir::Left => self.current_eq.move_cursor(false),
                NavDir::Right => self.current_eq.move_cursor(true),
            },
            FocusUi::Widget => {}
        }
    }

    fn run_equation(&mut self) {
        if self.current_eq.len == 0 {
            return;
        }
        let answer_str = self.engine.evaluate(self.current_eq.as_str());
        let mut new_hist_entry = EqEntry {
            equation: self.current_eq.data,
            equation_len: self.current_eq.len,
            result: [0; EqEntry::EQUATION_MAX_SIZE],
            result_len: 0,
        };
        Self::copy_str_to_buffer(
            &mut new_hist_entry.result,
            &mut new_hist_entry.result_len,
            &answer_str,
        );
        self.history_append(&new_hist_entry);
        self.current_eq.clear();
        self.current_result_len = 0;
    }

    fn update_realtime_result(&mut self) {
        let eq_str = self.current_eq.as_str();
        if eq_str.is_empty() {
            self.current_result_len = 0;
            return;
        }
        let answer_str = self.engine.evaluate(eq_str);
        Self::copy_str_to_buffer(
            &mut self.current_result,
            &mut self.current_result_len,
            &answer_str,
        );
    }

    fn copy_str_to_buffer(buffer: &mut [u8], len: &mut usize, s: &str) {
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(buffer.len());
        buffer[..copy_len].copy_from_slice(&bytes[..copy_len]);
        *len = copy_len;
    }

    fn history_append(&mut self, new_entry: &EqEntry) {
        self.eq_history[self.eq_history_write_idx] = *new_entry;
        self.eq_history_write_idx = (self.eq_history_write_idx + 1) % EQ_HISTORY_MAX;
        if self.eq_history_len < EQ_HISTORY_MAX {
            self.eq_history_len += 1;
        }
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

    fn copy_from_history(&mut self) {
        if let Some(idx) = self.history_selection_idx {
            let entry = self.eq_history[idx];
            self.current_eq.data = entry.equation;
            self.current_eq.len = entry.equation_len;
            self.history_selection_idx = None;
        }
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

    fn draw_history(&self, platform: &mut dyn IcPlatform) {
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
            let phys_idx = (most_recent_phys_idx + EQ_HISTORY_MAX - i as usize) % EQ_HISTORY_MAX;
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
    }

    fn draw_editor(&self, platform: &mut dyn IcPlatform) {
        let margin: u32 = 2;
        let equation_disp = core::str::from_utf8(&self.current_eq.data[..self.current_eq.len])
            .unwrap_or("Invalid UTF-8");
        let eq_scale = match self.current_eq.len {
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
        let cursor_x_pos = text_to_pos(
            &equation_disp,
            margin as f32,
            eq_scale,
            self.current_eq.cursor,
        );
        draw_text(
            platform,
            "|",
            cursor_x_pos as f32 - 3.0,
            eq_y,
            eq_scale,
            Rgb {
                r: 0xff,
                g: 0xff,
                b: 0x44,
            },
        );

        // draw result -------------

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
    }
}

impl IcApp for Calculator {
    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        let action = Self::get_action(key, ctx.is_shifted(), ctx.is_super());
        if let Some(act) = action {
            if self.focused_ui == FocusUi::Widget {
                let current_result_str =
                    core::str::from_utf8(&self.current_result[..self.current_result_len])
                        .unwrap_or("0");
                let handled =
                    self.engine
                        .on_widget_key(act, &mut self.current_eq, current_result_str);
                if !handled {
                    if act == KeyAction::MoveUp {
                        self.focused_ui = FocusUi::Equation;
                    }
                } else {
                    self.update_realtime_result();
                    return;
                }
            }
            match act {
                KeyAction::InsertChar(c) => self.current_eq.insert_char(c),
                KeyAction::Backspace => {
                    if self.history_selection_idx.is_none() {
                        self.current_eq.backspace()
                    } else {
                        self.delete_current_history_entry()
                    }
                }
                KeyAction::Clear => self.current_eq.clear(),
                KeyAction::Delete => self.current_eq.backspace_del(),
                KeyAction::Enter => {
                    if self.history_selection_idx.is_none() {
                        self.run_equation();
                    } else {
                        self.copy_from_history();
                    }
                }
                KeyAction::MoveUp => self.ui_nav(NavDir::Up),
                KeyAction::MoveDown => self.ui_nav(NavDir::Down),
                KeyAction::MoveLeft => self.ui_nav(NavDir::Left),
                KeyAction::MoveRight => self.ui_nav(NavDir::Right),
                KeyAction::Home => self.current_eq.move_cursor(false),
                KeyAction::End => self.current_eq.move_cursor(true),
                KeyAction::Mode => {
                    match self.engine_mode {
                        EngineMode::Programmer => {
                            self.engine_mode = EngineMode::Scientific;
                            self.engine = Box::new(ScientificEngine::default());
                        }
                        EngineMode::Scientific => {
                            self.engine_mode = EngineMode::Programmer;
                            self.engine = Box::new(ProgrammerEngine::default());
                        }
                    }
                    self.focused_ui = FocusUi::Equation;
                } 
            }
            self.update_realtime_result();
        }
    }

    fn update(&mut self, platform: &mut dyn IcPlatform, _ctx: &InputContext) {
        self.draw_history(platform);
        self.draw_editor(platform);
        let result_str =
            core::str::from_utf8(&self.current_result[..self.current_result_len]).unwrap_or("0");
        self.engine.draw_widgets(platform, result_str);
    }

    fn on_enter(&mut self) {
        todo!()
    }
}
