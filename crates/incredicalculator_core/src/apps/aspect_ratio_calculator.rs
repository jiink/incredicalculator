use crate::audio_engine;
use crate::fonts::FontId;
use crate::ui::{MathInput, MathInputDirection, MathInputEvent, MathInputMode};
use glam::{IVec2, Vec2};
use num_traits::{abs, clamp_max};
use rgb::RGB8;

use crate::{app::IcApp, text::draw_text};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FocusUi {
    Width1,
    Height1,
    Width2,
    Height2,
}
const INPUT_CHAR_LIMIT: usize = 16;
pub struct AspectRatioCalculator {
    focused_ui: FocusUi,
    input_box_width1: MathInput<INPUT_CHAR_LIMIT>,
    input_box_height1: MathInput<INPUT_CHAR_LIMIT>,
    input_box_width2: MathInput<INPUT_CHAR_LIMIT>,
    input_box_height2: MathInput<INPUT_CHAR_LIMIT>,
}

#[allow(dead_code)]
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
        let background = RGB8::new(0x1f, 0x5c, 0x55);
        let focused_background = RGB8::new(0xe8, 0xf2, 0x7a);
        let border = Some(RGB8::new(0x1c, 0x3e, 0x3a));
        AspectRatioCalculator {
            focused_ui: FocusUi::Width1,
            input_box_width1: MathInput::new(
                IVec2::new(17, 39),
                IVec2::new(129, 33),
                MathInputMode::Integer,
                2.0,
                10.0,
                background,
                focused_background,
                border,
            ),
            input_box_width2: MathInput::new(
                IVec2::new(174, 39),
                IVec2::new(129, 33),
                MathInputMode::Integer,
                2.0,
                10.0,
                background,
                focused_background,
                border,
            ),
            input_box_height1: MathInput::new(
                IVec2::new(17, 107),
                IVec2::new(129, 33),
                MathInputMode::Integer,
                2.0,
                10.0,
                background,
                focused_background,
                border,
            ),
            input_box_height2: MathInput::new(
                IVec2::new(174, 107),
                IVec2::new(129, 33),
                MathInputMode::Integer,
                2.0,
                10.0,
                background,
                focused_background,
                border,
            ),
        }
    }

    fn get_focused_input_box(&mut self) -> &mut MathInput<INPUT_CHAR_LIMIT> {
        match self.focused_ui {
            FocusUi::Width1 => &mut self.input_box_width1,
            FocusUi::Height1 => &mut self.input_box_height1,
            FocusUi::Width2 => &mut self.input_box_width2,
            FocusUi::Height2 => &mut self.input_box_height2,
        }
    }

    fn has_any_zeroes(&self) -> bool {
        self.input_box_width1.value_i32().unwrap_or(0) <= 0
            || self.input_box_height1.value_i32().unwrap_or(0) <= 0
            || self.input_box_width2.value_i32().unwrap_or(0) <= 0
            || self.input_box_height2.value_i32().unwrap_or(0) <= 0
    }

    fn update_math(&mut self) {
        // w1/h1 side is called ratio and w2/h2 side is called result.
        // Invalid or empty text is treated as zero until it can be parsed.
        let w1 = self.input_box_width1.value_i32().unwrap_or(0) as f32;
        let h1 = self.input_box_height1.value_i32().unwrap_or(0) as f32;
        let w2 = self.input_box_width2.value_i32().unwrap_or(0) as f32;
        let h2 = self.input_box_height2.value_i32().unwrap_or(0) as f32;
        if w1 == 0.0 || h1 == 0.0 {
            return;
        }

        let (target, value) = match self.focused_ui {
            FocusUi::Width1 => (&mut self.input_box_width2, (((w1 / h1) * h2) + 0.5) as i32),
            FocusUi::Height1 => (&mut self.input_box_height2, (((h1 / w1) * w2) + 0.5) as i32),
            FocusUi::Width2 => (&mut self.input_box_height2, (((h1 / w1) * w2) + 0.5) as i32),
            FocusUi::Height2 => (&mut self.input_box_width2, (((w1 / h1) * h2) + 0.5) as i32),
        };
        let text = alloc::format!("{}", value);
        let _ = target.set_text(&text);
    }

    fn handle_navigation(&mut self, direction: MathInputDirection) {
        let next_focus = match (self.focused_ui, direction) {
            (FocusUi::Width2, MathInputDirection::Left) => Some(FocusUi::Width1),
            (FocusUi::Height2, MathInputDirection::Left) => Some(FocusUi::Height1),
            (FocusUi::Width1, MathInputDirection::Right) => Some(FocusUi::Width2),
            (FocusUi::Height1, MathInputDirection::Right) => Some(FocusUi::Height2),
            (FocusUi::Height1, MathInputDirection::Up) => Some(FocusUi::Width1),
            (FocusUi::Height2, MathInputDirection::Up) => Some(FocusUi::Width2),
            (FocusUi::Width1, MathInputDirection::Down) => Some(FocusUi::Height1),
            (FocusUi::Width2, MathInputDirection::Down) => Some(FocusUi::Height2),
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
            Vec2::new(center_x as f32 - 1.0, top_left.y as f32 - 5.0),
            Vec2::new(antenna_l_pos.x as f32, antenna_l_pos.y as f32),
            base_col,
            2,
        );
        platform.draw_line(
            Vec2::new(center_x as f32 + 1.0, top_left.y as f32 - 5.0),
            Vec2::new(antenna_r_pos.x as f32, antenna_r_pos.y as f32),
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
        let width_to_height = self.input_box_width1.value_i32().unwrap_or(0) as f32
            / self.input_box_height1.value_i32().unwrap_or(1) as f32;
        let width = clamp_max((height as f32 * width_to_height) as i32, 320);
        let center_x = 320 / 2;
        let top_left = IVec2::new(center_x - width / 2, top_y);
        let bottom_right = IVec2::new(center_x + width / 2, top_y + height);
        self.draw_tv_frame(platform, top_left, bottom_right);
    }
}

