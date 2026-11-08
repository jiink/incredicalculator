#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(usize)]
pub enum IcKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Backspace,
    Divide,
    Multiply,
    Minus,
    Plus,
    Equals,
    Shift,
    Super,
    _Max,
}

impl IcKey {
    pub const COUNT: usize = IcKey::_Max as usize;

    /// Every key, in enum order.
    pub fn all() -> impl Iterator<Item = IcKey> {
        use num_traits::FromPrimitive;
        (0..Self::COUNT).filter_map(IcKey::from_usize)
    }

    /// The name scripts use for the key: its primary legend ("0"-"9", "/",
    /// "*", "-", "+", "="), or "backspace", "shift" or "super".
    pub fn name(self) -> &'static str {
        KEYS.get(self as usize).map_or("", |key| key.name)
    }

    /// The key a script refers to by `name`, as returned by [`IcKey::name`].
    pub fn from_name(name: &str) -> Option<IcKey> {
        Self::all().find(|key| key.name() == name)
    }

    /// What is printed on the key, and what each legend means.
    pub fn legends(self) -> &'static KeyLegends {
        KEYS.get(self as usize)
            .map_or(&BLANK_KEY.legends, |key| &key.legends)
    }
}

/// One legend on a keycap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Legend {
    /// What the press means to apps (a script's `on_key` `action`), or `None`
    /// when no app receives it: the shell keeps it, or the legend is blank.
    pub action: Option<&'static str>,
    /// Keycap text. May use symbols and emoji.
    pub label: &'static str,
    /// Keycap text for fonts with only Latin-1 characters.
    pub plain: &'static str,
}

/// The legends of one key: pressed alone, with Shift and with Super.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyLegends {
    pub primary: Legend,
    pub shift: Legend,
    pub super_: Legend,
}

impl KeyLegends {
    /// The legend that applies with these modifiers held. Super + Shift
    /// toggles the menu, so they never reach an app together.
    pub fn with(&self, shift: bool, super_held: bool) -> &Legend {
        if super_held {
            &self.super_
        } else if shift {
            &self.shift
        } else {
            &self.primary
        }
    }
}

struct KeyInfo {
    name: &'static str,
    legends: KeyLegends,
}

const fn legend(action: &'static str, label: &'static str, plain: &'static str) -> Legend {
    Legend {
        action: Some(action),
        label,
        plain,
    }
}

/// A legend whose action is the text printed on the key.
const fn same(text: &'static str) -> Legend {
    legend(text, text, text)
}

/// A legend whose press the shell keeps for itself.
const fn shell(label: &'static str, plain: &'static str) -> Legend {
    Legend {
        action: None,
        label,
        plain,
    }
}

const BLANK: Legend = shell("", "");

const fn key(name: &'static str, primary: Legend, shift: Legend, super_: Legend) -> KeyInfo {
    KeyInfo {
        name,
        legends: KeyLegends {
            primary,
            shift,
            super_,
        },
    }
}

const BLANK_KEY: KeyInfo = key("", BLANK, BLANK, BLANK);

