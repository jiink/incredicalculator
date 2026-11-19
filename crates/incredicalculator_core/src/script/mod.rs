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

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use rgb::RGB8;

use crate::graphics::vm1::PackedVitmap;

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
    /// The reference VM1 sample from the vitmap editor's tests: an "Idle"
    /// action with a triangle and an open line, and a two-frame "Run" action.
    pub const VM1_SAMPLE: &str =
        "VM1:AAEIAgRJZGxlAQECAAoUHgMQAADw8AAH/wAAgDIAAQIDAgECBAQDUnVuAAIAAA==";
}

/// Most log lines kept from one script call.
pub const MAX_LOG_LINES: usize = 8;
/// Most `math_input` boxes one script may make.
pub const MAX_INPUTS: usize = 12;
/// Characters a `math_input` box holds.
pub const INPUT_CAPACITY: usize = 40;
/// Most HID changes (keys, gamepad buttons and gamepad axes) a script may
/// make in one frame, before they are sent. Consecutive axis moves count once.
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
    /// Whether the person let this app session send keys and gamepad input.
    pub hid_permission: HidPermission,
}

/// Whether a scripted app may send keys and gamepad input to the computer.
/// Asked the first time it tries, once per app session.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HidPermission {
    /// The script hasn't tried yet.
    #[default]
    Unasked,
    /// The script tried; the app is asking the person.
    Asking,
    Allowed,
    Denied,
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
            hid_permission: HidPermission::Unasked,
        }
    }

    /// Queues a report for the computer, or fails once the per-call cap is
    /// reached. With `coalesce`, a gamepad report replaces a gamepad report
    /// queued just before it, since only the latest axis position matters.
    fn push_hid_report(&mut self, report: HidReport, coalesce: bool) -> Result<(), &'static str> {
        if coalesce
            && let (HidReport::Gamepad(_), Some(last @ HidReport::Gamepad(_))) =
                (report, self.hid_reports.last_mut())
        {
            *last = report;
            return Ok(());
        }
        if self.hid_reports.len() >= MAX_HID_REPORTS {
            return Err("too many HID changes (keys, gamepad) in one frame");
        }
        self.hid_reports.push(report);
        Ok(())
    }

    /// Applies `change` to the keys and gamepad the script holds, and queues
    /// the report it returns, if the person allowed HID input. The first
    /// change the computer would see starts asking them; until they allow
    /// it, changes are dropped. Returns whether the change was made.
    pub fn change_hid(
        &mut self,
        change: impl FnOnce(&mut HidState) -> Option<HidReport>,
        coalesce: bool,
    ) -> Result<bool, &'static str> {
        let mut next = self.hid;
        let Some(report) = change(&mut next) else {
            self.hid = next;
            return Ok(true);
        };
        match self.hid_permission {
            HidPermission::Allowed => {
                self.push_hid_report(report, coalesce)?;
                self.hid = next;
                Ok(true)
            }
            HidPermission::Unasked => {
                self.hid_permission = HidPermission::Asking;
                Ok(false)
            }
            HidPermission::Asking | HidPermission::Denied => Ok(false),
        }
    }

    /// Lets every key and gamepad control the script holds go.
    pub fn release_all_hid(&mut self) -> Result<(), &'static str> {
        let mut hid = self.hid;
        for report in hid.release_all() {
            self.push_hid_report(report, false)?;
        }
        self.hid = hid;
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

/// Settings a script declares about itself in its header comment, read
/// without running the script. Anything left out falls back to a default
/// chosen by [`ScriptApp`](crate::apps::ScriptApp).
///
/// The header is the run of line comments at the very top of the file; the
/// first line that isn't one ends it. Inside it, lines like
/// `-- title: My App` set a field. Keys are lowercase, the first line with a
/// key wins, and other comment lines (unknown keys, prose) are ignored.
#[derive(Debug, Default)]
pub struct ScriptMeta {
    /// `title:` the name in the menu.
    pub title: Option<String>,
    /// `author:`
    pub author: Option<String>,
    /// `desc:` a one-line description.
    pub desc: Option<String>,
    /// `color:` the menu card color, written `0x2a6f97` or `#2a6f97`.
    pub color: Option<u32>,
    /// `realtime: true` runs `update` every frame rather than only on input.
    pub realtime: bool,
    /// `icon:` a `VM1:` vitmap string, drawn on the menu card.
    pub icon: Option<PackedVitmap>,
}

impl ScriptMeta {
    /// Reads the header of `source`. A field with a bad value is left out,
    /// and the first such problem is returned alongside, as a message for
    /// the error view.
    pub fn parse(source: &str) -> (Self, Option<String>) {
        let mut meta = Self::default();
        let mut problem = None;
        let mut realtime_seen = false;
        for line in source.lines() {
            let Some(comment) = line.trim().strip_prefix(ActiveEngine::LINE_COMMENT) else {
                break;
            };
            let Some((key, value)) = comment.split_once(':') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            if value.is_empty() {
                continue;
            }
            let mut fail = |message: String| {
                problem.get_or_insert_with(|| format!("header: {key}: {message}"));
            };
            match key {
                "title" if meta.title.is_none() => meta.title = Some(value.into()),
                "author" if meta.author.is_none() => meta.author = Some(value.into()),
                "desc" if meta.desc.is_none() => meta.desc = Some(value.into()),
                "color" if meta.color.is_none() => match parse_color(value) {
                    Some(color) => meta.color = Some(color),
                    None => fail(format!("expected a color like 0x2a6f97, not \"{value}\"")),
                },
                "realtime" if !realtime_seen => {
                    realtime_seen = true;
                    match value {
                        "true" => meta.realtime = true,
                        "false" => {}
                        _ => fail(format!("expected true or false, not \"{value}\"")),
                    }
                }
                "icon" if meta.icon.is_none() => match PackedVitmap::decode(value) {
                    Ok(icon) => meta.icon = Some(icon),
                    Err(e) => fail(format!("not a valid vitmap: {e}")),
                },
                _ => {}
            }
        }
        (meta, problem)
    }
}

