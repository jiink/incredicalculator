use super::*;
use crate::platform::rgb8_hex;
use alloc::vec::Vec;

#[derive(Default)]
struct LinePlatform {
    lines: Vec<(Vec2, Vec2)>,
}

impl IcPlatform for LinePlatform {
    fn draw_line(&mut self, start: Vec2, end: Vec2, _: RGB8, _: u32) {
        self.lines.push((start, end));
    }
    fn draw_polygon(&mut self, _: &[Vec2], _: RGB8) {}
    fn draw_rectangle(&mut self, _: glam::IVec2, _: glam::IVec2, _: RGB8, _: u32, _: Option<RGB8>) {
    }
    fn draw_rectangle_rounded(
        &mut self,
        _: glam::IVec2,
        _: glam::IVec2,
        _: RGB8,
        _: u32,
        _: Option<RGB8>,
        _: u32,
    ) {
    }
    fn draw_triangle(
        &mut self,
        _: glam::IVec2,
        _: glam::IVec2,
        _: glam::IVec2,
        _: RGB8,
        _: u32,
        _: Option<RGB8>,
    ) {
    }
    fn clear(&mut self, _: RGB8) {}
    fn log(&mut self, _: fmt::Arguments) {}
    fn millis(&self) -> u64 {
        0
    }
    fn get_battery_soc(&self) -> Option<i32> {
        None
    }
    fn get_brightness(&self) -> u8 {
        128
    }
    fn set_brightness(&mut self, _: u8) {}
    fn get_volume(&self) -> u8 {
        128
    }
    fn set_volume(&mut self, _: u8) {}
    fn get_screen_timeout_seconds(&self) -> u32 {
        60
    }
    fn set_screen_timeout_seconds(&mut self, _: u32) {}
}

const SCALE: f32 = 6.0;

fn style(align: HAlign) -> TextStyle {
    TextStyle::new(SCALE, rgb8_hex(0)).align(align)
}

fn draw(text: &str, x: f32, align: HAlign) -> Vec<(Vec2, Vec2)> {
    let mut platform = LinePlatform::default();
    draw_text(&mut platform, text, Vec2::new(x, 20.0), style(align));
    platform.lines
}

fn assert_same(a: &[(Vec2, Vec2)], b: &[(Vec2, Vec2)]) {
    assert_eq!(a.len(), b.len());
    for (sa, sb) in a.iter().zip(b) {
        assert!(sa.0.abs_diff_eq(sb.0, 1e-4) && sa.1.abs_diff_eq(sb.1, 1e-4));
    }
}

#[test]
fn text_width_uses_widest_line() {
    let wide = text_width("MMMM", SCALE, fonts::FontId::Futural);
    let narrow = text_width("1", SCALE, fonts::FontId::Futural);
    assert!(wide > narrow);
    assert_eq!(text_width("MMMM\n1", SCALE, fonts::FontId::Futural), wide);
    assert_eq!(text_width("1\nMMMM", SCALE, fonts::FontId::Futural), wide);
    assert_eq!(text_width("1\r", SCALE, fonts::FontId::Futural), narrow);
}

#[test]
fn center_and_right_shift_by_line_width() {
    let text = "12+3";
    let width = text_width(text, SCALE, fonts::FontId::Futural);

    let center = draw(text, 100.0, HAlign::Center);
    assert_same(&center, &draw(text, 100.0 - width * 0.5, HAlign::Left));

    let right = draw(text, 100.0, HAlign::Right);
    assert_same(&right, &draw(text, 100.0 - width, HAlign::Left));
}

#[test]
fn alignment_applies_per_line() {
    let first = "MMMM";
    let second = "1";
    let first_width = text_width(first, SCALE, fonts::FontId::Futural);
    let second_width = text_width(second, SCALE, fonts::FontId::Futural);

    let mut expected = draw(first, 100.0 - first_width, HAlign::Left);
    expected.extend(draw("\n1", 100.0 - second_width, HAlign::Left));

    let text = alloc::format!("{first}\n{second}");
    assert_same(&draw(&text, 100.0, HAlign::Right), &expected);
}

#[test]
fn text_height_matches_drawn_strokes() {
    for text in ["Shift+Super", "ag\nMM", "1\n\n2"] {
        let lines = draw(text, 0.0, HAlign::Left);
        let (min_y, max_y) = lines
            .iter()
            .flat_map(|(a, b)| [a.y, b.y])
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), y| {
                (lo.min(y), hi.max(y))
            });
        let (top, bottom) = text_ink_bounds(text, SCALE, fonts::FontId::Futural).unwrap();
        assert!((top + 20.0 - min_y).abs() < 1e-3, "{text:?}");
        assert!((bottom + 20.0 - max_y).abs() < 1e-3, "{text:?}");
        let height = text_height(text, SCALE, fonts::FontId::Futural);
        assert!((height - (max_y - min_y)).abs() < 1e-3, "{text:?}");
    }
    assert_eq!(text_height("", SCALE, fonts::FontId::Futural), 0.0);
}
