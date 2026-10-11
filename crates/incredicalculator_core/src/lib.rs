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
    use glam::{Affine2, Vec2};
    use rgb::RGB8;

    use crate::platform::IcPlatform;

    include!(concat!(env!("OUT_DIR"), "/generated_graphics.rs"));

    /// Draws a frame from the vitmap's default action at `position`, scaling
    /// coordinates independently on each local axis.
    ///
    /// Returns `false` when `frame` is outside the vitmap's frame list.
    pub fn draw_vitmap(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        frame: usize,
        position: Vec2,
        scale: Vec2,
    ) -> bool {
        draw_vitmap_aff(
            platform,
            vitmap,
            frame,
            Affine2::from_scale_angle_translation(scale, 0.0, position),
        )
    }

    /// Draws a frame from the vitmap's default action using a local-to-screen
    /// affine transform.
    ///
    /// `transform` maps coordinates from vitmap-local space to screen space.
    /// Returns `false` when `frame` is outside the vitmap's frame list.
    pub fn draw_vitmap_aff(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        frame: usize,
        transform: Affine2,
    ) -> bool {
        draw_vitmap_action_aff(platform, vitmap, 0, frame, transform)
    }

    /// Draws a frame from a selected vitmap action at `position`, scaling
    /// coordinates independently on each local axis.
    ///
    /// Returns `false` when `action` or `frame` is outside the vitmap data.
    pub fn draw_vitmap_action(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        action: usize,
        frame: usize,
        position: Vec2,
        scale: Vec2,
    ) -> bool {
        draw_vitmap_action_aff(
            platform,
            vitmap,
            action,
            frame,
            Affine2::from_scale_angle_translation(scale, 0.0, position),
        )
    }

    /// Draws a frame from a selected vitmap action using a local-to-screen
    /// affine transform.
    ///
    /// `transform` maps coordinates from vitmap-local space to screen space.
    /// Returns `false` when `action` or `frame` is outside the vitmap data.
    pub fn draw_vitmap_action_aff(
        platform: &mut dyn IcPlatform,
        vitmap: &Vitmap,
        action: usize,
        frame: usize,
        transform: Affine2,
    ) -> bool {
        let Some(action) = vitmap.actions.get(action) else {
            return false;
        };
        let Some(frame) = action.frames.get(frame) else {
            return false;
        };
        let transform_point =
            |point: Point| transform.transform_point2(Vec2::new(point.x, point.y));

        for polygon in frame.shapes {
            if !polygon.open && polygon.points.len() >= 3 {
                let points: Vec<Vec2> = polygon
                    .points
                    .iter()
                    .copied()
                    .map(transform_point)
                    .collect();
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
                let start = transform_point(polygon.points[index]);
                let end = transform_point(polygon.points[(index + 1) % polygon.points.len()]);
                platform.draw_line(start, end, border_color, border_width);
            }
        }

        true
    }
}
mod fonts;
mod text;
mod ui;
