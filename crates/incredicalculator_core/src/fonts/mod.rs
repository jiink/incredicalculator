mod types;
pub use types::{HERSHEY_LIFT, HersheyFont, HersheyGlyph, HersheyVertex};
mod font_classic;
mod font_futural;
pub use font_classic::FONT_CLASSIC;
pub use font_futural::FONT_FUTURAL;

use core::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontId {
    /// Whichever font the user picked in Settings (see [`system_font`]).
    System,
    Futural,
    Classic,
}

impl FontId {
    /// Concrete fonts the user can pick as the system font, in cycle order.
    pub const SELECTABLE: [FontId; 2] = [FontId::Futural, FontId::Classic];

    pub const fn label(self) -> &'static str {
        match self {
            FontId::System => "System",
            FontId::Futural => "Futural",
            FontId::Classic => "Classic",
        }
    }

    /// Next selectable font after this one, wrapping around.
    pub fn next(self) -> Self {
        let index = Self::SELECTABLE
            .iter()
            .position(|&f| f == self.resolve())
            .unwrap_or(0);
        Self::SELECTABLE[(index + 1) % Self::SELECTABLE.len()]
    }

    /// Index into [`FontId::SELECTABLE`], used for persisting the choice.
    pub fn to_index(self) -> u8 {
        Self::SELECTABLE
            .iter()
            .position(|&f| f == self.resolve())
            .unwrap_or(0) as u8
    }

    pub fn from_index(index: u8) -> Option<Self> {
        Self::SELECTABLE.get(index as usize).copied()
    }

    /// Replaces `System` with the font currently selected system-wide.
    #[inline]
    pub fn resolve(self) -> Self {
        match self {
            FontId::System => system_font(),
            other => other,
        }
    }
}

// Index into FontId::SELECTABLE. Plain load/store only (no read-modify-write),
// which is cheap and safe on every target.
static SYSTEM_FONT: AtomicU8 = AtomicU8::new(0);

/// The font that `FontId::System` currently renders with.
pub fn system_font() -> FontId {
    FontId::from_index(SYSTEM_FONT.load(Ordering::Relaxed)).unwrap_or(FontId::Futural)
}

/// Changes the font used by everything drawn with `FontId::System`.
pub fn set_system_font(font: FontId) {
    SYSTEM_FONT.store(font.to_index(), Ordering::Relaxed);
}

#[inline]
pub fn get_font(id: FontId) -> &'static HersheyFont {
    match id.resolve() {
        FontId::Classic => &FONT_CLASSIC,
        FontId::Futural | FontId::System => &FONT_FUTURAL,
    }
}
