use culsynth::context::Context;
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
    pub fn next_sample(&mut self) -> f32 {
        self.synth.voice.next(
            &self.synth.ctx,
            None,
            &self.synth.cached_input,
            &self.synth.ch_input,
            self.synth.params.clone(),
        )
    }
    pub fn note_on(&mut self, midi_note: u8) {
        self.synth.cached_input.note = midi_note as f32;
        self.synth.cached_input.velocity = 0.8;
        self.synth.cached_input.gate = true;
    }
    pub fn note_off(&mut self) {
        self.synth.cached_input.gate = false;
    }
}

struct CulSynthSource {
    voice: Voice<f32>,
    params: VoiceParams<f32>,
    ch_input: VoiceChannelInput<f32>,
    ctx: Context<f32>,
    // Cached copy so we don't block if the mutex is contended.
    cached_input: VoiceInput<f32>,
    //control: Arc<Mutex<SynthControl>>,
}

impl CulSynthSource {
    fn new() -> Self {
        let mut params = VoiceParams::<f32>::default();
        params.oscs_p.primary.saw = 0.1;
        params.oscs_p.primary.tri = 1.0;
        params.amp_env_p.attack = 0.01;
        params.amp_env_p.release = 0.1;
        params.filt_p.cutoff = 120.0;
        params.filt_p.low_mix = 1.0;
        params.ring_p.mix_a = 1.0;

        Self {
            voice: Voice::<f32>::new(),
            params,
            ch_input: VoiceChannelInput::<f32>::default(),
            ctx: Context::<f32>::new(48000.0),
            cached_input: VoiceInput::<f32>::default(),
            //control,
        }
    }
}
