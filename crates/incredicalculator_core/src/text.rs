//! Hershey stroke-font text rendering.
//!
//! This replaces the old packed-nibble character format with native Hershey
//! coordinates (`left`, `right`, and a sequence of vertices separated by LIFT).
//!
//! The font data below intentionally contains only ONE example glyph (A).
//! Replace the example arrays with your complete Hershey font tables.
//!
//! Coordinate convention:
//!   * X/Y are in Hershey units.
//!   * Positive Y goes down on the display, matching the renderer you had.
//!   * LIFT ends the current stroke without drawing a connecting line.

use core::fmt;

use crate::fonts;
use crate::fonts::{HersheyFont, HersheyGlyph, HERSHEY_LIFT};
use glam::Vec2;
use rgb::*;

use crate::platform::IcPlatform;

#[inline]
fn get_glyph(font: &'static HersheyFont, c: u8) -> &'static HersheyGlyph {
    let first = font.first_char as usize;

    if (c as usize) >= first {
        let index = c as usize - first;
        if index < font.glyphs.len() {
            return &font.glyphs[index];
        }
    }

    font.missing_glyph
}

#[inline]
fn glyph_advance(font: &'static HersheyFont, c: u8, glyph: &'static HersheyGlyph) -> f32 {
    if c == b' ' {
        font.space_advance as f32
    } else {
        let advance = glyph.advance();
        if advance > 0 {
            advance as f32
        } else {
            1.0
        }
    }
}

#[inline]
fn draw_stroke_point(platform: &mut dyn IcPlatform, sx: f32, sy: f32, color: RGB8, thickness: u32) {
    // IcPlatform exposes draw_line but not a point primitive in the code
    // supplied. A tiny line gives standalone Hershey points visible output.
    let half = (thickness.max(1) as f32) * 0.5;
    platform.draw_line(
        Vec2::new(sx - half, sy - half),
        Vec2::new(sx + half, sy + half),
        color,
        thickness.max(1),
    );
}

/// Sum of glyph advances up to the end of the first line in `text`, in screen
/// pixels. Carriage returns are ignored
fn line_width(font: &'static HersheyFont, font_scale: f32, text: &[u8]) -> f32 {
    let mut width = 0.0;
    for &c in text {
        if c == b'\n' {
            break;
        }
        if c == b'\r' {
            continue;
        }
        width += glyph_advance(font, c, get_glyph(font, c)) * font_scale;
    }
    width
}

// -----------------------------------------------------------------------------
// PUBLIC RENDERING API
// -----------------------------------------------------------------------------

/// Horizontal alignment of each line of text relative to the anchor X.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HAlign {
    /// The anchor is the left edge of each line.
    #[default]
    Left,
    /// The anchor is the horizontal center of each line.
    Center,
    /// The anchor is the right edge of each line.
    Right,
}

