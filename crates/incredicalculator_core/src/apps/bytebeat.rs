use super::music::{self, BG_COLOR, CELL_COLOR, DIM_TEXT_COLOR, SOUNDING_CELL_COLOR, TEXT_COLOR};
use crate::app::{IcApp, InputContext};
use crate::audio_engine::{AudioEngine, SAMPLE_RATE_HZ};
use crate::fonts::FontId;
use crate::graphics::{self, Vitmap};
use crate::input::IcKey;
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{self, HAlign, TextStyle};
use crate::ui::{MathInput, MathInputDirection, MathInputEvent, MathInputMode};
use bitwise_expr::{IntFormat, Program};
use glam::{IVec2, Vec2};
use rgb::RGB8;

const EXPRESSION_LEN: usize = 64;
/// Like `int t` in the C programs bytebeat came from.
const FORMAT: IntFormat = IntFormat::new(32, true);
/// Rates `t` can count at. 8 kHz is the classic one.
const T_RATES_HZ: [u32; 6] = [8_000, 11_025, 16_000, 22_050, 32_000, 44_100];
/// Bytebeat output is full-scale and harsh, so play it at half level.
const OUTPUT_ATTENUATION_SHIFT: u32 = 1;
/// Larger is a lower cutoff for the filter that removes DC offset.
const DC_FILTER_SHIFT: u32 = 8;

const PRESETS: [&str; 7] = [
    "t*(42&t>>10)",
    "10*(t>>7|t|t>>6)+4*(t&t>>13|t>>6)",
    "t*((t>>12|t>>8)&63&t>>4)",
    "(t*(t>>5|t>>8))>>(t>>16)",
    "t*(t>>11&t>>8&123&t>>3)",
    "t*5&t>>7|t*3&t*4>>10",
    "(t>>6|t|t>>(t>>16))*10+(t>>11&7)",
];

const EDITOR_LEFT: i32 = 6;
const EDITOR_RIGHT: i32 = 256;
const EDITOR_TOP: i32 = 26;
const EDITOR_BOTTOM: i32 = 90;
const EDITOR_MARGIN: f32 = 5.0;
const EDITOR_LINE_SPACING: f32 = 14.0;
const EDITOR_MAX_LINES: usize = 4;
const SCOPE_TOP: i32 = 114;
const SCOPE_BOTTOM: i32 = 196;
const SCOPE_POINTS: u32 = 125;

const ERROR_COLOR: RGB8 = rgb8_hex(0xff5c7a);
const PLAYING_COLOR: RGB8 = rgb8_hex(0x5cff9d);
const CURSOR_COLOR: RGB8 = SOUNDING_CELL_COLOR;

/// Plays an integer expression of `t` as audio, where `t` counts up at a
/// fixed rate and the low byte of the result is the next sample.
pub struct Bytebeat {
    input: MathInput<EXPRESSION_LEN>,
    program: Option<Program>,
    playing: bool,
    parse_failed: bool,
    preset_idx: usize,
    rate_idx: usize,
    t: u32,
    /// Accumulates `t`'s rate each output sample; `t` advances on overflow
    /// past the output rate.
    phase: u32,
    /// The `t` that `value` was computed for.
    value_t: Option<u32>,
    value: u8,
    /// Running average of the output in 1/256ths, removed so a constant
    /// level doesn't sit on the speaker.
    dc_q8: i32,
}

impl Bytebeat {
    pub fn new() -> Bytebeat {
        let mut input = MathInput::new(
            IVec2::new(EDITOR_LEFT, EDITOR_TOP),
            IVec2::new(EDITOR_RIGHT - EDITOR_LEFT, EDITOR_BOTTOM - EDITOR_TOP),
            MathInputMode::Bytebeat,
            1.0,
            1.0,
            CELL_COLOR,
            CELL_COLOR,
            None,
        );
        let _ = input.set_text(PRESETS[0]);
        Self {
            input,
            program: None,
            playing: false,
            parse_failed: false,
            preset_idx: 0,
            rate_idx: 0,
            t: 0,
            phase: 0,
            value_t: None,
            value: 0x80,
            dc_q8: 0,
        }
    }

