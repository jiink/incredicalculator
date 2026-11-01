use crate::app::{IcApp, InputContext};
use crate::audio_engine::{AudioEngine, AudioPatch, MAX_VOICES, NoteId};
use crate::graphics::{self, Vitmap};
use crate::input::IcKey;
use crate::platform::{CANVAS_WIDTH, IcPlatform, rgb8_hex};
use crate::text::{self, HAlign, TextStyle};
use glam::{IVec2, Vec2};
use rgb::RGB8;

const DIRECT_SOURCE_BASE: u32 = 0x534b_0000;
const ARP_SOURCE_BASE: u32 = 0x534b_0100;

/// Keys in playing order: 1 is the root, 0 is the tenth scale degree.
const NUMBER_KEYS: [IcKey; KEY_COUNT] = [
    IcKey::Num1,
    IcKey::Num2,
    IcKey::Num3,
    IcKey::Num4,
    IcKey::Num5,
    IcKey::Num6,
    IcKey::Num7,
    IcKey::Num8,
    IcKey::Num9,
    IcKey::Num0,
];
const KEY_COUNT: usize = 10;

const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

const MIN_OCTAVE: i8 = 1;
const MAX_OCTAVE: i8 = 7;
const DEFAULT_OCTAVE: i8 = 4;
const MIN_BPM: u16 = 40;
const MAX_BPM: u16 = 240;
const BPM_STEP: u16 = 5;
const DEFAULT_BPM: u16 = 120;
const MAX_ARP_OCTAVES: u8 = 3;
/// Portion of each arp step during which the note is gated on.
const ARP_GATE_PERCENT: u64 = 60;

const BG_COLOR: RGB8 = rgb8_hex(0x1d1b3a);
const TEXT_COLOR: RGB8 = rgb8_hex(0xffffff);
const DIM_TEXT_COLOR: RGB8 = rgb8_hex(0xa9a6d6);
const CELL_COLOR: RGB8 = rgb8_hex(0x34306a);
const ROOT_CELL_COLOR: RGB8 = rgb8_hex(0x4a3f8f);
const HELD_CELL_COLOR: RGB8 = rgb8_hex(0x7a5cff);
const SOUNDING_CELL_COLOR: RGB8 = rgb8_hex(0xffc94a);
const SOUNDING_TEXT_COLOR: RGB8 = rgb8_hex(0x1d1b3a);

struct Scale {
    name: &'static str,
    steps: &'static [u8],
}

