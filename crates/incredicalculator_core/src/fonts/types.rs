#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HersheyVertex {
    pub x: i16,
    pub y: i16,
}

pub const HERSHEY_LIFT: HersheyVertex = HersheyVertex {
    x: i16::MIN,
    y: i16::MIN,
};

#[derive(Clone, Copy, Debug)]
pub struct HersheyGlyph {
    pub left: i16,
    pub right: i16,
    pub vertices: &'static [HersheyVertex],
}

#[derive(Clone, Copy, Debug)]
pub struct HersheyFont {
    pub name: &'static str,
    pub first_char: u8,
    pub glyphs: &'static [HersheyGlyph],
    pub space_advance: i16,
    pub line_height: i16,
    pub missing_glyph: &'static HersheyGlyph,
    pub units_per_em: f32,
}

impl HersheyGlyph {
    #[inline]
    pub const fn advance(&self) -> i16 {
        self.right - self.left
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}