    /// Plays the expression being edited. While already playing, `t` keeps
    /// counting so edits can be heard live.
    fn play(&mut self) {
        match Program::compile(self.input.text(), FORMAT) {
            Ok(program) => {
                if !self.playing {
                    self.restart();
                }
                self.program = Some(program);
                self.value_t = None;
                self.playing = true;
                self.parse_failed = false;
            }
            Err(_) => self.parse_failed = true,
        }
    }

    fn toggle_playing(&mut self) {
        if self.playing {
            self.playing = false;
        } else {
            self.play();
        }
    }

    fn restart(&mut self) {
        self.t = 0;
        self.phase = 0;
        self.value_t = None;
    }

    fn load_preset(&mut self, forward: bool) {
        self.preset_idx = music::cycle(self.preset_idx, PRESETS.len(), forward);
        let _ = self.input.set_text(PRESETS[self.preset_idx]);
        self.parse_failed = false;
        if self.playing {
            self.play();
        }
    }

    fn sample_at(program: &Program, t: u32) -> u8 {
        // Division by zero plays as silence rather than stopping.
        program.eval(t as i64).unwrap_or(0) as u8
    }

    fn draw(&self, platform: &mut dyn IcPlatform, ctx: &InputContext) {
        platform.clear(BG_COLOR);
        text::draw_text(
            platform,
            "Bytebeat",
            Vec2::new(8.0, 16.0),
            TextStyle::new(6.0, TEXT_COLOR),
        );

        self.draw_editor(platform);

        let (status, status_color) = if self.parse_failed {
            ("Can't parse that", ERROR_COLOR)
        } else if self.playing {
            ("Playing", PLAYING_COLOR)
        } else {
            ("Stopped", DIM_TEXT_COLOR)
        };
        text::draw_text(
            platform,
            status,
            Vec2::new(8.0, 104.0),
            TextStyle::new(4.5, status_color),
        );
        text::draw_text_f(
            platform,
            format_args!("t = {}", self.t),
            Vec2::new(EDITOR_RIGHT as f32, 104.0),
            TextStyle::new(4.0, DIM_TEXT_COLOR).align(HAlign::Right),
        );

        self.draw_scope(platform);

        let hint = if ctx.is_super() {
            "0: t   5: restart   8/2: presets"
        } else if ctx.is_shifted() {
            "0-5: A-F  6: ~  7/8: ( )  9: 0x"
        } else {
            "Super+0: t   Super+8/2: presets"
        };
        text::draw_text(
            platform,
            hint,
            Vec2::new(8.0, 214.0),
            TextStyle::new(3.5, DIM_TEXT_COLOR).thickness(1.0),
        );
        text::draw_text_f(
            platform,
            format_args!("t rate: {} Hz", T_RATES_HZ[self.rate_idx]),
            Vec2::new(8.0, 230.0),
            TextStyle::new(3.5, DIM_TEXT_COLOR).thickness(1.0),
        );

        let play = if self.playing { "Stop" } else { "Play" };
        let super_labels = ["Clr", "Undo", play, "Rate", "", ""];
        let labels = music::function_labels(
            ctx,
            &["Del", "/", "*", "-", "+", "Play"],
            &["&", "|", "%", "<<", ">>", "^"],
            &super_labels,
        );
        music::draw_function_column(platform, &labels, None);
    }

