use alloc::borrow::Cow;
use alloc::string::String;

use glam::Vec2;
use rgb::RGB8;

use crate::app::{AppIcon, IcApp, InputContext};
use crate::audio_engine::AudioEngine;
use crate::graphics::vm1::PackedVitmap;
use crate::input::IcKey;
use crate::platform::{CANVAS_HEIGHT, CANVAS_WIDTH, IcPlatform, rgb8_hex};
use crate::script::{ActiveEngine, HidPermission, ScriptEngine, ScriptHost, ScriptMeta};
use crate::text::{HAlign, TextStyle, draw_text, wrap_text};

/// Card colors for scripts that don't pick their own, chosen by name hash.
const DEFAULT_COLORS: [u32; 6] = [0x2A6F97, 0x6A4C93, 0x1B998B, 0xC05746, 0x8D6A9F, 0x3D5A80];

const ERROR_TEXT_SIZE: f32 = 5.0;
const ERROR_MARGIN: f32 = 8.0;

/// The HID permission prompt's box, centered on the screen.
const PROMPT_WIDTH: f32 = 280.0;
const PROMPT_HEIGHT: f32 = 150.0;
const PROMPT_PADDING: f32 = 12.0;
const PROMPT_TEXT_SIZE: f32 = 5.0;

/// An app whose behavior comes from a script, run by the engine chosen at
/// compile time.
pub struct ScriptApp {
    name: String,
    color: RGB8,
    realtime: bool,
    /// The header's icon, or `None` to show the name's initials.
    icon: Option<PackedVitmap>,
    /// A bad header field, shown instead of running the script.
    header_error: Option<String>,
    source: Cow<'static, str>,
    /// Only present while the app is open, so a closed script holds no
    /// engine memory. Its state starts over each time it is opened.
    engine: Option<ActiveEngine>,
    host: ScriptHost,
    /// Set when loading or a hook fails; the script is not run again until
    /// the app is reopened.
    error: Option<String>,
    /// Keys whose press the script was told about, so it only hears about
    /// releases of those.
    pressed: [bool; IcKey::COUNT],
}

impl ScriptApp {
    /// `file_name` (like `bouncer.lua`), without its extension, is the app
    /// name unless the script's header gives a title. The script itself
    /// doesn't run until the app is opened.
    pub fn new(file_name: &str, source: impl Into<Cow<'static, str>>) -> Self {
        let file_stem = file_name
            .strip_suffix(crate::platform::SCRIPT_EXTENSION)
            .and_then(|stem| stem.strip_suffix('.'))
            .unwrap_or(file_name);
        let source = source.into();
        let (meta, header_error) = ScriptMeta::parse(&source);
        Self {
            name: meta.title.unwrap_or_else(|| String::from(file_stem)),
            color: meta
                .color
                .map_or_else(|| default_color(file_stem), rgb8_hex),
            realtime: meta.realtime,
            icon: meta.icon,
            header_error,
            source,
            engine: None,
            host: ScriptHost::new(),
            error: None,
            pressed: [false; IcKey::COUNT],
        }
    }

    /// Replaces the host with a fresh one. Keys the script held on the
    /// computer are let up: the reports still to send, and the releases,
    /// carry over to the new host for `send_hid`.
    fn reset_host(&mut self) {
        let old = core::mem::take(&mut self.host);
        let mut outgoing = old.hid_reports;
        let mut hid = old.hid;
        outgoing.extend(hid.release_all());
        self.host.hid_reports = outgoing;
        self.host.hid_connected = old.hid_connected;
        self.pressed = [false; IcKey::COUNT];
    }

    /// Starts the script from scratch, with a fresh host.
    fn load(&mut self) {
        self.engine = None;
        self.error = None;
        self.reset_host();
        if let Some(error) = self.header_error.clone() {
            self.fail(error);
            return;
        }
        match ActiveEngine::load(&self.source, &mut self.host) {
            Ok(engine) => self.engine = Some(engine),
            Err(error) => self.fail(error),
        }
    }

    /// Records why the script stopped, and logs it with the script's own
    /// log lines so it shows up wherever those do, not only on the screen.
    /// It is kept even when the script used up its log lines.
    fn fail(&mut self, error: String) {
        self.host.logs.push(alloc::format!("error: {error}"));
        self.error = Some(error);
    }

