use crate::audio_engine::AudioEngine;
use crate::graphics::Vitmap;
use crate::input::{IcKey, KeyMode, KeyState};
use crate::platform::IcPlatform;
use rgb::RGB8;

pub struct InputContext<'a> {
    /// Physical key transitions with effective (possibly latched) modifier held states.
    pub key_states: &'a [KeyState; IcKey::COUNT],
    pub key_mode: KeyMode,
    /// Effective shift turned on this frame (physical press, or latch in Sticky/Toggle).
    pub shift_engaged: bool,
    /// Effective shift turned off this frame.
    pub shift_disengaged: bool,
}

impl<'a> InputContext<'a> {
    pub fn is_down(&self, key: IcKey) -> bool {
        self.key_states[key as usize].is_down
    }
    pub fn is_shifted(&self) -> bool {
        self.key_states[IcKey::Shift as usize].is_down
    }
    pub fn is_super(&self) -> bool {
        self.key_states[IcKey::Super as usize].is_down
    }
}

pub trait IcApp {
    fn name(&self) -> &str;
    fn icon(&self) -> &'static Vitmap;
    fn color(&self) -> RGB8;
    /// for things like games that shouldnt only update when you press a button
    fn requires_realtime_updates(&self) -> bool;
    fn on_enter(&mut self);
    fn on_key(&mut self, key: IcKey, ctx: &InputContext);
    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        audio: &mut AudioEngine,
    );
    /// Whether the app is generating its own samples through `render_audio`.
    /// While true, the platform keeps audio output running.
    fn produces_audio(&self) -> bool {
        false
    }
    /// Writes the app's next samples into `out_pcm`, at
    /// `audio_engine::SAMPLE_RATE_HZ`. The shell applies the volume and mixes
    /// them over the synth. Only called while `produces_audio` is true.
    fn render_audio(&mut self, _out_pcm: &mut [i16]) {}
}
