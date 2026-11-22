use crate::app::InputContext;
use crate::app::{AppIcon, IcApp};
use crate::apps::AspectRatioCalculator;
use crate::apps::Bytebeat;
use crate::apps::ChordPad;
use crate::apps::Info;
use crate::apps::ScaleKeys;
use crate::apps::{Calculator, EngineMode};
use crate::apps::RangeMapperCalculator;
use crate::apps::Grapher;
use crate::audio_engine::AudioEngine;
use crate::graphics;
use crate::graphics::draw_vitmap_aff;
use crate::graphics::{Vitmap, draw_vitmap};
use crate::input::IcKey;
use crate::input::{KeyMode, KeyState, ModifierKeys};
use crate::platform::rgb8_hex;
use crate::platform::{CANVAS_HEIGHT, CANVAS_WIDTH, IcPlatform};
use crate::settings::SettingsMenu;
use crate::text::*;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use glam::Affine2;
use glam::{IVec2, Vec2};
use num_traits::FromPrimitive;
use num_traits::clamp;
use rgb::Rgb;
use rgb::*;

const APPS_PER_PAGE: usize = 6;
const KEY_HINT_FADE_DELAY_MS: u64 = 2000;
const KEY_HINT_FADE_IN_MS: u64 = 700;

#[derive(Clone, Copy)]
enum SelectorAction {
    Launch(usize),
    PreviousPage,
    NextPage,
    OpenSettings,
    None,
}

#[derive(Clone, Copy)]
struct SelectorCard<'a> {
    name: &'a str,
    button: IcKey,
    icon: AppIcon<'a>,
    color: RGB8,
    action: SelectorAction,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(usize)]
pub enum Adjustable {
    Brightness,
    Volume,
}

/// Identifies a loaded scripted app, to tell whether its script has changed.
#[cfg(feature = "scripting")]
#[derive(PartialEq, Eq)]
struct ScriptKey {
    file_name: alloc::string::String,
    source_hash: u64,
}

pub struct IcShell {
    /// Compiled apps first, then scripted apps (one per entry in
    /// `script_keys`, in the same order).
    apps: Vec<Box<dyn IcApp>>,
    /// Platform's `scripts_version` when the scripted apps were last loaded.
    #[cfg(feature = "scripting")]
    scripts_version: Option<u32>,
    #[cfg(feature = "scripting")]
    script_keys: Vec<ScriptKey>,
    active_app_idx: Option<usize>,
    last_active_app_idx: Option<usize>,
    selector_page: usize,
    selector_hints_started_at: Option<u64>,
    /// The selector's key hints were still fading in on the last update, so
    /// the platform should keep redrawing until they're fully shown.
    selector_hints_fading: bool,
    selector_pressed: Option<IcKey>,
    settings_open: bool,
    settings_menu: SettingsMenu,
    settings_loaded: bool,
    key_states: [KeyState; IcKey::COUNT],
    show_startup_help: bool,
    modifiers: ModifierKeys,
    shift_was_engaged: bool,
    adjusting_something: Option<Adjustable>,
    /// Keys whose press the shell kept. Apps see them as up until released,
    /// so apps that poll held keys don't act on them too.
    held_by_shell: [bool; IcKey::COUNT],
    audio: AudioEngine,
}

impl IcShell {
    pub fn new() -> Self {
        // Scripted apps are added by `sync_scripts` on the first update.
        let apps: Vec<Box<dyn IcApp>> = vec![
            Box::new(Calculator::new(EngineMode::Programmer)),
            Box::new(Calculator::new(EngineMode::Scientific)),
            Box::new(AspectRatioCalculator::new()),
            Box::new(RangeMapperCalculator::new()),
            Box::new(Grapher::new()),
            Box::new(ScaleKeys::new()),
            Box::new(ChordPad::new()),
            Box::new(Bytebeat::new()),
            Box::new(Info::new()),
        ];
        Self {
            apps,
            #[cfg(feature = "scripting")]
            scripts_version: None,
            #[cfg(feature = "scripting")]
            script_keys: Vec::new(),
            active_app_idx: Some(0),
            last_active_app_idx: None,
            selector_page: 0,
            selector_hints_started_at: None,
            selector_hints_fading: false,
            selector_pressed: None,
            settings_open: false,
            settings_menu: SettingsMenu::new(),
            settings_loaded: false,
            key_states: [KeyState::default(); IcKey::COUNT],
            show_startup_help: true,
            modifiers: ModifierKeys::default(),
            shift_was_engaged: false,
            adjusting_something: None,
            held_by_shell: [false; IcKey::COUNT],
            audio: AudioEngine::new(),
        }
    }

