//! Vitmaps decoded at runtime from VM1 strings (`VM1:<base64>`), the format
//! the vitmap editor exports. Scripts make them with `vitmap[[VM1:...]]`.
//!
//! [`PackedVitmap::decode`] checks the whole payload once and records where
//! each frame starts. The shapes stay in their compact encoded form and are
//! read straight from the bytes as they are drawn, so a vitmap costs about as
//! much memory as its encoded data.
//!
//! Layout of the payload (all integers little-endian, counts are LEB128
//! varints):
//!
//! ```text
//! u8 flags (0x01 = 16-bit coordinates)  u8 scale shift  u8 fps (0 = 12)
//! varint action count, then per action:
//!     varint name length, name bytes (UTF-8), u8 flags (0x01 = loops)
//!     varint frame count, then per frame:
//!         varint shape count, then per shape:
//!             u8 flags (0x01 = open, 0x02 = has alpha, 0x04 = has border)
//!             u8 r, g, b  [u8 a]  [u16 border width / 100, u8 r, g, b]
//!             varint point count, then i8 or i16 x, y pairs
//! ```
//!
//! Coordinates are divided by `1 << shift`.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use base64::Engine;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};

use super::{Color, Point, ShapeView};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    MissingPrefix,
    UnsupportedVersion(String),
    BadBase64,
    Truncated,
    BadVarint,
    BadUtf8,
    UnknownFlags,
    BadScale,
    TrailingBytes,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => f.write_str("it should start with \"VM1:\""),
            Self::UnsupportedVersion(version) => {
                write!(f, "{version} vitmaps aren't supported, only VM1")
            }
            Self::BadBase64 => f.write_str("the data after \"VM1:\" isn't valid base64"),
            Self::Truncated => f.write_str("the data ends too soon (was it cut off?)"),
            Self::BadVarint => f.write_str("the data has a malformed count"),
            Self::BadUtf8 => f.write_str("an action name isn't valid UTF-8"),
            Self::UnknownFlags => f.write_str("the data uses flags this firmware doesn't know"),
            Self::BadScale => f.write_str("the coordinate scale is out of range"),
            Self::TrailingBytes => f.write_str("there is extra data after the last action"),
        }
    }
}

impl core::error::Error for DecodeError {}

const FLAG_WIDE: u8 = 0x01;
const ACTION_LOOP: u8 = 0x01;
const SHAPE_OPEN: u8 = 0x01;
const SHAPE_ALPHA: u8 = 0x02;
const SHAPE_BORDER: u8 = 0x04;

/// Standard alphabet (URL-safe characters are mapped to it first), padding
/// optional, unused low bits of the last character ignored.
const BASE64: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_decode_padding_mode(DecodePaddingMode::Indifferent)
        .with_decode_allow_trailing_bits(true),
);

/// A vitmap decoded from a VM1 string.
#[derive(Debug)]
pub struct PackedVitmap {
    /// The decoded payload. Only ever read at offsets [`Self::decode`]
    /// checked.
    bytes: Box<[u8]>,
    wide: bool,
    scale: f32,
    fps: u8,
    actions: Box<[PackedAction]>,
}

#[derive(Debug)]
pub struct PackedAction {
    name: Box<str>,
    looping: bool,
    /// Offset in the payload of each frame's shape count.
    frames: Box<[u32]>,
}

impl PackedAction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn looping(&self) -> bool {
        self.looping
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

impl PackedVitmap {
    /// Decodes a string like `VM1:AAEI...`. Whitespace around and inside
    /// the base64 data is ignored.
    pub fn decode(text: &str) -> Result<Self, DecodeError> {
        let text = text.trim();
        let colon = text.find(':').ok_or(DecodeError::MissingPrefix)?;
        let (prefix, body) = (&text[..colon], &text[colon + 1..]);
        if !prefix.starts_with("VM") {
            return Err(DecodeError::MissingPrefix);
        }
        if prefix != "VM1" {
            return Err(DecodeError::UnsupportedVersion(String::from(prefix)));
        }

        let body: Vec<u8> = body
            .bytes()
            .filter(|c| !c.is_ascii_whitespace())
            .map(|c| match c {
                b'-' => b'+',
                b'_' => b'/',
                c => c,
            })
            .collect();
        let bytes = BASE64
            .decode(body)
            .map_err(|_| DecodeError::BadBase64)?
            .into_boxed_slice();

        let mut r = Reader::new(&bytes);
        let flags = r.u8()?;
        if flags & !FLAG_WIDE != 0 {
            return Err(DecodeError::UnknownFlags);
        }
        let wide = flags & FLAG_WIDE != 0;
        let shift = r.u8()?;
        if shift > 15 {
            return Err(DecodeError::BadScale);
        }
        let fps = match r.u8()? {
            0 => 12,
            n => n,
        };

        let action_count = r.varint()?;
        let mut actions = Vec::new();
        for _ in 0..action_count {
            let name = r.str()?.into();
            let action_flags = r.u8()?;
            if action_flags & !ACTION_LOOP != 0 {
                return Err(DecodeError::UnknownFlags);
            }
            let frame_count = r.varint()?;
            let mut frames = Vec::new();
            for _ in 0..frame_count {
                frames.push(r.i as u32);
                let shape_count = r.varint()?;
                for _ in 0..shape_count {
                    let shape = r.shape_header()?;
                    let point_bytes = if wide { 4 } else { 2 };
                    let len = shape
                        .point_count
                        .checked_mul(point_bytes)
                        .ok_or(DecodeError::Truncated)?;
                    r.take(len)?;
                }
            }
            actions.push(PackedAction {
                name,
                looping: action_flags & ACTION_LOOP != 0,
                frames: frames.into_boxed_slice(),
            });
        }
        if r.i != bytes.len() {
            return Err(DecodeError::TrailingBytes);
        }

        Ok(Self {
            bytes,
            wide,
            scale: (1u32 << shift) as f32,
            fps,
            actions: actions.into_boxed_slice(),
        })
    }

