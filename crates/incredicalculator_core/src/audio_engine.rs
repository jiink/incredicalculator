use culsynth::{EnvParamFxP, NoteFxP, SampleFxP};
use culsynth::context::ContextFxP;
use culsynth::devices::{Amp, Device, Env, EnvParams, Osc, OscParams};

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
        // This intentionally uses CulSynth's component devices rather than
        // `Voice<i16>`. The full Voice evaluates two oscillators, modulation,
        // ring modulation, a filter envelope, and a modulated filter per
        // sample even when this patch does not need those stages.
        let oscillator = self.synth.osc.next(
            &self.synth.ctx,
            self.synth.note,
            self.synth.osc_params.clone(),
        );
        let amplitude = self.synth.amp_env.next(
            &self.synth.ctx,
            self.synth.gate,
            self.synth.amp_env_params.clone(),
        );
        let sample: SampleFxP = self.synth.amp.next(&self.synth.ctx, oscillator.tri, amplitude);
        sample.to_bits().saturating_mul(8)
    }

    pub fn note_on(&mut self, midi_note: u8) {
        self.synth.note = NoteFxP::from_num(midi_note);
        self.synth.gate = true;
    }

    pub fn note_off(&mut self) {
        self.synth.gate = false;
    }

}

struct CulSynthSource {
    ctx: ContextFxP,
    osc: Osc<i16>,
    osc_params: OscParams<i16>,
    amp_env: Env<i16>,
    amp_env_params: EnvParams<i16>,
    amp: Amp<i16>,
    note: NoteFxP,
    gate: bool,
}

impl CulSynthSource {
    fn new() -> Self {
        let mut amp_env_params = EnvParams::<i16>::default();
        amp_env_params.attack = EnvParamFxP::from_num(0.2);
        amp_env_params.release = EnvParamFxP::from_num(1.5);

        Self {
            ctx: ContextFxP::new_480(),
            osc: Osc::<i16>::new(),
            osc_params: OscParams::<i16>::default(),
            amp_env: Env::<i16>::default(),
            amp_env_params,
            amp: Amp::<i16>::default(),
            note: NoteFxP::from_num(0),
            gate: false,
        }
    }
}