    /// Wraps the expression across lines, breaking anywhere since bytebeat
    /// expressions rarely have spaces.
    fn draw_editor(&self, platform: &mut dyn IcPlatform) {
        platform.draw_rectangle_rounded(
            IVec2::new(EDITOR_LEFT, EDITOR_TOP),
            IVec2::new(EDITOR_RIGHT, EDITOR_BOTTOM),
            CELL_COLOR,
            0,
            Some(CELL_COLOR),
            4,
        );
        let text = self.input.text();
        let max_width = (EDITOR_RIGHT - EDITOR_LEFT) as f32 - EDITOR_MARGIN * 2.0;
        let (scale, lines, line_count) = [5.0, 4.0, 3.2]
            .iter()
            .map(|&scale| {
                let (lines, count) = wrap_lines(text, scale, max_width);
                (scale, lines, count)
            })
            .find(|(_, _, count)| *count <= EDITOR_MAX_LINES)
            .unwrap_or_else(|| {
                let (lines, count) = wrap_lines(text, 3.2, max_width);
                (3.2, lines, count)
            });

        let left = EDITOR_LEFT as f32 + EDITOR_MARGIN;
        let cursor = self.input.cursor();
        for (line_idx, &(start, end)) in lines[..line_count].iter().enumerate() {
            let y = EDITOR_TOP as f32 + 6.0 + scale * 1.2 + EDITOR_LINE_SPACING * line_idx as f32;
            let line = &text[start..end];
            text::draw_text(
                platform,
                line,
                Vec2::new(left, y),
                TextStyle::new(scale, TEXT_COLOR).weight(0.4),
            );
            // The cursor sits at the start of the next line rather than the
            // end of a full one, except at the very end of the text.
            let is_last = line_idx + 1 == line_count;
            if cursor >= start && (cursor < end || (is_last && cursor == end)) {
                let x = text::text_to_pos(line, left, scale, cursor - start, FontId::Futural);
                platform.draw_line(
                    Vec2::new(x, y - scale * 1.4),
                    Vec2::new(x, y + scale * 0.8),
                    CURSOR_COLOR,
                    2,
                );
            }
        }
    }

    /// The waveform of the last few hundred values of `t`.
    fn draw_scope(&self, platform: &mut dyn IcPlatform) {
        platform.draw_rectangle_rounded(
            IVec2::new(EDITOR_LEFT, SCOPE_TOP),
            IVec2::new(EDITOR_RIGHT, SCOPE_BOTTOM),
            CELL_COLOR,
            0,
            Some(rgb8_hex(0x141229)),
            4,
        );
        let Some(program) = self.program.as_ref().filter(|_| self.playing) else {
            return;
        };
        let width = (EDITOR_RIGHT - EDITOR_LEFT - 8) as f32;
        let height = (SCOPE_BOTTOM - SCOPE_TOP - 8) as f32;
        let point = |i: u32| {
            let t = self.t.wrapping_sub(2 * (SCOPE_POINTS - i));
            let value = Self::sample_at(program, t) as f32;
            Vec2::new(
                EDITOR_LEFT as f32 + 4.0 + width * i as f32 / SCOPE_POINTS as f32,
                SCOPE_BOTTOM as f32 - 4.0 - height * value / 255.0,
            )
        };
        let mut previous = point(0);
        for i in 1..=SCOPE_POINTS {
            let next = point(i);
            platform.draw_line(previous, next, PLAYING_COLOR, 1);
            previous = next;
        }
    }
}

/// Splits `text` into at most `EDITOR_MAX_LINES + 1` lines no wider than
/// `max_width`, returning byte ranges and how many lines were used. One extra
/// line is allowed so callers can tell the text did not fit.
fn wrap_lines(
    text: &str,
    scale: f32,
    max_width: f32,
) -> ([(usize, usize); EDITOR_MAX_LINES + 1], usize) {
    let mut lines = [(0, 0); EDITOR_MAX_LINES + 1];
    let mut count = 0;
    let mut start = 0;
    while count < lines.len() {
        let mut end = start;
        while end < text.len()
            && text::text_width(&text[start..end + 1], scale, FontId::Futural) <= max_width
        {
            end += 1;
        }
        // Always make progress, even if one character is too wide.
        if end == start && end < text.len() {
            end += 1;
        }
        lines[count] = (start, end);
        count += 1;
        if end >= text.len() {
            break;
        }
        start = end;
    }
    if count == lines.len() && lines[count - 1].1 < text.len() {
        // Still didn't fit; report it as overflowing.
        lines[count - 1].1 = text.len();
    }
    (lines, count)
}

impl IcApp for Bytebeat {
    fn requires_realtime_updates(&self) -> bool {
        // Keeps the scope and `t` moving.
        self.playing
    }

