//! Keys a scripted app can press on the computer the calculator is plugged
//! into, as a USB keyboard and media-key (consumer control) device, and a
//! gamepad it can play with.
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
/// Report ID of the gamepad report: 16 buttons, a hat switch (the d-pad),
/// two sticks and two triggers.
pub const GAMEPAD_REPORT_ID: u8 = 3;
/// Longest report, including its ID byte.
pub const MAX_REPORT_LEN: usize = 10;
/// Non-modifier keyboard keys that can be held at once.
pub const MAX_HELD_KEYS: usize = 6;
/// Highest consumer usage the media report can carry.
pub const MAX_MEDIA_USAGE: u16 = 0x03FF;
/// Gamepad buttons, numbered 1 to this.
pub const GAMEPAD_BUTTONS: u8 = 16;
/// Hat switch value for a centered d-pad (out of its 0-7 range, so null).
const HAT_CENTERED: u8 = 8;

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
    // Gamepad
    0x05, 0x01,       // Usage Page (Generic Desktop)
    0x09, 0x05,       // Usage (Game Pad)
    0xA1, 0x01,       // Collection (Application)
    0x85, GAMEPAD_REPORT_ID,
    0x05, 0x09,       //   Usage Page (Button)
    0x19, 0x01,       //   Usage Minimum (1)
    0x29, GAMEPAD_BUTTONS, // Usage Maximum (16)
    0x15, 0x00,       //   Logical Minimum (0)
    0x25, 0x01,       //   Logical Maximum (1)
    0x75, 0x01,       //   Report Size (1)
    0x95, GAMEPAD_BUTTONS, // Report Count (16)
    0x81, 0x02,       //   Input (Data, Variable, Absolute): buttons
    0x05, 0x01,       //   Usage Page (Generic Desktop)
    0x09, 0x39,       //   Usage (Hat Switch)
    0x15, 0x00,       //   Logical Minimum (0)
    0x25, 0x07,       //   Logical Maximum (7)
    0x35, 0x00,       //   Physical Minimum (0)
    0x46, 0x3B, 0x01, //   Physical Maximum (315)
    0x65, 0x14,       //   Unit (Degrees)
    0x75, 0x04,       //   Report Size (4)
    0x95, 0x01,       //   Report Count (1)
    0x81, 0x42,       //   Input (Data, Variable, Absolute, Null State): hat
    0x65, 0x00,       //   Unit (None)
    0x45, 0x00,       //   Physical Maximum (0): physical = logical again
    0x75, 0x04,       //   Report Size (4)
    0x95, 0x01,       //   Report Count (1)
    0x81, 0x01,       //   Input (Constant): padding
    // Triggers share the sticks' -127..127 range but only use 0..127, so a
    // let-go trigger sits at the axis center. A 0..255 trigger rests at
    // one end, which mapping screens read as an axis held all the way.
    0x09, 0x30,       //   Usage (X): left stick
    0x09, 0x31,       //   Usage (Y)
    0x09, 0x32,       //   Usage (Z): right stick
    0x09, 0x35,       //   Usage (Rz)
    0x09, 0x33,       //   Usage (Rx): left trigger
    0x09, 0x34,       //   Usage (Ry): right trigger
    0x15, 0x81,       //   Logical Minimum (-127)
    0x25, 0x7F,       //   Logical Maximum (127)
    0x75, 0x08,       //   Report Size (8)
    0x95, 0x06,       //   Report Count (6)
    0x81, 0x02,       //   Input (Data, Variable, Absolute): sticks, triggers
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

/// A gamepad control a script can press: a numbered button, a d-pad
/// direction, or a trigger pulled all the way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadButton {
    /// Button 1-16, stored as 0-15.
    Button(u8),
    Dpad(Dpad),
    /// A trigger pulled fully in.
    Trigger(Trigger),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dpad {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Left,
    Right,
}

/// A gamepad axis a script can set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadAxis {
    /// Stick axis, -1.0 to 1.0: left stick X, Y, right stick X, Y. Y is
    /// positive downwards, as gamepads report it.
    Stick(u8),
    /// Trigger, 0.0 (let go) to 1.0 (pulled fully).
    Trigger(Trigger),
}

/// Named gamepad buttons, by their usual place on an Xbox-style pad. Games
/// see only button numbers, so these are a convention, not a promise.
const PAD_BUTTON_NAMES: &[(&str, u8)] = &[
    ("a", 1),
    ("b", 2),
    ("x", 3),
    ("y", 4),
    ("lb", 5),
    ("rb", 6),
    ("back", 7),
    ("start", 8),
    ("l3", 9),
    ("r3", 10),
    ("home", 11),
];

