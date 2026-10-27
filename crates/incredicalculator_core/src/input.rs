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
    Func1,
    Func2,
    Func3,
    Func4,
    Func5,
    Func6,
    Shift,
    Super,
    _Max,
}

impl IcKey {
    pub const COUNT: usize = IcKey::_Max as usize;
}

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