    fn on_enter(&mut self) {
        // Like the other apps, start silent.
        self.playing = false;
    }

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        if ctx.is_super() && !ctx.is_shifted() {
            // F1 and F2.
            match key {
                IcKey::Multiply => return self.toggle_playing(),
                IcKey::Minus => {
                    self.rate_idx = music::cycle(self.rate_idx, T_RATES_HZ.len(), true);
                    return;
                }
                IcKey::Num5 => return self.restart(),
                _ => {}
            }
        }
        match self.input.handle_key(key, ctx) {
            MathInputEvent::Submitted => self.play(),
            MathInputEvent::Changed => self.parse_failed = false,
            MathInputEvent::Navigate(MathInputDirection::Up) => self.load_preset(false),
            MathInputEvent::Navigate(MathInputDirection::Down) => self.load_preset(true),
            _ => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        self.draw(platform, ctx);
    }

    fn produces_audio(&self) -> bool {
        self.playing && self.program.is_some()
    }

    fn render_audio(&mut self, out_pcm: &mut [i16]) {
        let Some(program) = self.program.as_ref() else {
            out_pcm.fill(0);
            return;
        };
        let rate = T_RATES_HZ[self.rate_idx];
        for sample in out_pcm.iter_mut() {
            if self.value_t != Some(self.t) {
                self.value = Self::sample_at(program, self.t);
                self.value_t = Some(self.t);
            }
            let level = (self.value as i32 - 128) << 8;
            self.dc_q8 += ((level << 8) - self.dc_q8) >> DC_FILTER_SHIFT;
            let filtered = (level - (self.dc_q8 >> 8)) >> OUTPUT_ATTENUATION_SHIFT;
            *sample = filtered.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

            self.phase += rate;
            while self.phase >= SAMPLE_RATE_HZ {
                self.phase -= SAMPLE_RATE_HZ;
                self.t = self.t.wrapping_add(1);
            }
        }
    }

    fn name(&self) -> &str {
        "Bytebeat"
    }

    fn icon(&self) -> &'static Vitmap {
        &graphics::MUSIC_ICON
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0x5cff9d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_counts_at_the_chosen_rate() {
        let mut app = Bytebeat::new();
        app.play();
        assert!(app.produces_audio());
        let mut out = [0i16; 480];
        app.render_audio(&mut out);
        assert_eq!(app.t, 80);

        app.rate_idx = T_RATES_HZ.iter().position(|&rate| rate == 44_100).unwrap();
        app.restart();
        let mut out = [0i16; SAMPLE_RATE_HZ as usize];
        app.render_audio(&mut out);
        assert_eq!(app.t, 44_100);
    }

    #[test]
    fn output_follows_the_low_byte() {
        let mut app = Bytebeat::new();
        app.input.set_text("t*64").unwrap();
        app.play();
        app.rate_idx = T_RATES_HZ.iter().position(|&rate| rate == 8_000).unwrap();
        // t*64 cycles 0, 64, 128, 192: a rising staircase repeating every
        // four steps of six output samples each.
        let mut out = [0i16; 48];
        app.render_audio(&mut out);
        assert!(out[0] < out[6] && out[6] < out[12] && out[12] < out[18]);
        assert!(out[24] < out[18]);
    }

    #[test]
    fn bad_expressions_keep_the_last_good_one_playing() {
        let mut app = Bytebeat::new();
        app.play();
        app.input.set_text("t>>").unwrap();
        app.play();
        assert!(app.parse_failed);
        assert!(app.produces_audio());
        assert_eq!(
            app.program,
            Some(Program::compile(PRESETS[0], FORMAT).unwrap())
        );
    }

    #[test]
    fn editing_while_playing_keeps_t() {
        let mut app = Bytebeat::new();
        app.play();
        let mut out = [0i16; 600];
        app.render_audio(&mut out);
        app.load_preset(true);
        assert_eq!(app.input.text(), PRESETS[1]);
        assert_eq!(app.t, 100);
        app.toggle_playing();
        app.toggle_playing();
        assert_eq!(app.t, 0);
    }

    #[test]
    fn long_expressions_wrap() {
        let (lines, count) = wrap_lines(PRESETS[1], 5.0, 240.0);
        assert!(count >= 1 && count <= EDITOR_MAX_LINES);
        assert_eq!(lines[0].0, 0);
        assert_eq!(lines[count - 1].1, PRESETS[1].len());
        for pair in lines[..count].windows(2) {
            assert_eq!(pair[0].1, pair[1].0);
        }
    }
}
