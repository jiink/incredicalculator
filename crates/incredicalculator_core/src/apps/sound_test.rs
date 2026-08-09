use crate::app::{IcApp, InputContext};
use crate::audio_engine::{AudioEngine, AudioPatch, NoteId};
use crate::input::IcKey;
use crate::text;
use rgb::RGB8;

const SOUND_TEST_SOURCE_BASE: u32 = 0x534f_0000;
const ENV_PRESETS_MS: [u16; 5] = [0, 100, 500, 1_000, 2_500];
const VIBRATO_ON_DEPTH_CENTS: u8 = 5;
const WAVE_MIX_TOTAL: u8 = 204;

pub struct SoundTest {
    patch: AudioPatch,
}

impl SoundTest {
    pub fn new() -> SoundTest {
        Self {
            patch: AudioPatch::default(),
        }
    }

    fn cycle_attack(&mut self) {
        self.patch.attack_ms = next_envelope_preset(self.patch.attack_ms);
    }

    fn cycle_release(&mut self) {
        self.patch.release_ms = next_envelope_preset(self.patch.release_ms);
    }

    fn toggle_vibrato(&mut self) {
        self.patch.vibrato_depth_cents = if self.patch.vibrato_depth_cents == 0 {
            VIBRATO_ON_DEPTH_CENTS
        } else {
            0
        };
    }

    fn adjust_square_mix(&mut self, amount: i16) {
        let square = (self.patch.square_mix as i16 + amount)
            .clamp(0, WAVE_MIX_TOTAL as i16) as u8;
        self.patch.square_mix = square;
        self.patch.triangle_mix = WAVE_MIX_TOTAL - square;
    }
}

impl IcApp for SoundTest {
    fn requires_realtime_updates(&self) -> bool {
        false
    }

    fn on_enter(&mut self) {}

    fn on_key(&mut self, key: IcKey, _ctx: &InputContext) {
        match key {
            IcKey::Func2 => self.toggle_vibrato(),
            IcKey::Func3 => self.cycle_attack(),
            IcKey::Func4 => self.cycle_release(),
            IcKey::Func5 => self.adjust_square_mix(-16),
            IcKey::Func6 => self.adjust_square_mix(16),
            _ => {}
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn crate::platform::IcPlatform,
        ctx: &InputContext,
        audio: &mut AudioEngine,
    ) {
        audio.set_patch(self.patch);

        let note_keys = [
            (IcKey::Num0, 60),
            (IcKey::Num1, 61),
            (IcKey::Num2, 62),
            (IcKey::Num3, 63),
            (IcKey::Num4, 64),
            (IcKey::Num5, 65),
            (IcKey::Num6, 66),
            (IcKey::Num7, 67),
            (IcKey::Num8, 68),
            (IcKey::Num9, 69),
        ];

        let mut held_notes = 0;
        for (index, (key, midi_note)) in note_keys.iter().enumerate() {
            let held = ctx.is_down(*key);
            if held {
                held_notes += 1;
            }
            // The source ID happens to be associated with a key in this app,
            // but the audio API itself has no hardware-key dependency.
            audio.set_note(
                NoteId::new(SOUND_TEST_SOURCE_BASE + index as u32),
                held.then_some(*midi_note),
            );
        }

        platform.clear(RGB8::new(200, 200, 200));
        text::draw_text_f(
            platform,
            format_args!("Held: {} / {}", held_notes, crate::audio_engine::MAX_VOICES),
            4.0,
            4.0,
            3.0,
            RGB8::new(0, 0, 0),
        );
        text::draw_text_f(
            platform,
            format_args!(
                "F2 Vibrato: {}",
                if self.patch.vibrato_depth_cents == 0 { "Off" } else { "On" },
            ),
            4.0,
            28.0,
            2.0,
            RGB8::new(0, 0, 0),
        );
        text::draw_text_f(
            platform,
            format_args!("F3 Attack: {} ms", self.patch.attack_ms),
            4.0,
            48.0,
            2.0,
            RGB8::new(0, 0, 0),
        );
        text::draw_text_f(
            platform,
            format_args!("F4 Release: {} ms", self.patch.release_ms),
            4.0,
            68.0,
            2.0,
            RGB8::new(0, 0, 0),
        );
        text::draw_text_f(
            platform,
            format_args!(
                "F5/F6 Triangle/Square: {}/{}",
                self.patch.triangle_mix,
                self.patch.square_mix,
            ),
            4.0,
            88.0,
            2.0,
            RGB8::new(0, 0, 0),
        );
    }

    fn name(&self) -> &str {
        "Sound test"
    }
}

fn next_envelope_preset(value_ms: u16) -> u16 {
    ENV_PRESETS_MS
        .iter()
        .copied()
        .find(|&preset_ms| preset_ms > value_ms)
        .unwrap_or(ENV_PRESETS_MS[0])
}
