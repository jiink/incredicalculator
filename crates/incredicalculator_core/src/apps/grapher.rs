//! Plots `y = f(x)` as the formula is typed. The curve is sampled once per
//! pixel column, a few columns each frame, so a slow formula doesn't stall
//! the UI; the plot fills in from left to right.
use crate::app::{AppIcon, IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::input::IcKey;
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{HAlign, TextStyle, draw_text, draw_text_f};
use crate::ui::{MathInput, MathInputEvent, MathInputMode};
use alloc::rc::Rc;
use alloc::string::String;
use bumpalo::Bump;
use exp_rs::{EvalContext, Expression};
use glam::{IVec2, Vec2};
use rgb::RGB8;

const FORMULA_LEN: usize = 48;

const INPUT_LEFT: i32 = 30;
const INPUT_TOP: i32 = 4;
const INPUT_WIDTH: i32 = 286;
const INPUT_HEIGHT: i32 = 30;

const GRAPH_LEFT: i32 = 4;
const GRAPH_RIGHT: i32 = 316;
const GRAPH_TOP: i32 = 40;
const GRAPH_BOTTOM: i32 = 236;
/// One sample per pixel column, both edges included.
const SAMPLE_COUNT: usize = (GRAPH_RIGHT - GRAPH_LEFT) as usize + 1;

/// How long sampling may take each frame before the rest is left for the
/// next one. At least one sample is taken per frame regardless.
const FRAME_BUDGET_MS: u64 = 8;
/// A segment that jumps from above the view to below it (or back) by more
/// than this many view heights is taken to be an asymptote, and not drawn.
const ASYMPTOTE_SPANS: f32 = 20.0;

const BG_COLOR: RGB8 = rgb8_hex(0x10141c);
const GRAPH_BG_COLOR: RGB8 = rgb8_hex(0x161c28);
const GRID_COLOR: RGB8 = rgb8_hex(0x26304a);
const AXIS_COLOR: RGB8 = rgb8_hex(0x7c88a8);
const CURVE_COLOR: RGB8 = rgb8_hex(0x4fd1ff);
const TEXT_COLOR: RGB8 = rgb8_hex(0xffffff);
const DIM_TEXT_COLOR: RGB8 = rgb8_hex(0x7c88a8);
const ERROR_COLOR: RGB8 = rgb8_hex(0xff5c7a);
const INPUT_BG_COLOR: RGB8 = rgb8_hex(0x26304a);
const INPUT_FOCUS_BG_COLOR: RGB8 = rgb8_hex(0x33405e);

#[derive(Clone, Copy)]
struct View {
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
}

impl View {
    const DEFAULT: View = View {
        x_min: -1.0,
        x_max: 1.0,
        y_min: -1.0,
        y_max: 1.0,
    };

    fn sample_x(&self, idx: usize) -> f32 {
        self.x_min + (self.x_max - self.x_min) * idx as f32 / (SAMPLE_COUNT - 1) as f32
    }

    fn screen_x(&self, x: f32) -> f32 {
        GRAPH_LEFT as f32
            + (x - self.x_min) / (self.x_max - self.x_min) * (GRAPH_RIGHT - GRAPH_LEFT) as f32
    }

    fn screen_y(&self, y: f32) -> f32 {
        GRAPH_TOP as f32
            + (self.y_max - y) / (self.y_max - self.y_min) * (GRAPH_BOTTOM - GRAPH_TOP) as f32
    }
}

pub struct Grapher {
    input: MathInput<FORMULA_LEN>,
    /// Holds the built-in functions and constants.
    context: Rc<EvalContext>,
    /// The formula as exp-rs takes it, once it has parsed.
    formula: Option<String>,
    /// Whether the typed formula fails to parse or evaluate.
    invalid: bool,
    view: View,
    samples: [f32; SAMPLE_COUNT],
    /// How many of `samples`, from the left, are for the current formula.
    sampled: usize,
}

impl Grapher {
    pub fn new() -> Grapher {
        Grapher {
            input: MathInput::new(
                IVec2::new(INPUT_LEFT, INPUT_TOP),
                IVec2::new(INPUT_WIDTH, INPUT_HEIGHT),
                MathInputMode::Graph,
                2.0,
                9.0,
                INPUT_BG_COLOR,
                INPUT_FOCUS_BG_COLOR,
                None,
            ),
            context: Rc::new(EvalContext::new()),
            formula: None,
            invalid: false,
            view: View::DEFAULT,
            samples: [f32::NAN; SAMPLE_COUNT],
            sampled: 0,
        }
    }

    /// Takes in the newly typed formula and starts plotting it over.
    fn formula_changed(&mut self) {
        self.sampled = 0;
        self.formula = None;
        self.invalid = false;
        if self.input.is_empty() {
            return;
        }
        let formula = expand_implicit_multiplication(self.input.text());
        let arena = Bump::new();
        if exp_rs::parse_expression(&formula, &arena).is_ok() {
            self.formula = Some(formula);
        } else {
            self.invalid = true;
        }
    }

    /// Samples the formula from where it left off, until the frame's time
    /// budget runs out or the curve is complete.
    fn sample_some(&mut self, platform: &dyn IcPlatform) {
        let Some(formula) = &self.formula else {
            return;
        };
        if self.sampled >= SAMPLE_COUNT {
            return;
        }
        // The parsed expression borrows its arena, so it is rebuilt each
        // frame rather than kept. Parsing a short formula is cheap next to
        // evaluating it a few hundred times.
        let arena = Bump::new();
        let mut expression = Expression::new(&arena);
        let ok = expression.add_parameter("x", 0.0).is_ok()
            && expression.add_expression(formula).is_ok();
        if !ok {
            self.formula = None;
            self.invalid = true;
            return;
        }

        let started = platform.millis();
        loop {
            let x = self.view.sample_x(self.sampled);
            let _ = expression.set_param(0, x as exp_rs::Real);
            if expression.eval(&self.context).is_err() {
                // Unknown names and the like fail at every x, not just some.
                self.formula = None;
                self.invalid = true;
                return;
            }
            self.samples[self.sampled] = expression.get_result(0).map_or(f32::NAN, |y| y as f32);
            self.sampled += 1;
            if self.sampled >= SAMPLE_COUNT
                || platform.millis().saturating_sub(started) >= FRAME_BUDGET_MS
            {
                break;
            }
        }
    }

    fn draw_grid(&self, platform: &mut dyn IcPlatform) {
        let view = self.view;
        let left = GRAPH_LEFT as f32;
        let right = GRAPH_RIGHT as f32;
        let top = GRAPH_TOP as f32;
        let bottom = GRAPH_BOTTOM as f32;

        let x_step = grid_step(view.x_max - view.x_min);
        let mut x = libm::ceilf(view.x_min / x_step) * x_step;
        while x <= view.x_max {
            let sx = view.screen_x(x);
            platform.draw_line(Vec2::new(sx, top), Vec2::new(sx, bottom), GRID_COLOR, 1);
            x += x_step;
        }
        let y_step = grid_step(view.y_max - view.y_min);
        let mut y = libm::ceilf(view.y_min / y_step) * y_step;
        while y <= view.y_max {
            let sy = view.screen_y(y);
            platform.draw_line(Vec2::new(left, sy), Vec2::new(right, sy), GRID_COLOR, 1);
            y += y_step;
        }

        // Axes, where they are in view; otherwise labels go along the edges.
        let axis_x = view.screen_x(0.0).clamp(left, right);
        let axis_y = view.screen_y(0.0).clamp(top, bottom);
        if view.x_min <= 0.0 && 0.0 <= view.x_max {
            platform.draw_line(
                Vec2::new(axis_x, top),
                Vec2::new(axis_x, bottom),
                AXIS_COLOR,
                1,
            );
        }
        if view.y_min <= 0.0 && 0.0 <= view.y_max {
            platform.draw_line(
                Vec2::new(left, axis_y),
                Vec2::new(right, axis_y),
                AXIS_COLOR,
                1,
            );
        }

        let label = TextStyle::new(4.0, DIM_TEXT_COLOR);
        let below_axis = (axis_y + 11.0).min(bottom - 3.0);
        draw_text_f(
            platform,
            format_args!("{}", view.x_min),
            Vec2::new(left + 3.0, below_axis),
            label,
        );
        draw_text_f(
            platform,
            format_args!("{}", view.x_max),
            Vec2::new(right - 3.0, below_axis),
            label.align(HAlign::Right),
        );
        let beside_axis = (axis_x + 4.0).min(right - 30.0);
        draw_text_f(
            platform,
            format_args!("{}", view.y_max),
            Vec2::new(beside_axis, top + 11.0),
            label,
        );
        draw_text_f(
            platform,
            format_args!("{}", view.y_min),
            Vec2::new(beside_axis, bottom - 3.0),
            label,
        );
    }

    fn draw_curve(&self, platform: &mut dyn IcPlatform) {
        let top = GRAPH_TOP as f32;
        let bottom = GRAPH_BOTTOM as f32;
        let height = bottom - top;
        let mut previous: Option<Vec2> = None;
        for idx in 0..self.sampled {
            let y = self.samples[idx];
            if !y.is_finite() {
                previous = None;
                continue;
            }
            let point = Vec2::new(
                self.view.screen_x(self.view.sample_x(idx)),
                self.view.screen_y(y),
            );
            if let Some(start) = previous {
                if let Some((a, b)) = clip_to_rows(start, point, top, bottom, height) {
                    platform.draw_line(a, b, CURVE_COLOR, 2);
                }
            }
            previous = Some(point);
        }
    }
}

/// Clips the segment from `a` to `b` to the rows from `top` to `bottom`, or
/// returns `None` if none of it is in view.
fn clip_to_rows(a: Vec2, b: Vec2, top: f32, bottom: f32, height: f32) -> Option<(Vec2, Vec2)> {
    if (a.y < top && b.y < top) || (a.y > bottom && b.y > bottom) {
        return None;
    }
    let spans_view = (a.y < top && b.y > bottom) || (a.y > bottom && b.y < top);
    if spans_view && (b.y - a.y).abs() > ASYMPTOTE_SPANS * height {
        return None;
    }
    let clip = |p: Vec2, q: Vec2| -> Vec2 {
        let edge = if p.y < top {
            top
        } else if p.y > bottom {
            bottom
        } else {
            return p;
        };
        let t = (edge - p.y) / (q.y - p.y);
        Vec2::new(p.x + (q.x - p.x) * t, edge)
    };
    Some((clip(a, b), clip(b, a)))
}

/// Spacing for about four to ten grid lines across `range`: 1, 2 or 5 times
/// a power of ten.
fn grid_step(range: f32) -> f32 {
    let rough = range / 4.0;
    let magnitude = libm::powf(10.0, libm::floorf(libm::log10f(rough)));
    let fraction = rough / magnitude;
    let nice = if fraction < 2.0 {
        1.0
    } else if fraction < 5.0 {
        2.0
    } else {
        5.0
    };
    nice * magnitude
}

/// Adds the `*` a person leaves out, as in `2x`, `x(x+1)`, `(x+1)(x-1)` or
/// `2(x+1)`, since exp-rs requires it.
fn expand_implicit_multiplication(formula: &str) -> String {
    use exp_rs::{lexer::Lexer, types::TokenKind};

    let mut expanded = String::with_capacity(formula.len() + 8);
    let mut lexer = Lexer::new(formula);
    let mut previous = TokenKind::End;
    // A function name before `(` is a call, but `x` is never a function.
    let mut previous_is_x = false;
    let mut copied = 0;
    while let Some(token) = lexer.next_token() {
        if token.kind == TokenKind::Error {
            break;
        }
        let ends_operand = matches!(previous, TokenKind::Number | TokenKind::Close)
            || (previous == TokenKind::Variable && previous_is_x);
        let starts_operand = matches!(
            token.kind,
            TokenKind::Open | TokenKind::Number | TokenKind::Variable
        );
        let is_call = previous == TokenKind::Variable && !previous_is_x;
        if ends_operand && starts_operand && !is_call {
            expanded.push_str(&formula[copied..token.position]);
            expanded.push('*');
            copied = token.position;
        }
        previous_is_x = token.kind == TokenKind::Variable && token.text.as_deref() == Some("x");
        previous = token.kind;
    }
    expanded.push_str(&formula[copied..]);
    expanded
}

impl IcApp for Grapher {
    fn name(&self) -> &str {
        "Grapher"
    }

    fn icon(&self) -> AppIcon<'_> {
        AppIcon::Initials("y=")
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0x0b4a6e)
    }

    fn requires_realtime_updates(&self) -> bool {
        // Only while there is still more of the curve to sample.
        self.formula.is_some() && self.sampled < SAMPLE_COUNT
    }

    fn on_enter(&mut self) {}

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        match self.input.handle_key(key, ctx) {
            MathInputEvent::Changed => self.formula_changed(),
            MathInputEvent::Submitted
            | MathInputEvent::Navigate(_)
            | MathInputEvent::Handled
            | MathInputEvent::Ignored => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        _ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        self.sample_some(platform);

        platform.clear(BG_COLOR);
        platform.draw_rectangle(
            IVec2::new(GRAPH_LEFT, GRAPH_TOP),
            IVec2::new(GRAPH_RIGHT, GRAPH_BOTTOM),
            GRID_COLOR,
            1,
            Some(GRAPH_BG_COLOR),
        );
        self.draw_grid(platform);
        self.draw_curve(platform);

        draw_text(
            platform,
            "y=",
            Vec2::new(6.0, 26.0),
            TextStyle::new(
                7.0,
                if self.invalid {
                    ERROR_COLOR
                } else {
                    TEXT_COLOR
                },
            ),
        );
        self.input.draw(platform, true);

        if self.input.is_empty() {
            draw_text(
                platform,
                "Super+0 types x",
                Vec2::new(GRAPH_LEFT as f32 + 6.0, GRAPH_TOP as f32 + 14.0),
                TextStyle::new(5.0, DIM_TEXT_COLOR),
            );
        } else if self.invalid {
            draw_text(
                platform,
                "Can't graph this",
                Vec2::new(GRAPH_LEFT as f32 + 6.0, GRAPH_TOP as f32 + 14.0),
                TextStyle::new(5.0, ERROR_COLOR),
            );
        }
    }
}
