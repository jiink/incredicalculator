//! Keys a scripted app can press on the computer the calculator is plugged
//! into, as a USB keyboard and media-key (consumer control) device.
//!
//! Apps don't send USB reports themselves: they change a [`HidState`], which
//! returns the [`HidReport`] for each change, and the platform sends the
//! reports in order with [`IcPlatform::send_hid_report`]. The report bytes
//! and [`REPORT_DESCRIPTOR`] are defined together here so they always agree.
//!
//! [`IcPlatform::send_hid_report`]: crate::platform::IcPlatform::send_hid_report

use core::fmt;

/// Report ID of the keyboard report: modifier bits, a reserved byte and up to
/// six held keys.
pub const KEYBOARD_REPORT_ID: u8 = 1;
/// Report ID of the media-key report: one 16-bit consumer usage, 0 for none.
pub const MEDIA_REPORT_ID: u8 = 2;
/// Longest report, including its ID byte.
pub const MAX_REPORT_LEN: usize = 9;
/// Non-modifier keyboard keys that can be held at once.
pub const MAX_HELD_KEYS: usize = 6;
/// Highest consumer usage the media report can carry.
pub const MAX_MEDIA_USAGE: u16 = 0x03FF;

/// HID report descriptor for [`HidReport::to_bytes`].
#[rustfmt::skip]
pub const REPORT_DESCRIPTOR: &[u8] = &[
    // Keyboard
    0x05, 0x01,       // Usage Page (Generic Desktop)
    0x09, 0x06,       // Usage (Keyboard)
    0xA1, 0x01,       // Collection (Application)
    0x85, KEYBOARD_REPORT_ID,
    0x05, 0x07,       //   Usage Page (Keyboard/Keypad)
    0x19, 0xE0,       //   Usage Minimum (Left Control)
    0x29, 0xE7,       //   Usage Maximum (Right GUI)
    0x15, 0x00,       //   Logical Minimum (0)
    0x25, 0x01,       //   Logical Maximum (1)
    0x75, 0x01,       //   Report Size (1)
    0x95, 0x08,       //   Report Count (8)
    0x81, 0x02,       //   Input (Data, Variable, Absolute): modifiers
    0x75, 0x08,       //   Report Size (8)
    0x95, 0x01,       //   Report Count (1)
    0x81, 0x01,       //   Input (Constant): reserved
    0x75, 0x08,       //   Report Size (8)
    0x95, 0x06,       //   Report Count (6)
    0x15, 0x00,       //   Logical Minimum (0)
    0x26, 0xFF, 0x00, //   Logical Maximum (255)
    0x19, 0x00,       //   Usage Minimum (0)
    0x2A, 0xFF, 0x00, //   Usage Maximum (255)
    0x81, 0x00,       //   Input (Data, Array): held keys
    0xC0,             // End Collection
    // Media keys
    0x05, 0x0C,       // Usage Page (Consumer)
    0x09, 0x01,       // Usage (Consumer Control)
    0xA1, 0x01,       // Collection (Application)
    0x85, MEDIA_REPORT_ID,
    0x15, 0x00,       //   Logical Minimum (0)
    0x26, 0xFF, 0x03, //   Logical Maximum (0x3FF)
    0x19, 0x00,       //   Usage Minimum (0)
    0x2A, 0xFF, 0x03, //   Usage Maximum (0x3FF)
    0x75, 0x10,       //   Report Size (16)
    0x95, 0x01,       //   Report Count (1)
    0x81, 0x00,       //   Input (Data, Array)
    0xC0,             // End Collection
];

/// A key on the computer's side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidKey {
    /// A Keyboard/Keypad page usage. 0xE0-0xE7 are the modifiers.
    Keyboard(u8),
    /// A Consumer page usage, such as volume up.
    Media(u16),
}

impl HidKey {
    /// The key a script names, like `"a"`, `"left_ctrl"` or `"volume_up"`.
    pub fn from_name(name: &str) -> Option<HidKey> {
        if let Some(&(_, usage)) = KEYBOARD_NAMES.iter().find(|(n, _)| *n == name) {
            return Some(HidKey::Keyboard(usage));
        }
        if let Some(&(_, usage)) = MEDIA_NAMES.iter().find(|(n, _)| *n == name) {
            return Some(HidKey::Media(usage));
        }
        let bytes = name.as_bytes();
        match bytes {
            [c @ b'a'..=b'z'] => Some(HidKey::Keyboard(0x04 + (c - b'a'))),
            [b'0'] => Some(HidKey::Keyboard(0x27)),
            [c @ b'1'..=b'9'] => Some(HidKey::Keyboard(0x1E + (c - b'1'))),
            _ => {
                let n: u8 = name.strip_prefix('f')?.parse().ok()?;
                match n {
                    1..=12 => Some(HidKey::Keyboard(0x3A + n - 1)),
                    13..=24 => Some(HidKey::Keyboard(0x68 + n - 13)),
                    _ => None,
                }
            }
        }
    }

