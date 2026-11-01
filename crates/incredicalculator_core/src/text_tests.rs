use super::*;

const SCALE: f32 = 6.0;

#[test]
fn text_width_uses_widest_line() {
    let wide = text_width("MMMM", SCALE, fonts::FontId::Futural);
    let narrow = text_width("1", SCALE, fonts::FontId::Futural);
    assert!(wide > narrow);
    assert_eq!(text_width("MMMM\n1", SCALE, fonts::FontId::Futural), wide);
    assert_eq!(text_width("1\nMMMM", SCALE, fonts::FontId::Futural), wide);
    assert_eq!(text_width("1\r", SCALE, fonts::FontId::Futural), narrow);
}

