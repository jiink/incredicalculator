use culsynth::{EnvParamFxP, NoteFxP, SampleFxP, ScalarFxP, LfoFreqFxP, IScalarFxP};
use culsynth::context::ContextFxP;
use culsynth::voice::{Voice, VoiceChannelInput, VoiceInput, VoiceParams};
use culsynth::voice::modulation::{ModMatrix, ModSrc, ModDest}; // Added

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
        // 1. Determine whether we need to upload the matrix or reuse the cached one.
        let matrix_param = if self.synth.mod_matrix_dirty {
            self.synth.mod_matrix_dirty = false;
            Some(&self.synth.mod_matrix)
        } else {
            None
        };

        let sample_fxp: SampleFxP = self.synth.voice.next(
            &self.synth.ctx,
            matrix_param, // Only passed on the first run (or when dirty)
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

    /// Dynamically update the vibrato depth (can be hooked up to mod-wheel, etc.)
    pub fn set_vibrato_depth(&mut self, depth: f32) {
        self.synth.mod_matrix.rows[ModSrc::Lfo1 as usize].1[0] = (
            ModDest::Osc1Fine,
            IScalarFxP::from_num(depth),
        );
        // Mark dirty so the voice updates its internal matrix on the next sample
        self.synth.mod_matrix_dirty = true;
    }
}

struct CulSynthSource {
    voice: Voice<i16>,
    params: VoiceParams<i16>,
    ch_input: VoiceChannelInput<i16>,
    ctx: ContextFxP,
    cached_input: VoiceInput<i16>,
    mod_matrix: ModMatrix<i16>,
    mod_matrix_dirty: bool,
}

impl CulSynthSource {
    fn new() -> Self {
        let mut params = VoiceParams::<i16>::default();
        params.oscs_p.primary.saw = ScalarFxP::from_num(0.0);
        params.oscs_p.primary.tri = ScalarFxP::from_num(0.8);
        params.amp_env_p.attack = EnvParamFxP::from_num(0.2);
        params.amp_env_p.release = EnvParamFxP::from_num(1.5);
        params.filt_p.cutoff = NoteFxP::from_num(120.0);
        params.filt_p.low_mix = ScalarFxP::from_num(0.999);
        params.ring_p.mix_a = ScalarFxP::from_num(0.999);

        // 2. Configure LFO1 frequency for vibrato rate
        params.lfo1_p.freq = LfoFreqFxP::from_num(6.0);    // 6.0 Hz vibrato speed
        params.lfo1_p.depth = ScalarFxP::from_num(0.999); // Pass full LFO output to matrix

        // 3. Map LFO1 -> Oscillator 1 Fine pitch in the ModMatrix
        let mut mod_matrix = ModMatrix::<i16>::default();
        mod_matrix.rows[ModSrc::Lfo1 as usize].1[0] = (
            ModDest::Osc1Fine,
            IScalarFxP::from_num(0.15), // Initial vibrato depth
        );

        Self {
            voice: Voice::<i16>::new(),
            params,
            ch_input: VoiceChannelInput::<i16>::default(),
            ctx: ContextFxP::new_480(),
            cached_input: VoiceInput::<i16>::default(),
            mod_matrix,
            mod_matrix_dirty: true, // Initialized as true so the first sample loads the matrix
        }
    }
}