    /// Frames per second the vitmap was made to play at.
    pub fn fps(&self) -> u8 {
        self.fps
    }

    pub fn actions(&self) -> &[PackedAction] {
        &self.actions
    }

    /// Index of the first action called `name`.
    pub fn action_index(&self, name: &str) -> Option<usize> {
        self.actions.iter().position(|a| &*a.name == name)
    }

    /// The shapes of one frame, or `None` when `action` or `frame` is out of
    /// range.
    pub fn shapes(&self, action: usize, frame: usize) -> Option<PackedShapes<'_>> {
        let offset = *self.actions.get(action)?.frames.get(frame)? as usize;
        let mut reader = Reader::new(&self.bytes);
        reader.i = offset;
        let remaining = reader.varint().ok()?;
        Some(PackedShapes {
            reader,
            remaining,
            wide: self.wide,
            scale: self.scale,
        })
    }
}

/// The shapes of one frame of a [`PackedVitmap`], read as they are iterated.
pub struct PackedShapes<'a> {
    reader: Reader<'a>,
    remaining: usize,
    wide: bool,
    scale: f32,
}

impl<'a> Iterator for PackedShapes<'a> {
    type Item = ShapeView<PackedPoints<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        // The payload was checked when it was decoded, so these reads only
        // fail if that check has a bug; then the frame just ends early.
        let shape = self.reader.shape_header().ok()?;
        let point_bytes = if self.wide { 4 } else { 2 };
        let len = shape.point_count.checked_mul(point_bytes)?;
        let bytes = self.reader.take(len).ok()?;
        Some(ShapeView {
            points: PackedPoints {
                bytes,
                wide: self.wide,
                scale: self.scale,
            },
            color: shape.color,
            border_width: shape.border_width,
            border_color: shape.border_color,
            open: shape.open,
        })
    }
}

/// The points of one shape of a [`PackedVitmap`].
#[derive(Clone)]
pub struct PackedPoints<'a> {
    bytes: &'a [u8],
    wide: bool,
    scale: f32,
}

impl Iterator for PackedPoints<'_> {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        let (x, y, rest) = if self.wide {
            let [x0, x1, y0, y1, rest @ ..] = self.bytes else {
                return None;
            };
            let x = i16::from_le_bytes([*x0, *x1]) as f32;
            (x, i16::from_le_bytes([*y0, *y1]) as f32, rest)
        } else {
            let [x, y, rest @ ..] = self.bytes else {
                return None;
            };
            (*x as i8 as f32, *y as i8 as f32, rest)
        };
        self.bytes = rest;
        Some(Point {
            x: x / self.scale,
            y: y / self.scale,
        })
    }
}

