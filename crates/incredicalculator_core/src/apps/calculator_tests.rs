use super::*;
use crate::input::{KeyMode, KeyState, ModifierKeys};
use num_traits::FromPrimitive;

struct ShiftTapHarness {
    states: [KeyState; IcKey::COUNT],
    modifiers: ModifierKeys,
    key_mode: KeyMode,
    shift_was_engaged: bool,
}

impl ShiftTapHarness {
    fn new(key_mode: KeyMode) -> Self {
        let mut modifiers = ModifierKeys::default();
        modifiers.set_mode(key_mode);
        Self {
            states: [KeyState::default(); IcKey::COUNT],
            modifiers,
            key_mode,
            shift_was_engaged: false,
        }
    }

    fn set(&mut self, key: IcKey, down: bool) {
        self.states[key as usize].is_down = down;
    }

    fn frame(&mut self, calculator: &mut Calculator) {
        for state in self.states.iter_mut() {
            state.just_pressed = state.is_down && !state.was_down;
            state.just_released = !state.is_down && state.was_down;
            state.was_down = state.is_down;
        }
        let effective = self.modifiers.apply(&self.states);
        let shift_engaged = effective[IcKey::Shift as usize].is_down;
        let ctx = InputContext {
            key_states: &effective,
            key_mode: self.key_mode,
            shift_engaged: shift_engaged && !self.shift_was_engaged,
            shift_disengaged: !shift_engaged && self.shift_was_engaged,
        };
        self.shift_was_engaged = shift_engaged;
        for i in 0..IcKey::COUNT {
            if effective[i].just_pressed {
                calculator.on_key(IcKey::from_usize(i).unwrap(), &ctx);
            }
        }
        calculator.handle_shift_tap(&ctx);
    }

    fn press(&mut self, calculator: &mut Calculator, key: IcKey) {
        self.set(key, true);
        self.frame(calculator);
    }

    fn release(&mut self, calculator: &mut Calculator, key: IcKey) {
        self.set(key, false);
        self.frame(calculator);
    }
}

#[test]
fn scientific_parentheses_imply_multiplication() {
    let engine = ScientificEngine::default();
    for (equation, expected) in [
        ("3(4)", "12"),
        ("3 (4)", "12"),
        ("(3)(4)", "12"),
        ("(3)4", "12"),
        ("(3) .5", "1.5"),
        ("2(3(4))", "24"),
        ("2(3+4)^2", "98"),
        ("-3(-4)", "12"),
        ("1.5(4)", "6"),
        ("1e-2(4)", "0.04"),
        ("2(sqrt(9))", "6"),
        ("sin(0)(4)", "0"),
    ] {
        assert_eq!(engine.evaluate(equation), expected, "{equation}");
    }
}

#[test]
fn scientific_preprocessing_preserves_explicit_syntax() {
    let engine = ScientificEngine::default();
    for equation in [
        "3*4",
        "2+3*4",
        "sqrt(9)",
        "sin (0)",
        "atan2(0,1)",
        "log10(100)",
        "pow(2,3)",
        "1e-2+4",
        "3+()",
        "(",
        "3 4",
        "unknown2(4)",
        "3@4",
        "",
    ] {
        assert_eq!(
            ScientificEngine::expand_implicit_multiplication(equation),
            equation
        );
        let expected = match exp_rs::interp(equation, None) {
            Ok(value) => format!("{}", value),
            Err(error) => error.to_string(),
        };
        assert_eq!(engine.evaluate(equation), expected, "{equation}");
    }
}

#[test]
fn shift_tap_inserts_once_on_release_and_updates_result() {
    let mut calculator = Calculator::new(EngineMode::Scientific);
    calculator.current_eq.insert_char(b'1');
    let mut h = ShiftTapHarness::new(KeyMode::Normal);
    h.press(&mut calculator, IcKey::Shift);
    h.frame(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "1");
    h.release(&mut calculator, IcKey::Shift);
    h.frame(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "1.");
    assert_eq!(
        &calculator.current_result[..calculator.current_result_len],
        b"1"
    );
}

