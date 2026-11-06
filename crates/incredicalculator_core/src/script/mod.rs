//! Scripted apps: apps whose behavior comes from a script instead of
//! compiled-in Rust.
//!
//! The script engine is chosen at compile time with a Cargo feature
//! (`script-rhai` for now). Every engine implements [`ScriptEngine`], and
//! [`ScriptApp`](crate::apps::ScriptApp) wraps whichever one is active as an
//! [`IcApp`](crate::app::IcApp), so the shell never knows which language a
//! scripted app is written in.
//!
//! Engines never touch the platform. A script's drawing calls are recorded
//! into [`ScriptHost::commands`], which the app replays on the platform after
//! the script returns. That keeps engine bindings free of platform lifetimes
//! and makes a per-frame cap on drawing work easy to enforce.

use alloc::string::String;
use alloc::vec::Vec;
use glam::{IVec2, Vec2};
use rgb::RGB8;

use crate::input::IcKey;
use crate::text::HAlign;

#[cfg(feature = "script-rhai")]
mod rhai_engine;

/// The engine selected by Cargo features.
#[cfg(feature = "script-rhai")]
pub type ActiveEngine = rhai_engine::RhaiEngine;

/// Hard-coded example script, shown in the apps menu until scripts can be
/// loaded from storage.
#[cfg(feature = "script-rhai")]
pub const EXAMPLE_SCRIPT: (&str, &str) = ("bouncer", include_str!("../../scripts/bouncer.rhai"));

/// Most drawing commands one script call may record. Scripts that draw more
/// stop with an error rather than growing the heap without bound.
pub const MAX_DRAW_COMMANDS: usize = 512;
/// Most log lines kept from one script call.
pub const MAX_LOG_LINES: usize = 8;

/// One drawing call made by a script, replayed on the platform later.
#[derive(Clone, Debug)]
pub enum DrawCmd {
    Clear(RGB8),
    Line {
        start: Vec2,
        end: Vec2,
        color: RGB8,
        width: u32,
    },
    Rect {
        start: IVec2,
        end: IVec2,
        color: RGB8,
        /// Outline width; 0 means filled.
        outline: u32,
        radius: u32,
    },
    Triangle {
        points: [IVec2; 3],
        color: RGB8,
    },
    Polygon {
        points: Vec<Vec2>,
        color: RGB8,
    },
    Text {
        text: String,
        pos: Vec2,
        size: f32,
        color: RGB8,
        align: HAlign,
    },
}

/// Everything a script can see of, or do to, the outside world during one
/// call. The app fills in the inputs before each call and drains the outputs
/// afterwards.
pub struct ScriptHost {
    pub millis: u64,
    pub keys_down: [bool; IcKey::COUNT],
    pub commands: Vec<DrawCmd>,
    pub logs: Vec<String>,
}

impl ScriptHost {
    pub const fn new() -> Self {
        Self {
            millis: 0,
            keys_down: [false; IcKey::COUNT],
            commands: Vec::new(),
            logs: Vec::new(),
        }
    }

    /// Records a drawing command, or fails once the per-call cap is reached.
    pub fn push(&mut self, command: DrawCmd) -> Result<(), &'static str> {
        if self.commands.len() >= MAX_DRAW_COMMANDS {
            return Err("too many drawing calls in one frame");
        }
        self.commands.push(command);
        Ok(())
    }

    pub fn log(&mut self, line: &str) {
        if self.logs.len() < MAX_LOG_LINES {
            self.logs.push(String::from(line));
        }
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

    /// Compiles `source` and runs its top level, which may declare metadata.
    fn load(source: &str) -> Result<(Self, ScriptMeta), String>;
    /// Runs the script's `init` hook. Called once, the first time the app is
    /// opened.
    fn init(&mut self, host: &mut ScriptHost) -> Result<(), String>;
    /// Runs the script's `update` hook. Drawing calls are only honored here.
    fn update(&mut self, host: &mut ScriptHost) -> Result<(), String>;
    /// Runs the script's `on_key` hook with the key's script name (see
    /// [`key_name`]).
    fn on_key(&mut self, key: IcKey, host: &mut ScriptHost) -> Result<(), String>;
}

/// The name scripts use for a key: "0"-"9", "f1"-"f6", "shift" or "super".
pub fn key_name(key: IcKey) -> &'static str {
    match key {
        IcKey::Num0 => "0",
        IcKey::Num1 => "1",
        IcKey::Num2 => "2",
        IcKey::Num3 => "3",
        IcKey::Num4 => "4",
        IcKey::Num5 => "5",
        IcKey::Num6 => "6",
        IcKey::Num7 => "7",
        IcKey::Num8 => "8",
        IcKey::Num9 => "9",
        IcKey::Func1 => "f1",
        IcKey::Func2 => "f2",
        IcKey::Func3 => "f3",
        IcKey::Func4 => "f4",
        IcKey::Func5 => "f5",
        IcKey::Func6 => "f6",
        IcKey::Shift => "shift",
        IcKey::Super => "super",
        IcKey::_Max => "",
    }
}

/// The key a script refers to by `name`, as returned by [`key_name`].
pub fn key_from_name(name: &str) -> Option<IcKey> {
    use num_traits::FromPrimitive;
    (0..IcKey::COUNT)
        .filter_map(IcKey::from_usize)
        .find(|&key| key_name(key) == name)
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
