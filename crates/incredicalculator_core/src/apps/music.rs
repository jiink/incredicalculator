//! Pieces shared by the music apps: sounds, note names, and the screen
//! layout that mirrors the device's keypad and function keys.

use crate::app::InputContext;
use crate::audio_engine::AudioPatch;
use crate::input::IcKey;
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{self, HAlign, TextStyle};
use core::fmt;
use glam::{IVec2, Vec2};
use rgb::RGB8;

pub const KEY_COUNT: usize = 10;
/// Indexed by key number.
pub const NUMBER_KEYS: [IcKey; KEY_COUNT] = [
    IcKey::Num0,
    IcKey::Num1,
    IcKey::Num2,
    IcKey::Num3,
    IcKey::Num4,
    IcKey::Num5,
    IcKey::Num6,
    IcKey::Num7,
    IcKey::Num8,
    IcKey::Num9,
];

pub const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

pub const MIN_OCTAVE: i8 = 1;
pub const MAX_OCTAVE: i8 = 7;
pub const DEFAULT_OCTAVE: i8 = 4;

pub const BG_COLOR: RGB8 = rgb8_hex(0x1d1b3a);
pub const TEXT_COLOR: RGB8 = rgb8_hex(0xffffff);
pub const DIM_TEXT_COLOR: RGB8 = rgb8_hex(0xa9a6d6);
pub const CELL_COLOR: RGB8 = rgb8_hex(0x34306a);
pub const ROOT_CELL_COLOR: RGB8 = rgb8_hex(0x4a3f8f);
pub const HELD_CELL_COLOR: RGB8 = rgb8_hex(0x7a5cff);
pub const SOUNDING_CELL_COLOR: RGB8 = rgb8_hex(0xffc94a);
pub const SOUNDING_TEXT_COLOR: RGB8 = rgb8_hex(0x1d1b3a);

pub struct SoundPreset {
    pub name: &'static str,
    pub patch: AudioPatch,
}

pub const SOUND_PRESETS: [SoundPreset; 5] = [
    SoundPreset {
        name: "Soft",
        patch: AudioPatch {
            attack_ms: 20,
            release_ms: 600,
            triangle_mix: 204,
            square_mix: 0,
            vibrato_depth_cents: 5,
        },
    },
    SoundPreset {
        name: "Pluck",
        patch: AudioPatch {
            attack_ms: 0,
            release_ms: 250,
            triangle_mix: 170,
            square_mix: 34,
            vibrato_depth_cents: 0,
        },
    },
    SoundPreset {
        name: "Reed",
        patch: AudioPatch {
            attack_ms: 30,
            release_ms: 300,
            triangle_mix: 102,
            square_mix: 102,
            vibrato_depth_cents: 8,
        },
    },
    SoundPreset {
        name: "Chip",
        patch: AudioPatch {
            attack_ms: 0,
            release_ms: 80,
            triangle_mix: 0,
            square_mix: 204,
            vibrato_depth_cents: 0,
        },
    },
    SoundPreset {
        name: "Pad",
        patch: AudioPatch {
            attack_ms: 500,
            release_ms: 2_500,
            triangle_mix: 170,
            square_mix: 34,
            vibrato_depth_cents: 6,
        },
    },
];

/// Step `idx` forward or backward through `len` items, wrapping around.
pub fn cycle(idx: usize, len: usize, forward: bool) -> usize {
    if forward {
        (idx + 1) % len
    } else {
        (idx + len - 1) % len
    }
}

/// Octave number of a MIDI note, where middle C (60) is C4.
pub fn octave_of(midi_note: u8) -> i16 {
    midi_note as i16 / 12 - 1
}

pub const KEYPAD_CELL_W: i32 = 44;
pub const KEYPAD_CELL_H: i32 = 42;
/// Left edge of the panel between the keypad and the function column.
pub const PANEL_LEFT: f32 = 158.0;

