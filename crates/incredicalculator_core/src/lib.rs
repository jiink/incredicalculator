#![no_std]
// IC stands for Incredicalculator
#[macro_use]
extern crate num_derive;
extern crate alloc;

pub mod app;
pub mod apps;
pub mod audio_engine;
pub mod hid;
pub mod input;
pub mod platform;
#[cfg(feature = "scripting")]
pub mod script;
pub mod settings;
pub mod shell;

#[cfg(all(test, feature = "scripting"))]
mod fake_platform;

/// See build.rs.
pub const FIRMWARE_VERSION: &str = env!("INCREDICALCULATOR_VERSION");
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
        let shapes = frame.shapes.iter().map(|polygon| ShapeView {
            points: polygon.points.iter().copied(),
            color: polygon.color,
            border_width: polygon.border_width,
            border_color: polygon.border_color,
            open: polygon.open,
        });
        draw_shapes(platform, shapes, transform);
        true
    }

    /// One shape of a vitmap frame, borrowed from wherever the frame is
    /// stored: the compiled-in [`Vitmap`]s or a script's
    /// [`vm1::PackedVitmap`].
    pub struct ShapeView<P> {
        pub points: P,
        pub color: Color,
        /// 0 means no border.
        pub border_width: f32,
        pub border_color: Color,
        /// Open shapes are a polyline: border only, no fill, no closing edge.
        pub open: bool,
    }

    /// Points of one shape kept on the stack while drawing it; shapes with
    /// more go on the heap.
    const STACK_POINTS: usize = 64;

    /// Draws `shapes` in order, each one's border first and then its fill.
    /// A fill with zero alpha is skipped; other alpha is drawn opaque.
    ///
    /// `transform` maps coordinates from vitmap-local space to screen space.
    pub fn draw_shapes<P: Iterator<Item = Point>>(
        platform: &mut dyn IcPlatform,
        shapes: impl IntoIterator<Item = ShapeView<P>>,
        transform: Affine2,
    ) {
        // Borders scale with the shape so they keep their thickness relative
        // to it. Under a stretch the axis scales differ, so this uses their
        // geometric mean (the square root of the area scale).
        let border_scale = libm::sqrtf(transform.matrix2.determinant().abs());
        let mut stack = [Vec2::ZERO; STACK_POINTS];
        let mut heap = Vec::new();
        for shape in shapes {
            heap.clear();
            let mut count = 0;
            for point in shape.points {
                let point = transform.transform_point2(Vec2::new(point.x, point.y));
                if count < STACK_POINTS {
                    stack[count] = point;
                } else {
                    if count == STACK_POINTS {
                        heap.extend_from_slice(&stack);
                    }
                    heap.push(point);
                }
                count += 1;
            }
            let points = if count <= STACK_POINTS {
                &stack[..count]
            } else {
                &heap[..]
            };

            let border_width = if shape.border_width > 0.0 {
                (libm::roundf(shape.border_width * border_scale) as u32).max(1)
            } else {
                0
            };
            if border_width > 0 && points.len() >= 2 {
                let border_color = RGB8::new(
                    shape.border_color.r,
                    shape.border_color.g,
                    shape.border_color.b,
                );
                let edge_count = if shape.open {
                    points.len() - 1
                } else {
                    points.len()
                };
                for index in 0..edge_count {
                    let end = points[(index + 1) % points.len()];
                    platform.draw_line(points[index], end, border_color, border_width);
                }
            }

            if !shape.open && points.len() >= 3 && shape.color.a > 0 {
                let color = RGB8::new(shape.color.r, shape.color.g, shape.color.b);
                platform.draw_polygon(points, color);
            }
        }
    }

    #[cfg(feature = "scripting")]
    pub mod vm1;
}
pub mod fonts;
mod text;
mod ui;