    /// Reloads the scripted apps when the platform's installed scripts have
    /// changed. Unchanged scripts keep their running app. An open app whose
    /// script was edited restarts with the new code; one whose script was
    /// deleted closes back to the menu.
    #[cfg(feature = "scripting")]
    fn sync_scripts(&mut self, platform: &mut dyn IcPlatform) {
        use crate::apps::ScriptApp;
        use alloc::string::String;

        let version = platform.scripts_version();
        if self.scripts_version == Some(version) {
            return;
        }
        self.scripts_version = Some(version);

        let files: Vec<(String, String)> = match platform.list_scripts() {
            None => {
                crate::script::EXAMPLE_SCRIPTS
                    .iter()
                    .map(|&(file_name, source)| (file_name.into(), source.into()))
                    .collect()
            }
            Some(mut names) => {
                names.sort();
                names
                    .into_iter()
                    .filter_map(|name| platform.read_script(&name).map(|source| (name, source)))
                    .collect()
            }
        };

        let compiled = self.apps.len() - self.script_keys.len();
        let old_keys = core::mem::take(&mut self.script_keys);
        let mut old_apps: Vec<Option<Box<dyn IcApp>>> = self
            .apps
            .split_off(compiled)
            .into_iter()
            .map(Some)
            .collect();
        // For each old script: its new index, and whether its code changed.
        let mut moved: Vec<Option<(usize, bool)>> = vec![None; old_keys.len()];
        for (file_name, source) in files {
            let key = ScriptKey {
                source_hash: fnv1a(source.as_bytes()),
                file_name,
            };
            let old = old_keys.iter().position(|k| k.file_name == key.file_name);
            let reused = old
                .filter(|&i| old_keys[i] == key)
                .and_then(|i| old_apps[i].take());
            if let Some(i) = old {
                moved[i] = Some((self.apps.len(), reused.is_none()));
            }
            let app = reused.unwrap_or_else(|| Box::new(ScriptApp::new(&key.file_name, source)));
            self.apps.push(app);
            self.script_keys.push(key);
        }

        let remap = |idx: usize| match idx.checked_sub(compiled) {
            None => Some((idx, false)),
            Some(i) => moved[i],
        };
        if let Some(idx) = self.active_app_idx {
            match remap(idx) {
                Some((new_idx, changed)) => {
                    self.active_app_idx = Some(new_idx);
                    if changed {
                        self.apps[new_idx].on_enter();
                    }
                }
                None => {
                    self.active_app_idx = None;
                    self.selector_hints_started_at = Some(platform.millis());
                    self.audio.release_all();
                }
            }
        }
        self.last_active_app_idx = self.last_active_app_idx.and_then(remap).map(|(idx, _)| idx);
        // Scripts that were edited or removed: let up any keys they held.
        for mut app in old_apps.into_iter().flatten() {
            app.on_exit();
            app.send_hid(platform);
        }
        let pages = self.apps.len().div_ceil(APPS_PER_PAGE).max(1);
        self.selector_page = self.selector_page.min(pages - 1);
    }

    pub fn audio_mut(&mut self) -> &mut AudioEngine {
        &mut self.audio
    }

