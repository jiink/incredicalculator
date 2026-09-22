mod types;
pub use types::{HersheyFont, HersheyGlyph, HersheyVertex, HERSHEY_LIFT};
mod font_futural;
pub use font_futural::FONT_FUTURAL;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontId {
    Futural,

    // Later:
    // Simplex,
    // Duplex,
}

#[inline]
pub const fn get_font(id: FontId) -> &'static HersheyFont {
    match id {
        FontId::Futural => &FONT_FUTURAL,
    }
}
