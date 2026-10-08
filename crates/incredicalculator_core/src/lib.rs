#![no_std]
// IC stands for Incredicalculator
#[macro_use]
extern crate num_derive;
extern crate alloc;

pub mod app;
pub mod apps;
pub mod audio_engine;
pub mod input;
pub mod platform;
pub mod shell;
pub mod graphics {
    use alloc::vec::Vec;
    use glam::{Mat2, Vec2};
    use rgb::RGB8;

    use crate::platform::IcPlatform;

    include!(concat!(env!("OUT_DIR"), "/generated_graphics.rs"));

    /// Draws a frame from the vitmap's default action after applying its
    /// local-to-screen transform.
    ///
    /// `position` is the screen position of the vitmap origin, `rotation` is
    /// in radians, and `scale` is applied independently on each local axis.
    /// Returns `false` when `frame` is outside the vitmap's frame list.
    pub fn draw_vitmap(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        frame: usize,
        position: Vec2,
        rotation: f32,
        scale: Vec2,
    ) -> bool {
        draw_vitmap_action(platform, vitmap, 0, frame, position, rotation, scale)
    }

    /// Draws a frame from a selected vitmap action.
    ///
    /// `position` is the screen position of the vitmap origin, `rotation` is
    /// in radians, and `scale` is applied independently on each local axis.
    /// Returns `false` when `action` or `frame` is outside the vitmap data.
    pub fn draw_vitmap_action(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        action: usize,
        frame: usize,
        position: Vec2,
        rotation: f32,
        scale: Vec2,
    ) -> bool {
        let Some(action) = vitmap.actions.get(action) else {
            return false;
        };
        let Some(frame) = action.frames.get(frame) else {
            return false;
        };
        let rotation = Mat2::from_angle(rotation);
        let transform =
            |point: Point| position + rotation * Vec2::new(point.x * scale.x, point.y * scale.y);

        for polygon in frame.shapes {
            if !polygon.open && polygon.points.len() >= 3 {
                let points: Vec<Vec2> = polygon.points.iter().copied().map(transform).collect();
                platform.draw_polygon(
                    &points,
                    RGB8::new(polygon.color.r, polygon.color.g, polygon.color.b),
                );
            }

            let border_width = if polygon.border_width > 0.0 {
                (polygon.border_width as u32).max(1)
            } else {
                0
            };
            if border_width == 0 || polygon.points.len() < 2 {
                continue;
            }

            let border_color = RGB8::new(
                polygon.border_color.r,
                polygon.border_color.g,
                polygon.border_color.b,
            );
            let edge_count = if polygon.open {
                polygon.points.len() - 1
            } else {
                polygon.points.len()
            };
            for index in 0..edge_count {
                let start = transform(polygon.points[index]);
                let end = transform(polygon.points[(index + 1) % polygon.points.len()]);
                platform.draw_line(start, end, border_color, border_width);
            }
        }

        true
    }
}
mod fonts;
mod text;
mod ui;