    /// Runs one hook, recording any failure and dropping the engine so the
    /// script stops running.
    fn run(&mut self, hook: impl FnOnce(&mut ActiveEngine, &mut ScriptHost) -> Result<(), String>) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        if let Err(error) = hook(engine, &mut self.host) {
            self.fail(error);
            self.engine = None;
            // A stopped script can't let its keys up itself.
            let released: alloc::vec::Vec<_> = self.host.hid.release_all().collect();
            self.host.hid_reports.extend(released);
        }
    }

    fn read_keys(&mut self, ctx: &InputContext) {
        for (down, state) in self.host.keys_down.iter_mut().zip(ctx.key_states) {
            *down = state.is_down;
        }
    }

    fn flush_logs(&mut self, platform: &mut dyn IcPlatform) {
        for line in self.host.logs.drain(..) {
            platform.log(format_args!("[{}] {}", self.name, line));
        }
    }

    fn draw_error(&self, platform: &mut dyn IcPlatform, error: &str) {
        // Covers whatever the script drew before it failed.
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
        let hint = if platform.usb_drive_connected() {
            "Fix it on the INCREDICALC drive; saving reloads it."
        } else {
            "Reopen the app to retry."
        };
        draw_text(
            platform,
            hint,
            Vec2::new(ERROR_MARGIN, 225.0),
            TextStyle::new(ERROR_TEXT_SIZE, rgb8_hex(0xC0A0A0)),
        );
    }

    /// Asks whether the script may send keys and gamepad input, over
    /// whatever it drew.
    fn draw_hid_prompt(&self, platform: &mut dyn IcPlatform) {
        let left = (CANVAS_WIDTH as f32 - PROMPT_WIDTH) / 2.0;
        let top = (CANVAS_HEIGHT as f32 - PROMPT_HEIGHT) / 2.0;
        platform.draw_rectangle_rounded(
            glam::IVec2::new(left as i32, top as i32),
            glam::IVec2::new((left + PROMPT_WIDTH) as i32, (top + PROMPT_HEIGHT) as i32),
            rgb8_hex(0xFFB040),
            3,
            Some(rgb8_hex(0x1E1A12)),
            8,
        );
        let center = CANVAS_WIDTH as f32 / 2.0;
        draw_text(
            platform,
            "Allow USB input?",
            Vec2::new(center, top + 26.0),
            TextStyle::new(7.0, rgb8_hex(0xFFD080))
                .thickness(3.0)
                .align(HAlign::Center),
        );
        let message = alloc::format!(
            "\"{}\" wants to press keys or gamepad buttons on the connected computer.",
            self.name
        );
        let wrapped = wrap_text(
            &message,
            PROMPT_WIDTH - 2.0 * PROMPT_PADDING,
            PROMPT_TEXT_SIZE,
        );
        draw_text(
            platform,
            &wrapped,
            Vec2::new(center, top + 54.0),
            TextStyle::new(PROMPT_TEXT_SIZE, rgb8_hex(0xFFFFFF)).align(HAlign::Center),
        );
        let bottom = top + PROMPT_HEIGHT - PROMPT_PADDING;
        let quarter = PROMPT_WIDTH / 4.0;
        draw_text(
            platform,
            "Enter: Allow",
            Vec2::new(left + quarter, bottom),
            TextStyle::new(PROMPT_TEXT_SIZE, rgb8_hex(0x80FF90))
                .thickness(2.5)
                .align(HAlign::Center),
        );
        draw_text(
            platform,
            "Backspace: Deny",
            Vec2::new(left + 3.0 * quarter, bottom),
            TextStyle::new(PROMPT_TEXT_SIZE, rgb8_hex(0xFF9080))
                .thickness(2.5)
                .align(HAlign::Center),
        );
    }
}

impl IcApp for ScriptApp {
    fn name(&self) -> &str {
        &self.name
    }

