use super::*;
use crate::input::KeyState;
use num_traits::FromPrimitive;

fn shift_tap_frame(calculator: &mut Calculator, states: &mut [KeyState; IcKey::COUNT]) {
    for state in states.iter_mut() {
        state.just_pressed = state.is_down && !state.was_down;
        state.just_released = !state.is_down && state.was_down;
        state.was_down = state.is_down;
    }
    let ctx = InputContext {
        key_states: states,
        key_mode: KeyMode::Normal,
    };
    for i in 0..IcKey::COUNT {
        if states[i].just_pressed {
            calculator.on_key(IcKey::from_usize(i).unwrap(), &ctx);
        }
    }
    calculator.handle_shift_tap(&ctx);
}

#[test]
fn calculator_text_fitting_uses_glyph_widths() {
    let narrow = FittedText::new("iiiiiiiiiiii", 10.0, 100.0, 100.0, 0.0);
    let wide = FittedText::new("WWWWWWWWWWWW", 10.0, 100.0, 100.0, 0.0);
    assert!(narrow.scale > wide.scale);
    assert_eq!(FittedText::new("3(4)", 10.0, 312.0, 100.0, 0.0).scale, 10.0);
}

#[test]
fn calculator_text_fitting_stays_in_its_row() {
    for text in [
        "123456789012345678901234567890123456789012345678",
        "888888888888888888888888888888888888888888888888",
        "((((1+2)*3)/4)^5)",
        "Syntax error: unexpected function call",
        "-1.2345678901234567e-100",
        "gyp()[]",
        "1",
    ] {
        for (max_scale, width, height, center_y) in [
            (10.0, 312.0, 28.0, 162.0),
            (10.0, 288.0, 26.0, 194.0),
            (6.0, 298.0, 13.0, 116.5),
            (6.0, 284.0, 13.0, 136.5),
        ] {
            let layout = FittedText::new(text, max_scale, width, height, center_y);
            assert!(layout.scale > 0.0 && layout.scale <= max_scale);
            assert!(text_width(text, layout.scale, FontId::Futural) <= width + 0.001);
            let (min_y, max_y) = text_vertical_bounds(text, FontId::Futural).unwrap();
            let font_scale = layout.scale / fonts::get_font(FontId::Futural).units_per_em;
            let top = layout.y + min_y * font_scale;
            let bottom = layout.y + max_y * font_scale;
            assert!(top >= center_y - height * 0.5 - 0.001, "{text}");
            assert!(bottom <= center_y + height * 0.5 + 0.001, "{text}");
            assert!(((top + bottom) * 0.5 - center_y).abs() < 0.001);
            for cursor in 0..=text.len() {
                let x = text_to_pos(text, 4.0, layout.scale, cursor, FontId::Futural);
                assert!(x >= 4.0 && x <= 4.0 + width + 0.001);
            }
        }
    }
}

#[test]
fn calculator_cursor_fits_empty_and_single_stroke_text() {
    for text in ["", " ", "-", ".", "_"] {
        let layout = FittedText::new(text, 10.0, 312.0, 32.0, 162.0);
        assert!(layout.y.is_finite());
        assert!(layout.height >= 4.0 && layout.height <= 32.0);
    }
    let layout = FittedText::new("123", 10.0, 0.0, 0.0, 162.0);
    assert_eq!(layout.scale, 0.0);
    assert_eq!(layout.height, 0.0);
    assert_eq!(layout.y, 162.0);
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
fn shift_tap_does_not_insert_a_dot_in_latching_modes() {
    for key_mode in [KeyMode::Sticky, KeyMode::Toggle] {
        let mut calculator = Calculator::new(EngineMode::Scientific);
        let mut states = [KeyState::default(); IcKey::COUNT];
        states[IcKey::Shift as usize].just_pressed = true;
        states[IcKey::Shift as usize].is_down = true;
        calculator.handle_shift_tap(&InputContext {
            key_states: &states,
            key_mode,
        });
        states[IcKey::Shift as usize].just_pressed = false;
        states[IcKey::Shift as usize].just_released = true;
        calculator.handle_shift_tap(&InputContext {
            key_states: &states,
            key_mode,
        });
        assert_eq!(calculator.current_eq.as_str(), "");
    }
}

#[test]
fn shift_tap_inserts_once_on_release_and_updates_result() {
    let mut calculator = Calculator::new(EngineMode::Scientific);
    calculator.current_eq.insert_char(b'1');
    let mut states = [KeyState::default(); IcKey::COUNT];
    states[IcKey::Shift as usize].is_down = true;
    shift_tap_frame(&mut calculator, &mut states);
    shift_tap_frame(&mut calculator, &mut states);
    assert_eq!(calculator.current_eq.as_str(), "1");
    states[IcKey::Shift as usize].is_down = false;
    shift_tap_frame(&mut calculator, &mut states);
    shift_tap_frame(&mut calculator, &mut states);
    assert_eq!(calculator.current_eq.as_str(), "1.");
    assert_eq!(
        &calculator.current_result[..calculator.current_result_len],
        b"1"
    );
}

#[test]
fn shift_tap_is_cancelled_by_every_other_key() {
    for i in 0..IcKey::COUNT {
        if i == IcKey::Shift as usize {
            continue;
        }
        let mut calculator = Calculator::new(EngineMode::Scientific);
        let mut states = [KeyState::default(); IcKey::COUNT];
        states[IcKey::Shift as usize].is_down = true;
        shift_tap_frame(&mut calculator, &mut states);
        states[i].is_down = true;
        shift_tap_frame(&mut calculator, &mut states);
        states[i].is_down = false;
        shift_tap_frame(&mut calculator, &mut states);
        let len = calculator.current_eq.len;
        states[IcKey::Shift as usize].is_down = false;
        shift_tap_frame(&mut calculator, &mut states);
        assert_eq!(calculator.current_eq.len, len, "interrupting key {i}");
    }
}

#[test]
fn shift_tap_rejects_overlapping_keys_including_release_frame() {
    for key_first in [false, true] {
        let mut calculator = Calculator::new(EngineMode::Scientific);
        let mut states = [KeyState::default(); IcKey::COUNT];
        if key_first {
            states[IcKey::Num1 as usize].is_down = true;
            shift_tap_frame(&mut calculator, &mut states);
        }
        states[IcKey::Shift as usize].is_down = true;
        shift_tap_frame(&mut calculator, &mut states);
        states[IcKey::Shift as usize].is_down = false;
        states[IcKey::Num1 as usize].is_down = true;
        shift_tap_frame(&mut calculator, &mut states);
        assert!(!calculator.current_eq.as_str().contains('.'));
    }
}

#[test]
fn shift_tap_does_not_affect_programmer_or_survive_reentry() {
    for mode in [EngineMode::Programmer, EngineMode::Scientific] {
        let mut calculator = Calculator::new(mode);
        let mut states = [KeyState::default(); IcKey::COUNT];
        states[IcKey::Shift as usize].is_down = true;
        shift_tap_frame(&mut calculator, &mut states);
        if mode == EngineMode::Scientific {
            calculator.on_enter();
        }
        states[IcKey::Shift as usize].is_down = false;
        shift_tap_frame(&mut calculator, &mut states);
        assert_eq!(calculator.current_eq.as_str(), "");
    }
}