/// The keypad's legends, in `IcKey` order. Every platform's keycaps and the
/// script API follow this table.
const KEYS: [KeyInfo; IcKey::COUNT] = [
    key("0", same("0"), same("A"), same(".")),
    key("1", same("1"), same("B"), legend("end", "End", "End")),
    key("2", same("2"), same("C"), legend("down", "v", "v")),
    key("3", same("3"), same("D"), shell("🔆", "Brt")),
    key("4", same("4"), same("E"), legend("left", "<", "<")),
    key("5", same("5"), same("F"), legend("ok", "✔️", "Sel")),
    key("6", same("6"), same("~"), legend("right", ">", ">")),
    key("7", same("7"), same("("), legend("home", "Hm", "Hm")),
    key("8", same("8"), same(")"), legend("up", "^", "^")),
    key("9", same("9"), same("0x"), shell("🔊", "Vol")),
    key(
        "backspace",
        legend("backspace", "⌫", "Bk"),
        same("&"),
        legend("clear", "Clr", "Clr"),
    ),
    key("/", same("/"), same("|"), legend("undo", "↩", "Und")),
    key("*", same("*"), same("%"), same("F1")),
    key("-", same("-"), same("<<"), same("F2")),
    key("+", same("+"), same(">>"), same("F3")),
    key("=", legend("enter", "=", "="), same("^"), same("F4")),
    key("shift", shell("⇪", "Shft"), BLANK, BLANK),
    key("super", shell("Ⓢ", "§"), BLANK, BLANK),
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyMode {
    #[default]
    Normal,
    Sticky,
    Toggle,
}

impl KeyMode {
    pub fn next(self) -> Self {
        match self {
            Self::Normal => Self::Sticky,
            Self::Sticky => Self::Toggle,
            Self::Toggle => Self::Normal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Sticky => "Sticky",
            Self::Toggle => "Toggle",
        }
    }
}

#[derive(Default)]
struct ModifierLatch {
    active: bool,
    used: bool,
}

#[derive(Default)]
pub struct ModifierKeys {
    mode: KeyMode,
    shift: ModifierLatch,
    super_key: ModifierLatch,
}

impl ModifierKeys {
    pub fn clear_latches(&mut self) {
        self.shift = ModifierLatch::default();
        self.super_key = ModifierLatch::default();
    }

    pub fn set_mode(&mut self, mode: KeyMode) {
        if self.mode != mode {
            self.mode = mode;
            self.clear_latches();
        }
    }

    pub fn apply(&mut self, physical: &[KeyState; IcKey::COUNT]) -> [KeyState; IcKey::COUNT] {
        let mut effective = *physical;
        let key_pressed = physical[..IcKey::Shift as usize]
            .iter()
            .any(|state| state.just_pressed);
        let key_down = physical[..IcKey::Shift as usize]
            .iter()
            .any(|state| state.is_down);
        for (key, latch) in [
            (IcKey::Shift, &mut self.shift),
            (IcKey::Super, &mut self.super_key),
        ] {
            let state = &mut effective[key as usize];
            if self.mode == KeyMode::Normal {
                continue;
            }
            if self.mode == KeyMode::Sticky && latch.used && !key_down && !state.is_down {
                latch.active = false;
                latch.used = false;
            }
            if state.just_pressed {
                latch.active = !latch.active;
                latch.used = false;
            }
            if self.mode == KeyMode::Sticky {
                if key_pressed && (latch.active || state.is_down) {
                    latch.used = true;
                }
                state.is_down |= latch.active;
            } else {
                state.is_down = latch.active;
            }
            // Modifier edges remain physical; only the held state is logical.
        }
        effective
    }
}

#[derive(Clone, Copy)]
pub struct KeyState {
    pub is_down: bool,
    pub was_down: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        KeyState {
            is_down: false,
            was_down: false,
            just_pressed: false,
            just_released: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(
        modifiers: &mut ModifierKeys,
        states: &mut [KeyState; IcKey::COUNT],
        down: &[IcKey],
    ) -> [KeyState; IcKey::COUNT] {
        for (i, state) in states.iter_mut().enumerate() {
            state.is_down = down.iter().any(|key| *key as usize == i);
            state.just_pressed = state.is_down && !state.was_down;
            state.just_released = !state.is_down && state.was_down;
            state.was_down = state.is_down;
        }
        modifiers.apply(states)
    }

    #[test]
    fn normal_modifiers_follow_physical_keys() {
        let mut modifiers = ModifierKeys::default();
        let mut states = [KeyState::default(); IcKey::COUNT];
        for down in [&[IcKey::Shift, IcKey::Super][..], &[][..]] {
            let effective = frame(&mut modifiers, &mut states, down);
            for i in 0..IcKey::COUNT {
                assert_eq!(effective[i].is_down, states[i].is_down);
                assert_eq!(effective[i].just_pressed, states[i].just_pressed);
                assert_eq!(effective[i].just_released, states[i].just_released);
            }
        }
    }

    #[test]
    fn sticky_modifiers_apply_to_one_key_until_it_is_released() {
        for modifier in [IcKey::Shift, IcKey::Super] {
            let mut modifiers = ModifierKeys::default();
            modifiers.set_mode(KeyMode::Sticky);
            let mut states = [KeyState::default(); IcKey::COUNT];
            frame(&mut modifiers, &mut states, &[modifier]);
            let effective = frame(&mut modifiers, &mut states, &[]);
            assert!(effective[modifier as usize].is_down);
            assert!(effective[modifier as usize].just_released);
            for _ in 0..2 {
                let effective = frame(&mut modifiers, &mut states, &[IcKey::Num1]);
                assert!(effective[modifier as usize].is_down);
            }
            let effective = frame(&mut modifiers, &mut states, &[]);
            assert!(!effective[modifier as usize].is_down);
            let effective = frame(&mut modifiers, &mut states, &[IcKey::Num2]);
            assert!(!effective[modifier as usize].is_down);
        }
    }

    #[test]
    fn sticky_modifiers_support_held_chords_and_tap_to_cancel() {
        let mut modifiers = ModifierKeys::default();
        modifiers.set_mode(KeyMode::Sticky);
        let mut states = [KeyState::default(); IcKey::COUNT];
        frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        frame(&mut modifiers, &mut states, &[IcKey::Shift, IcKey::Num1]);
        let effective = frame(&mut modifiers, &mut states, &[IcKey::Num1]);
        assert!(effective[IcKey::Shift as usize].is_down);
        let effective = frame(&mut modifiers, &mut states, &[]);
        assert!(!effective[IcKey::Shift as usize].is_down);
        frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        frame(&mut modifiers, &mut states, &[]);
        frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        let effective = frame(&mut modifiers, &mut states, &[]);
        assert!(!effective[IcKey::Shift as usize].is_down);
    }

    #[test]
    fn sticky_chord_releasing_key_first_does_not_rearm_modifier() {
        let mut modifiers = ModifierKeys::default();
        modifiers.set_mode(KeyMode::Sticky);
        let mut states = [KeyState::default(); IcKey::COUNT];
        frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        frame(&mut modifiers, &mut states, &[IcKey::Shift, IcKey::Num1]);
        let effective = frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        assert!(effective[IcKey::Shift as usize].is_down);
        let effective = frame(&mut modifiers, &mut states, &[]);
        assert!(!effective[IcKey::Shift as usize].is_down);
    }

    #[test]
    fn toggle_modifiers_persist_until_pressed_again() {
        for modifier in [IcKey::Shift, IcKey::Super] {
            let mut modifiers = ModifierKeys::default();
            modifiers.set_mode(KeyMode::Toggle);
            let mut states = [KeyState::default(); IcKey::COUNT];
            frame(&mut modifiers, &mut states, &[modifier]);
            for down in [&[][..], &[IcKey::Num1][..], &[][..], &[IcKey::Num2][..]] {
                let effective = frame(&mut modifiers, &mut states, down);
                assert!(effective[modifier as usize].is_down);
            }
            let effective = frame(&mut modifiers, &mut states, &[modifier]);
            assert!(!effective[modifier as usize].is_down);
            let effective = frame(&mut modifiers, &mut states, &[]);
            assert!(!effective[modifier as usize].is_down);
        }
    }

    #[test]
    fn sticky_modifiers_can_be_combined_and_mode_change_clears_latches() {
        let mut modifiers = ModifierKeys::default();
        modifiers.set_mode(KeyMode::Sticky);
        let mut states = [KeyState::default(); IcKey::COUNT];
        frame(&mut modifiers, &mut states, &[IcKey::Shift]);
        frame(&mut modifiers, &mut states, &[]);
        frame(&mut modifiers, &mut states, &[IcKey::Super]);
        let effective = frame(&mut modifiers, &mut states, &[]);
        assert!(effective[IcKey::Shift as usize].is_down);
        assert!(effective[IcKey::Super as usize].is_down);
        modifiers.set_mode(KeyMode::Toggle);
        let effective = frame(&mut modifiers, &mut states, &[]);
        assert!(!effective[IcKey::Shift as usize].is_down);
        assert!(!effective[IcKey::Super as usize].is_down);
    }
}
