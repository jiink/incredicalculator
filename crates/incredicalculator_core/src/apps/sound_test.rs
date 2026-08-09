use crate::app::{IcApp, InputContext};
use crate::audio_engine::{AudioEngine, AudioPatch, NoteId};
use crate::input::IcKey;
use crate::text;
use rgb::RGB8;

const SOUND_TEST_SOURCE_BASE: u32 = 0x534f_0000;
const ENV_STEP_MS: i32 = 50;
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

    fn adjust_attack(&mut self, amount_ms: i32) {
        self.patch.attack_ms = adjust_envelope_ms(self.patch.attack_ms, amount_ms);
    }

    fn adjust_release(&mut self, amount_ms: i32) {
        self.patch.release_ms = adjust_envelope_ms(self.patch.release_ms, amount_ms);
    }

    fn adjust_square_mix(&mut self, amount: i16) {
        let square = (self.patch.square_mix as i16 + amount)
            .clamp(0, WAVE_MIX_TOTAL as i16) as u8;
        self.patch.square_mix = square;
        self.patch.triangle_mix = WAVE_MIX_TOTAL - square;
    }
}

impl IcApp for SoundTest {
    fn on_enter(&mut self) {}

    fn on_key(&mut self, key: IcKey, _ctx: &InputContext) {
        match key {
            IcKey::Func1 => self.adjust_attack(-ENV_STEP_MS),
            IcKey::Func2 => self.adjust_attack(ENV_STEP_MS),
            IcKey::Func3 => self.adjust_release(-ENV_STEP_MS),
            IcKey::Func4 => self.adjust_release(ENV_STEP_MS),
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
            format_args!("F1/F2 Attack: {} ms", self.patch.attack_ms),
            4.0,
            28.0,
            2.5,
            RGB8::new(0, 0, 0),
        );
        text::draw_text_f(
            platform,
            format_args!("F3/F4 Release: {} ms", self.patch.release_ms),
            4.0,
            48.0,
            2.5,
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
            68.0,
            2.5,
            RGB8::new(0, 0, 0),
        );
    }

    fn name(&self) -> &str {
        "Sound test"
    }
}

fn adjust_envelope_ms(value: u16, amount: i32) -> u16 {
    (value as i32 + amount).clamp(5, 7_900) as u16
}
