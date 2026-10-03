
use crate::app::{IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::fonts::FontId;
use crate::input::IcKey;
use crate::platform::{IcPlatform, CANVAS_WIDTH};
use crate::text::{draw_text, draw_text_f};
use crate::ui::{MathInput, MathInputDirection, MathInputEvent, MathInputMode};
use alloc::format;
use glam::IVec2;
use rgb::RGB8;

const INPUT_CAPACITY: usize = 6;
const LOCAL_COLOR_BG_0: RGB8 = RGB8::new(0x20, 0x2a, 0x2d);
const LOCAL_COLOR_BG_1: RGB8 = RGB8::new(0x1f, 0x5c, 0x55);
const LOCAL_COLOR_BG_FOCUS_0: RGB8 = RGB8::new(0xe8, 0xf2, 0x7a);
const LOCAL_COLOR_BORDER_0: RGB8 = RGB8::new(0x12, 0x3b, 0x38);
const LOCAL_COLOR_FG_0: RGB8 = RGB8::new(0xf4, 0xf1, 0xdf);
const LOCAL_COLOR_FG_SUBTLE_0: RGB8 = RGB8::new(0x91, 0xd8, 0xca);

#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryMode {
    Rgb,
    Unit,
    Hex,
    Rgb565,
}

pub struct ColorCalculator {
    mode: EntryMode,
    focused_channel: usize,
    channel_inputs: [MathInput<INPUT_CAPACITY>; 3],
    unit_inputs: [MathInput<INPUT_CAPACITY>; 3],
    hex_input: MathInput<INPUT_CAPACITY>,
    rgb565_input: MathInput<INPUT_CAPACITY>,
    color: RGB8,
}

impl ColorCalculator {
    fn make_input(pos: IVec2, size: IVec2, mode: MathInputMode) -> MathInput<INPUT_CAPACITY> {
        MathInput::new(
            pos,
            size,
            mode,
            4.0,
            8.0,
            LOCAL_COLOR_BG_1,
            LOCAL_COLOR_BG_FOCUS_0,
            Some(LOCAL_COLOR_BORDER_0),
        )
    }

    pub fn new() -> Self {
        let color = RGB8::new(0x33, 0x66, 0x99);
        let y = 74;
        let mut calculator = Self {
            mode: EntryMode::Rgb,
            focused_channel: 0,
            channel_inputs: [
                Self::make_input(IVec2::new(8, y), IVec2::new(50, 28), MathInputMode::Integer),
                Self::make_input(IVec2::new(66, y), IVec2::new(50, 28), MathInputMode::Integer),
                Self::make_input(IVec2::new(124, y), IVec2::new(50, 28), MathInputMode::Integer),
            ],
            unit_inputs: [
                Self::make_input(IVec2::new(8, y), IVec2::new(50, 28), MathInputMode::Expression),
                Self::make_input(IVec2::new(66, y), IVec2::new(50, 28), MathInputMode::Expression),
                Self::make_input(IVec2::new(124, y), IVec2::new(50, 28), MathInputMode::Expression),
            ],
            hex_input: Self::make_input(
                IVec2::new(8, y),
                IVec2::new(166, 28),
                MathInputMode::Hexadecimal,
            ),
            rgb565_input: Self::make_input(
                IVec2::new(8, y),
                IVec2::new(166, 28),
                MathInputMode::Hexadecimal,
            ),
            color,
        };
        calculator.sync_inputs();
        calculator
    }

    fn focused_input_mut(&mut self) -> &mut MathInput<INPUT_CAPACITY> {
        match self.mode {
            EntryMode::Rgb => &mut self.channel_inputs[self.focused_channel],
            EntryMode::Unit => &mut self.unit_inputs[self.focused_channel],
            EntryMode::Hex => &mut self.hex_input,
            EntryMode::Rgb565 => &mut self.rgb565_input,
        }
    }

