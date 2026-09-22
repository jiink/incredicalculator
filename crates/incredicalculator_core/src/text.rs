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

use glam::{IVec2, Vec2};
use num_traits::ToPrimitive;
use rgb::*;
use num_traits::float::FloatCore;
use core::cmp;
use crate::fonts::{HersheyFont, HersheyGlyph, HersheyVertex, HERSHEY_LIFT};
use crate::fonts;

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
fn draw_stroke_point(
    platform: &mut dyn IcPlatform,
    sx: f32,
    sy: f32,
    color: RGB8,
    thickness: u32,
) {
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

// -----------------------------------------------------------------------------
// PUBLIC RENDERING API
// -----------------------------------------------------------------------------

/// Draw text using a selected Hershey font.
///
/// `x` / `y` are the text origin used by the old renderer.
/// `scale` converts Hershey units to screen pixels.
///
/// Because the platform code supplied only exposes draw_line, `thickness`
/// is derived from scale. If you want explicit thickness, add it as another
/// argument and replace the `scale.round()` line below.
pub fn draw_text(
    platform: &mut dyn IcPlatform,
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    thickness: f32,
    color: RGB8,
    font_id: fonts::FontId,
) {
    let font = fonts::get_font(font_id);
    let font_scale = scale / font.units_per_em;
    if font_scale <= 0.0 {
        return;
    }

    let mut current_x = x;
    let mut current_y = y;
    let line_height = font.line_height as f32 * font_scale;

    for c in text.bytes() {
        if c == b'\n' {
            current_x = x;
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
                    thickness as u32,
                );
            } else {
                // Draw isolated points only when this vertex is a one-point
                // stroke (next item is LIFT or this is the final item).
                let is_last = index + 1 >= glyph.vertices.len();
                let next_is_lift =
                    !is_last && glyph.vertices[index + 1] == HERSHEY_LIFT;

                if glyph.vertices.len() == 1 || is_last || next_is_lift {
                    draw_stroke_point(platform, sx, sy, color, thickness as u32);
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
    x: f32,
    y: f32,
    scale: f32,
    thickness: f32,
    color: RGB8,
    font_id: fonts::FontId,
) {
    let mut buf = [0u8; 128];

    let text = format_no_std::show(&mut buf, arg).unwrap();
    draw_text(platform, text, x, y, scale, thickness, color, font_id);
}

/// Return the X position after `cursor` characters.
///
/// This follows the behavior of your existing text_to_pos helper and treats
/// newline/carriage-return as line-control characters.
pub fn text_to_pos(
    text: &str,
    x: f32,
    scale: f32,
    cursor: usize,
    font_id: fonts::FontId,
) -> f32 {
    let font = fonts::get_font(font_id);
    if scale <= 0.0 || cursor == 0 {
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
            current_x = x;
            continue;
        }

        let glyph = get_glyph(font, c);
        let advance_x = glyph_advance(font, c, glyph) * scale;

        current_x += advance_x;
        counter += 1;

        if counter >= cursor {
            return current_x;
        }
    }

    current_x
}