    fn modifier_bit(self) -> Option<u8> {
        match self {
            HidKey::Keyboard(usage @ 0xE0..=0xE7) => Some(1 << (usage - 0xE0)),
            _ => None,
        }
    }
}

/// Named keyboard keys besides letters, digits and F1-F24.
const KEYBOARD_NAMES: &[(&str, u8)] = &[
    ("enter", 0x28),
    ("escape", 0x29),
    ("backspace", 0x2A),
    ("tab", 0x2B),
    ("space", 0x2C),
    ("minus", 0x2D),
    ("equal", 0x2E),
    ("left_bracket", 0x2F),
    ("right_bracket", 0x30),
    ("backslash", 0x31),
    ("semicolon", 0x33),
    ("quote", 0x34),
    ("grave", 0x35),
    ("comma", 0x36),
    ("period", 0x37),
    ("slash", 0x38),
    ("caps_lock", 0x39),
    ("print_screen", 0x46),
    ("scroll_lock", 0x47),
    ("pause", 0x48),
    ("insert", 0x49),
    ("home", 0x4A),
    ("page_up", 0x4B),
    ("delete", 0x4C),
    ("end", 0x4D),
    ("page_down", 0x4E),
    ("right", 0x4F),
    ("left", 0x50),
    ("down", 0x51),
    ("up", 0x52),
    ("num_lock", 0x53),
    ("kp_divide", 0x54),
    ("kp_multiply", 0x55),
    ("kp_minus", 0x56),
    ("kp_plus", 0x57),
    ("kp_enter", 0x58),
    ("kp_1", 0x59),
    ("kp_2", 0x5A),
    ("kp_3", 0x5B),
    ("kp_4", 0x5C),
    ("kp_5", 0x5D),
    ("kp_6", 0x5E),
    ("kp_7", 0x5F),
    ("kp_8", 0x60),
    ("kp_9", 0x61),
    ("kp_0", 0x62),
    ("kp_period", 0x63),
    ("menu", 0x65),
    ("left_ctrl", 0xE0),
    ("left_shift", 0xE1),
    ("left_alt", 0xE2),
    ("left_gui", 0xE3),
    ("right_ctrl", 0xE4),
    ("right_shift", 0xE5),
    ("right_alt", 0xE6),
    ("right_gui", 0xE7),
    ("ctrl", 0xE0),
    ("shift", 0xE1),
    ("alt", 0xE2),
    ("gui", 0xE3),
];

/// Named media keys (Consumer page usages).
const MEDIA_NAMES: &[(&str, u16)] = &[
    ("volume_up", 0xE9),
    ("volume_down", 0xEA),
    ("mute", 0xE2),
    ("play_pause", 0xCD),
    ("next_track", 0xB5),
    ("prev_track", 0xB6),
    ("stop", 0xB7),
    ("brightness_up", 0x6F),
    ("brightness_down", 0x70),
    ("calculator", 0x192),
];

fn keyboard_name(usage: u8) -> Option<&'static str> {
    KEYBOARD_NAMES
        .iter()
        .find(|(_, u)| *u == usage)
        .map(|(name, _)| *name)
}

fn media_name(usage: u16) -> Option<&'static str> {
    MEDIA_NAMES
        .iter()
        .find(|(_, u)| *u == usage)
        .map(|(name, _)| *name)
}

/// The whole state of one report, sent each time it changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidReport {
    Keyboard {
        modifiers: u8,
        keys: [u8; MAX_HELD_KEYS],
    },
    Media(u16),
}

impl HidReport {
    /// The report as sent over USB, starting with its report ID.
    pub fn to_bytes(&self, buf: &mut [u8; MAX_REPORT_LEN]) -> usize {
        match *self {
            HidReport::Keyboard { modifiers, keys } => {
                buf[0] = KEYBOARD_REPORT_ID;
                buf[1] = modifiers;
                buf[2] = 0;
                buf[3..9].copy_from_slice(&keys);
                9
            }
            HidReport::Media(usage) => {
                buf[0] = MEDIA_REPORT_ID;
                buf[1..3].copy_from_slice(&usage.to_le_bytes());
                3
            }
        }
    }
}

