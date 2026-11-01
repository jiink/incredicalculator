use super::music::{
    self, CELL_COLOR, DEFAULT_OCTAVE, DIM_TEXT_COLOR, HELD_CELL_COLOR, KEY_COUNT, KEYPAD_CELL_W,
    MAX_OCTAVE, MIN_OCTAVE, NOTE_NAMES, NUMBER_KEYS, PANEL_LEFT, ROOT_CELL_COLOR, SOUND_PRESETS,
    SOUNDING_CELL_COLOR, SOUNDING_TEXT_COLOR, TEXT_COLOR,
};
use crate::app::{IcApp, InputContext};
use crate::audio_engine::{AudioEngine, NoteId};
use crate::graphics::{self, Vitmap};
use crate::input::IcKey;
use crate::platform::{IcPlatform, rgb8_hex};
use crate::text::{self, HAlign, TextStyle};
use core::fmt;
use glam::Vec2;
use rgb::RGB8;

/// Room between the panel's left edge and the function column.
const PANEL_WIDTH: f32 = 98.0;
const TONE_SOURCE_BASE: u32 = 0x4350_0000;
const TONE_COUNT: usize = 3;
/// Starts in the Pad sound, which suits held chords.
const DEFAULT_SOUND_IDX: usize = 4;
/// Key 1 plays the key's home chord.
const TONIC_KEY: usize = 1;
/// The seventh most chords take when Shift has changed their quality.
const MINOR_SEVENTH: u8 = 10;

const STRUM_SPEEDS: [StrumSpeed; 3] = [
    StrumSpeed {
        name: "Fast",
        gap_ms: 15,
    },
    StrumSpeed {
        name: "Med",
        gap_ms: 35,
    },
    StrumSpeed {
        name: "Slow",
        gap_ms: 70,
    },
];

/// Chords for keys 0-9 in a major key. 1-7 are the diatonic chords; 8, 9
/// and 0 are common borrowed chords (V of V, flat seven, minor four).
const MAJOR_KEY_CHORDS: [ChordSlot; KEY_COUNT] = [
    ChordSlot::new("iv", 5, Quality::Minor, 10),
    ChordSlot::new("I", 0, Quality::Major, 11),
    ChordSlot::new("ii", 2, Quality::Minor, 10),
    ChordSlot::new("iii", 4, Quality::Minor, 10),
    ChordSlot::new("IV", 5, Quality::Major, 11),
    ChordSlot::new("V", 7, Quality::Major, 10),
    ChordSlot::new("vi", 9, Quality::Minor, 10),
    ChordSlot::new("viio", 11, Quality::Diminished, 10),
    ChordSlot::new("II", 2, Quality::Major, 10),
    ChordSlot::new("bVII", 10, Quality::Major, 11),
];

/// Chords for keys 0-9 in a minor key. 1-7 are the natural minor chords;
/// 8, 9 and 0 are the major five, the Dorian four and the Neapolitan.
const MINOR_KEY_CHORDS: [ChordSlot; KEY_COUNT] = [
    ChordSlot::new("bII", 1, Quality::Major, 11),
    ChordSlot::new("i", 0, Quality::Minor, 10),
    ChordSlot::new("iio", 2, Quality::Diminished, 10),
    ChordSlot::new("III", 3, Quality::Major, 11),
    ChordSlot::new("iv", 5, Quality::Minor, 10),
    ChordSlot::new("v", 7, Quality::Minor, 10),
    ChordSlot::new("VI", 8, Quality::Major, 11),
    ChordSlot::new("VII", 10, Quality::Major, 10),
    ChordSlot::new("V", 7, Quality::Major, 10),
    ChordSlot::new("IV", 5, Quality::Major, 10),
];

