//! Shows the platform's [`IcPlatform::info`] text, refreshed twice a second.
//! A dot by the title flashes on each refresh and fades until the next, so
//! it is clear the readings are live.
use crate::app::{AppIcon, IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::fonts::get_font;
use crate::graphics;
use crate::input::IcKey;
use crate::platform::{CANVAS_WIDTH, IcPlatform, rgb8_hex};
use crate::text::{TextStyle, draw_text, text_width, wrap_text};
use alloc::string::String;
use glam::{IVec2, Vec2};
use rgb::RGB8;

const REFRESH_MS: u64 = 500;
const TEXT_SIZE: f32 = 5.0;
const MARGIN: f32 = 5.0;
/// Extra space between lines, beyond the font's own line height.
const EXTRA_LINE_SPACING: f32 = 2.0;
const TITLE: &str = "System Information";
/// Same size and place as the Settings title.
const TITLE_SIZE: f32 = 6.0;
const TITLE_POS: Vec2 = Vec2::new(5.0, 12.0);
/// The first line of the platform's text, where Settings starts its list.
const BODY_Y: f32 = 32.0;
const DOT_RADIUS: i32 = 4;
/// Gap between the end of the title and the live dot.
const DOT_GAP: f32 = 8.0;
/// How bright the live dot is just before a refresh, out of 1.
const DOT_MIN_BRIGHTNESS: f32 = 0.2;

pub struct Info {
    /// The platform's text, already wrapped to the screen width.
    text: String,
    /// When `text` was last fetched, or `None` to fetch it on the next frame.
    fetched_at: Option<u64>,
}

impl Info {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            fetched_at: None,
        }
    }
}

impl IcApp for Info {
    fn name(&self) -> &str {
        "Info"
    }

    fn icon(&self) -> AppIcon<'_> {
        AppIcon::Static(&graphics::INFO)
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0x306082)
    }

    // So the readings stay current without key presses.
    fn requires_realtime_updates(&self) -> bool {
        true
    }

    fn on_enter(&mut self) {
        self.fetched_at = None;
    }

    fn on_exit(&mut self) {
        self.text = String::new();
    }

    fn on_key(&mut self, _key: IcKey, _ctx: &InputContext) {}

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        _ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        let now = platform.millis();
        if self
            .fetched_at
            .is_none_or(|at| now.saturating_sub(at) >= REFRESH_MS)
        {
            let max_width = CANVAS_WIDTH as f32 - 2.0 * MARGIN;
            self.text = wrap_text(&platform.info(), max_width, TEXT_SIZE);
            self.fetched_at = Some(now);
        }

        platform.clear(rgb8_hex(0x306082));
        let title_style = TextStyle::new(TITLE_SIZE, rgb8_hex(0xffffff));
        draw_text(platform, TITLE, TITLE_POS, title_style);
        let title_width = text_width(TITLE, TITLE_SIZE, title_style.font);
        let age = self.fetched_at.map_or(0, |at| now.saturating_sub(at));
        draw_live_dot(platform, TITLE_POS.x + title_width + DOT_GAP, age);

        let style = TextStyle::new(TEXT_SIZE, rgb8_hex(0xffffff));
        let font = get_font(style.font);
        let line_step =
            font.line_height as f32 * TEXT_SIZE / font.units_per_em + EXTRA_LINE_SPACING;
        for (index, line) in self.text.split('\n').enumerate() {
            let y = BODY_Y + index as f32 * line_step;
            draw_text(platform, line, Vec2::new(MARGIN, y), style);
        }
    }
}

/// A green dot at `x`, centered on the title, that is brightest right after
/// a refresh (`age` 0 ms) and fades until the next one.
fn draw_live_dot(platform: &mut dyn IcPlatform, x: f32, age: u64) {
    let fade = 1.0 - (age as f32 / REFRESH_MS as f32).min(1.0);
    let brightness = DOT_MIN_BRIGHTNESS + (1.0 - DOT_MIN_BRIGHTNESS) * fade;
    let channel = |full: u8| (full as f32 * brightness) as u8;
    let color = RGB8::new(channel(0x40), channel(0xe0), channel(0x60));
    // Capitals span y - 1.2 × size to y + 0.9 × size.
    let center = IVec2::new(x as i32, (TITLE_POS.y - 0.15 * TITLE_SIZE) as i32);
    let half = IVec2::splat(DOT_RADIUS);
    platform.draw_rectangle_rounded(
        center - half,
        center + half,
        color,
        0,
        Some(color),
        DOT_RADIUS as u32,
    );
}