impl IcApp for AspectRatioCalculator {
    fn name(&self) -> &str {
        "Aspect Ratio"
    }

    fn requires_realtime_updates(&self) -> bool {
        false
    }

    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, key: crate::input::IcKey, ctx: &crate::app::InputContext) {
        let event = self.get_focused_input_box().handle_key(key, ctx);
        match event {
            MathInputEvent::Changed => self.update_math(),
            MathInputEvent::Submitted => {
                self.focused_ui = match self.focused_ui {
                    FocusUi::Width1 => FocusUi::Height1,
                    FocusUi::Height1 => FocusUi::Width2,
                    FocusUi::Width2 => FocusUi::Height2,
                    FocusUi::Height2 => FocusUi::Width1,
                };
            }
            MathInputEvent::Navigate(direction) => self.handle_navigation(direction),
            MathInputEvent::Handled | MathInputEvent::Ignored => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn crate::platform::IcPlatform,
        _ctx: &crate::app::InputContext,
        _audio: &mut audio_engine::AudioEngine,
    ) {
        platform.clear(RGB8::new(0xff, 0xb3, 0x3f));
        draw_text(
            platform,
            "Aspect Ratio Calculator",
            5.0,
            12.0,
            6.0,
            2.0,
            RGB8::new(0, 0, 0),
            FontId::Futural,
        );
        self.input_box_width1
            .draw(platform, self.focused_ui == FocusUi::Width1);
        self.input_box_height1
            .draw(platform, self.focused_ui == FocusUi::Height1);
        self.input_box_width2
            .draw(platform, self.focused_ui == FocusUi::Width2);
        self.input_box_height2
            .draw(platform, self.focused_ui == FocusUi::Height2);
        platform.draw_line(
            Vec2::new(152.0, 83.0),
            Vec2::new((152 + 17) as f32, 83.0),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            Vec2::new(152.0, 93.0),
            Vec2::new((152 + 17) as f32, 93.0),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            Vec2::new(17.0, 89.0),
            Vec2::new((17 + 129) as f32, 89.0),
            RGB8::new(0, 0, 0),
            3,
        );
        platform.draw_line(
            Vec2::new(175.0, 89.0),
            Vec2::new((175 + 129) as f32, 89.0),
            RGB8::new(0, 0, 0),
            3,
        );
        self.draw_ratio_visualizer(platform);
    }
}