/// `0x2a6f97`, `#2a6f97` or `2a6f97`.
fn parse_color(value: &str) -> Option<u32> {
    let digits = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .or_else(|| value.strip_prefix('#'))
        .unwrap_or(value);
    if digits.is_empty() || digits.len() > 6 {
        return None;
    }
    u32::from_str_radix(digits, 16).ok()
}

/// A loaded script, ready to have its hooks called. Errors are human-readable
/// messages meant for the on-screen error view.
///
/// Every hook is optional in the script; calling a hook the script does not
/// define does nothing.
pub trait ScriptEngine: Sized {
    /// File extension of scripts for this engine, without the dot.
    const FILE_EXTENSION: &'static str;
    /// What starts a line comment, for reading the [`ScriptMeta`] header.
    const LINE_COMMENT: &'static str;

    /// Compiles `source` and runs its top level, which may make
    /// `math_input` boxes in `host`.
    fn load(source: &str, host: &mut ScriptHost) -> Result<Self, String>;
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

#[cfg(test)]
mod header_tests {
    use super::*;

    #[test]
    fn reads_every_field() {
        let source = format!(
            "-- title:    My App\n\
             -- author:   Someone\n\
             -- desc:     Does: things\n\
             -- color:    0x2a6f97\n\
             -- realtime: true\n\
             -- icon:     {}\n\
             function update() end\n",
            test_sources::VM1_SAMPLE
        );
        let (meta, problem) = ScriptMeta::parse(&source);
        assert_eq!(problem, None);
        assert_eq!(meta.title.as_deref(), Some("My App"));
        assert_eq!(meta.author.as_deref(), Some("Someone"));
        // Only the first colon separates the key.
        assert_eq!(meta.desc.as_deref(), Some("Does: things"));
        assert_eq!(meta.color, Some(0x2a6f97));
        assert!(meta.realtime);
        let icon = meta.icon.expect("icon");
        assert_eq!(icon.actions()[0].name(), "Idle");
    }

    #[test]
    fn missing_header_gives_defaults() {
        let (meta, problem) = ScriptMeta::parse("function update() end");
        assert_eq!(problem, None);
        assert!(meta.title.is_none() && meta.color.is_none() && meta.icon.is_none());
        assert!(!meta.realtime);
        assert_eq!(ScriptMeta::parse("").1, None);
    }

    #[test]
    fn header_ends_at_the_first_non_comment_line() {
        let (meta, _) = ScriptMeta::parse("-- title: Top\n\n-- color: 0x123456\n");
        assert_eq!(meta.title.as_deref(), Some("Top"));
        assert_eq!(meta.color, None);

        let (meta, _) = ScriptMeta::parse("x = 1\n-- title: Late\n");
        assert_eq!(meta.title, None);
    }

    #[test]
    fn first_key_wins_and_other_lines_are_ignored() {
        let source = "-- My App: a description in prose\n\
                      --\n\
                      -- Title: not a field (keys are lowercase)\n\
                      -- note: unknown keys are ignored\n\
                      -- title:\n\
                      -- title: First\n\
                      -- title: Second\n\
                      -- realtime: false\n\
                      -- realtime: true\n";
        let (meta, problem) = ScriptMeta::parse(source);
        assert_eq!(problem, None);
        // An empty value doesn't count as the first.
        assert_eq!(meta.title.as_deref(), Some("First"));
        assert!(!meta.realtime);
    }

    #[test]
    fn colors_take_several_spellings() {
        for value in ["0x2a6f97", "0X2A6F97", "#2a6f97", "2a6f97"] {
            let (meta, problem) = ScriptMeta::parse(&format!("-- color: {value}"));
            assert_eq!(problem, None, "{value}");
            assert_eq!(meta.color, Some(0x2a6f97), "{value}");
        }
    }

    #[test]
    fn bad_values_are_left_out_and_reported() {
        let bad = |source: &str| {
            let (meta, problem) = ScriptMeta::parse(source);
            (meta, problem.expect("a problem"))
        };

        let (meta, problem) = bad("-- color: blue");
        assert!(meta.color.is_none());
        assert!(problem.starts_with("header: color:"), "{problem}");

        for color in ["#1234567", "0x"] {
            assert!(ScriptMeta::parse(&format!("-- color: {color}")).1.is_some());
        }

        let (meta, problem) = bad("-- realtime: yes");
        assert!(!meta.realtime);
        assert!(problem.starts_with("header: realtime:"), "{problem}");

        let (meta, problem) = bad("-- icon: VM1:AAAA!");
        assert!(meta.icon.is_none());
        assert!(problem.contains("not a valid vitmap"), "{problem}");

        // The other fields are still read, and the first problem is kept.
        let (meta, problem) = bad("-- color: red\n-- title: Still\n-- realtime: maybe");
        assert_eq!(meta.title.as_deref(), Some("Still"));
        assert!(problem.starts_with("header: color:"), "{problem}");
    }
}
