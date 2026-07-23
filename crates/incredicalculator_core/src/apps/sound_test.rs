use crate::{app::IcApp, input::IcKey, text};
use culsynth::context::Context;
use culsynth::voice::{Voice, VoiceChannelInput, VoiceInput, VoiceParams};
use rgb::RGB8;

pub struct SoundTest {
    active_note: Option<u8>,
}

impl SoundTest {
    pub fn new() -> SoundTest {
        SoundTest {
            active_note: None,
        }
    }
}

impl IcApp for SoundTest {
    fn on_enter(&mut self) {
        ()
    }

    fn on_key(&mut self, _key: crate::input::IcKey, _ctx: &crate::app::InputContext) {
        ()
    }

    fn update(
        &mut self,
        platform: &mut dyn crate::platform::IcPlatform,
        ctx: &crate::app::InputContext,
        audio: &mut crate::audio_engine::AudioEngine
    ) {
        let num_keys = [
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

        let mut p: Option<u8> = None;
        for (k, note) in num_keys {
            if ctx.is_down(k) {
                p = Some(note);
                break;
            }
        }
        self.active_note = p;
        platform.clear(RGB8::new(200, 200, 200));
        text::draw_text_f(
            platform,
            format_args!("N: {}", self.active_note.unwrap_or(0)),
            4.0,
            4.0,
            5.0,
            RGB8::new(0, 0, 0),
        );

        if let Some(n) = self.active_note {
            audio.note_on(n);
        } else {
            audio.note_off();
        }

    }
    
    fn name(&self) -> &str {
        "Sound test"
    }
}