/// Lists the held keys by name, like `keyboard: left_ctrl c` or `media: none`.
impl fmt::Display for HidReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            HidReport::Keyboard { modifiers, keys } => {
                f.write_str("keyboard:")?;
                let held = (0..8)
                    .filter(|bit| modifiers & (1 << bit) != 0)
                    .map(|bit| 0xE0 + bit)
                    .chain(keys.iter().copied().filter(|&k| k != 0));
                let mut any = false;
                for usage in held {
                    any = true;
                    match usage {
                        0x04..=0x1D => write!(f, " {}", (b'a' + usage - 0x04) as char)?,
                        0x1E..=0x26 => write!(f, " {}", usage - 0x1E + 1)?,
                        0x27 => f.write_str(" 0")?,
                        0x3A..=0x45 => write!(f, " f{}", usage - 0x3A + 1)?,
                        0x68..=0x73 => write!(f, " f{}", usage - 0x68 + 13)?,
                        _ => match keyboard_name(usage) {
                            Some(name) => write!(f, " {name}")?,
                            None => write!(f, " 0x{usage:02X}")?,
                        },
                    }
                }
                if !any {
                    f.write_str(" none")?;
                }
                Ok(())
            }
            HidReport::Media(0) => f.write_str("media: none"),
            HidReport::Media(usage) => match media_name(usage) {
                Some(name) => write!(f, "media: {name}"),
                None => write!(f, "media: 0x{usage:03X}"),
            },
        }
    }
}

/// All six key slots are taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooManyKeys;

/// The keys one app holds down on the computer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HidState {
    modifiers: u8,
    keys: [u8; MAX_HELD_KEYS],
    /// One media key at a time; 0 for none.
    media: u16,
}

impl HidState {
    pub const fn new() -> Self {
        Self {
            modifiers: 0,
            keys: [0; MAX_HELD_KEYS],
            media: 0,
        }
    }

    fn keyboard_report(&self) -> HidReport {
        HidReport::Keyboard {
            modifiers: self.modifiers,
            keys: self.keys,
        }
    }

    /// Holds `key` down. Returns the report to send, or `None` if it was
    /// already held. A new media key replaces the one held before.
    pub fn press(&mut self, key: HidKey) -> Result<Option<HidReport>, TooManyKeys> {
        if let Some(bit) = key.modifier_bit() {
            if self.modifiers & bit != 0 {
                return Ok(None);
            }
            self.modifiers |= bit;
            return Ok(Some(self.keyboard_report()));
        }
        match key {
            HidKey::Keyboard(0) => Ok(None),
            HidKey::Keyboard(usage) => {
                if self.keys.contains(&usage) {
                    return Ok(None);
                }
                let slot = self.keys.iter_mut().find(|k| **k == 0).ok_or(TooManyKeys)?;
                *slot = usage;
                Ok(Some(self.keyboard_report()))
            }
            HidKey::Media(usage) => {
                if self.media == usage || usage == 0 {
                    return Ok(None);
                }
                self.media = usage;
                Ok(Some(HidReport::Media(usage)))
            }
        }
    }

    /// Lets `key` up. Returns the report to send, or `None` if it wasn't held.
    pub fn release(&mut self, key: HidKey) -> Option<HidReport> {
        if let Some(bit) = key.modifier_bit() {
            if self.modifiers & bit == 0 {
                return None;
            }
            self.modifiers &= !bit;
            return Some(self.keyboard_report());
        }
        match key {
            HidKey::Keyboard(0) => None,
            HidKey::Keyboard(usage) => {
                let slot = self.keys.iter_mut().find(|k| **k == usage)?;
                *slot = 0;
                Some(self.keyboard_report())
            }
            HidKey::Media(usage) => {
                if self.media != usage || usage == 0 {
                    return None;
                }
                self.media = 0;
                Some(HidReport::Media(0))
            }
        }
    }

    /// Lets every key up, returning the reports to send (none if nothing
    /// was held).
    pub fn release_all(&mut self) -> impl Iterator<Item = HidReport> {
        let keyboard = (self.modifiers != 0 || self.keys.iter().any(|&k| k != 0)).then_some(
            HidReport::Keyboard {
                modifiers: 0,
                keys: [0; MAX_HELD_KEYS],
            },
        );
        let media = (self.media != 0).then_some(HidReport::Media(0));
        *self = Self::new();
        keyboard.into_iter().chain(media)
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::new()
    }
}