    fn sync_inputs(&mut self) {
        let channels = [self.color.r, self.color.g, self.color.b];
        for (index, channel) in channels.into_iter().enumerate() {
            let _ = self.channel_inputs[index].set_text(&format!("{channel}"));
            let _ = self.unit_inputs[index].set_text(&format!("{:.3}", channel as f32 / 255.0));
        }
        let _ = self.hex_input.set_text(&format!("{:06X}", self.rgb24()));
        let _ = self.rgb565_input.set_text(&format!("{:04X}", self.to_rgb565()));
    }

    fn set_mode(&mut self, mode: EntryMode) {
        self.mode = mode;
        self.focused_channel = 0;
        self.sync_inputs();
    }

    fn update_color_from_input(&mut self) {
        match self.mode {
            EntryMode::Rgb => {
                let value = self.channel_inputs[self.focused_channel]
                    .value_i32()
                    .unwrap_or(0);
                let clamped_value = value.clamp(0, 255);
                if value != clamped_value {
                    let _ = self.channel_inputs[self.focused_channel]
                        .set_text(&format!("{clamped_value}"));
                }
                let channel = clamped_value as u8;
                match self.focused_channel {
                    0 => self.color.r = channel,
                    1 => self.color.g = channel,
                    _ => self.color.b = channel,
                }
            }
            EntryMode::Unit => {
                let value = self.unit_inputs[self.focused_channel]
                    .evaluate()
                    .unwrap_or(0.0);
                let clamped_value = value.clamp(0.0, 1.0);
                if value != clamped_value {
                    let _ = self.unit_inputs[self.focused_channel]
                        .set_text(&format!("{clamped_value:.3}"));
                }
                let channel = (clamped_value * 255.0 + 0.5) as u8;
                match self.focused_channel {
                    0 => self.color.r = channel,
                    1 => self.color.g = channel,
                    _ => self.color.b = channel,
                }
            }
            EntryMode::Hex => {
                let value = u32::from_str_radix(self.hex_input.text(), 16).unwrap_or(0);
                self.color = RGB8::new(
                    (value >> 16) as u8,
                    (value >> 8) as u8,
                    value as u8,
                );
            }
            EntryMode::Rgb565 => {
                let value = u32::from_str_radix(self.rgb565_input.text(), 16).unwrap_or(0);
                self.color = Self::from_rgb565(value.min(u16::MAX as u32) as u16);
            }
        }
    }

    fn handle_navigation(&mut self, direction: MathInputDirection) {
        if matches!(self.mode, EntryMode::Rgb | EntryMode::Unit) {
            match direction {
                MathInputDirection::Left if self.focused_channel > 0 => {
                    self.focused_channel -= 1;
                }
                MathInputDirection::Right if self.focused_channel < 2 => {
                    self.focused_channel += 1;
                }
                _ => {}
            }
        }
    }

    fn rgb24(&self) -> u32 {
        ((self.color.r as u32) << 16) | ((self.color.g as u32) << 8) | self.color.b as u32
    }

    fn to_rgb565(&self) -> u16 {
        let red = (self.color.r as u16 * 31 + 127) / 255;
        let green = (self.color.g as u16 * 63 + 127) / 255;
        let blue = (self.color.b as u16 * 31 + 127) / 255;
        (red << 11) | (green << 5) | blue
    }

    fn from_rgb565(value: u16) -> RGB8 {
        let red = ((value >> 11) & 0x1f) as u8;
        let green = ((value >> 5) & 0x3f) as u8;
        let blue = (value & 0x1f) as u8;
        RGB8::new(
            (red << 3) | (red >> 2),
            (green << 2) | (green >> 4),
            (blue << 3) | (blue >> 2),
        )
    }