struct StrumSpeed {
    name: &'static str,
    gap_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Quality {
    Major,
    Minor,
    Diminished,
}

impl Quality {
    fn flipped(self) -> Self {
        match self {
            Self::Major => Self::Minor,
            Self::Minor | Self::Diminished => Self::Major,
        }
    }
}

struct ChordSlot {
    numeral: &'static str,
    /// Semitones above the key's root.
    offset: u8,
    quality: Quality,
    /// The seventh this chord takes in the key.
    seventh: u8,
}

impl ChordSlot {
    const fn new(numeral: &'static str, offset: u8, quality: Quality, seventh: u8) -> Self {
        Self {
            numeral,
            offset,
            quality,
            seventh,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Chord {
    /// Pitch class of the chord's root, 0 = C.
    root: u8,
    quality: Quality,
    /// Semitones from the root to a seventh, which replaces the fifth so the
    /// chord still fits in three voices.
    seventh: Option<u8>,
}

impl Chord {
    fn intervals(self) -> [u8; TONE_COUNT] {
        let third = match self.quality {
            Quality::Major => 4,
            Quality::Minor | Quality::Diminished => 3,
        };
        let fifth = match self.quality {
            Quality::Diminished => 6,
            Quality::Major | Quality::Minor => 7,
        };
        [0, third, self.seventh.unwrap_or(fifth)]
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match (self.quality, self.seventh) {
            (Quality::Major, None) => "",
            (Quality::Minor, None) => "m",
            (Quality::Diminished, None) => "dim",
            (Quality::Major, Some(11)) => "maj7",
            (Quality::Major, Some(_)) => "7",
            (Quality::Minor, Some(_)) => "m7",
            (Quality::Diminished, Some(_)) => "m7b5",
        };
        write!(f, "{}{}", NOTE_NAMES[self.root as usize], suffix)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Strum {
    Off,
    Up,
    Down,
}

impl Strum {
    fn next(self) -> Self {
        match self {
            Self::Off => Self::Up,
            Self::Up => Self::Down,
            Self::Down => Self::Off,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Up => "Up",
            Self::Down => "Down",
        }
    }
}

#[derive(Clone, Copy)]
struct HeldChord {
    chord: Chord,
    order: u32,
}

/// The chord currently sounding, whose tones may still be strumming in.
#[derive(Clone, Copy)]
struct Strike {
    key_idx: usize,
    order: u32,
    chord: Chord,
    /// Lowest to highest.
    notes: [u8; TONE_COUNT],
    start_ms: [u64; TONE_COUNT],
    started: [bool; TONE_COUNT],
}

/// Plays a whole chord from each number key, Omnichord style.
///
/// The newest held key sounds. Shift flips a chord between major and minor
/// and Super adds its seventh, both decided when the key goes down.
pub struct ChordPad {
    root: u8,
    minor_key: bool,
    octave: i8,
    sound_idx: usize,
    strum: Strum,
    strum_speed_idx: usize,
    /// Moves each chord to the inversion nearest the last one.
    smooth_voicing: bool,
    /// Keeps the last chord ringing after its key is released.
    hold: bool,
    held: [Option<HeldChord>; KEY_COUNT],
    next_order: u32,
    current: Option<Strike>,
    last_voicing: Option<[u8; TONE_COUNT]>,
}

impl ChordPad {
    pub fn new() -> ChordPad {
        Self {
            root: 0,
            minor_key: false,
            octave: DEFAULT_OCTAVE,
            sound_idx: DEFAULT_SOUND_IDX,
            strum: Strum::Off,
            strum_speed_idx: 1,
            smooth_voicing: true,
            hold: false,
            held: [None; KEY_COUNT],
            next_order: 0,
            current: None,
            last_voicing: None,
        }
    }

    fn chord_table(&self) -> &'static [ChordSlot; KEY_COUNT] {
        if self.minor_key {
            &MINOR_KEY_CHORDS
        } else {
            &MAJOR_KEY_CHORDS
        }
    }

    fn chord_for(&self, key_idx: usize, flip: bool, add_seventh: bool) -> Chord {
        let slot = &self.chord_table()[key_idx];
        let (quality, seventh) = if flip {
            (slot.quality.flipped(), MINOR_SEVENTH)
        } else {
            (slot.quality, slot.seventh)
        };
        Chord {
            root: (self.root + slot.offset) % 12,
            quality,
            seventh: add_seventh.then_some(seventh),
        }
    }

    /// Lowest note of the home chord in root position.
    fn anchor_note(&self) -> i16 {
        (self.octave as i16 + 1) * 12 + self.root as i16
    }

    fn voice(&self, chord: Chord) -> [u8; TONE_COUNT] {
        let anchor = self.anchor_note();
        // Keep roots within a fifth or so of the home chord, so V sits
        // below I rather than far above it.
        let mut root = anchor + (chord.root as i16 - self.root as i16).rem_euclid(12);
        if root - anchor > 6 {
            root -= 12;
        }
        let root_position = chord.intervals().map(|interval| root + interval as i16);

        let best = match self.last_voicing.filter(|_| self.smooth_voicing) {
            None => root_position,
            Some(previous) => {
                let mut best = root_position;
                let mut best_cost = i16::MAX;
                for inversion in 0..TONE_COUNT {
                    let mut notes = root_position;
                    for note in notes.iter_mut().take(inversion) {
                        *note += 12;
                    }
                    notes.sort_unstable();
                    for shift in [-12, 0, 12] {
                        let candidate = notes.map(|note| note + shift);
                        let movement: i16 = candidate
                            .iter()
                            .zip(previous.iter())
                            .map(|(&note, &prev)| (note - prev as i16).abs())
                            .sum();
                        // Favour small movements, but drift back toward the
                        // chosen octave so long progressions don't wander.
                        let cost = movement * 2 + (candidate[0] - anchor).abs();
                        if cost < best_cost {
                            best = candidate;
                            best_cost = cost;
                        }
                    }
                }
                best
            }
        };
        best.map(|note| note.clamp(0, 127) as u8)
    }

    fn update_held_keys(&mut self, ctx: &InputContext) {
        for (key_idx, key) in NUMBER_KEYS.iter().enumerate() {
            match (ctx.is_down(*key), self.held[key_idx]) {
                (true, None) => {
                    self.held[key_idx] = Some(HeldChord {
                        chord: self.chord_for(key_idx, ctx.is_shifted(), ctx.is_super()),
                        order: self.next_order,
                    });
                    self.next_order = self.next_order.wrapping_add(1);
                }
                (false, Some(_)) => self.held[key_idx] = None,
                _ => {}
            }
        }
    }

    fn strike(&mut self, key_idx: usize, held: HeldChord, now: u64, audio: &mut AudioEngine) {
        let notes = self.voice(held.chord);
        self.last_voicing = Some(notes);
        let gap_ms = match self.strum {
            Strum::Off => 0,
            Strum::Up | Strum::Down => STRUM_SPEEDS[self.strum_speed_idx].gap_ms,
        };
        let start_ms = core::array::from_fn(|tone| {
            let position = match self.strum {
                Strum::Down => TONE_COUNT - 1 - tone,
                Strum::Off | Strum::Up => tone,
            };
            now + gap_ms * position as u64
        });
        // Re-strike every tone, even ones shared with the last chord.
        for tone in 0..TONE_COUNT {
            audio.set_note(tone_source(tone), None);
        }
        self.current = Some(Strike {
            key_idx,
            order: held.order,
            chord: held.chord,
            notes,
            start_ms,
            started: [false; TONE_COUNT],
        });
    }

    fn start_due_tones(&mut self, now: u64, audio: &mut AudioEngine) {
        let Some(strike) = self.current.as_mut() else {
            return;
        };
        for tone in 0..TONE_COUNT {
            if !strike.started[tone] && now >= strike.start_ms[tone] {
                audio.set_note(tone_source(tone), Some(strike.notes[tone]));
                strike.started[tone] = true;
            }
        }
    }

    fn release(&mut self, audio: &mut AudioEngine) {
        if self.current.take().is_some() {
            for tone in 0..TONE_COUNT {
                audio.set_note(tone_source(tone), None);
            }
        }
    }

    fn draw(&self, platform: &mut dyn IcPlatform, ctx: &InputContext) {
        music::draw_header(
            platform,
            format_args!(
                "{} {}",
                NOTE_NAMES[self.root as usize],
                if self.minor_key { "Minor" } else { "Major" }
            ),
            format_args!(
                "Octave {}   Sound: {}",
                self.octave, SOUND_PRESETS[self.sound_idx].name
            ),
        );

        self.draw_keypad(platform, ctx);
        self.draw_panel(platform);

        let labels = music::function_labels(
            ctx,
            &["Strum", "Hold", "Oct-", "Oct+", "Voice", "Mode"],
            &["Speed", "", "Root-", "Root+", "", ""],
            ["Snd+", "Snd-", "", ""],
        );
        let hold_lit = self.hold && !ctx.is_super() && !ctx.is_shifted();
        music::draw_function_column(platform, &labels, hold_lit.then_some(1));
    }

    fn draw_keypad(&self, platform: &mut dyn IcPlatform, ctx: &InputContext) {
        let table = self.chord_table();
        let sounding_key = self.current.map(|strike| strike.key_idx);

        for key_idx in 0..KEY_COUNT {
            let origin = music::keypad_cell_origin(key_idx);
            let held = self.held[key_idx];
            let sounding = sounding_key == Some(key_idx);
            let fill = if sounding {
                SOUNDING_CELL_COLOR
            } else if held.is_some() {
                HELD_CELL_COLOR
            } else if key_idx == TONIC_KEY {
                ROOT_CELL_COLOR
            } else {
                CELL_COLOR
            };
            let (text_color, numeral_color) = if sounding {
                (SOUNDING_TEXT_COLOR, SOUNDING_TEXT_COLOR)
            } else {
                (TEXT_COLOR, DIM_TEXT_COLOR)
            };
            music::draw_keypad_cell(platform, origin, fill);

            // Idle keys preview what the held modifiers would play.
            let chord = match held {
                Some(held) => held.chord,
                None if sounding => self.current.unwrap().chord,
                None => self.chord_for(key_idx, ctx.is_shifted(), ctx.is_super()),
            };
            let center_x = (origin.x + KEYPAD_CELL_W / 2) as f32;
            let y = origin.y as f32;
            music::draw_text_fit(
                platform,
                format_args!("{}", chord),
                Vec2::new(center_x, y + 18.0),
                TextStyle::new(4.5, text_color).align(HAlign::Center),
                (KEYPAD_CELL_W - 6) as f32,
            );
            text::draw_text_f(
                platform,
                format_args!("{}  {}", key_idx, table[key_idx].numeral),
                Vec2::new(center_x, y + 34.0),
                TextStyle::new(3.5, numeral_color)
                    .thickness(1.0)
                    .align(HAlign::Center),
            );
        }
    }

    fn draw_panel(&self, platform: &mut dyn IcPlatform) {
        const LEFT: f32 = PANEL_LEFT;
        match self.current {
            Some(strike) => {
                music::draw_text_fit(
                    platform,
                    format_args!("{}", strike.chord),
                    Vec2::new(LEFT, 80.0),
                    TextStyle::new(8.0, SOUNDING_CELL_COLOR),
                    PANEL_WIDTH,
                );
                let [low, mid, high] = strike.notes;
                text::draw_text_f(
                    platform,
                    format_args!(
                        "{}{} {}{} {}{}",
                        NOTE_NAMES[low as usize % 12],
                        music::octave_of(low),
                        NOTE_NAMES[mid as usize % 12],
                        music::octave_of(mid),
                        NOTE_NAMES[high as usize % 12],
                        music::octave_of(high),
                    ),
                    Vec2::new(LEFT, 102.0),
                    TextStyle::new(4.0, DIM_TEXT_COLOR),
                );
            }
            None => text::draw_text(
                platform,
                "-",
                Vec2::new(LEFT, 80.0),
                TextStyle::new(8.0, DIM_TEXT_COLOR),
            ),
        }

        let style = TextStyle::new(4.5, TEXT_COLOR);
        text::draw_text_f(
            platform,
            format_args!("Strum: {}", self.strum.label()),
            Vec2::new(LEFT, 132.0),
            style,
        );
        if self.strum != Strum::Off {
            text::draw_text_f(
                platform,
                format_args!("Speed: {}", STRUM_SPEEDS[self.strum_speed_idx].name),
                Vec2::new(LEFT, 150.0),
                style,
            );
        }
        text::draw_text_f(
            platform,
            format_args!(
                "Voice: {}",
                if self.smooth_voicing {
                    "Smooth"
                } else {
                    "Root"
                }
            ),
            Vec2::new(LEFT, 170.0),
            style,
        );
        if self.hold {
            text::draw_text(
                platform,
                "Hold",
                Vec2::new(LEFT, 196.0),
                TextStyle::new(5.0, SOUNDING_CELL_COLOR),
            );
        }
    }
}

fn tone_source(tone: usize) -> NoteId {
    NoteId::new(TONE_SOURCE_BASE + tone as u32)
}

impl IcApp for ChordPad {
    fn requires_realtime_updates(&self) -> bool {
        // Only while a strum is still bringing in its tones.
        self.current
            .is_some_and(|strike| strike.started.iter().any(|started| !started))
    }

    fn on_enter(&mut self) {
        // The shell releases our notes when leaving, so start from silence.
        self.held = [None; KEY_COUNT];
        self.current = None;
        self.last_voicing = None;
    }

    fn on_key(&mut self, key: IcKey, ctx: &InputContext) {
        if ctx.is_super() {
            // Super+F5/F6 belong to the shell's volume and brightness.
            match key {
                IcKey::Func1 => {
                    self.sound_idx = music::cycle(self.sound_idx, SOUND_PRESETS.len(), true)
                }
                IcKey::Func2 => {
                    self.sound_idx = music::cycle(self.sound_idx, SOUND_PRESETS.len(), false)
                }
                _ => {}
            }
        } else if ctx.is_shifted() {
            match key {
                IcKey::Func1 => {
                    self.strum_speed_idx =
                        music::cycle(self.strum_speed_idx, STRUM_SPEEDS.len(), true)
                }
                IcKey::Func3 => self.root = (self.root + 11) % 12,
                IcKey::Func4 => self.root = (self.root + 1) % 12,
                _ => {}
            }
        } else {
            match key {
                IcKey::Func1 => self.strum = self.strum.next(),
                IcKey::Func2 => self.hold = !self.hold,
                IcKey::Func3 => self.octave = (self.octave - 1).max(MIN_OCTAVE),
                IcKey::Func4 => self.octave = (self.octave + 1).min(MAX_OCTAVE),
                IcKey::Func5 => self.smooth_voicing = !self.smooth_voicing,
                IcKey::Func6 => self.minor_key = !self.minor_key,
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

        // The newest held key wins; releasing it falls back to an older one.
        let newest = (0..KEY_COUNT)
            .filter_map(|key_idx| self.held[key_idx].map(|held| (key_idx, held)))
            .max_by_key(|(_, held)| held.order);
        match newest {
            Some((key_idx, held))
                if self.current.map(|strike| strike.order) != Some(held.order) =>
            {
                self.strike(key_idx, held, now, audio)
            }
            Some(_) => {}
            None if !self.hold => self.release(audio),
            None => {}
        }
        self.start_due_tones(now, audio);

        self.draw(platform, ctx);
    }

    fn name(&self) -> &str {
        "Chord pad"
    }

    fn icon(&self) -> &'static Vitmap {
        &graphics::MUSIC_ICON
    }

    fn color(&self) -> RGB8 {
        rgb8_hex(0xffc94a)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::string::ToString;

    #[test]
    fn modifiers_change_chord_quality_and_add_sevenths() {
        let pad = ChordPad::new();
        assert_eq!(pad.chord_for(1, false, false).to_string(), "C");
        assert_eq!(pad.chord_for(1, false, true).to_string(), "Cmaj7");
        assert_eq!(pad.chord_for(7, false, false).to_string(), "Bdim");
        assert_eq!(pad.chord_for(7, false, true).to_string(), "Bm7b5");
        assert_eq!(pad.chord_for(3, true, false).to_string(), "E");
        assert_eq!(pad.chord_for(3, true, true).to_string(), "E7");
        assert_eq!(pad.chord_for(5, false, true).to_string(), "G7");
        assert_eq!(pad.chord_for(0, false, false).to_string(), "Fm");

        let mut pad = ChordPad::new();
        pad.minor_key = true;
        pad.root = 9; // A
        assert_eq!(pad.chord_for(1, false, false).to_string(), "Am");
        assert_eq!(pad.chord_for(8, false, true).to_string(), "E7");
        assert_eq!(pad.chord_for(0, false, false).to_string(), "Bb");
    }

    #[test]
    fn root_voicing_keeps_chords_near_the_home_chord() {
        let mut pad = ChordPad::new();
        pad.smooth_voicing = false;
        assert_eq!(pad.voice(pad.chord_for(1, false, false)), [60, 64, 67]);
        assert_eq!(pad.voice(pad.chord_for(4, false, false)), [65, 69, 72]);
        // V drops below the home chord rather than climbing above it.
        assert_eq!(pad.voice(pad.chord_for(5, false, false)), [55, 59, 62]);
        assert_eq!(pad.voice(pad.chord_for(5, false, true)), [55, 59, 65]);
    }

    #[test]
    fn smooth_voicing_moves_to_the_nearest_inversion() {
        let mut pad = ChordPad::new();
        pad.last_voicing = Some([60, 64, 67]);
        // C to F keeps C and moves the others by a step.
        assert_eq!(pad.voice(pad.chord_for(4, false, false)), [60, 65, 69]);
        pad.last_voicing = Some([60, 65, 69]);
        assert_eq!(pad.voice(pad.chord_for(5, false, false)), [59, 62, 67]);
    }

    #[test]
    fn strums_stagger_the_tones() {
        let mut pad = ChordPad::new();
        let mut audio = AudioEngine::new();
        let held = HeldChord {
            chord: pad.chord_for(1, false, false),
            order: 0,
        };
        pad.strum = Strum::Up;
        pad.strike(1, held, 1_000, &mut audio);
        assert_eq!(pad.current.unwrap().start_ms, [1_000, 1_035, 1_070]);
        pad.start_due_tones(1_040, &mut audio);
        assert_eq!(pad.current.unwrap().started, [true, true, false]);
        assert!(pad.requires_realtime_updates());

        pad.strum = Strum::Down;
        pad.strike(1, held, 2_000, &mut audio);
        assert_eq!(pad.current.unwrap().start_ms, [2_070, 2_035, 2_000]);
    }
}