const SCALES: [Scale; 9] = [
    Scale {
        name: "Major",
        steps: &[0, 2, 4, 5, 7, 9, 11],
    },
    Scale {
        name: "Minor",
        steps: &[0, 2, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "Major Pent",
        steps: &[0, 2, 4, 7, 9],
    },
    Scale {
        name: "Minor Pent",
        steps: &[0, 3, 5, 7, 10],
    },
    Scale {
        name: "Blues",
        steps: &[0, 3, 5, 6, 7, 10],
    },
    Scale {
        name: "Dorian",
        steps: &[0, 2, 3, 5, 7, 9, 10],
    },
    Scale {
        name: "Mixolydian",
        steps: &[0, 2, 4, 5, 7, 9, 10],
    },
    Scale {
        name: "Harm Minor",
        steps: &[0, 2, 3, 5, 7, 8, 11],
    },
    Scale {
        name: "Chromatic",
        steps: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    },
];

struct SoundPreset {
    name: &'static str,
    patch: AudioPatch,
}

const SOUND_PRESETS: [SoundPreset; 5] = [
    SoundPreset {
        name: "Soft",
        patch: AudioPatch {
            attack_ms: 20,
            release_ms: 600,
            triangle_mix: 204,
            square_mix: 0,
            vibrato_depth_cents: 5,
        },
    },
    SoundPreset {
        name: "Pluck",
        patch: AudioPatch {
            attack_ms: 0,
            release_ms: 250,
            triangle_mix: 170,
            square_mix: 34,
            vibrato_depth_cents: 0,
        },
    },
    SoundPreset {
        name: "Reed",
        patch: AudioPatch {
            attack_ms: 30,
            release_ms: 300,
            triangle_mix: 102,
            square_mix: 102,
            vibrato_depth_cents: 8,
        },
    },
    SoundPreset {
        name: "Chip",
        patch: AudioPatch {
            attack_ms: 0,
            release_ms: 80,
            triangle_mix: 0,
            square_mix: 204,
            vibrato_depth_cents: 0,
        },
    },
    SoundPreset {
        name: "Pad",
        patch: AudioPatch {
            attack_ms: 500,
            release_ms: 2_500,
            triangle_mix: 170,
            square_mix: 34,
            vibrato_depth_cents: 6,
        },
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArpMode {
    Off,
    Up,
    Down,
    UpDown,
    Played,
    Random,
}

impl ArpMode {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Up,
            Self::Up => Self::Down,
            Self::Down => Self::UpDown,
            Self::UpDown => Self::Played,
            Self::Played => Self::Random,
            Self::Random => Self::Off,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Up => "Up",
            Self::Down => "Down",
            Self::UpDown => "Up/Down",
            Self::Played => "As played",
            Self::Random => "Random",
        }
    }
}

#[derive(Clone, Copy)]
enum ArpRate {
    Quarter,
    Eighth,
    EighthTriplet,
    Sixteenth,
}

impl ArpRate {
    fn next(self) -> Self {
        match self {
            Self::Quarter => Self::Eighth,
            Self::Eighth => Self::EighthTriplet,
            Self::EighthTriplet => Self::Sixteenth,
            Self::Sixteenth => Self::Quarter,
        }
    }

    fn steps_per_beat(self) -> u64 {
        match self {
            Self::Quarter => 1,
            Self::Eighth => 2,
            Self::EighthTriplet => 3,
            Self::Sixteenth => 4,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Quarter => "1/4",
            Self::Eighth => "1/8",
            Self::EighthTriplet => "1/8T",
            Self::Sixteenth => "1/16",
        }
    }
}

#[derive(Clone, Copy)]
struct HeldNote {
    midi_note: u8,
    /// Press order, used by the "as played" arp mode.
    order: u32,
}

/// A keyboard locked to a scale, with an optional arpeggiator.
///
/// The number keys play scale degrees, so any combination sounds in key.
/// While the arp is on, held (or latched) notes feed the arp instead of
/// sounding directly.
pub struct ScaleKeys {
    root: u8,
    scale_idx: usize,
    octave: i8,
    sound_idx: usize,
    held: [Option<HeldNote>; KEY_COUNT],
    /// Notes kept after release while latch is on; a press after every key
    /// has been released starts a new latched set.
    latched: [Option<HeldNote>; KEY_COUNT],
    latch: bool,
    next_order: u32,
    arp_mode: ArpMode,
    arp_rate: ArpRate,
    arp_octaves: u8,
    bpm: u16,
    arp_step: u32,
    arp_next_step_ms: Option<u64>,
    arp_gate_off_ms: Option<u64>,
    /// Arp notes rotate between sources so release tails can overlap.
    arp_voice: usize,
    /// Key whose note the arp is currently gating, for the display.
    arp_sounding: Option<usize>,
    rng: u32,
}

impl ScaleKeys {
    pub fn new() -> ScaleKeys {
        Self {
            root: 0,
            scale_idx: 0,
            octave: DEFAULT_OCTAVE,
            sound_idx: 0,
            held: [None; KEY_COUNT],
            latched: [None; KEY_COUNT],
            latch: false,
            next_order: 0,
            arp_mode: ArpMode::Off,
            arp_rate: ArpRate::Eighth,
            arp_octaves: 1,
            bpm: DEFAULT_BPM,
            arp_step: 0,
            arp_next_step_ms: None,
            arp_gate_off_ms: None,
            arp_voice: 0,
            arp_sounding: None,
            rng: 1,
        }
    }

    fn scale(&self) -> &'static Scale {
        &SCALES[self.scale_idx]
    }

    fn midi_note_for(&self, key_idx: usize, sharp: bool) -> u8 {
        let steps = self.scale().steps;
        let octave = self.octave as i16 + 1 + (key_idx / steps.len()) as i16;
        let note =
            octave * 12 + self.root as i16 + steps[key_idx % steps.len()] as i16 + sharp as i16;
        note.clamp(0, 127) as u8
    }

    /// Notes that are currently sounding or feeding the arp.
    fn active_notes(&self) -> &[Option<HeldNote>; KEY_COUNT] {
        if self.latch {
            &self.latched
        } else {
            &self.held
        }
    }

    fn has_active_notes(&self) -> bool {
        self.active_notes().iter().any(Option::is_some)
    }

    fn toggle_latch(&mut self) {
        self.latch = !self.latch;
        self.latched = if self.latch {
            self.held
        } else {
            [None; KEY_COUNT]
        };
    }

    fn adjust_octave(&mut self, amount: i8) {
        self.octave = (self.octave + amount).clamp(MIN_OCTAVE, MAX_OCTAVE);
    }

    fn adjust_bpm(&mut self, increase: bool) {
        self.bpm = if increase {
            (self.bpm + BPM_STEP).min(MAX_BPM)
        } else {
            (self.bpm - BPM_STEP).max(MIN_BPM)
        };
    }

    fn cycle_scale(&mut self, forward: bool) {
        self.scale_idx = if forward {
            (self.scale_idx + 1) % SCALES.len()
        } else {
            (self.scale_idx + SCALES.len() - 1) % SCALES.len()
        };
    }

    fn cycle_sound(&mut self, forward: bool) {
        self.sound_idx = if forward {
            (self.sound_idx + 1) % SOUND_PRESETS.len()
        } else {
            (self.sound_idx + SOUND_PRESETS.len() - 1) % SOUND_PRESETS.len()
        };
    }

    fn update_held_keys(&mut self, ctx: &InputContext) {
        for (key_idx, key) in NUMBER_KEYS.iter().enumerate() {
            let down = ctx.is_down(*key);
            match (down, self.held[key_idx]) {
                (true, None) => {
                    // The pitch is fixed at press time so changing octave,
                    // scale, or shift doesn't retune a note mid-hold.
                    let note = HeldNote {
                        midi_note: self.midi_note_for(key_idx, ctx.is_shifted()),
                        order: self.next_order,
                    };
                    self.next_order = self.next_order.wrapping_add(1);
                    if self.latch {
                        if self.held.iter().all(Option::is_none) {
                            self.latched = [None; KEY_COUNT];
                        }
                        self.latched[key_idx] = Some(note);
                    }
                    self.held[key_idx] = Some(note);
                }
                (false, Some(_)) => self.held[key_idx] = None,
                _ => {}
            }
        }
    }

    fn arp_interval_ms(&self) -> u64 {
        60_000 / (self.bpm as u64 * self.arp_rate.steps_per_beat())
    }

    fn next_random(&mut self) -> u32 {
        // xorshift32
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    /// The key and pitch the arp plays on `step`, or `None` with no notes.
    fn arp_note(&mut self, step: u32) -> Option<(usize, u8)> {
        let mut pool = [(
            0usize,
            HeldNote {
                midi_note: 0,
                order: 0,
            },
        ); KEY_COUNT];
        let mut len = 0;
        for (key_idx, note) in self.active_notes().iter().enumerate() {
            if let Some(note) = note {
                pool[len] = (key_idx, *note);
                len += 1;
            }
        }
        if len == 0 {
            return None;
        }
        let pool = &mut pool[..len];
        if self.arp_mode == ArpMode::Played {
            pool.sort_unstable_by_key(|(_, note)| note.order);
        } else {
            pool.sort_unstable_by_key(|(_, note)| note.midi_note);
        }

        let total = len * self.arp_octaves as usize;
        let step = step as usize;
        let idx = match self.arp_mode {
            ArpMode::Off | ArpMode::Up | ArpMode::Played => step % total,
            ArpMode::Down => total - 1 - step % total,
            ArpMode::UpDown if total > 1 => {
                let cycle = 2 * total - 2;
                let pos = step % cycle;
                if pos < total { pos } else { cycle - pos }
            }
            ArpMode::UpDown => 0,
            ArpMode::Random => self.next_random() as usize % total,
        };
        let (key_idx, note) = pool[idx % len];
        let midi_note = (note.midi_note as usize + 12 * (idx / len)).min(127) as u8;
        Some((key_idx, midi_note))
    }

    fn arp_source(&self) -> NoteId {
        NoteId::new(ARP_SOURCE_BASE + self.arp_voice as u32)
    }

    fn stop_arp(&mut self, audio: &mut AudioEngine) {
        for voice in 0..MAX_VOICES {
            audio.set_note(NoteId::new(ARP_SOURCE_BASE + voice as u32), None);
        }
        self.arp_next_step_ms = None;
        self.arp_gate_off_ms = None;
        self.arp_sounding = None;
    }

    fn update_arp(&mut self, now: u64, audio: &mut AudioEngine) {
        if self.arp_mode == ArpMode::Off || !self.has_active_notes() {
            self.stop_arp(audio);
            return;
        }

        if self.arp_gate_off_ms.is_some_and(|off_ms| now >= off_ms) {
            audio.set_note(self.arp_source(), None);
            self.arp_gate_off_ms = None;
            self.arp_sounding = None;
        }

        let step_ms = match self.arp_next_step_ms {
            Some(step_ms) => step_ms,
            None => {
                // Start on the first note as soon as a key goes down.
                self.arp_step = 0;
                self.rng = (now as u32).wrapping_mul(0x9e37_79b9) | 1;
                now
            }
        };
        if now < step_ms {
            return;
        }

        let interval = self.arp_interval_ms();
        if let Some((key_idx, midi_note)) = self.arp_note(self.arp_step) {
            audio.set_note(self.arp_source(), None);
            self.arp_voice = (self.arp_voice + 1) % MAX_VOICES;
            audio.set_note(self.arp_source(), Some(midi_note));
            self.arp_sounding = Some(key_idx);
            self.arp_gate_off_ms = Some(now + interval * ARP_GATE_PERCENT / 100);
        }
        self.arp_step = self.arp_step.wrapping_add(1);
        // Keep steps on the tempo grid, but skip ahead rather than rushing
        // through missed steps after a stall.
        let next_step_ms = step_ms + interval;
        self.arp_next_step_ms = Some(if next_step_ms <= now {
            now + interval
        } else {
            next_step_ms
        });
    }

    fn draw(&self, platform: &mut dyn IcPlatform, ctx: &InputContext) {
        platform.clear(BG_COLOR);

        text::draw_text_f(
            platform,
            format_args!("{} {}", NOTE_NAMES[self.root as usize], self.scale().name),
            Vec2::new(8.0, 17.0),
            TextStyle::new(8.0, TEXT_COLOR),
        );
        text::draw_text_f(
            platform,
            format_args!(
                "Octave {}   Sound: {}",
                self.octave, SOUND_PRESETS[self.sound_idx].name
            ),
            Vec2::new(8.0, 44.0),
            TextStyle::new(5.0, DIM_TEXT_COLOR),
        );

        self.draw_keys(platform, ctx.is_shifted());

        text::draw_text_f(
            platform,
            format_args!(
                "Arp: {}{}",
                self.arp_mode.label(),
                if self.latch { "   Latch" } else { "" },
            ),
            Vec2::new(8.0, 150.0),
            TextStyle::new(6.0, TEXT_COLOR),
        );
        if self.arp_mode != ArpMode::Off {
            text::draw_text_f(
                platform,
                format_args!(
                    "{}   {} BPM   {} oct",
                    self.arp_rate.label(),
                    self.bpm,
                    self.arp_octaves,
                ),
                Vec2::new(8.0, 168.0),
                TextStyle::new(5.0, DIM_TEXT_COLOR),
            );
        }

        self.draw_function_bar(platform, ctx);
    }

    fn draw_keys(&self, platform: &mut dyn IcPlatform, shifted: bool) {
        const CELL_W: i32 = 29;
        const CELL_GAP: i32 = 2;
        const TOP: i32 = 56;
        const BOTTOM: i32 = 128;
        let left = (CANVAS_WIDTH as i32
            - (CELL_W * KEY_COUNT as i32 + CELL_GAP * (KEY_COUNT as i32 - 1)))
            / 2;
        let steps_len = self.scale().steps.len();
        let active = self.active_notes();

        for key_idx in 0..KEY_COUNT {
            let x = left + key_idx as i32 * (CELL_W + CELL_GAP);
            let active_note = active[key_idx].or(self.held[key_idx]);
            let sounding = match self.arp_mode {
                ArpMode::Off => active_note.is_some(),
                _ => self.arp_sounding == Some(key_idx),
            };
            let fill = if sounding {
                SOUNDING_CELL_COLOR
            } else if active_note.is_some() {
                HELD_CELL_COLOR
            } else if key_idx % steps_len == 0 {
                ROOT_CELL_COLOR
            } else {
                CELL_COLOR
            };
            let text_color = if sounding {
                SOUNDING_TEXT_COLOR
            } else {
                TEXT_COLOR
            };
            platform.draw_rectangle_rounded(
                IVec2::new(x, TOP),
                IVec2::new(x + CELL_W, BOTTOM),
                fill,
                0,
                Some(fill),
                4,
            );

            let center_x = (x + CELL_W / 2) as f32;
            text::draw_text_f(
                platform,
                format_args!("{}", (key_idx + 1) % KEY_COUNT),
                Vec2::new(center_x, TOP as f32 + 18.0),
                TextStyle::new(7.0, text_color).align(HAlign::Center),
            );
            // Idle keys preview what Shift would play.
            let midi_note = active_note
                .map(|note| note.midi_note)
                .unwrap_or_else(|| self.midi_note_for(key_idx, shifted));
            text::draw_text(
                platform,
                NOTE_NAMES[midi_note as usize % 12],
                Vec2::new(center_x, TOP as f32 + 42.0),
                TextStyle::new(4.5, text_color).align(HAlign::Center),
            );
            text::draw_text_f(
                platform,
                format_args!("{}", midi_note as i16 / 12 - 1),
                Vec2::new(center_x, TOP as f32 + 60.0),
                TextStyle::new(4.0, text_color).align(HAlign::Center),
            );
        }
    }

    /// Labels for F1-F6, following the held modifier.
    fn draw_function_bar(&self, platform: &mut dyn IcPlatform, ctx: &InputContext) {
        const CELL_W: i32 = 44;
        const CELL_GAP: i32 = 2;
        const LEFT: i32 = 6;
        // Stays above the shell's modifier icon in the bottom-right corner.
        const TOP: i32 = 182;
        const BOTTOM: i32 = 204;
        let labels = if ctx.is_super() {
            ["Snd+", "Snd-", "", "", "Vol", "Light"]
        } else if ctx.is_shifted() {
            ["Rate", "ArpOct", "Root-", "Root+", "Scl-", "Scl+"]
        } else {
            ["Arp", "Latch", "Oct-", "Oct+", "BPM-", "BPM+"]
        };
        let latch_lit = !ctx.is_super() && !ctx.is_shifted() && self.latch;

        for (idx, label) in labels.iter().enumerate() {
            if label.is_empty() {
                continue;
            }
            let x = LEFT + idx as i32 * (CELL_W + CELL_GAP);
            let lit = idx == 1 && latch_lit;
            let fill = if lit { HELD_CELL_COLOR } else { CELL_COLOR };
            platform.draw_rectangle_rounded(
                IVec2::new(x, TOP),
                IVec2::new(x + CELL_W, BOTTOM),
                fill,
                0,
                Some(fill),
                4,
            );
            text::draw_text(
                platform,
                label,
                Vec2::new((x + CELL_W / 2) as f32, TOP as f32 + 15.0),
                TextStyle::new(4.0, TEXT_COLOR).align(HAlign::Center),
            );
        }
    }
}

impl IcApp for ScaleKeys {
    fn requires_realtime_updates(&self) -> bool {
        self.arp_mode != ArpMode::Off && self.has_active_notes()
    }

    fn on_enter(&mut self) {
        // The shell releases our notes when leaving, so start from silence.
        self.held = [None; KEY_COUNT];
        self.latched = [None; KEY_COUNT];
        self.arp_next_step_ms = None;
        self.arp_gate_off_ms = None;
        self.arp_sounding = None;
    }

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        if ctx.is_super() {
            // Super+F5/F6 belong to the shell's volume and brightness.
            match key {
                IcKey::Func1 => self.cycle_sound(true),
                IcKey::Func2 => self.cycle_sound(false),
                _ => {}
            }
        } else if ctx.is_shifted() {
            match key {
                IcKey::Func1 => self.arp_rate = self.arp_rate.next(),
                IcKey::Func2 => self.arp_octaves = self.arp_octaves % MAX_ARP_OCTAVES + 1,
                IcKey::Func3 => self.root = (self.root + 11) % 12,
                IcKey::Func4 => self.root = (self.root + 1) % 12,
                IcKey::Func5 => self.cycle_scale(false),
                IcKey::Func6 => self.cycle_scale(true),
                _ => {}
            }
        } else {
            match key {
                IcKey::Func1 => self.arp_mode = self.arp_mode.next(),
                IcKey::Func2 => self.toggle_latch(),
                IcKey::Func3 => self.adjust_octave(-1),
                IcKey::Func4 => self.adjust_octave(1),
                IcKey::Func5 => self.adjust_bpm(false),
                IcKey::Func6 => self.adjust_bpm(true),
                _ => {}
            }
        }
    }

    fn update(
        &mut self,
        platform: &mut dyn IcPlatform,
        ctx: &InputContext,
        audio: &mut AudioEngine,
    ) {
        let now = platform.millis();
        audio.set_patch(SOUND_PRESETS[self.sound_idx].patch);
        self.update_held_keys(ctx);

        let arp_on = self.arp_mode != ArpMode::Off;
        let active = *self.active_notes();
        for (key_idx, note) in active.iter().enumerate() {
            audio.set_note(
                NoteId::new(DIRECT_SOURCE_BASE + key_idx as u32),
                note.filter(|_| !arp_on).map(|note| note.midi_note),
            );
        }
        self.update_arp(now, audio);

        self.draw(platform, ctx);
    }

    fn name(&self) -> &str {
        "Scale keys"
    }

    fn icon(&self) -> &'static Vitmap {
        &graphics::MUSIC_ICON
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0x7a5cff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_notes(keys: &mut ScaleKeys, notes: &[(usize, u8)]) {
        for (order, &(key_idx, midi_note)) in notes.iter().enumerate() {
            keys.held[key_idx] = Some(HeldNote {
                midi_note,
                order: order as u32,
            });
        }
    }

    fn arp_sequence(keys: &mut ScaleKeys, steps: u32) -> [u8; 8] {
        let mut out = [0; 8];
        for step in 0..steps {
            out[step as usize] = keys.arp_note(step).unwrap().1;
        }
        out
    }

    #[test]
    fn number_keys_follow_the_scale_across_octaves() {
        let mut keys = ScaleKeys::new();
        let notes: [u8; KEY_COUNT] = core::array::from_fn(|k| keys.midi_note_for(k, false));
        assert_eq!(notes, [60, 62, 64, 65, 67, 69, 71, 72, 74, 76]);

        keys.scale_idx = 3; // Minor Pent
        keys.root = 9; // A
        keys.octave = 3;
        assert_eq!(keys.midi_note_for(0, false), 57);
        assert_eq!(keys.midi_note_for(5, false), 69);
        assert_eq!(keys.midi_note_for(1, true), 61);
    }

    #[test]
    fn arp_modes_order_notes() {
        let mut keys = ScaleKeys::new();
        // Pressed high, low, middle.
        with_notes(&mut keys, &[(4, 67), (0, 60), (2, 64)]);

        keys.arp_mode = ArpMode::Up;
        assert_eq!(arp_sequence(&mut keys, 4)[..4], [60, 64, 67, 60]);
        keys.arp_mode = ArpMode::Down;
        assert_eq!(arp_sequence(&mut keys, 4)[..4], [67, 64, 60, 67]);
        keys.arp_mode = ArpMode::UpDown;
        assert_eq!(arp_sequence(&mut keys, 5)[..5], [60, 64, 67, 64, 60]);
        keys.arp_mode = ArpMode::Played;
        assert_eq!(arp_sequence(&mut keys, 4)[..4], [67, 60, 64, 67]);

        keys.arp_mode = ArpMode::Up;
        keys.arp_octaves = 2;
        assert_eq!(
            arp_sequence(&mut keys, 7)[..7],
            [60, 64, 67, 72, 76, 79, 60]
        );
    }

    #[test]
    fn latch_keeps_notes_until_a_new_chord_starts() {
        let mut keys = ScaleKeys::new();
        keys.toggle_latch();
        with_notes(&mut keys, &[(0, 60), (2, 64)]);
        keys.latched = keys.held;
        keys.held = [None; KEY_COUNT];
        assert!(keys.has_active_notes());

        let mut states = [crate::input::KeyState::default(); IcKey::COUNT];
        states[IcKey::Num5 as usize].is_down = true;
        let ctx = InputContext {
            key_states: &states,
            key_mode: Default::default(),
            shift_engaged: false,
            shift_disengaged: false,
        };
        keys.update_held_keys(&ctx);
        let latched: usize = keys.latched.iter().filter(|note| note.is_some()).count();
        assert_eq!(latched, 1);
        assert_eq!(keys.latched[4].unwrap().midi_note, 67);
    }
}
