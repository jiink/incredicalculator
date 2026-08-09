use culsynth::{EnvParamFxP, NoteFxP, SampleFxP};
use culsynth::context::ContextFxP;
use culsynth::devices::{Amp, Device, Env, EnvParams, Osc, OscParams};

pub const MAX_VOICES: usize = 4;

/// Identifies a logical note source owned by an app, sequencer, or control.
///
/// A source is deliberately independent from the MIDI pitch and from hardware
/// keys: two sources may play the same pitch at the same time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteId(u32);

impl NoteId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

/// Patch parameters shared by every active voice.
#[derive(Clone, Copy)]
pub struct AudioPatch {
    pub attack_ms: u16,
    pub release_ms: u16,
    pub triangle_mix: u8,
    pub square_mix: u8,
}

impl Default for AudioPatch {
    fn default() -> Self {
        Self {
            attack_ms: 200,
            release_ms: 1_500,
            // Preserve the former triangle's approximate 0.8 gain while
            // leaving headroom for four voices in the final mixer.
            triangle_mix: 204,
            square_mix: 0,
        }
    }
}

pub struct AudioEngine {
    context: ContextFxP,
    patch: AudioPatch,
    amp_env_params: EnvParams<i16>,
    voices: [SynthVoice; MAX_VOICES],
    next_start_order: u32,
}

impl AudioEngine {
    pub fn new() -> Self {
        let patch = AudioPatch::default();
        Self {
            context: ContextFxP::new_480(),
            patch,
            amp_env_params: env_params_for_patch(patch),
            voices: core::array::from_fn(|_| SynthVoice::new()),
            next_start_order: 0,
        }
    }

    pub fn patch(&self) -> AudioPatch {
        self.patch
    }

    pub fn set_patch(&mut self, patch: AudioPatch) {
        self.patch = sanitize_patch(patch);
        self.amp_env_params = env_params_for_patch(self.patch);
    }

    /// Set or release the note belonging to `source`.
    ///
    /// Calling this repeatedly with an unchanged note is a no-op. A source
    /// does not name a physical voice; the engine assigns a free or released
    /// voice and steals the oldest held voice only as a last resort.
    pub fn set_note(&mut self, source: NoteId, midi_note: Option<u8>) {
        match midi_note {
            Some(midi_note) => self.start_or_update_note(source, midi_note),
            None => {
                if let Some(voice) = self
                    .voices
                    .iter_mut()
                    .find(|voice| voice.source == Some(source))
                {
                    voice.release();
                }
            }
        }
    }

    /// Gate every active note off, preserving its normal release envelope.
    pub fn release_all(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.release();
        }
    }

    pub fn next_sample(&mut self) -> i16 {
        let mut mixed = 0i32;
        for voice in self.voices.iter_mut() {
            mixed += voice.next_sample(
                &self.context,
                &self.amp_env_params,
                self.patch.triangle_mix,
                self.patch.square_mix,
            ) as i32;
        }

        // Keep enough headroom that four full-scale notes do not clip.
        (mixed / MAX_VOICES as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16
    }

    fn start_or_update_note(&mut self, source: NoteId, midi_note: u8) {
        if let Some(voice) = self
            .voices
            .iter_mut()
            .find(|voice| voice.source == Some(source))
        {
            if voice.gate && voice.midi_note == midi_note {
                return;
            }
            voice.start(source, midi_note, self.next_start_order);
            self.next_start_order = self.next_start_order.wrapping_add(1);
            return;
        }

        let index = self.choose_voice();
        self.voices[index].start(source, midi_note, self.next_start_order);
        self.next_start_order = self.next_start_order.wrapping_add(1);
    }

    fn choose_voice(&self) -> usize {
        // Prefer unused voices, then the quietest release tail, then the
        // oldest held note if every voice is currently active.
        if let Some(index) = self.voices.iter().position(|voice| voice.source.is_none()) {
            return index;
        }
        if let Some((index, _)) = self
            .voices
            .iter()
            .enumerate()
            .filter(|(_, voice)| !voice.gate)
            .min_by_key(|(_, voice)| voice.last_level)
        {
            return index;
        }
        self.voices
            .iter()
            .enumerate()
            .min_by_key(|(_, voice)| voice.start_order)
            .map(|(index, _)| index)
            .unwrap_or(0)
    }
}

struct SynthVoice {
    osc: Osc<i16>,
    osc_params: OscParams<i16>,
    amp_env: Env<i16>,
    amp: Amp<i16>,
    source: Option<NoteId>,
    midi_note: u8,
    gate: bool,
    start_order: u32,
    last_level: u16,
}

impl SynthVoice {
    fn new() -> Self {
        Self {
            osc: Osc::<i16>::new(),
            osc_params: OscParams::<i16>::default(),
            amp_env: Env::<i16>::default(),
            amp: Amp::<i16>::default(),
            source: None,
            midi_note: 0,
            gate: false,
            start_order: 0,
            last_level: 0,
        }
    }

    fn start(&mut self, source: NoteId, midi_note: u8, start_order: u32) {
        // A newly assigned source must not inherit the old note's phase or
        // release level, especially when this voice was selected for stealing.
        self.osc = Osc::<i16>::new();
        self.amp_env = Env::<i16>::default();
        self.source = Some(source);
        self.midi_note = midi_note;
        self.gate = true;
        self.start_order = start_order;
        self.last_level = 0;
    }

    fn release(&mut self) {
        self.gate = false;
    }

    fn next_sample(
        &mut self,
        context: &ContextFxP,
        amp_env_params: &EnvParams<i16>,
        triangle_mix: u8,
        square_mix: u8,
    ) -> i16 {
        let Some(_) = self.source else {
            return 0;
        };

        let oscillator = self.osc.next(
            context,
            NoteFxP::from_num(self.midi_note),
            self.osc_params.clone(),
        );
        let mixed_wave_bits = ((oscillator.tri.to_bits() as i32 * triangle_mix as i32)
            + (oscillator.sq.to_bits() as i32 * square_mix as i32))
            / u8::MAX as i32;
        let mixed_wave = SampleFxP::from_bits(
            mixed_wave_bits.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        );
        let amplitude = self
            .amp_env
            .next(context, self.gate, amp_env_params.clone());
        let sample: SampleFxP = self.amp.next(context, mixed_wave, amplitude);
        let pcm = sample.to_bits().saturating_mul(8);
        self.last_level = pcm.unsigned_abs();
        pcm
    }
}

fn sanitize_patch(mut patch: AudioPatch) -> AudioPatch {
    const MIN_ENV_MS: u16 = 5;
    const MAX_ENV_MS: u16 = 7_900;

    patch.attack_ms = patch.attack_ms.clamp(MIN_ENV_MS, MAX_ENV_MS);
    patch.release_ms = patch.release_ms.clamp(MIN_ENV_MS, MAX_ENV_MS);
    let total_mix = patch.triangle_mix as u16 + patch.square_mix as u16;
    if total_mix > u8::MAX as u16 {
        patch.triangle_mix =
            ((patch.triangle_mix as u16 * u8::MAX as u16) / total_mix) as u8;
        patch.square_mix = u8::MAX - patch.triangle_mix;
    }
    patch
}

fn env_params_for_patch(patch: AudioPatch) -> EnvParams<i16> {
    let mut params = EnvParams::<i16>::default();
    params.attack = EnvParamFxP::from_num(patch.attack_ms as f32 / 1_000.0);
    params.release = EnvParamFxP::from_num(patch.release_ms as f32 / 1_000.0);
    params
}