    pub fn key_down(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            return;
        }
        self.key_states[key as usize].is_down = true;
    }

    pub fn key_up(&mut self, key: IcKey) {
        if key == IcKey::_Max {
            return;
        }
        self.key_states[key as usize].is_down = false;
    }

    pub fn fill_audio(&mut self, out_pcm: &mut [i16]) {
        for sample in out_pcm.iter_mut() {
            let s = self.audio.next_sample();
            *sample = s;
        }
        self.mix_app_audio(out_pcm);
    }

    /// Mixes the open app's own audio (see [`IcShell::app_produces_audio`]),
    /// at the current volume, into `out_pcm`. `fill_audio` already does
    /// this; it's separate for platforms whose synth runs in remote mode.
    pub fn mix_app_audio(&mut self, out_pcm: &mut [i16]) {
        if !self.app_produces_audio() {
            return;
        }
        let Some(app) = self.active_app_idx.map(|idx| &mut self.apps[idx]) else {
            return;
        };
        let volume = self.audio.volume() as i32;
        let mut app_pcm = [0i16; 256];
        for out_chunk in out_pcm.chunks_mut(app_pcm.len()) {
            let app_chunk = &mut app_pcm[..out_chunk.len()];
            app.render_audio(app_chunk);
            for (out, &app_sample) in out_chunk.iter_mut().zip(app_chunk.iter()) {
                let mixed = *out as i32 + app_sample as i32 * volume / u8::MAX as i32;
                *out = mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            }
        }
    }

    pub fn has_active_audio(&self) -> bool {
        self.audio.is_active() || self.app_produces_audio()
    }

    /// Whether the open app is generating samples of its own, besides the
    /// synth. Only the open app plays its own audio, so leaving it falls silent.
    pub fn app_produces_audio(&self) -> bool {
        self.audio.volume() != 0
            && self
                .active_app_idx
                .is_some_and(|idx| self.apps[idx].produces_audio())
    }

    pub fn requires_realtime_updates(&self) -> bool {
        match self.active_app_idx {
            Some(idx) => self.apps[idx].requires_realtime_updates(),
            None => self.selector_hints_fading,
        }
    }

    /// Compact RAM readout in the status area, left of the battery icon:
    /// `H now(peak)/size  S0 peak/size  S1 peak/size`, in KiB. Turns red when
    /// any peak passes 80% of its budget.
    fn draw_memory_overlay(platform: &mut dyn IcPlatform) {
        const RIGHT_EDGE: f32 = 268.0;
        const BASELINE: f32 = 13.0;
        const SCALE: f32 = 4.0;
        struct KiB(usize);
        impl core::fmt::Display for KiB {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                let kib = self.0 as f32 / 1024.0;
                if kib < 10.0 {
                    write!(f, "{kib:.1}")
                } else {
                    write!(f, "{kib:.0}")
                }
            }
        }

        let (text, color) = match platform.memory_stats() {
            Some(stats) => {
                let text = alloc::format!(
                    "H {}({})/{}K  S0 {}/{}K  S1 {}/{}K",
                    KiB(stats.heap_used),
                    KiB(stats.heap_peak),
                    KiB(stats.heap_size),
                    KiB(stats.stack_peaks[0]),
                    KiB(stats.stack_sizes[0]),
                    KiB(stats.stack_peaks[1]),
                    KiB(stats.stack_sizes[1]),
                );
                let color = if stats.worst_fraction() > 0.8 {
                    rgb8_hex(0xff5050)
                } else {
                    rgb8_hex(0xffffff)
                };
                (text, color)
            }
            None => (
                alloc::string::String::from("memory stats n/a"),
                rgb8_hex(0xaaaaaa),
            ),
        };
        let style = TextStyle::new(SCALE, color).align(HAlign::Right);
        let width = text_width(&text, SCALE, style.font);
        platform.draw_rectangle(
            IVec2::new((RIGHT_EDGE - width) as i32 - 4, 2),
            IVec2::new(RIGHT_EDGE as i32 + 3, 18),
            rgb8_hex(0x000000),
            0,
            Some(rgb8_hex(0x000000)),
        );
        draw_text(platform, &text, Vec2::new(RIGHT_EDGE, BASELINE), style);
    }

    fn draw_battery(&mut self, platform: &mut dyn IcPlatform) {
        let soc = platform.get_battery_soc();
        let batt_percentage = soc.map(|val| clamp(val, 0, 100));
        let batt_icon_pos = IVec2::new(282, 3);
        let batt_icon_w = 34;
        let batt_icon_h = 17;
        let fill_w = (batt_percentage.unwrap_or(100) * batt_icon_w) / 100;
        platform.draw_rectangle(
            batt_icon_pos,
            batt_icon_pos + IVec2::new(batt_icon_w, batt_icon_h),
            RGB8::new(0, 0, 0),
            0,
            Some(RGB::new(0x80, 0x80, 0x80)),
        );
        if fill_w > 0 {
            platform.draw_rectangle(
                batt_icon_pos,
                batt_icon_pos + IVec2::new(fill_w, batt_icon_h),
                RGB8::new(0, 0, 0),
                0,
                Some(RGB::new(0xff, 0xff, 0xff)),
            );
        }
        if platform.is_charging() {
            draw_vitmap(
                platform,
                &graphics::CHARGING,
                0,
                Vec2::new((batt_icon_pos.x - 10) as f32, 12.0),
                Vec2::new(1.0, 1.0),
            );
        }
        match batt_percentage {
            Some(pct) => {
                let x_pos = match pct {
                    100.. => 284.0,
                    10.. => 290.0,
                    _ => 295.0,
                };
                draw_text_f(
                    platform,
                    format_args!("{}", pct),
                    Vec2::new(x_pos, 12.0),
                    TextStyle::new(5.0, Rgb::new(0, 0, 0)),
                );
            }
            None => {
                draw_text_f(
                    platform,
                    format_args!("N/A"),
                    Vec2::new(284.0, 12.0),
                    TextStyle::new(5.0, Rgb::new(0, 0, 0)),
                );
            }
        }
    }

    fn adjust_adjustable(platform: &mut dyn IcPlatform, adjustable: Adjustable, amount: i32) {
        if amount == 0 {
            return;
        }
        let og_val_32 = Self::get_adjustable(platform, adjustable) as i32;
        let new_val_32 = og_val_32.saturating_add(amount);
        let new_val = new_val_32.clamp(0, 255) as u8;
        match adjustable {
            Adjustable::Brightness => {
                platform.set_brightness(new_val);
            }
            Adjustable::Volume => {
                platform.set_volume(new_val);
            }
        }
    }

    fn get_adjustable(platform: &dyn IcPlatform, adjustable: Adjustable) -> u8 {
        match adjustable {
            Adjustable::Brightness => platform.get_brightness(),
            Adjustable::Volume => platform.get_volume(),
        }
    }

    fn draw_adjustable_readout(platform: &mut dyn IcPlatform, adjustable: Adjustable, value: u8) {
        let white = rgb8_hex(0xFFFFFF);
        let black = rgb8_hex(0x000000);

        // Outer panel
        let panel_margin = 6;
        let panel_h = 84;
        let center_y = CANVAS_HEIGHT as i32 / 2;
        let panel_start = IVec2::new(panel_margin, center_y - panel_h / 2);
        let panel_end = IVec2::new(CANVAS_WIDTH as i32 - panel_margin, center_y + panel_h / 2);
        platform.draw_rectangle(panel_start, panel_end, white, 2, Some(black));

        let icon_size = 56;
        let icon_start = IVec2::new(panel_start.x + 8, center_y - icon_size / 2);
        let icon_end = icon_start + IVec2::splat(icon_size);
        let icon = match adjustable {
            Adjustable::Brightness => &graphics::BRIGHTNESS,
            Adjustable::Volume if value == 0 => &graphics::VOLUME_OFF,
            Adjustable::Volume => &graphics::VOLUME_HIGH,
        };
        draw_vitmap(
            platform,
            icon,
            0,
            (icon_start + icon_end).as_vec2() / 2.0,
            Vec2::splat(3.0),
        );

        // Right-hand content column
        let content_left = icon_end.x + 16;
        let content_right = panel_end.x - 10;
        let row_center_y = panel_start.y + 24;
        let button_size = 22;

        let draw_button = |platform: &mut dyn IcPlatform, left: i32, plus: bool| {
            let start = IVec2::new(left, row_center_y - button_size / 2);
            let end = start + IVec2::splat(button_size);
            platform.draw_rectangle(start, end, white, 0, Some(white));
            let mid = (start + end).as_vec2() / 2.0;
            let arm = button_size as f32 / 2.0 - 4.0;
            platform.draw_line(
                Vec2::new(mid.x - arm, mid.y),
                Vec2::new(mid.x + arm, mid.y),
                black,
                4,
            );
            if plus {
                platform.draw_line(
                    Vec2::new(mid.x, mid.y - arm),
                    Vec2::new(mid.x, mid.y + arm),
                    black,
                    4,
                );
            }
        };
        draw_button(platform, content_left, false);
        draw_button(platform, content_right - button_size, true);

        let percentage = (value as u32 * 100 + 127) / 255;
        let pct_text = alloc::format!("{}%", percentage);
        let pct_style = TextStyle::new(9.0, white).align(HAlign::Center);
        let (pct_ink_top, pct_ink_bottom) =
            text_ink_bounds(&pct_text, pct_style.scale, pct_style.font).unwrap_or_default();
        draw_text(
            platform,
            &pct_text,
            Vec2::new(
                (content_left + content_right) as f32 / 2.0,
                row_center_y as f32 - (pct_ink_top + pct_ink_bottom) / 2.0,
            ),
            pct_style,
        );

        // Level bar
        let bar_stroke = 2;
        let bar_start = IVec2::new(content_left, panel_end.y - 34);
        let bar_end = IVec2::new(content_right, panel_end.y - 10);
        platform.draw_rectangle(bar_start, bar_end, white, bar_stroke as u32, Some(black));

        let inset = bar_stroke + 2;
        let max_inner_w = (bar_end.x - bar_start.x) - inset * 2;
        let fill_w = (value as i32 * max_inner_w) / 255;
        if fill_w > 0 {
            platform.draw_rectangle(
                IVec2::new(bar_start.x + inset, bar_start.y + inset),
                IVec2::new(bar_start.x + inset + fill_w, bar_end.y - inset),
                black,
                0,
                Some(rgb8_hex(0x6ABE30)),
            );
        }
    }

    fn draw_startup_help(platform: &mut dyn IcPlatform) {
        let text = "Shift+Super: open apps menu";
        let style = TextStyle::new(6.0, rgb8_hex(0xffffff)).align(HAlign::Center);
        let padding = 10.0;
        let margin = 12.0;
        let center_x = CANVAS_WIDTH as f32 / 2.0;
        let half_w = text_width(text, style.scale, style.font) / 2.0 + padding;
        let (ink_top, _) = text_ink_bounds(text, style.scale, style.font).unwrap_or_default();
        let box_h = text_height(text, style.scale, style.font) + padding * 2.0;
        let end_y = CANVAS_HEIGHT as f32 - margin;
        let start = Vec2::new(center_x - half_w, end_y - box_h);
        let end = Vec2::new(center_x + half_w, end_y);
        platform.draw_rectangle_rounded(
            start.as_ivec2(),
            end.as_ivec2(),
            rgb8_hex(0xffffff),
            2,
            Some(rgb8_hex(0x222222)),
            6,
        );
        draw_text(
            platform,
            text,
            Vec2::new(center_x, start.y + padding - ink_top),
            style,
        );
    }

    fn draw_modifier_hint(platform: &mut dyn IcPlatform, color: RGB8, icon: &Vitmap) {
        let inset = 3;
        platform.draw_rectangle(
            IVec2::splat(inset),
            IVec2::new(
                CANVAS_WIDTH as i32 - 1 - inset,
                CANVAS_HEIGHT as i32 - 1 - inset,
            ),
            rgb8_hex(0xffffff),
            2,
            None,
        );
        platform.draw_rectangle(
            IVec2::splat(0),
            IVec2::new(CANVAS_WIDTH as i32 - 1, CANVAS_HEIGHT as i32 - 1),
            color,
            4,
            None,
        );

        let icon_position = Vec2::new((CANVAS_WIDTH - 18) as f32, 18.0);
        platform.draw_rectangle(
            icon_position.as_ivec2() - IVec2::splat(16),
            IVec2::new(CANVAS_WIDTH as i32, icon_position.y as i32 + 16),
            rgb8_hex(0xffffff),
            0,
            Some(color),
        );

        draw_vitmap(platform, icon, 0, icon_position, Vec2::splat(1.5));
    }

    fn selector_cards(&self) -> [SelectorCard<'_>; 9] {
        let mut cards = [
            SelectorCard {
                name: "",
                button: IcKey::Num7,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num8,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num9,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num4,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num5,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "",
                button: IcKey::Num6,
                icon: AppIcon::Static(&graphics::SAMPLE_GRAPHIC),
                color: rgb8_hex(0x282820),
                action: SelectorAction::None,
            },
            SelectorCard {
                name: "Previous",
                button: IcKey::Num1,
                icon: AppIcon::Static(&graphics::LEFT_ARROW),
                color: rgb8_hex(0xffffff),
                action: SelectorAction::PreviousPage,
            },
            SelectorCard {
                name: "Settings",
                button: IcKey::Num2,
                icon: AppIcon::Static(&graphics::SETTINGS_ICON),
                color: rgb8_hex(0xffffff),
                action: SelectorAction::OpenSettings,
            },
            SelectorCard {
                name: "Next",
                button: IcKey::Num3,
                icon: AppIcon::Static(&graphics::RIGHT_ARROW),
                color: rgb8_hex(0xffffff),
                action: SelectorAction::NextPage,
            },
        ];

        let first_app_idx = self.selector_page * APPS_PER_PAGE;
        for (slot, card) in cards.iter_mut().take(APPS_PER_PAGE).enumerate() {
            let app_idx = first_app_idx + slot;
            if let Some(app) = self.apps.get(app_idx) {
                card.name = app.name();
                card.icon = app.icon();
                card.color = app.color();
                card.action = SelectorAction::Launch(app_idx);
            }
        }

        cards
    }

    fn selector_action(&self, key: IcKey) -> Option<SelectorAction> {
        self.selector_cards()
            .iter()
            .find(|card| card.button == key)
            .map(|card| card.action)
            .filter(|action| !matches!(action, SelectorAction::None))
    }

    fn run_selector_action(&mut self, key: IcKey) {
        match self.selector_action(key) {
            Some(SelectorAction::Launch(idx)) => {
                self.active_app_idx = Some(idx);
                self.apps[idx].on_enter();
            }
            Some(SelectorAction::PreviousPage) if self.selector_page > 0 => {
                self.selector_page -= 1;
            }
            Some(SelectorAction::NextPage)
                if (self.selector_page + 1) * APPS_PER_PAGE < self.apps.len() =>
            {
                self.selector_page += 1;
            }
            Some(SelectorAction::OpenSettings) => {
                self.settings_open = true;
                self.settings_menu.open();
            }
            _ => {}
        }
    }

    fn draw_app_selector(&self, platform: &mut dyn IcPlatform, hint_opacity: Option<u8>) {
        platform.clear(rgb8_hex(0x000000));
        let cards = self.selector_cards();
        let pressed_card = self
            .selector_pressed
            .and_then(|key| cards.iter().find(|card| card.button == key));
        // Pressing an app card hides the others; navigation cards keep the grid visible.
        let hide_unpressed = pressed_card.is_some_and(|card| {
            !matches!(
                card.action,
                SelectorAction::PreviousPage
                    | SelectorAction::NextPage
                    | SelectorAction::OpenSettings
            )
        });
        if let Some(card) = pressed_card {
            draw_text(
                platform,
                card.name,
                Vec2::new(5.0, 12.0),
                TextStyle::new(6.0, rgb8_hex(0xffffff)),
            );
        } else {
            let page_count = (self.apps.len() + APPS_PER_PAGE - 1) / APPS_PER_PAGE;
            draw_text_f(
                platform,
                format_args!("Apps - Page {} of {}", self.selector_page + 1, page_count),
                Vec2::new(5.0, 12.0),
                TextStyle::new(6.0, rgb8_hex(0xffffff)),
            );
        }

        for (index, card) in cards.iter().enumerate() {
            let column = (index % 3) as i32;
            let row = (index / 3) as i32;
            let start = IVec2::new(4 + column * 106, 27 + row * 69);
            let end = start + IVec2::new(100, 64);
            let is_empty = matches!(card.action, SelectorAction::None) && index < APPS_PER_PAGE;
            let is_pressed = self.selector_pressed == Some(card.button);
            if is_empty || (hide_unpressed && !is_pressed) {
                continue;
            }
            if is_pressed {
                let white = rgb8_hex(0xffffff);
                let black = rgb8_hex(0x000000);
                let ring = IVec2::splat(10);
                platform.draw_rectangle_rounded(start - ring, end + ring, white, 0, Some(white), 7);
                let gap = IVec2::splat(5);
                platform.draw_rectangle_rounded(start - gap, end + gap, black, 0, Some(black), 6);
            }
            platform.draw_rectangle_rounded(start, end, rgb8_hex(0xffffff), 0, Some(card.color), 5);
            card.icon.draw(
                platform,
                Vec2::new(start.x as f32 + 50.0, start.y as f32 + 32.0),
                3.0,
            );
            if let Some(opacity) = hint_opacity.filter(|_| pressed_card.is_none()) {
                let hint_start = start;
                let shadow_color = mix_colors(rgb8_hex(0x000000), card.color, opacity);
                draw_text_f(
                    platform,
                    format_args!("{}", button_number(card.button)),
                    Vec2::new(hint_start.x as f32 + 6.0, hint_start.y as f32 + 22.0),
                    TextStyle::new(10.0, shadow_color).thickness(4.0),
                );
                let text_color = mix_colors(rgb8_hex(0xffffff), card.color, opacity);
                draw_text_f(
                    platform,
                    format_args!("{}", button_number(card.button)),
                    Vec2::new(hint_start.x as f32 + 4.0, hint_start.y as f32 + 20.0),
                    TextStyle::new(10.0, text_color).thickness(4.0),
                );
            }
        }
    }

    pub fn update(&mut self, platform: &mut dyn IcPlatform) {
        if !self.settings_loaded {
            self.settings_menu.load(platform);
            self.settings_loaded = true;
        }
        #[cfg(feature = "scripting")]
        self.sync_scripts(platform);
        for s in self.key_states.iter_mut() {
            s.just_pressed = s.is_down && !s.was_down;
            s.just_released = !s.is_down && s.was_down;
            s.was_down = s.is_down;
        }
        let key_mode = self.settings_menu.key_mode();
        self.modifiers.set_mode(key_mode);
        let mut effective_states = self.modifiers.apply(&self.key_states);
        // Super + Shift toggles the menu; latched modifiers count toward the chord.
        let menu_chord = effective_states[IcKey::Super as usize].is_down
            && effective_states[IcKey::Shift as usize].is_down
            && (self.key_states[IcKey::Super as usize].just_pressed
                || self.key_states[IcKey::Shift as usize].just_pressed);
        if menu_chord {
            self.adjusting_something = None;
            self.modifiers.clear_latches();
            effective_states[IcKey::Super as usize].is_down = false;
            effective_states[IcKey::Shift as usize].is_down = false;
        }
        let shift_engaged = effective_states[IcKey::Shift as usize].is_down;
        let ctx = InputContext {
            key_states: &effective_states,
            key_mode,
            shift_engaged: shift_engaged && !self.shift_was_engaged,
            shift_disengaged: !shift_engaged && self.shift_was_engaged,
        };
        self.shift_was_engaged = shift_engaged;
        if self.key_states.iter().any(|state| state.just_pressed) {
            self.show_startup_help = false;
        }
        let backspace_closes_menu = self.active_app_idx.is_none()
            && !self.settings_open
            && self.key_states[IcKey::Backspace as usize].just_pressed;
        for i in 0..IcKey::COUNT {
            if !self.key_states[i].just_pressed {
                continue;
            }
            let Some(key) = IcKey::from_usize(i) else {
                continue;
            };
            if menu_chord && matches!(key, IcKey::Shift | IcKey::Super) {
                continue;
            }
            let mut input_consumed_by_shell: bool = false;
            let adjust_amt = match key {
                IcKey::Minus => -32,
                IcKey::Plus => 32,
                _ => 0,
            };
            if let Some(adjustable) = self.adjusting_something
                && adjust_amt != 0
            {
                // +/- adjust even with Super engaged (otherwise they'd be F2/F3), and
                // consume a latched Super so it doesn't linger after adjusting.
                Self::adjust_adjustable(platform, adjustable, adjust_amt);
                self.modifiers.clear_super_latch();
                input_consumed_by_shell = true;
            } else if ctx.is_super() {
                let adjustable = match key {
                    IcKey::Num3 => Some(Adjustable::Brightness),
                    IcKey::Num9 => Some(Adjustable::Volume),
                    _ => None,
                };
                if adjustable.is_some() {
                    self.adjusting_something = adjustable;
                    // In Toggle mode Super would stay latched; release it so the
                    // user doesn't press Super again (which would dismiss the UI).
                    self.modifiers.clear_super_latch();
                    input_consumed_by_shell = true;
                }
            } else if self.adjusting_something.is_some() {
                input_consumed_by_shell = true;
                self.adjusting_something = None;
            }
            self.held_by_shell[i] |= input_consumed_by_shell;
            if let Some(app_idx) = self.active_app_idx {
                if !input_consumed_by_shell {
                    self.apps[app_idx].on_key(key, &ctx);
                }
            } else if !input_consumed_by_shell {
                if self.settings_open {
                    if self.settings_menu.handle_key(key, &ctx, platform) {
                        self.settings_open = false;
                        self.settings_menu.close(platform);
                    }
                } else if self.selector_action(key).is_some() {
                    // Highlight now; the action runs on release.
                    self.selector_pressed = Some(key);
                }
            }
        }
        if let Some(app_idx) = self.active_app_idx {
            for i in 0..IcKey::COUNT {
                if !self.key_states[i].just_released || self.held_by_shell[i] {
                    continue;
                }
                if let Some(key) = IcKey::from_usize(i) {
                    self.apps[app_idx].on_key_up(key, &ctx);
                }
            }
        }
        if backspace_closes_menu
            || menu_chord
            || self.active_app_idx.is_some()
            || self.settings_open
        {
            self.selector_pressed = None;
        } else if let Some(key) = self.selector_pressed
            && self.key_states[key as usize].just_released
        {
            self.selector_pressed = None;
            self.run_selector_action(key);
        }
        if backspace_closes_menu || menu_chord {
            if let Some(idx) = self.active_app_idx {
                self.apps[idx].on_exit();
                self.apps[idx].send_hid(platform);
                self.last_active_app_idx = self.active_app_idx;
                self.active_app_idx = None;
                self.selector_hints_started_at = Some(platform.millis());
                // release its sound instead of leaving a held note behind.
                self.audio.release_all();
            } else if self.settings_open {
                self.settings_open = false;
                self.settings_menu.close(platform);
            } else if let Some(prev) = self.last_active_app_idx {
                self.active_app_idx = Some(prev);
                self.apps[prev].on_enter();
            }
        }
        self.audio.set_volume(platform.get_volume());
        let mut app_states = effective_states;
        for (i, held) in self.held_by_shell.iter_mut().enumerate() {
            *held &= self.key_states[i].is_down;
            if *held {
                app_states[i] = KeyState::default();
            }
        }
        let app_ctx = InputContext {
            key_states: &app_states,
            ..ctx
        };
        self.selector_hints_fading = false;
        if let Some(appidx) = self.active_app_idx {
            self.apps[appidx].update(platform, &app_ctx, &mut self.audio);
            self.apps[appidx].send_hid(platform);
        } else if self.settings_open {
            self.settings_menu.draw(platform);
        } else {
            let hint_opacity = self.selector_hints_started_at.map(|started_at| {
                let elapsed = platform.millis().saturating_sub(started_at);
                let fade_elapsed = elapsed.saturating_sub(KEY_HINT_FADE_DELAY_MS);
                (fade_elapsed.min(KEY_HINT_FADE_IN_MS) * u8::MAX as u64 / KEY_HINT_FADE_IN_MS) as u8
            });
            self.selector_hints_fading = hint_opacity.is_some_and(|opacity| opacity < u8::MAX);
            self.draw_app_selector(platform, hint_opacity);
        }
        if let Some(adjustable) = self.adjusting_something {
            let v = Self::get_adjustable(platform, adjustable);
            Self::draw_adjustable_readout(platform, adjustable, v);
        }
        self.draw_battery(platform);
        if self.settings_menu.show_memory() {
            Self::draw_memory_overlay(platform);
        }
        if self.show_startup_help {
            Self::draw_startup_help(platform);
        }
        if ctx.is_shifted() {
            Self::draw_modifier_hint(platform, rgb8_hex(0x0000ff), &graphics::SHIFT);
        } else if ctx.is_super() {
            Self::draw_modifier_hint(platform, rgb8_hex(0xff0000), &graphics::SUPER);
        }
        self.modifiers.set_mode(self.settings_menu.key_mode());
    }
}

