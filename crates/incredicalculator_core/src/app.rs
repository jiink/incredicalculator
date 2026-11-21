use crate::audio_engine::AudioEngine;
#[cfg(feature = "scripting")]
use crate::graphics::vm1::PackedVitmap;
use crate::graphics::{Vitmap, draw_vitmap};
use crate::input::{IcKey, KeyMode, KeyState};
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{HAlign, TextStyle, draw_text, text_width};
use glam::Vec2;
use rgb::RGB8;

/// An app's icon in the menu, about 16 × 16 units around its origin.
#[derive(Clone, Copy)]
pub enum AppIcon<'a> {
    /// A vitmap compiled into the firmware.
    Static(&'a Vitmap),
    /// A vitmap from a script's header.
    #[cfg(feature = "scripting")]
    Packed(&'a PackedVitmap),
    /// The first two letters of this name, for an app without a picture.
    Initials(&'a str),
}

/// Height of an [`AppIcon::Initials`] capital, in icon units.
const INITIALS_CAP_HEIGHT: f32 = 12.0;
/// Widest an [`AppIcon::Initials`] pair may be, in icon units; wider letters
/// are drawn smaller.
const INITIALS_MAX_WIDTH: f32 = 18.0;

impl AppIcon<'_> {
    /// Draws the icon centered on `position`: a vitmap's first frame of its
    /// first action, or the initials.
    pub fn draw(self, platform: &mut dyn IcPlatform, position: Vec2, scale: f32) {
        match self {
            Self::Static(vitmap) => {
                draw_vitmap(platform, vitmap, 0, position, Vec2::splat(scale));
            }
            Self::Initials(name) => {
                let mut buffer = [0u8; 8];
                let initials = initials(name, &mut buffer);
                // Capitals are about 2.1 × size tall and span from
                // y - 1.2 × size to y + 0.9 × size.
                let mut size = INITIALS_CAP_HEIGHT / 2.1 * scale;
                let style = |size| {
                    TextStyle::new(size, rgb8_hex(0xffffff))
                        .weight(0.4)
                        .align(HAlign::Center)
                };
                let width = text_width(initials, size, style(size).font);
                let max_width = INITIALS_MAX_WIDTH * scale;
                if width > max_width {
                    size *= max_width / width;
                }
                let baseline = Vec2::new(position.x, position.y + 0.15 * size);
                draw_text(platform, initials, baseline, style(size));
            }
            #[cfg(feature = "scripting")]
            Self::Packed(vitmap) => {
                if let Some(shapes) = vitmap.shapes(0, 0) {
                    let transform = glam::Affine2::from_scale_angle_translation(
                        Vec2::splat(scale),
                        0.0,
                        position,
                    );
                    crate::graphics::draw_shapes(platform, shapes, transform);
                }
            }
        }
    }
}

fn initials<'b>(name: &str, buffer: &'b mut [u8; 8]) -> &'b str {
    let mut len = 0;
    for c in name.chars().filter(|c| !c.is_whitespace()).take(4) {
        len += c.encode_utf8(&mut buffer[len..]).len();
    }
    core::str::from_utf8(&buffer[..len]).unwrap_or_default()
}

#[cfg(test)]
#[test]
fn initials_are_the_first_x_non_space_characters() {
    let mut buffer = [0u8; 8];
    assert_eq!(initials("Ohm's Law", &mut buffer), "Ohm'");
    assert_eq!(initials("  A Game", &mut buffer), "AGam");
    assert_eq!(initials("X", &mut buffer), "X");
    assert_eq!(initials("", &mut buffer), "");
    assert_eq!(initials("ééé", &mut buffer), "ééé");
}

pub struct InputContext<'a> {
    /// Physical key transitions with effective (possibly latched) modifier held states.
    pub key_states: &'a [KeyState; IcKey::COUNT],
    pub key_mode: KeyMode,
    /// Effective shift turned on this frame (physical press, or latch in Sticky/Toggle).
    pub shift_engaged: bool,
    /// Effective shift turned off this frame.
    pub shift_disengaged: bool,
}

impl<'a> InputContext<'a> {
    pub fn is_down(&self, key: IcKey) -> bool {
        self.key_states[key as usize].is_down
    }
    pub fn is_shifted(&self) -> bool {
        self.key_states[IcKey::Shift as usize].is_down
    }
    pub fn is_super(&self) -> bool {
        self.key_states[IcKey::Super as usize].is_down
    }
}

pub trait IcApp {
    fn name(&self) -> &str;
    fn icon(&self) -> AppIcon<'_>;
    fn color(&self) -> RGB8;
    /// for things like games that shouldnt only update when you press a button
    fn requires_realtime_updates(&self) -> bool;
    fn on_enter(&mut self);
    /// Called when the user leaves the app for the menu. Apps can release
    /// memory they only need while open.
    fn on_exit(&mut self) {}
    fn on_key(&mut self, key: IcKey, ctx: &InputContext);
    /// Called when a key is released. Releases of keys the shell kept for
    /// itself are not passed on.
    fn on_key_up(&mut self, _key: IcKey, _ctx: &InputContext) {}
    /// Sends the keyboard and media-key reports the app has queued for the
    /// computer (see [`crate::hid`]). The shell calls it after `update` and
    /// after `on_exit`, so keys an app held are let up when it closes.
    fn send_hid(&mut self, _platform: &mut dyn IcPlatform) {}
    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        audio: &mut AudioEngine,
    );
    /// Whether the app is generating its own samples through `render_audio`.
    /// While true, the platform keeps audio output running.
    fn produces_audio(&self) -> bool {
        false
    }
    /// Writes the app's next samples into `out_pcm`, at
    /// `audio_engine::SAMPLE_RATE_HZ`. The shell applies the volume and mixes
    /// them over the synth. Only called while `produces_audio` is true.
    fn render_audio(&mut self, _out_pcm: &mut [i16]) {}
}