impl HAlign {
    #[inline]
    fn factor(self) -> f32 {
        match self {
            HAlign::Left => 0.0,
            HAlign::Center => 0.5,
            HAlign::Right => 1.0,
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub scale: f32,
    pub color: RGB8,
    pub thickness: f32,
    pub font: fonts::FontId,
    pub align: HAlign,
}

impl TextStyle {
    pub const fn new(scale: f32, color: RGB8) -> Self {
        Self {
            scale,
            color,
            thickness: 2.0,
            font: fonts::FontId::Futural,
            align: HAlign::Left,
        }
    }

    pub const fn align(self, align: HAlign) -> Self {
        Self { align, ..self }
    }

    pub const fn thickness(self, thickness: f32) -> Self {
        Self { thickness, ..self }
    }

    pub const fn font(self, font: fonts::FontId) -> Self {
        Self { font, ..self }
    }
}

/// Draw text using a selected Hershey font.
///
/// `pos.x` is the anchor for every line, interpreted according to
/// `style.align`. `pos.y` is the origin of the first line; following lines
/// advance by the font's line height.
pub fn draw_text(platform: &mut dyn IcPlatform, text: &str, pos: Vec2, style: TextStyle) {
    let font = fonts::get_font(style.font);
    let font_scale = style.scale / font.units_per_em;
    if font_scale <= 0.0 {
        return;
    }

    let bytes = text.as_bytes();
    let align = style.align.factor();
    let line_x = |line: &[u8]| pos.x - line_width(font, font_scale, line) * align;
    let thickness = style.thickness as u32;
    let color = style.color;

    let mut current_x = line_x(bytes);
    let mut current_y = pos.y;
    let line_height = font.line_height as f32 * font_scale;

    for (i, &c) in bytes.iter().enumerate() {
        if c == b'\n' {
            current_x = line_x(&bytes[i + 1..]);
            current_y += line_height;
            continue;
        }

        if c == b'\r' {
            continue;
        }

        let glyph = get_glyph(font, c);
        let advance_x = glyph_advance(font, c, glyph) * font_scale;

        // Same left-bearing behavior as your C++ renderer:
        // current_x is the left edge of the glyph cell.
        let x_offset = -(glyph.left as f32) * font_scale;

        let mut pen_down = false;
        let mut last_sx = 0.0_f32;
        let mut last_sy = 0.0_f32;

        for (index, vertex) in glyph.vertices.iter().enumerate() {
            if *vertex == HERSHEY_LIFT {
                pen_down = false;
                continue;
            }

            let sx = current_x + x_offset + vertex.x as f32 * font_scale;
            let sy = current_y + vertex.y as f32 * font_scale;

            if pen_down {
                platform.draw_line(
                    Vec2::new(last_sx, last_sy),
                    Vec2::new(sx, sy),
                    color,
                    thickness,
                );
            } else {
                // Draw isolated points only when this vertex is a one-point
                // stroke (next item is LIFT or this is the final item).
                let is_last = index + 1 >= glyph.vertices.len();
                let next_is_lift = !is_last && glyph.vertices[index + 1] == HERSHEY_LIFT;

                if glyph.vertices.len() == 1 || is_last || next_is_lift {
                    draw_stroke_point(platform, sx, sy, color, thickness);
                }
            }

            last_sx = sx;
            last_sy = sy;
            pen_down = true;
        }

        current_x += advance_x;
    }
}

/// Formatting-friendly wrapper, matching your old draw_text_f helper.
pub fn draw_text_f(
    platform: &mut dyn IcPlatform,
    arg: fmt::Arguments<'_>,
    pos: Vec2,
    style: TextStyle,
) {
    let mut buf = [0u8; 128];

    let text = format_no_std::show(&mut buf, arg).unwrap();
    draw_text(platform, text, pos, style);
}

/// Return the X position after `cursor` characters of left-aligned text
/// drawn at `x`.
///
/// Newlines reset to `x` and carriage returns are ignored, matching
/// draw_text. Neither counts toward `cursor`.
pub fn text_to_pos(text: &str, x: f32, scale: f32, cursor: usize, font_id: fonts::FontId) -> f32 {
    let font = fonts::get_font(font_id);
    let font_scale = scale / font.units_per_em;

    if font_scale <= 0.0 || cursor == 0 {
        return x;
    }

    let mut current_x = x;
    let mut counter = 0_usize;

    for c in text.bytes() {
        if c == b'\n' {
            current_x = x;
            continue;
        }

        if c == b'\r' {
            continue;
        }

        let glyph = get_glyph(font, c);
        let advance_x = glyph_advance(font, c, glyph) * font_scale;

        current_x += advance_x;
        counter += 1;

        if counter >= cursor {
            return current_x;
        }
    }
    current_x
}

/// Width of the widest line in `text`, in screen pixels.
pub fn text_width(text: &str, scale: f32, font_id: fonts::FontId) -> f32 {
    let font = fonts::get_font(font_id);
    let font_scale = scale / font.units_per_em;
    if font_scale <= 0.0 {
        return 0.0;
    }

    text.as_bytes()
        .split(|&c| c == b'\n')
        .map(|line| line_width(font, font_scale, line))
        .fold(0.0, f32::max)
}

/// Top and bottom of the drawn strokes, in screen pixels relative to the
/// `pos.y` passed to draw_text. Later lines are offset by the line height.
pub fn text_ink_bounds(text: &str, scale: f32, font_id: fonts::FontId) -> Option<(f32, f32)> {
    let font = fonts::get_font(font_id);
    let font_scale = scale / font.units_per_em;
    let mut bounds: Option<(f32, f32)> = None;

    for (line_index, line) in text.split('\n').enumerate() {
        let Some((min_y, max_y)) = text_vertical_bounds(line, font_id) else {
            continue;
        };
        let line_y = line_index as f32 * font.line_height as f32;
        let top = (line_y + min_y) * font_scale;
        let bottom = (line_y + max_y) * font_scale;
        bounds = Some(match bounds {
            Some((t, b)) => (t.min(top), b.max(bottom)),
            None => (top, bottom),
        });
    }

    bounds
}

/// Height of the drawn strokes in `text`, in screen pixels.
pub fn text_height(text: &str, scale: f32, font_id: fonts::FontId) -> f32 {
    text_ink_bounds(text, scale, font_id).map_or(0.0, |(top, bottom)| bottom - top)
}

pub fn text_vertical_bounds(text: &str, font_id: fonts::FontId) -> Option<(f32, f32)> {
    let font = fonts::get_font(font_id);
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for byte in text.bytes() {
        if byte == b'\n' || byte == b'\r' {
            continue;
        }
        for vertex in get_glyph(font, byte).vertices {
            if *vertex != HERSHEY_LIFT {
                min_y = min_y.min(vertex.y as f32);
                max_y = max_y.max(vertex.y as f32);
            }
        }
    }

    (min_y <= max_y).then_some((min_y, max_y))
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