fn button_number(button: IcKey) -> u8 {
    match button {
        IcKey::Num1 => 1,
        IcKey::Num2 => 2,
        IcKey::Num3 => 3,
        IcKey::Num4 => 4,
        IcKey::Num5 => 5,
        IcKey::Num6 => 6,
        IcKey::Num7 => 7,
        IcKey::Num8 => 8,
        IcKey::Num9 => 9,
        _ => 0,
    }
}

fn mix_colors(color_a: RGB8, color_b: RGB8, color_a_weight: u8) -> RGB8 {
    let color_a_weight = color_a_weight as u16;
    let color_b_weight = u8::MAX as u16 - color_a_weight;
    let mix_channel = |color_a: u8, color_b: u8| {
        ((color_a as u16 * color_a_weight + color_b as u16 * color_b_weight + 127) / u8::MAX as u16)
            as u8
    };

    RGB8::new(
        mix_channel(color_a.r, color_b.r),
        mix_channel(color_a.g, color_b.g),
        mix_channel(color_a.b, color_b.b),
    )
}

/// FNV-1a, to notice when a script's source changes.
#[cfg(feature = "scripting")]
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |hash, &b| {
        (hash ^ b as u64).wrapping_mul(0x0000_0100_0000_01B3)
    })
}

#[cfg(all(test, feature = "scripting"))]
mod tests {
    use super::*;
    use crate::fake_platform::FakePlatform;
    use alloc::string::{String, ToString};