const STICK_NAMES: [&str; 4] = ["lx", "ly", "rx", "ry"];

impl PadButton {
    /// The control a script names, like `"a"`, `"start"`, `"up"` or `"lt"`.
    pub fn from_name(name: &str) -> Option<PadButton> {
        if let Some(&(_, number)) = PAD_BUTTON_NAMES.iter().find(|(n, _)| *n == name) {
            return Some(PadButton::Button(number - 1));
        }
        Some(match name {
            "select" => PadButton::Button(6),
            "up" => PadButton::Dpad(Dpad::Up),
            "right" => PadButton::Dpad(Dpad::Right),
            "down" => PadButton::Dpad(Dpad::Down),
            "left" => PadButton::Dpad(Dpad::Left),
            "lt" => PadButton::Trigger(Trigger::Left),
            "rt" => PadButton::Trigger(Trigger::Right),
            _ => return None,
        })
    }

    /// Button `number`, 1-16.
    pub fn from_number(number: i64) -> Option<PadButton> {
        (1..=GAMEPAD_BUTTONS as i64)
            .contains(&number)
            .then(|| PadButton::Button(number as u8 - 1))
    }
}

impl PadAxis {
    /// The axis a script names: `"lx"`, `"ly"`, `"rx"`, `"ry"`, `"lt"` or
    /// `"rt"`.
    pub fn from_name(name: &str) -> Option<PadAxis> {
        match name {
            "lt" => Some(PadAxis::Trigger(Trigger::Left)),
            "rt" => Some(PadAxis::Trigger(Trigger::Right)),
            _ => STICK_NAMES
                .iter()
                .position(|n| *n == name)
                .map(|i| PadAxis::Stick(i as u8)),
        }
    }
}

/// The whole state of the gamepad, as one report carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GamepadReport {
    /// Bit n is button n + 1.
    pub buttons: u16,
    /// 0 (up) to 7 (up-left) clockwise, or 8 when centered.
    pub hat: u8,
    /// Left X, left Y, right X, right Y, -127 to 127.
    pub sticks: [i8; 4],
    /// Left, right, 0 (let go) to 127.
    pub triggers: [i8; 2],
}

impl GamepadReport {
    /// Nothing pressed, sticks centered, triggers let go.
    pub const NEUTRAL: Self = Self {
        buttons: 0,
        hat: HAT_CENTERED,
        sticks: [0; 4],
        triggers: [0; 2],
    };
}

/// The held gamepad controls, from which [`GamepadReport`] is made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GamepadState {
    buttons: u16,
    /// Bit per [`Dpad`] direction, so opposite directions can both be held.
    dpad: u8,
    sticks: [i8; 4],
    triggers: [i8; 2],
}

impl GamepadState {
    const fn new() -> Self {
        Self {
            buttons: 0,
            dpad: 0,
            sticks: [0; 4],
            triggers: [0; 2],
        }
    }

    /// The hat switch value for the held d-pad directions. Opposite
    /// directions cancel out.
    fn hat(&self) -> u8 {
        let held = |d: Dpad| self.dpad & (1 << d as u8) != 0;
        let vertical = held(Dpad::Down) as i8 - held(Dpad::Up) as i8;
        let horizontal = held(Dpad::Right) as i8 - held(Dpad::Left) as i8;
        match (vertical, horizontal) {
            (-1, 0) => 0,
            (-1, 1) => 1,
            (0, 1) => 2,
            (1, 1) => 3,
            (1, 0) => 4,
            (1, -1) => 5,
            (0, -1) => 6,
            (-1, -1) => 7,
            _ => HAT_CENTERED,
        }
    }

    fn report(&self) -> GamepadReport {
        GamepadReport {
            buttons: self.buttons,
            hat: self.hat(),
            sticks: self.sticks,
            triggers: self.triggers,
        }
    }
}

/// `x` rounded to the nearest integer, halves away from zero (no `round` in
/// `core`).
fn round_half_away(x: f32) -> f32 {
    (x + 0.5f32.copysign(x)) as i32 as f32
}

