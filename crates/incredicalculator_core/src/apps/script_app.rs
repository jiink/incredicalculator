use alloc::borrow::Cow;
use alloc::string::String;

use glam::Vec2;
use rgb::RGB8;

use crate::app::{IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::graphics::{self, Vitmap};
use crate::input::IcKey;
use crate::platform::{CANVAS_WIDTH, IcPlatform, rgb8_hex};
use crate::script::{ActiveEngine, DrawCmd, ScriptEngine, ScriptHost, ScriptMeta};
use crate::text::{TextStyle, draw_text, wrap_text};

/// Card colors for scripts that don't pick their own, chosen by name hash.
const DEFAULT_COLORS: [u32; 6] = [0x2A6F97, 0x6A4C93, 0x1B998B, 0xC05746, 0x8D6A9F, 0x3D5A80];

const ERROR_TEXT_SIZE: f32 = 5.0;
const ERROR_MARGIN: f32 = 8.0;

/// An app whose behavior comes from a script, run by the engine chosen at
/// compile time.
pub struct ScriptApp {
    name: String,
    color: RGB8,
    realtime: bool,
    source: Cow<'static, str>,
    /// Only present while the app is open, so a closed script holds no
    /// engine memory. Its state starts over each time it is opened.
    engine: Option<ActiveEngine>,
    host: ScriptHost,
    /// Set when loading or a hook fails; the script is not run again until
    /// the app is reopened.
    error: Option<String>,
}

impl ScriptApp {
    /// `file_name` (like `bouncer.lua`), without its extension, is the app
    /// name unless the script declares one.
    pub fn new(file_name: &str, source: impl Into<Cow<'static, str>>) -> Self {
        let file_stem = file_name
            .strip_suffix(crate::platform::SCRIPT_EXTENSION)
            .and_then(|stem| stem.strip_suffix('.'))
            .unwrap_or(file_name);
        let mut app = Self {
            name: String::from(file_stem),
            color: default_color(file_stem),
            realtime: false,
            source: source.into(),
            engine: None,
            host: ScriptHost::new(),
            error: None,
        };
        // Load once for the menu's name and color, then free the engine.
        app.load();
        app.engine = None;
        app
    }

    fn load(&mut self) {
        self.engine = None;
        self.error = None;
        match ActiveEngine::load(&self.source) {
            Ok((engine, meta)) => {
                self.apply_meta(meta);
                self.engine = Some(engine);
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn apply_meta(&mut self, meta: ScriptMeta) {
        if let Some(name) = meta.name {
            self.name = name;
        }
        if let Some(color) = meta.color {
            self.color = rgb8_hex(color);
        }
        self.realtime = meta.realtime;
    }

    /// Runs one hook, recording any failure and dropping the engine so the
    /// script stops running.
    fn run(&mut self, hook: impl FnOnce(&mut ActiveEngine, &mut ScriptHost) -> Result<(), String>) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        if let Err(error) = hook(engine, &mut self.host) {
            self.error = Some(error);
            self.engine = None;
        }
    }

    fn flush_logs(&mut self, platform: &mut dyn IcPlatform) {
        for line in self.host.logs.drain(..) {
            platform.log(format_args!("[{}] {}", self.name, line));
        }
    }

    fn draw_error(&self, platform: &mut dyn IcPlatform, error: &str) {
        platform.clear(rgb8_hex(0x3A0D0D));
        draw_text(
            platform,
            "Script error",
            Vec2::new(ERROR_MARGIN, 24.0),
            TextStyle::new(8.0, rgb8_hex(0xFF8080)).thickness(3.0),
        );
        let wrapped = wrap_text(
            error,
            CANVAS_WIDTH as f32 - 2.0 * ERROR_MARGIN,
            ERROR_TEXT_SIZE,
        );
        draw_text(
            platform,
            &wrapped,
            Vec2::new(ERROR_MARGIN, 50.0),
            TextStyle::new(ERROR_TEXT_SIZE, rgb8_hex(0xFFFFFF)),
        );
        draw_text(
            platform,
            "Reopen the app to retry.",
            Vec2::new(ERROR_MARGIN, 225.0),
            TextStyle::new(ERROR_TEXT_SIZE, rgb8_hex(0xC0A0A0)),
        );
    }
}

impl IcApp for ScriptApp {
    fn name(&self) -> &str {
        &self.name
    }

    fn icon(&self) -> &'static Vitmap {
        &graphics::CODE_ICON
    }

    fn color(&self) -> RGB8 {
        self.color
    }

    fn requires_realtime_updates(&self) -> bool {
        self.realtime && self.error.is_none()
    }

    fn on_enter(&mut self) {
        self.load();
        self.run(|engine, host| engine.init(host));
        // Drawing is only honored during `update`.
        self.host.commands.clear();
    }

    fn on_exit(&mut self) {
        self.engine = None;
        self.host = ScriptHost::new();
    }

    fn on_key(&mut self, key: IcKey, _ctx: &InputContext) {
        self.run(|engine, host| engine.on_key(key, host));
        self.host.commands.clear();
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        self.host.millis = platform.millis();
        for (down, state) in self.host.keys_down.iter_mut().zip(ctx.key_states) {
            *down = state.is_down;
        }
        self.host.commands.clear();
        self.run(|engine, host| engine.update(host));
        self.flush_logs(platform);

        if let Some(error) = &self.error {
            self.draw_error(platform, error);
            return;
        }
        for command in self.host.commands.drain(..) {
            replay(platform, command);
        }
    }
}

fn replay(platform: &mut dyn IcPlatform, command: DrawCmd) {
    match command {
        DrawCmd::Clear(color) => platform.clear(color),
        DrawCmd::Line {
            start,
            end,
            color,
            width,
        } => platform.draw_line(start, end, color, width),
        DrawCmd::Rect {
            start,
            end,
            color,
            outline,
            radius,
        } => {
            let fill = (outline == 0).then_some(color);
            if radius > 0 {
                platform.draw_rectangle_rounded(start, end, color, outline, fill, radius);
            } else {
                platform.draw_rectangle(start, end, color, outline, fill);
            }
        }
        DrawCmd::Triangle { points, color } => {
            platform.draw_triangle(points[0], points[1], points[2], color, 0, Some(color))
        }
        DrawCmd::Polygon { points, color } => platform.draw_polygon(&points, color),
        DrawCmd::Text {
            text,
            pos,
            size,
            color,
            align,
        } => draw_text(
            platform,
            &text,
            pos,
            TextStyle::new(size, color).align(align),
        ),
    }
}

fn default_color(name: &str) -> RGB8 {
    // FNV-1a, so a script keeps its color across boots.
    let hash = name.bytes().fold(0x811C_9DC5u32, |h, b| {
        (h ^ b as u32).wrapping_mul(0x0100_0193)
    });
    rgb8_hex(DEFAULT_COLORS[hash as usize % DEFAULT_COLORS.len()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::script::test_sources;
    use crate::text::text_width;

    #[test]
    fn wrap_keeps_lines_within_width() {
        let text = "Function not found: update_everything_please (line 12, position 3)";
        let wrapped = wrap_text(text, 120.0, ERROR_TEXT_SIZE);
        let style = TextStyle::new(ERROR_TEXT_SIZE, RGB8::default());
        assert!(wrapped.lines().count() > 1);
        for line in wrapped.lines() {
            assert!(text_width(line, style.scale, style.font) <= 120.0, "{line}");
        }
    }

    #[test]
    fn default_color_is_stable() {
        assert_eq!(default_color("bouncer"), default_color("bouncer"));
    }

    #[test]
    fn example_script_loads_with_metadata() {
        let (name, source) = crate::script::EXAMPLE_SCRIPT;
        let app = ScriptApp::new(name, source);
        assert!(app.error.is_none(), "{:?}", app.error);
        assert_eq!(app.name(), "Bouncer");
        assert_eq!(ScriptApp::new("my-app.lua", "").name(), "my-app");
        assert!(app.requires_realtime_updates());
    }

    #[test]
    fn example_script_draws_and_handles_keys() {
        let (name, source) = crate::script::EXAMPLE_SCRIPT;
        let (mut engine, _) = ActiveEngine::load(source).unwrap();
        let mut host = ScriptHost::new();
        engine.init(&mut host).unwrap();
        for key in [IcKey::Num5, IcKey::Num8, IcKey::Func2] {
            engine.on_key(key, &mut host).unwrap();
        }
        for frame in 0..60 {
            host.millis = frame * 16;
            host.commands.clear();
            engine.update(&mut host).unwrap();
            assert!(!host.commands.is_empty(), "{name} drew nothing");
            assert!(matches!(host.commands[0], DrawCmd::Clear(_)));
        }
    }

    #[test]
    fn engine_only_lives_while_open() {
        let (name, source) = crate::script::EXAMPLE_SCRIPT;
        let mut app = ScriptApp::new(name, source);
        assert!(app.engine.is_none());
        app.on_enter();
        assert!(app.engine.is_some());
        app.on_exit();
        assert!(app.engine.is_none());
    }

    #[test]
    fn errors_are_reported_not_panicked() {
        let mut app = ScriptApp::new("broken", test_sources::RUNTIME_ERROR);
        assert!(app.error.is_none());
        app.on_enter();
        app.run(|engine, host| engine.update(host));
        assert!(app.error.as_deref().unwrap().contains("undefined_fn"));
        assert!(app.engine.is_none());
        // Reopening retries from a clean slate.
        app.on_enter();
        assert!(app.error.is_none());

        let app = ScriptApp::new("syntax", test_sources::SYNTAX_ERROR);
        assert!(app.error.is_some());
        assert_eq!(app.name(), "syntax");
    }

    #[test]
    fn endless_loops_are_stopped() {
        let (mut engine, _) = ActiveEngine::load(test_sources::ENDLESS_LOOP).unwrap();
        let error = engine.update(&mut ScriptHost::new()).unwrap_err();
        assert!(!error.is_empty());
    }

    #[test]
    fn drawing_is_capped() {
        let (mut engine, _) = ActiveEngine::load(test_sources::DRAW_FLOOD).unwrap();
        let mut host = ScriptHost::new();
        let error = engine.update(&mut host).unwrap_err();
        assert!(error.contains("too many drawing calls"), "{error}");
        assert_eq!(host.commands.len(), crate::script::MAX_DRAW_COMMANDS);
    }
}