    fn script_names(shell: &IcShell) -> Vec<String> {
        let compiled = shell.apps.len() - shell.script_keys.len();
        shell.apps[compiled..]
            .iter()
            .map(|app| app.name().to_string())
            .collect()
    }

    fn open_last_app(shell: &mut IcShell) -> usize {
        let idx = shell.apps.len() - 1;
        shell.active_app_idx = Some(idx);
        shell.apps[idx].on_enter();
        idx
    }

    fn app_index(shell: &IcShell, name: &str) -> Option<usize> {
        shell.apps.iter().position(|app| app.name() == name)
    }

    #[test]
    fn an_open_app_reloads_when_its_script_is_saved() {
        let mut platform = FakePlatform::with_scripts(&[("a.lua", "-- title: One")]);
        let mut shell = IcShell::new();
        shell.update(&mut platform);
        let idx = app_index(&shell, "One").unwrap();
        shell.active_app_idx = Some(idx);
        shell.apps[idx].on_enter();
        shell.update(&mut platform);

        // A computer saves a new version to the drive while the app is open.
        platform.set_script("a.lua", Some("-- title: Two"));
        shell.update(&mut platform);
        let idx = shell.active_app_idx.expect("still open");
        assert_eq!(shell.apps[idx].name(), "Two");
    }