/// The whole state of one report, sent each time it changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HidReport {
    Keyboard {
        modifiers: u8,
        keys: [u8; MAX_HELD_KEYS],
    },
    Media(u16),
    Gamepad(GamepadReport),
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
            HidReport::Gamepad(pad) => {
                buf[0] = GAMEPAD_REPORT_ID;
                buf[1..3].copy_from_slice(&pad.buttons.to_le_bytes());
                buf[3] = pad.hat;
                for (byte, stick) in buf[4..8].iter_mut().zip(pad.sticks) {
                    *byte = stick as u8;
                }
                for (byte, trigger) in buf[8..10].iter_mut().zip(pad.triggers) {
                    *byte = trigger as u8;
                }
                10
            }
        }
    }

    /// Which of the three reports this is: 0 keyboard, 1 media, 2 gamepad.
    pub fn kind_index(&self) -> usize {
        match self {
            HidReport::Keyboard { .. } => 0,
            HidReport::Media(_) => 1,
            HidReport::Gamepad(_) => 2,
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
            HidReport::Gamepad(pad) => {
                f.write_str("gamepad:")?;
                if pad == GamepadReport::NEUTRAL {
                    return f.write_str(" none");
                }
                for bit in 0..GAMEPAD_BUTTONS {
                    if pad.buttons & (1 << bit) == 0 {
                        continue;
                    }
                    match PAD_BUTTON_NAMES.iter().find(|(_, n)| *n == bit + 1) {
                        Some((name, _)) => write!(f, " {name}")?,
                        None => write!(f, " {}", bit + 1)?,
                    }
                }
                const HAT_NAMES: [&str; 8] = [
                    "up",
                    "up_right",
                    "right",
                    "down_right",
                    "down",
                    "down_left",
                    "left",
                    "up_left",
                ];
                if let Some(name) = HAT_NAMES.get(pad.hat as usize) {
                    write!(f, " {name}")?;
                }
                for (name, value) in STICK_NAMES.iter().zip(pad.sticks) {
                    if value != 0 {
                        write!(f, " {name}={:.2}", value as f32 / 127.0)?;
                    }
                }
                for (name, value) in ["lt", "rt"].iter().zip(pad.triggers) {
                    if value != 0 {
                        write!(f, " {name}={:.2}", value as f32 / 127.0)?;
                    }
                }
                Ok(())
            }
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
    pad: GamepadState,
}

impl HidState {
    pub const fn new() -> Self {
        Self {
            modifiers: 0,
            keys: [0; MAX_HELD_KEYS],
            media: 0,
            pad: GamepadState::new(),
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

    /// Changes the gamepad with `change`, returning its report if anything
    /// changed.
    fn change_pad(&mut self, change: impl FnOnce(&mut GamepadState)) -> Option<HidReport> {
        let before = self.pad.report();
        change(&mut self.pad);
        let after = self.pad.report();
        // Holding opposite d-pad directions changes nothing the computer sees.
        (after != before).then_some(HidReport::Gamepad(after))
    }

    /// Holds a gamepad control down. Returns the report to send, or `None`
    /// if nothing the computer sees changed.
    pub fn pad_press(&mut self, button: PadButton) -> Option<HidReport> {
        self.change_pad(|pad| match button {
            PadButton::Button(n) => pad.buttons |= 1 << n,
            PadButton::Dpad(d) => pad.dpad |= 1 << d as u8,
            PadButton::Trigger(t) => pad.triggers[t as usize] = i8::MAX,
        })
    }

    /// Lets a gamepad control go. Returns the report to send, or `None` if
    /// it wasn't held.
    pub fn pad_release(&mut self, button: PadButton) -> Option<HidReport> {
        self.change_pad(|pad| match button {
            PadButton::Button(n) => pad.buttons &= !(1 << n),
            PadButton::Dpad(d) => pad.dpad &= !(1 << d as u8),
            PadButton::Trigger(t) => pad.triggers[t as usize] = 0,
        })
    }

    /// Moves a gamepad axis: a stick to -1.0..=1.0, or a trigger to
    /// 0.0..=1.0. Values outside the range are clamped. Returns the report to
    /// send, or `None` if the axis was already there.
    pub fn pad_axis(&mut self, axis: PadAxis, value: f32) -> Option<HidReport> {
        let value = if value.is_nan() { 0.0 } else { value };
        self.change_pad(|pad| match axis {
            PadAxis::Stick(i) => {
                pad.sticks[i as usize] = round_half_away(value.clamp(-1.0, 1.0) * 127.0) as i8
            }
            PadAxis::Trigger(t) => {
                pad.triggers[t as usize] = round_half_away(value.clamp(0.0, 1.0) * 127.0) as i8
            }
        })
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
        let pad = (self.pad.report() != GamepadReport::NEUTRAL)
            .then_some(HidReport::Gamepad(GamepadReport::NEUTRAL));
        *self = Self::new();
        keyboard.into_iter().chain(media).chain(pad)
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::new()
    }
}
