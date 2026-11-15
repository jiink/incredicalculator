//! Scripted apps: apps whose behavior comes from a Lua script instead of
//! compiled-in Rust. Built with the `scripting` feature.
//!
//! The Lua bindings implement [`ScriptEngine`], and
//! [`ScriptApp`](crate::apps::ScriptApp) wraps the engine as an
//! [`IcApp`](crate::app::IcApp), so the shell and the app logic stay separate
//! from the language bindings.
//!
//! A script's drawing calls go straight to the platform, which the app lends
//! to [`ScriptEngine::update`] for the length of the call, just as compiled
//! apps draw during their own `update`. Nothing is buffered.

use alloc::string::String;
use alloc::vec::Vec;
use rgb::RGB8;

use crate::hid::{HidReport, HidState};
use crate::input::IcKey;
use crate::platform::IcPlatform;
use crate::text::HAlign;
use crate::ui::MathInput;

mod lua_engine;

/// The engine that runs scripted apps.
pub type ActiveEngine = lua_engine::LuaEngine;

/// Example script bundled into the firmware, as (file name, source).
pub const EXAMPLE_SCRIPT: (&str, &str) = (
    "bouncer.lua",
    include_str!("../../../../example_scripts/bouncer.lua"),
);

/// Small scripts for tests.
#[cfg(test)]
pub(crate) mod test_sources {
    pub const RUNTIME_ERROR: &str = "function update() x = undefined_fn() end";
    pub const SYNTAX_ERROR: &str = "function update( end";
    pub const ENDLESS_LOOP: &str = "function update() while true do end end";
}

/// Most log lines kept from one script call.
pub const MAX_LOG_LINES: usize = 8;
/// Most `math_input` boxes one script may make.
pub const MAX_INPUTS: usize = 12;
/// Characters a `math_input` box holds.
pub const INPUT_CAPACITY: usize = 40;
/// Most HID key changes (`hid_press`/`hid_release`) a script may make in one
/// frame, before they are sent.
pub const MAX_HID_REPORTS: usize = 32;

/// A `math_input` box: the number and expression entry the compiled apps
/// use, with the same keys.
pub type ScriptInput = MathInput<INPUT_CAPACITY>;

/// Everything a script can see of, or do to, the outside world during one
/// call. The app fills in the inputs before each call and drains the outputs
/// afterwards.
pub struct ScriptHost {
    pub millis: u64,
    /// Held keys, with Shift and Super as the key mode (sticky, toggle)
    /// makes them.
    pub keys_down: [bool; IcKey::COUNT],
    /// The key being handled, during `on_key`.
    pub key: Option<IcKey>,
    pub logs: Vec<String>,
    /// The script's `math_input` boxes. They last while the app is open.
    pub inputs: Vec<ScriptInput>,
    /// Keys the script holds down on the computer.
    pub hid: HidState,
    /// Reports for the changes to `hid`, in order, waiting to be sent.
    pub hid_reports: Vec<HidReport>,
    /// Whether the computer can receive the script's key presses.
    pub hid_connected: bool,
}

impl ScriptHost {
    pub const fn new() -> Self {
        Self {
            millis: 0,
            keys_down: [false; IcKey::COUNT],
            key: None,
            logs: Vec::new(),
            inputs: Vec::new(),
            hid: HidState::new(),
            hid_reports: Vec::new(),
            hid_connected: false,
        }
    }

    /// Queues a report for the computer, or fails once the per-call cap is
    /// reached.
    pub fn push_hid_report(&mut self, report: HidReport) -> Result<(), &'static str> {
        if self.hid_reports.len() >= MAX_HID_REPORTS {
            return Err("too many hid_press/hid_release changes in one frame");
        }
        self.hid_reports.push(report);
        Ok(())
    }

    pub fn shifted(&self) -> bool {
        self.keys_down[IcKey::Shift as usize]
    }

    pub fn super_held(&self) -> bool {
        self.keys_down[IcKey::Super as usize]
    }

    pub fn log(&mut self, line: &str) {
        if self.logs.len() < MAX_LOG_LINES {
            self.logs.push(String::from(line));
        }
    }
}

impl core::fmt::Debug for ScriptHost {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ScriptHost")
            .field("millis", &self.millis)
            .field("key", &self.key)
            .field("logs", &self.logs)
            .field("inputs", &self.inputs.len())
            .finish_non_exhaustive()
    }
}

impl Default for ScriptHost {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings a script declares about itself. Anything left out falls back to
/// a default chosen by [`ScriptApp`](crate::apps::ScriptApp).
#[derive(Clone, Debug, Default)]
pub struct ScriptMeta {
    pub name: Option<String>,
    /// 0xRRGGBB.
    pub color: Option<u32>,
    /// Whether `update` should run every frame rather than only on input.
    pub realtime: bool,
}

/// A loaded script, ready to have its hooks called. Errors are human-readable
/// messages meant for the on-screen error view.
///
/// Every hook is optional in the script; calling a hook the script does not
/// define does nothing.
pub trait ScriptEngine: Sized {
    /// File extension of scripts for this engine, without the dot.
    const FILE_EXTENSION: &'static str;

    /// Compiles `source` and runs its top level, which may declare metadata
    /// and make `math_input` boxes in `host`.
    fn load(source: &str, host: &mut ScriptHost) -> Result<(Self, ScriptMeta), String>;
    /// Runs the script's `init` hook. Called once, the first time the app is
    /// opened.
    fn init(&mut self, host: &mut ScriptHost) -> Result<(), String>;
    /// Runs the script's `update` hook, drawing on `platform` as the script
    /// calls the drawing functions. Drawing only works here; in other hooks
    /// it is an error.
    fn update(
        &mut self,
        host: &mut ScriptHost,
        platform: &mut dyn IcPlatform,
    ) -> Result<(), String>;
    /// Runs the script's `on_key` hook with the key's script name (see
    /// [`IcKey::name`]) and what its legend means with the modifiers in
    /// `host.keys_down` (see [`key_action`]).
    fn on_key(&mut self, key: IcKey, host: &mut ScriptHost) -> Result<(), String>;
    /// Runs the script's `on_key_up` hook with the key's script name.
    fn on_key_up(&mut self, key: IcKey, host: &mut ScriptHost) -> Result<(), String>;
}

/// What a key press means by the legends printed on the keypad, given the
/// held modifiers, or `None` for Shift and Super themselves and for the
/// presses the shell keeps (see [`IcKey::legends`]).
pub fn key_action(key: IcKey, shift: bool, super_held: bool) -> Option<&'static str> {
    key.legends().with(shift, super_held).action
}

#[cfg(test)]
#[test]
fn every_key_has_one_meaning_per_modifier() {
    for key in IcKey::all().take_while(|&key| key != IcKey::Shift) {
        assert!(key_action(key, false, false).is_some(), "{key:?}");
        assert!(key_action(key, true, false).is_some(), "{key:?}");
    }
    assert_eq!(key_action(IcKey::Num9, false, true), None);
    assert_eq!(key_action(IcKey::Num9, true, false), Some("0x"));
    assert_eq!(key_action(IcKey::Shift, true, false), None);
}

pub(crate) fn align_from_name(name: &str) -> Option<HAlign> {
    match name {
        "left" => Some(HAlign::Left),
        "center" => Some(HAlign::Center),
        "right" => Some(HAlign::Right),
        _ => None,
    }
}

pub(crate) fn color_from_int(color: i64) -> RGB8 {
    crate::platform::rgb8_hex(color as u32 & 0xFF_FFFF)
}