    fn icon(&self) -> AppIcon<'_> {
        match &self.icon {
            Some(icon) => AppIcon::Packed(icon),
            None => AppIcon::Initials(&self.name),
        }
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
    }

    fn on_exit(&mut self) {
        self.engine = None;
        self.reset_host();
    }

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        if self.host.hid_permission == HidPermission::Asking {
            // The prompt takes every key until it is answered; the script
            // doesn't hear these presses, nor their releases.
            let answer = match key {
                IcKey::Equals => HidPermission::Allowed,
                IcKey::Backspace => HidPermission::Denied,
                _ => return,
            };
            self.host.hid_permission = answer;
            let verdict = if answer == HidPermission::Allowed {
                "allowed"
            } else {
                "denied"
            };
            self.host
                .log(&alloc::format!("USB keyboard/gamepad input {verdict}"));
            return;
        }
        self.pressed[key as usize] = true;
        self.read_keys(ctx);
        self.run(|engine, host| engine.on_key(key, host));
    }

    fn on_key_up(&mut self, key: IcKey, ctx: &InputContext) {
        if !core::mem::take(&mut self.pressed[key as usize]) {
            return;
        }
        self.read_keys(ctx);
        self.run(|engine, host| engine.on_key_up(key, host));
    }

    fn send_hid(&mut self, platform: &mut dyn IcPlatform) {
        for report in self.host.hid_reports.drain(..) {
            platform.send_hid_report(report);
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        _audio: &mut AudioEngine,
    ) {
        self.host.millis = platform.millis();
        self.host.hid_connected = platform.hid_connected();
        self.read_keys(ctx);
        self.run(|engine, host| engine.update(host, platform));
        self.flush_logs(platform);

        if let Some(error) = &self.error {
            self.draw_error(platform, error);
        }
        if self.host.hid_permission == HidPermission::Asking {
            self.draw_hid_prompt(platform);
        }
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
    use crate::fake_platform::FakePlatform;
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
        let mut host = ScriptHost::new();
        let mut engine = ActiveEngine::load(source, &mut host).unwrap();
        engine.init(&mut host).unwrap();
        for key in [IcKey::Num5, IcKey::Num8, IcKey::Divide] {
            engine.on_key(key, &mut host).unwrap();
        }
        for frame in 0..60 {
            host.millis = frame * 16;
            engine
                .update(&mut host, &mut FakePlatform::default())
                .unwrap_or_else(|e| panic!("{name}: {e}"));
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
        let mut platform = FakePlatform::default();
        app.run(|engine, host| engine.update(host, &mut platform));
        assert!(app.error.as_deref().unwrap().contains("undefined_fn"));
        assert!(app.engine.is_none());
        // Reopening retries from a clean slate.
        app.on_enter();
        assert!(app.error.is_none());

        // The script only runs, and fails, once it is opened.
        let mut app = ScriptApp::new("syntax", test_sources::SYNTAX_ERROR);
        assert!(app.error.is_none());
        assert_eq!(app.name(), "syntax");
        app.on_enter();
        assert!(app.error.is_some());
    }

    #[test]
    fn header_sets_menu_name_color_and_icon() {
        let source = alloc::format!(
            "-- title: Shapes\n-- color: #123456\n-- icon: {}\nfunction update() end",
            test_sources::VM1_SAMPLE
        );
        let app = ScriptApp::new("shapes.lua", source);
        assert_eq!(app.name(), "Shapes");
        assert_eq!(app.color(), rgb8_hex(0x123456));
        assert!(!app.requires_realtime_updates());
        let AppIcon::Packed(icon) = app.icon() else {
            panic!("expected the header's icon");
        };
        assert_eq!(icon.shapes(0, 0).unwrap().count(), 2);
        // Drawing the icon as the menu does doesn't need the script to run.
        app.icon()
            .draw(&mut FakePlatform::default(), Vec2::new(50.0, 32.0), 3.0);
        assert!(app.engine.is_none());
    }

    #[test]
    fn scripts_without_an_icon_show_their_initials() {
        let app = ScriptApp::new("plain.lua", "function update() end");
        let AppIcon::Initials(name) = app.icon() else {
            panic!("expected initials");
        };
        assert_eq!(name, "plain");
        assert_eq!(app.color(), default_color("plain"));
        app.icon()
            .draw(&mut FakePlatform::default(), Vec2::new(50.0, 32.0), 3.0);

        let app = ScriptApp::new("x.lua", "-- title: Ohm's Law");
        assert!(matches!(app.icon(), AppIcon::Initials("Ohm's Law")));
    }

    #[test]
    fn the_menu_does_not_run_scripts() {
        // A top level that would fail still shows its header in the menu.
        let app = ScriptApp::new("x.lua", "-- title: Broken\nerror('boom')");
        assert_eq!(app.name(), "Broken");
        assert!(app.error.is_none());
    }

    #[test]
    fn bad_header_shows_defaults_then_an_error_when_opened() {
        let source = "-- title: Bad Icon\n-- icon: VM1:AAAA!\nfunction update() end";
        let mut app = ScriptApp::new("bad.lua", source);
        assert_eq!(app.name(), "Bad Icon");
        assert!(matches!(app.icon(), AppIcon::Initials("Bad Icon")));
        assert!(app.error.is_none());

        app.on_enter();
        assert!(app.engine.is_none());
        let error = app.error.as_deref().expect("an error once opened");
        assert!(error.starts_with("header: icon:"), "{error}");
        assert!(!app.requires_realtime_updates());
    }

    #[test]
    fn endless_loops_are_stopped() {
        let mut host = ScriptHost::new();
        let mut engine = ActiveEngine::load(test_sources::ENDLESS_LOOP, &mut host).unwrap();
        let error = engine
            .update(&mut host, &mut FakePlatform::default())
            .unwrap_err();
        assert!(!error.is_empty());
    }
}