struct ShapeHeader {
    color: Color,
    border_width: f32,
    border_color: Color,
    open: bool,
    point_count: usize,
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn new(b: &'a [u8]) -> Self {
        Self { b, i: 0 }
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .i
            .checked_add(len)
            .filter(|&end| end <= self.b.len())
            .ok_or(DecodeError::Truncated)?;
        let bytes = &self.b[self.i..end];
        self.i = end;
        Ok(bytes)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes([self.u8()?, self.u8()?]))
    }

    fn varint(&mut self) -> Result<usize, DecodeError> {
        let mut result: u32 = 0;
        for n in 0..5 {
            let b = self.u8()?;
            let bits = ((b & 0x7f) as u32)
                .checked_shl(7 * n)
                .ok_or(DecodeError::BadVarint)?;
            result |= bits;
            if b & 0x80 == 0 {
                return Ok(result as usize);
            }
        }
        Err(DecodeError::BadVarint)
    }

    fn str(&mut self) -> Result<&'a str, DecodeError> {
        let len = self.varint()?;
        core::str::from_utf8(self.take(len)?).map_err(|_| DecodeError::BadUtf8)
    }

    fn shape_header(&mut self) -> Result<ShapeHeader, DecodeError> {
        let flags = self.u8()?;
        if flags & !(SHAPE_OPEN | SHAPE_ALPHA | SHAPE_BORDER) != 0 {
            return Err(DecodeError::UnknownFlags);
        }
        let (r, g, b) = (self.u8()?, self.u8()?, self.u8()?);
        let a = if flags & SHAPE_ALPHA != 0 {
            self.u8()?
        } else {
            255
        };
        let (border_width, border_color) = if flags & SHAPE_BORDER != 0 {
            let width = self.u16()? as f32 / 100.0;
            let (r, g, b) = (self.u8()?, self.u8()?, self.u8()?);
            (width, Color { r, g, b, a: 255 })
        } else {
            (
                0.0,
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            )
        };
        Ok(ShapeHeader {
            color: Color { r, g, b, a },
            border_width,
            border_color,
            open: flags & SHAPE_OPEN != 0,
            point_count: self.varint()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::test_sources::VM1_SAMPLE;
    use alloc::format;

    fn points(shape: &ShapeView<PackedPoints<'_>>) -> Vec<(f32, f32)> {
        shape.points.clone().map(|p| (p.x, p.y)).collect()
    }

    fn rgba(color: Color) -> (u8, u8, u8, u8) {
        (color.r, color.g, color.b, color.a)
    }

    #[test]
    fn decodes_the_reference_sample() {
        let v = PackedVitmap::decode(VM1_SAMPLE).unwrap();
        assert_eq!(v.fps(), 8);
        assert_eq!(v.actions().len(), 2);

        let idle = &v.actions()[0];
        assert_eq!(idle.name(), "Idle");
        assert!(idle.looping());
        assert_eq!(idle.frame_count(), 1);
        let shapes: Vec<_> = v.shapes(0, 0).unwrap().collect();
        assert_eq!(shapes.len(), 2);

        let tri = &shapes[0];
        assert_eq!(points(tri), [(8.0, 0.0), (0.0, -8.0), (-8.0, 0.0)]);
        assert_eq!(rgba(tri.color), (10, 20, 30, 255));
        assert_eq!(tri.border_width, 0.0);
        assert!(!tri.open);

        let line = &shapes[1];
        assert_eq!(points(line), [(0.5, 1.0), (2.0, 2.0)]);
        assert_eq!(line.color.a, 128);
        assert_eq!(line.border_width, 0.5);
        assert_eq!(rgba(line.border_color), (1, 2, 3, 255));
        assert!(line.open);

        let run = &v.actions()[1];
        assert_eq!(run.name(), "Run");
        assert!(!run.looping());
        assert_eq!(run.frame_count(), 2);
        assert_eq!(v.shapes(1, 0).unwrap().count(), 0);
        assert_eq!(v.shapes(1, 1).unwrap().count(), 0);
        assert_eq!(v.action_index("Run"), Some(1));
        assert_eq!(v.action_index("Walk"), None);
    }

    #[test]
    fn out_of_range_frames_have_no_shapes() {
        let v = PackedVitmap::decode(VM1_SAMPLE).unwrap();
        assert!(v.shapes(0, 1).is_none());
        assert!(v.shapes(1, 2).is_none());
        assert!(v.shapes(2, 0).is_none());
    }

    #[test]
    fn accepts_url_safe_unpadded_and_spaced_base64() {
        let body = VM1_SAMPLE.strip_prefix("VM1:").unwrap();
        let url_safe = body.replace('+', "-").replace('/', "_");
        let unpadded = url_safe.trim_end_matches('=');
        let (a, b) = unpadded.split_at(10);
        let text = format!("  VM1:{a}\n  {b}  \n");
        let v = PackedVitmap::decode(&text).unwrap();
        assert_eq!(v.actions()[1].name(), "Run");
    }

    #[test]
    fn rejects_bad_input() {
        use DecodeError::*;
        let err = |text: &str| PackedVitmap::decode(text).unwrap_err();
        assert_eq!(err("VM2:AAAA"), UnsupportedVersion("VM2".into()));
        assert_eq!(err(&VM1_SAMPLE[..20]), Truncated);
        assert_eq!(err("hello"), MissingPrefix);
        assert_eq!(err("XY1:AAAA"), MissingPrefix);
        assert_eq!(err("VM1:AA*A"), BadBase64);
        // Unknown header flags.
        assert_eq!(err("VM1:AgAAAA"), UnknownFlags);
        // A scale shift above 15.
        assert_eq!(err("VM1:ABAAAA"), BadScale);
        // A valid, empty vitmap, then one with a byte left over.
        assert!(PackedVitmap::decode("VM1:AAAAAA").is_ok());
        assert_eq!(err("VM1:AAAAAAA"), TrailingBytes);
    }
}