    fn draw_input(&self, platform: &mut dyn IcPlatform) {
        let y_offset = 60.0;
        platform.draw_rectangle(
            IVec2::new(0, y_offset as i32 + 12),
            IVec2::new(CANVAS_WIDTH as i32 - 1, y_offset as i32 + 44),
            LOCAL_COLOR_FG_0,
            2,
            Some(self.color),
        );
        let mode_label = match self.mode {
            EntryMode::Rgb => "RGB 0-255",
            EntryMode::Unit => "NORMALIZED CHANNELS 0-1",
            EntryMode::Hex => "HEX 0x",
            EntryMode::Rgb565 => "RGB565 0x",
        };
        draw_text(platform, mode_label, 8.0, y_offset, 6.0, 2.0, LOCAL_COLOR_FG_0, FontId::Futural);

        match self.mode {
            EntryMode::Rgb => {
                for i in 0..3 {
                    self.channel_inputs[i].draw(platform, i == self.focused_channel);
                }
            }
            EntryMode::Unit => {
                for i in 0..3 {
                    self.unit_inputs[i].draw(platform, i == self.focused_channel);
                }
            }
            EntryMode::Hex => self.hex_input.draw(platform, true),
            EntryMode::Rgb565 => self.rgb565_input.draw(platform, true),
        }
    }
}

impl IcApp for ColorCalculator {
    fn name(&self) -> &str {
        "Color Converter"
    }

    fn requires_realtime_updates(&self) -> bool {
        false
    }

    fn on_enter(&mut self) {}

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        if ctx.is_super() && !ctx.is_shifted() {
            let mode = match key {
                IcKey::Func1 => Some(EntryMode::Rgb),
                IcKey::Func2 => Some(EntryMode::Unit),
                IcKey::Func3 => Some(EntryMode::Hex),
                IcKey::Func4 => Some(EntryMode::Rgb565),
                _ => None,
            };
            if let Some(mode) = mode {
                self.set_mode(mode);
                return;
            }
        }

        let event = self.focused_input_mut().handle_key(key, ctx);
        match event {
            MathInputEvent::Changed => self.update_color_from_input(),
            MathInputEvent::Submitted => {
                if matches!(self.mode, EntryMode::Rgb | EntryMode::Unit) {
                    self.focused_channel = (self.focused_channel + 1) % 3;
                }
            }
            MathInputEvent::Navigate(direction) => self.handle_navigation(direction),
            MathInputEvent::Handled | MathInputEvent::Ignored => {}
        }
    }

    fn update(&mut self, platform: &mut dyn IcPlatform, _ctx: &InputContext, _audio: &mut AudioEngine) {
        platform.clear(LOCAL_COLOR_BG_0);
        draw_text(platform, self.name(), 8.0, 15.0, 6.0, 2.0, LOCAL_COLOR_FG_0, FontId::Futural);
        draw_text(
            platform,
            "Press F1-F4",
            8.0,
            35.0,
            6.0,
            2.0,
            LOCAL_COLOR_FG_SUBTLE_0,
            FontId::Futural,
        );
        self.draw_input(platform);
        let conv_y_offset = 124.0;
        draw_text_f(
            platform,
            format_args!("HEX 0x{:06X}", self.rgb24()),
            8.0,
            conv_y_offset,
            6.0,
            2.0,
            LOCAL_COLOR_FG_0,
            FontId::Futural,
        );
        draw_text_f(
            platform,
            format_args!("DEC ({}, {}, {})", self.color.r, self.color.g, self.color.b),
            8.0,
            conv_y_offset + 20.0,
            6.0,
            2.0,
            LOCAL_COLOR_FG_0,
            FontId::Futural,
        );
        draw_text_f(
            platform,
            format_args!(
                "FLOAT ({:.3}, {:.3}, {:.3})",
                self.color.r as f32 / 255.0,
                self.color.g as f32 / 255.0,
                self.color.b as f32 / 255.0,
            ),
            8.0,
            conv_y_offset + 40.0,
            6.0,
            2.0,
            LOCAL_COLOR_FG_0,
            FontId::Futural,
        );
        draw_text_f(
            platform,
            format_args!("RGB565 0x{:04X}", self.to_rgb565()),
            8.0,
            conv_y_offset + 60.0,
            6.0,
            2.0,
            LOCAL_COLOR_FG_0,
            FontId::Futural,
        );
    }
}

