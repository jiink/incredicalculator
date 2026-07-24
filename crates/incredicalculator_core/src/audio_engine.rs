use culsynth::{EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP};
use culsynth::context::{Context, ContextFxP};
use culsynth::voice::{Voice, VoiceChannelInput, VoiceInput, VoiceParams};
use num_traits::ToPrimitive;

pub struct AudioEngine {
    synth: CulSynthSource,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            synth: CulSynthSource::new(),
        }
    }
    pub fn next_sample(&mut self) -> i16 {
        let sample_fxp: SampleFxP = self.synth.voice.next(
            &self.synth.ctx,
            None,
            &self.synth.cached_input,
            &self.synth.ch_input,
            self.synth.params.clone(),
        );
        // Convert the underlying raw bits of SampleFxP (I4F12) to standard i16 PCM.
        sample_fxp.to_bits().saturating_mul(8)
    }
    pub fn note_on(&mut self, midi_note: u8) {
        self.synth.cached_input.note = NoteFxP::from_num(midi_note);
        self.synth.cached_input.velocity = ScalarFxP::from_num(0.8);
        self.synth.cached_input.gate = true;
    }
    pub fn note_off(&mut self) {
        self.synth.cached_input.gate = false;
    }
}

struct CulSynthSource {
    voice: Voice<i16>,
    params: VoiceParams<i16>,
    ch_input: VoiceChannelInput<i16>,
    ctx: ContextFxP,
    cached_input: VoiceInput<i16>,
}

impl CulSynthSource {
    fn new() -> Self {
        let mut params = VoiceParams::<i16>::default();
        params.oscs_p.primary.saw = ScalarFxP::from_num(0.1);
        params.oscs_p.primary.tri = ScalarFxP::from_num(0.999);
        params.amp_env_p.attack = EnvParamFxP::from_num(0.01);
        params.amp_env_p.release = EnvParamFxP::from_num(0.1);
        params.filt_p.cutoff = NoteFxP::from_num(120.0);
        params.filt_p.low_mix = ScalarFxP::from_num(0.999);
        params.ring_p.mix_a = ScalarFxP::from_num(0.999);

        Self {
            voice: Voice::<i16>::new(),
            params,
            ch_input: VoiceChannelInput::<i16>::default(),
            ctx: ContextFxP::new_480(),
            cached_input: VoiceInput::<i16>::default(),
        }
    }
}