    #[test]
    fn without_storage_the_bundled_scripts_are_shown() {
        let mut shell = IcShell::new();
        shell.update(&mut FakePlatform::default());
        assert_eq!(script_names(&shell), ["Bouncer", "Color Converter", "Face"]);
    }

    #[test]
    fn scripts_are_listed_by_file_name() {
        let mut platform = FakePlatform::with_scripts(&[("zeta.lua", ""), ("alpha.lua", "")]);
        let mut shell = IcShell::new();
        shell.update(&mut platform);
        assert_eq!(script_names(&shell), ["alpha", "zeta"]);

        platform.set_script("beta.lua", Some(""));
        shell.update(&mut platform);
        assert_eq!(script_names(&shell), ["alpha", "beta", "zeta"]);
    }

    #[test]
    fn unchanged_scripts_keep_their_app() {
        let mut platform = FakePlatform::with_scripts(&[("a.lua", ""), ("b.lua", "")]);
        let mut shell = IcShell::new();
        shell.update(&mut platform);
        let before: *const dyn IcApp = &*shell.apps[shell.apps.len() - 1];

        platform.set_script("a.lua", Some("x = 1"));
        shell.update(&mut platform);
        let after: *const dyn IcApp = &*shell.apps[shell.apps.len() - 1];
        assert!(core::ptr::addr_eq(before, after), "b.lua was reloaded");
    }

    #[test]
    fn editing_the_open_script_restarts_it() {
        let mut platform = FakePlatform::with_scripts(&[("app.lua", "-- title: Old")]);
        let mut shell = IcShell::new();
        shell.update(&mut platform);
        let idx = open_last_app(&mut shell);

        platform.set_script("app.lua", Some("-- title: New"));
        shell.update(&mut platform);
        assert_eq!(shell.active_app_idx, Some(idx));
        assert_eq!(shell.apps[idx].name(), "New");
    }

    #[test]
    fn deleting_the_open_script_returns_to_the_menu() {
        let mut platform = FakePlatform::with_scripts(&[("a.lua", ""), ("b.lua", "")]);
        let mut shell = IcShell::new();
        shell.update(&mut platform);
        open_last_app(&mut shell);

        platform.set_script("b.lua", None);
        shell.update(&mut platform);
        assert_eq!(shell.active_app_idx, None);
        assert_eq!(script_names(&shell), ["a"]);
    }
}
