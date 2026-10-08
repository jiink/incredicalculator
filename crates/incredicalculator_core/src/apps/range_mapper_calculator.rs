use crate::audio_engine;
use crate::fonts::FontId;
use crate::input::IcKey;
use crate::ui::{MathInput, MathInputDirection, MathInputEvent, MathInputMode};
use crate::{
    app::IcApp,
    platform::{IcPlatform, rgb8_hex},
    text::{draw_text, draw_text_f},
};
use glam::IVec2;

const INPUT_CHAR_LIMIT: usize = 24;

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
    input_box_in_val: MathInput<INPUT_CHAR_LIMIT>,
    input_box_in_min: MathInput<INPUT_CHAR_LIMIT>,
    input_box_in_max: MathInput<INPUT_CHAR_LIMIT>,
    input_box_out_min: MathInput<INPUT_CHAR_LIMIT>,
    input_box_out_max: MathInput<INPUT_CHAR_LIMIT>,
    answer: f32,
}

impl RangeMapperCalculator {
    pub fn new() -> RangeMapperCalculator {
        let background = rgb8_hex(0x3A9AFF);
        let focused_background = rgb8_hex(0xF1FF5E);
        RangeMapperCalculator {
            focused_ui: FocusUi::InValue,
            input_box_in_val: MathInput::new(
                IVec2::new(50, 7),
                IVec2::new(122, 36),
                MathInputMode::Expression,
                2.0,
                10.0,
                background,
                focused_background,
                None,
            ),
            input_box_in_min: MathInput::new(
                IVec2::new(50, 56),
                IVec2::new(122, 36),
                MathInputMode::Expression,
                2.0,
                10.0,
                background,
                focused_background,
                None,
            ),
            input_box_in_max: MathInput::new(
                IVec2::new(188, 56),
                IVec2::new(122, 36),
                MathInputMode::Expression,
                2.0,
                10.0,
                background,
                focused_background,
                None,
            ),
            input_box_out_min: MathInput::new(
                IVec2::new(50, 105),
                IVec2::new(122, 36),
                MathInputMode::Expression,
                2.0,
                10.0,
                background,
                focused_background,
                None,
            ),
            input_box_out_max: MathInput::new(
                IVec2::new(188, 105),
                IVec2::new(122, 36),
                MathInputMode::Expression,
                2.0,
                10.0,
                background,
                focused_background,
                None,
            ),
            answer: 0.0,
        }
    }

    fn get_focused_input_box(&mut self) -> &mut MathInput<INPUT_CHAR_LIMIT> {
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
        let x = self.input_box_in_val.evaluate().unwrap_or(0.0);
        let a = self.input_box_in_min.evaluate().unwrap_or(0.0);
        let b = self.input_box_in_max.evaluate().unwrap_or(0.0);
        let c = self.input_box_out_min.evaluate().unwrap_or(0.0);
        let d = self.input_box_out_max.evaluate().unwrap_or(0.0);
        self.answer = c + ((x - a) * (d - c) / (b - a));
    }

    fn handle_navigation(&mut self, direction: MathInputDirection) {
        let next_focus = match (self.focused_ui, direction) {
            (FocusUi::InValue, MathInputDirection::Right) => Some(FocusUi::InMax),
            (FocusUi::InValue, MathInputDirection::Down) => Some(FocusUi::InMin),
            (FocusUi::InMin, MathInputDirection::Left) => Some(FocusUi::InValue),
            (FocusUi::InMin, MathInputDirection::Right) => Some(FocusUi::InMax),
            (FocusUi::InMin, MathInputDirection::Up) => Some(FocusUi::InValue),
            (FocusUi::InMin, MathInputDirection::Down) => Some(FocusUi::OutMin),
            (FocusUi::InMax, MathInputDirection::Left) => Some(FocusUi::InMin),
            (FocusUi::InMax, MathInputDirection::Up) => Some(FocusUi::InValue),
            (FocusUi::InMax, MathInputDirection::Down) => Some(FocusUi::OutMax),
            (FocusUi::OutMin, MathInputDirection::Right) => Some(FocusUi::OutMax),
            (FocusUi::OutMin, MathInputDirection::Up) => Some(FocusUi::InMin),
            (FocusUi::OutMax, MathInputDirection::Left) => Some(FocusUi::OutMin),
            (FocusUi::OutMax, MathInputDirection::Up) => Some(FocusUi::InMax),
            _ => None,
        };

        if let Some(next_focus) = next_focus {
            self.focused_ui = next_focus;
            match direction {
                MathInputDirection::Left => self.get_focused_input_box().end(),
                MathInputDirection::Right => self.get_focused_input_box().home(),
                MathInputDirection::Up | MathInputDirection::Down => {}
            }
        }
    }
}

impl IcApp for RangeMapperCalculator {
    fn name(&self) -> &str {
        "Range Mapper"
    }

    fn requires_realtime_updates(&self) -> bool {
        false
    }

    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: IcKey, ctx: &crate::app::InputContext) {
        match self.get_focused_input_box().handle_key(key, ctx) {
            MathInputEvent::Changed => self.update_math(),
            MathInputEvent::Submitted => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::InValue => FocusUi::InMin,
                    FocusUi::InMin => FocusUi::InMax,
                    FocusUi::InMax => FocusUi::OutMin,
                    FocusUi::OutMin => FocusUi::OutMax,
                    FocusUi::OutMax => FocusUi::InValue,
                };
            }
            MathInputEvent::Navigate(direction) => self.handle_navigation(direction),
            MathInputEvent::Handled | MathInputEvent::Ignored => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        _ctx: &crate::app::InputContext,
        _audio: &mut audio_engine::AudioEngine,
    ) {
        platform.clear(rgb8_hex(0x1C0770));
        self.input_box_in_val
            .draw(platform, self.focused_ui == FocusUi::InValue);
        self.input_box_in_min
            .draw(platform, self.focused_ui == FocusUi::InMin);
        self.input_box_in_max
            .draw(platform, self.focused_ui == FocusUi::InMax);
        self.input_box_out_min
            .draw(platform, self.focused_ui == FocusUi::OutMin);
        self.input_box_out_max
            .draw(platform, self.focused_ui == FocusUi::OutMax);
        draw_text_f(
            platform,
            format_args!("{}", self.answer),
            49.0,
            175.0,
            10.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
        draw_text(
            platform,
            "Map",
            3.0,
            24.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
        draw_text(
            platform,
            "from",
            3.0,
            73.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
        draw_text(
            platform,
            "to",
            3.0,
            119.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
        draw_text(
            platform,
            "=",
            32.0,
            167.0,
            6.0,
            2.0,
            rgb8_hex(0xffffff),
            FontId::Futural,
        );
        draw_text(
            platform,
            "X = C + ((X-A)*(D-C) / B-A)",
            3.0,
            227.0,
            6.0,
            2.0,
            rgb8_hex(0x3A9AFF),
            FontId::Futural,
        )
    }
}