/// Top-left corner of a key's cell. The grid mirrors the device's keypad:
/// 7-8-9 on top, 0 alone at the bottom left.
pub fn keypad_cell_origin(key_idx: usize) -> IVec2 {
    const GAP: i32 = 3;
    const LEFT: i32 = 8;
    const TOP: i32 = 56;
    let (col, row) = match key_idx {
        0 => (0, 3),
        n => ((n - 1) % 3, 2 - (n - 1) / 3),
    };
    IVec2::new(
        LEFT + col as i32 * (KEYPAD_CELL_W + GAP),
        TOP + row as i32 * (KEYPAD_CELL_H + GAP),
    )
}

pub fn draw_keypad_cell(platform: &mut dyn IcPlatform, origin: IVec2, fill: RGB8) {
    platform.draw_rectangle_rounded(
        origin,
        origin + IVec2::new(KEYPAD_CELL_W, KEYPAD_CELL_H),
        fill,
        0,
        Some(fill),
        4,
    );
}

/// Clears the screen and draws the title and subtitle lines.
pub fn draw_header(platform: &mut dyn IcPlatform, title: fmt::Arguments, subtitle: fmt::Arguments) {
    platform.clear(BG_COLOR);
    text::draw_text_f(
        platform,
        title,
        Vec2::new(8.0, 17.0),
        TextStyle::new(8.0, TEXT_COLOR),
    );
    text::draw_text_f(
        platform,
        subtitle,
        Vec2::new(8.0, 44.0),
        TextStyle::new(5.0, DIM_TEXT_COLOR),
    );
}

/// Draws text at `style.scale`, shrinking it if needed to fit `max_width`.
pub fn draw_text_fit(
    platform: &mut dyn IcPlatform,
    arg: fmt::Arguments,
    pos: Vec2,
    style: TextStyle,
    max_width: f32,
) {
    let mut buf = [0u8; 32];
    let Ok(text) = format_no_std::show(&mut buf, arg) else {
        return;
    };
    let width = text::text_width(text, style.scale, style.font);
    let scale = if width > max_width {
        style.scale * max_width / width
    } else {
        style.scale
    };
    text::draw_text(platform, text, pos, TextStyle { scale, ..style });
}

/// The F1-F6 labels for whichever modifier is held. Super+F5/F6 always
/// belong to the shell's volume and brightness.
pub fn function_labels<'a>(
    ctx: &InputContext,
    plain: &'a [&'a str; 6],
    shifted: &'a [&'a str; 6],
    super_held: [&'a str; 4],
) -> [&'a str; 6] {
    if ctx.is_super() {
        let [f1, f2, f3, f4] = super_held;
        [f1, f2, f3, f4, "Vol", "Light"]
    } else if ctx.is_shifted() {
        *shifted
    } else {
        *plain
    }
}

/// Labels for F1-F6, which run down the right side of the device. Empty
/// labels are skipped, and `lit` highlights the label at that index.
pub fn draw_function_column(platform: &mut dyn IcPlatform, labels: &[&str; 6], lit: Option<usize>) {
    const LEFT: i32 = 262;
    const RIGHT: i32 = 312;
    const CELL_H: i32 = 28;
    const GAP: i32 = 2;
    // Fits between the shell's battery and its modifier icon.
    const TOP: i32 = 25;

    for (idx, label) in labels.iter().enumerate() {
        if label.is_empty() {
            continue;
        }
        let y = TOP + idx as i32 * (CELL_H + GAP);
        let fill = if lit == Some(idx) {
            HELD_CELL_COLOR
        } else {
            CELL_COLOR
        };
        platform.draw_rectangle_rounded(
            IVec2::new(LEFT, y),
            IVec2::new(RIGHT, y + CELL_H),
            fill,
            0,
            Some(fill),
            4,
        );
        text::draw_text(
            platform,
            label,
            Vec2::new((LEFT + RIGHT) as f32 / 2.0, y as f32 + 15.0),
            TextStyle::new(4.0, TEXT_COLOR).align(HAlign::Center),
        );
    }
}