#[test]
fn shift_tap_is_cancelled_by_every_other_key() {
    for key_mode in [KeyMode::Normal, KeyMode::Sticky, KeyMode::Toggle] {
        for i in 0..IcKey::COUNT {
            let key = IcKey::from_usize(i).unwrap();
            if key == IcKey::Shift {
                continue;
            }
            let mut calculator = Calculator::new(EngineMode::Scientific);
            let mut h = ShiftTapHarness::new(key_mode);
            h.press(&mut calculator, IcKey::Shift);
            h.press(&mut calculator, key);
            h.release(&mut calculator, key);
            let len = calculator.current_eq.len;
            h.release(&mut calculator, IcKey::Shift);
            if key_mode != KeyMode::Normal {
                h.press(&mut calculator, IcKey::Shift);
                h.release(&mut calculator, IcKey::Shift);
            }
            assert_eq!(
                calculator.current_eq.len, len,
                "{key_mode:?} interrupting key {i}"
            );
        }
    }
}

#[test]
fn shift_tap_while_key_already_held_inserts_a_dot() {
    let mut calculator = Calculator::new(EngineMode::Scientific);
    let mut h = ShiftTapHarness::new(KeyMode::Normal);
    h.press(&mut calculator, IcKey::Num1);
    h.press(&mut calculator, IcKey::Shift);
    h.release(&mut calculator, IcKey::Shift);
    h.release(&mut calculator, IcKey::Num1);
    assert_eq!(calculator.current_eq.as_str(), "1.");
}

#[test]
fn latching_modes_insert_a_dot_when_shift_is_engaged_then_disengaged() {
    for key_mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let mut calculator = Calculator::new(EngineMode::Scientific);
        let mut h = ShiftTapHarness::new(key_mode);
        h.press(&mut calculator, IcKey::Shift);
        h.release(&mut calculator, IcKey::Shift);
        assert_eq!(calculator.current_eq.as_str(), "", "{key_mode:?}");
        h.press(&mut calculator, IcKey::Shift);
        h.release(&mut calculator, IcKey::Shift);
        assert_eq!(calculator.current_eq.as_str(), ".", "{key_mode:?}");
    }
}

fn press_with(calculator: &mut Calculator, modifiers: &[IcKey], key: IcKey) {
    let mut states = [KeyState::default(); IcKey::COUNT];
    for &modifier in modifiers {
        states[modifier as usize].is_down = true;
    }
    let ctx = InputContext {
        key_states: &states,
        key_mode: KeyMode::Normal,
        shift_engaged: false,
        shift_disengaged: false,
    };
    calculator.on_key(key, &ctx);
}

fn undo(calculator: &mut Calculator) {
    press_with(calculator, &[IcKey::Super], IcKey::Num3);
}

#[test]
fn super_num3_undoes_one_key_press_at_a_time() {
    let mut calculator = Calculator::new(EngineMode::Scientific);
    press_with(&mut calculator, &[], IcKey::Num1);
    press_with(&mut calculator, &[], IcKey::Num2);
    press_with(&mut calculator, &[IcKey::Shift], IcKey::Num0);
    assert_eq!(calculator.current_eq.as_str(), "12sin(");

    // cursor moves are not undo steps, but undo restores the cursor
    press_with(&mut calculator, &[IcKey::Super], IcKey::Num7);
    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "12");
    assert_eq!(calculator.current_eq.cursor, 2);
    assert_eq!(calculator.current_result_len, 2);

    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "1");
    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "");
    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "");
}

#[test]
fn undo_restores_cleared_and_submitted_equations() {
    let mut calculator = Calculator::new(EngineMode::Programmer);
    press_with(&mut calculator, &[], IcKey::Num4);
    press_with(&mut calculator, &[IcKey::Super], IcKey::Num9);
    assert_eq!(calculator.current_eq.as_str(), "");
    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "4");

    press_with(&mut calculator, &[], IcKey::Func6);
    assert_eq!(calculator.current_eq.as_str(), "");
    assert_eq!(calculator.eq_history_len, 1);
    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "4");
    assert_eq!(calculator.eq_history_len, 1);
}

#[test]
fn shift_tap_dot_is_an_undo_step() {
    let mut calculator = Calculator::new(EngineMode::Scientific);
    let mut h = ShiftTapHarness::new(KeyMode::Normal);
    h.press(&mut calculator, IcKey::Num1);
    h.release(&mut calculator, IcKey::Num1);
    h.press(&mut calculator, IcKey::Shift);
    h.release(&mut calculator, IcKey::Shift);
    assert_eq!(calculator.current_eq.as_str(), "1.");

    undo(&mut calculator);
    assert_eq!(calculator.current_eq.as_str(), "1");
}
