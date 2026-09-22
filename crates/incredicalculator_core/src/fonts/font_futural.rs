use crate::fonts::{HersheyGlyph, HersheyVertex, HersheyFont};
use crate::fonts::types::HERSHEY_LIFT;

static FUTURAL_GLYPH_0_VERTICES: [HersheyVertex; 1] = [
    HERSHEY_LIFT,
];

pub const FUTURAL_GLYPH_0: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_0_VERTICES,
};

static FUTURAL_GLYPH_1_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 2 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 7 },
    HersheyVertex { x: -1, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: 1, y: 8 },
    HersheyVertex { x: 0, y: 7 },
];

pub const FUTURAL_GLYPH_1: HersheyGlyph = HersheyGlyph {
    left: -5,
    right: 5,
    vertices: &FUTURAL_GLYPH_1_VERTICES,
};

static FUTURAL_GLYPH_2_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -4, y: -12 },
    HersheyVertex { x: -4, y: -5 },
    HERSHEY_LIFT,
    HersheyVertex { x: 4, y: -12 },
    HersheyVertex { x: 4, y: -5 },
];

pub const FUTURAL_GLYPH_2: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_2_VERTICES,
};

static FUTURAL_GLYPH_3_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 1, y: -16 },
    HersheyVertex { x: -6, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -16 },
    HersheyVertex { x: 0, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -3 },
    HersheyVertex { x: 8, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: 3 },
    HersheyVertex { x: 7, y: 3 },
];

pub const FUTURAL_GLYPH_3: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 11,
    vertices: &FUTURAL_GLYPH_3_VERTICES,
};

static FUTURAL_GLYPH_4_VERTICES: [HersheyVertex; 26] = [
    HersheyVertex { x: -2, y: -16 },
    HersheyVertex { x: -2, y: 13 },
    HERSHEY_LIFT,
    HersheyVertex { x: 2, y: -16 },
    HersheyVertex { x: 2, y: 13 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -5, y: -11 },
    HersheyVertex { x: -7, y: -9 },
    HersheyVertex { x: -7, y: -7 },
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: -5, y: -4 },
    HersheyVertex { x: -3, y: -3 },
    HersheyVertex { x: 3, y: -1 },
    HersheyVertex { x: 5, y: 0 },
    HersheyVertex { x: 6, y: 1 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 7, y: 6 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -7, y: 6 },
];

pub const FUTURAL_GLYPH_4: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_4_VERTICES,
};

static FUTURAL_GLYPH_5_VERTICES: [HersheyVertex; 31] = [
    HersheyVertex { x: 9, y: -12 },
    HersheyVertex { x: -9, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -4, y: -12 },
    HersheyVertex { x: -2, y: -10 },
    HersheyVertex { x: -2, y: -8 },
    HersheyVertex { x: -3, y: -6 },
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: -7, y: -5 },
    HersheyVertex { x: -9, y: -7 },
    HersheyVertex { x: -9, y: -9 },
    HersheyVertex { x: -8, y: -11 },
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -4, y: -12 },
    HersheyVertex { x: -2, y: -11 },
    HersheyVertex { x: 1, y: -10 },
    HersheyVertex { x: 4, y: -10 },
    HersheyVertex { x: 7, y: -11 },
    HersheyVertex { x: 9, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: 5, y: 2 },
    HersheyVertex { x: 3, y: 3 },
    HersheyVertex { x: 2, y: 5 },
    HersheyVertex { x: 2, y: 7 },
    HersheyVertex { x: 4, y: 9 },
    HersheyVertex { x: 6, y: 9 },
    HersheyVertex { x: 8, y: 8 },
    HersheyVertex { x: 9, y: 6 },
    HersheyVertex { x: 9, y: 4 },
    HersheyVertex { x: 7, y: 2 },
    HersheyVertex { x: 5, y: 2 },
];

pub const FUTURAL_GLYPH_5: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_5_VERTICES,
};

static FUTURAL_GLYPH_6_VERTICES: [HersheyVertex; 34] = [
    HersheyVertex { x: 10, y: -3 },
    HersheyVertex { x: 10, y: -4 },
    HersheyVertex { x: 9, y: -5 },
    HersheyVertex { x: 8, y: -5 },
    HersheyVertex { x: 7, y: -4 },
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: 3 },
    HersheyVertex { x: 2, y: 6 },
    HersheyVertex { x: 0, y: 8 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -6, y: 9 },
    HersheyVertex { x: -8, y: 8 },
    HersheyVertex { x: -9, y: 7 },
    HersheyVertex { x: -10, y: 5 },
    HersheyVertex { x: -10, y: 3 },
    HersheyVertex { x: -9, y: 1 },
    HersheyVertex { x: -8, y: 0 },
    HersheyVertex { x: -1, y: -4 },
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: 1, y: -7 },
    HersheyVertex { x: 1, y: -9 },
    HersheyVertex { x: 0, y: -11 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -5, y: -9 },
    HersheyVertex { x: -5, y: -7 },
    HersheyVertex { x: -4, y: -4 },
    HersheyVertex { x: -2, y: -1 },
    HersheyVertex { x: 3, y: 6 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 7, y: 9 },
    HersheyVertex { x: 9, y: 9 },
    HersheyVertex { x: 10, y: 8 },
    HersheyVertex { x: 10, y: 7 },
];

pub const FUTURAL_GLYPH_6: HersheyGlyph = HersheyGlyph {
    left: -13,
    right: 13,
    vertices: &FUTURAL_GLYPH_6_VERTICES,
};

static FUTURAL_GLYPH_7_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: -1, y: -11 },
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 1, y: -11 },
    HersheyVertex { x: 1, y: -9 },
    HersheyVertex { x: 0, y: -7 },
    HersheyVertex { x: -1, y: -6 },
];

pub const FUTURAL_GLYPH_7: HersheyGlyph = HersheyGlyph {
    left: -5,
    right: 5,
    vertices: &FUTURAL_GLYPH_7_VERTICES,
};

static FUTURAL_GLYPH_8_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 4, y: -16 },
    HersheyVertex { x: 2, y: -14 },
    HersheyVertex { x: 0, y: -11 },
    HersheyVertex { x: -2, y: -7 },
    HersheyVertex { x: -3, y: -2 },
    HersheyVertex { x: -3, y: 2 },
    HersheyVertex { x: -2, y: 7 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 2, y: 14 },
    HersheyVertex { x: 4, y: 16 },
];

pub const FUTURAL_GLYPH_8: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_8_VERTICES,
};

static FUTURAL_GLYPH_9_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: -4, y: -16 },
    HersheyVertex { x: -2, y: -14 },
    HersheyVertex { x: 0, y: -11 },
    HersheyVertex { x: 2, y: -7 },
    HersheyVertex { x: 3, y: -2 },
    HersheyVertex { x: 3, y: 2 },
    HersheyVertex { x: 2, y: 7 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: -2, y: 14 },
    HersheyVertex { x: -4, y: 16 },
];

pub const FUTURAL_GLYPH_9: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_9_VERTICES,
};

static FUTURAL_GLYPH_10_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -6 },
    HersheyVertex { x: 0, y: 6 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: -3 },
    HersheyVertex { x: 5, y: 3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 5, y: -3 },
    HersheyVertex { x: -5, y: 3 },
];

pub const FUTURAL_GLYPH_10: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_10_VERTICES,
};

static FUTURAL_GLYPH_11_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -9 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -9, y: 0 },
    HersheyVertex { x: 9, y: 0 },
];

pub const FUTURAL_GLYPH_11: HersheyGlyph = HersheyGlyph {
    left: -13,
    right: 13,
    vertices: &FUTURAL_GLYPH_11_VERTICES,
};

static FUTURAL_GLYPH_12_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 0, y: 6 },
    HersheyVertex { x: -1, y: 5 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 1, y: 7 },
    HersheyVertex { x: -1, y: 9 },
];

pub const FUTURAL_GLYPH_12: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_12_VERTICES,
};

static FUTURAL_GLYPH_13_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: -9, y: 0 },
    HersheyVertex { x: 9, y: 0 },
];

pub const FUTURAL_GLYPH_13: HersheyGlyph = HersheyGlyph {
    left: -13,
    right: 13,
    vertices: &FUTURAL_GLYPH_13_VERTICES,
};

static FUTURAL_GLYPH_14_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: -1, y: 5 },
    HersheyVertex { x: 0, y: 6 },
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 0, y: 4 },
];

pub const FUTURAL_GLYPH_14: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_14_VERTICES,
};

static FUTURAL_GLYPH_15_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 9, y: -16 },
    HersheyVertex { x: -9, y: 16 },
];

pub const FUTURAL_GLYPH_15: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_15_VERTICES,
};

static FUTURAL_GLYPH_16_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -6, y: -8 },
    HersheyVertex { x: -7, y: -3 },
    HersheyVertex { x: -7, y: 0 },
    HersheyVertex { x: -6, y: 5 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 5 },
    HersheyVertex { x: 7, y: 0 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 6, y: -8 },
    HersheyVertex { x: 4, y: -11 },
    HersheyVertex { x: 1, y: -12 },
    HersheyVertex { x: -1, y: -12 },
];

pub const FUTURAL_GLYPH_16: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_16_VERTICES,
};

static FUTURAL_GLYPH_17_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: -4, y: -8 },
    HersheyVertex { x: -2, y: -9 },
    HersheyVertex { x: 1, y: -12 },
    HersheyVertex { x: 1, y: 9 },
];

pub const FUTURAL_GLYPH_17: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_17_VERTICES,
};

static FUTURAL_GLYPH_18_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: -6, y: -7 },
    HersheyVertex { x: -6, y: -8 },
    HersheyVertex { x: -5, y: -10 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 4, y: -11 },
    HersheyVertex { x: 5, y: -10 },
    HersheyVertex { x: 6, y: -8 },
    HersheyVertex { x: 6, y: -6 },
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 3, y: -1 },
    HersheyVertex { x: -7, y: 9 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_18: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_18_VERTICES,
};

static FUTURAL_GLYPH_19_VERTICES: [HersheyVertex; 15] = [
    HersheyVertex { x: -5, y: -12 },
    HersheyVertex { x: 6, y: -12 },
    HersheyVertex { x: 0, y: -4 },
    HersheyVertex { x: 3, y: -4 },
    HersheyVertex { x: 5, y: -3 },
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 7, y: 1 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -6, y: 7 },
    HersheyVertex { x: -7, y: 5 },
];

pub const FUTURAL_GLYPH_19: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_19_VERTICES,
};

static FUTURAL_GLYPH_20_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: -7, y: 2 },
    HersheyVertex { x: 8, y: 2 },
    HERSHEY_LIFT,
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: 3, y: 9 },
];

pub const FUTURAL_GLYPH_20: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_20_VERTICES,
};

static FUTURAL_GLYPH_21_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: 5, y: -12 },
    HersheyVertex { x: -5, y: -12 },
    HersheyVertex { x: -6, y: -3 },
    HersheyVertex { x: -5, y: -4 },
    HersheyVertex { x: -2, y: -5 },
    HersheyVertex { x: 1, y: -5 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 7, y: 1 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -6, y: 7 },
    HersheyVertex { x: -7, y: 5 },
];

pub const FUTURAL_GLYPH_21: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_21_VERTICES,
};

static FUTURAL_GLYPH_22_VERTICES: [HersheyVertex; 23] = [
    HersheyVertex { x: 6, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: -3, y: -11 },
    HersheyVertex { x: -5, y: -8 },
    HersheyVertex { x: -6, y: -3 },
    HersheyVertex { x: -6, y: 2 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 7, y: 2 },
    HersheyVertex { x: 6, y: -1 },
    HersheyVertex { x: 4, y: -3 },
    HersheyVertex { x: 1, y: -4 },
    HersheyVertex { x: 0, y: -4 },
    HersheyVertex { x: -3, y: -3 },
    HersheyVertex { x: -5, y: -1 },
    HersheyVertex { x: -6, y: 2 },
];

pub const FUTURAL_GLYPH_22: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_22_VERTICES,
};

static FUTURAL_GLYPH_23_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: -3, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: -12 },
];

pub const FUTURAL_GLYPH_23: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_23_VERTICES,
};

static FUTURAL_GLYPH_24_VERTICES: [HersheyVertex; 29] = [
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -5, y: -11 },
    HersheyVertex { x: -6, y: -9 },
    HersheyVertex { x: -6, y: -7 },
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: 1, y: -3 },
    HersheyVertex { x: 4, y: -2 },
    HersheyVertex { x: 6, y: 0 },
    HersheyVertex { x: 7, y: 2 },
    HersheyVertex { x: 7, y: 5 },
    HersheyVertex { x: 6, y: 7 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -6, y: 7 },
    HersheyVertex { x: -7, y: 5 },
    HersheyVertex { x: -7, y: 2 },
    HersheyVertex { x: -6, y: 0 },
    HersheyVertex { x: -4, y: -2 },
    HersheyVertex { x: -1, y: -3 },
    HersheyVertex { x: 3, y: -4 },
    HersheyVertex { x: 5, y: -5 },
    HersheyVertex { x: 6, y: -7 },
    HersheyVertex { x: 6, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -2, y: -12 },
];

pub const FUTURAL_GLYPH_24: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_24_VERTICES,
};

static FUTURAL_GLYPH_25_VERTICES: [HersheyVertex; 23] = [
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 5, y: -2 },
    HersheyVertex { x: 3, y: 0 },
    HersheyVertex { x: 0, y: 1 },
    HersheyVertex { x: -1, y: 1 },
    HersheyVertex { x: -4, y: 0 },
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: -7, y: -5 },
    HersheyVertex { x: -7, y: -6 },
    HersheyVertex { x: -6, y: -9 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 3, y: -11 },
    HersheyVertex { x: 5, y: -9 },
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 6, y: 0 },
    HersheyVertex { x: 5, y: 5 },
    HersheyVertex { x: 3, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -6, y: 6 },
];

pub const FUTURAL_GLYPH_25: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_25_VERTICES,
};

static FUTURAL_GLYPH_26_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: -1, y: -2 },
    HersheyVertex { x: 0, y: -1 },
    HersheyVertex { x: 1, y: -2 },
    HersheyVertex { x: 0, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: -1, y: 5 },
    HersheyVertex { x: 0, y: 6 },
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 0, y: 4 },
];

pub const FUTURAL_GLYPH_26: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_26_VERTICES,
};

static FUTURAL_GLYPH_27_VERTICES: [HersheyVertex; 13] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: -1, y: -2 },
    HersheyVertex { x: 0, y: -1 },
    HersheyVertex { x: 1, y: -2 },
    HersheyVertex { x: 0, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 0, y: 6 },
    HersheyVertex { x: -1, y: 5 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 1, y: 7 },
    HersheyVertex { x: -1, y: 9 },
];

pub const FUTURAL_GLYPH_27: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_27_VERTICES,
};

static FUTURAL_GLYPH_28_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 8, y: -9 },
    HersheyVertex { x: -8, y: 0 },
    HersheyVertex { x: 8, y: 9 },
];

pub const FUTURAL_GLYPH_28: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_28_VERTICES,
};

static FUTURAL_GLYPH_29_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -9, y: -3 },
    HersheyVertex { x: 9, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: -9, y: 3 },
    HersheyVertex { x: 9, y: 3 },
];

pub const FUTURAL_GLYPH_29: HersheyGlyph = HersheyGlyph {
    left: -13,
    right: 13,
    vertices: &FUTURAL_GLYPH_29_VERTICES,
};

static FUTURAL_GLYPH_30_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: -8, y: -9 },
    HersheyVertex { x: 8, y: 0 },
    HersheyVertex { x: -8, y: 9 },
];

pub const FUTURAL_GLYPH_30: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_30_VERTICES,
};

static FUTURAL_GLYPH_31_VERTICES: [HersheyVertex; 20] = [
    HersheyVertex { x: -6, y: -7 },
    HersheyVertex { x: -6, y: -8 },
    HersheyVertex { x: -5, y: -10 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 4, y: -11 },
    HersheyVertex { x: 5, y: -10 },
    HersheyVertex { x: 6, y: -8 },
    HersheyVertex { x: 6, y: -6 },
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 4, y: -3 },
    HersheyVertex { x: 0, y: -1 },
    HersheyVertex { x: 0, y: 2 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 7 },
    HersheyVertex { x: -1, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: 1, y: 8 },
    HersheyVertex { x: 0, y: 7 },
];

pub const FUTURAL_GLYPH_31: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_31_VERTICES,
};

static FUTURAL_GLYPH_32_VERTICES: [HersheyVertex; 55] = [
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 4, y: -6 },
    HersheyVertex { x: 2, y: -7 },
    HersheyVertex { x: -1, y: -7 },
    HersheyVertex { x: -3, y: -6 },
    HersheyVertex { x: -4, y: -5 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -5, y: 1 },
    HersheyVertex { x: -4, y: 3 },
    HersheyVertex { x: -2, y: 4 },
    HersheyVertex { x: 1, y: 4 },
    HersheyVertex { x: 3, y: 3 },
    HersheyVertex { x: 4, y: 1 },
    HERSHEY_LIFT,
    HersheyVertex { x: -1, y: -7 },
    HersheyVertex { x: -3, y: -5 },
    HersheyVertex { x: -4, y: -2 },
    HersheyVertex { x: -4, y: 1 },
    HersheyVertex { x: -3, y: 3 },
    HersheyVertex { x: -2, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 5, y: -7 },
    HersheyVertex { x: 4, y: 1 },
    HersheyVertex { x: 4, y: 3 },
    HersheyVertex { x: 6, y: 4 },
    HersheyVertex { x: 8, y: 4 },
    HersheyVertex { x: 10, y: 2 },
    HersheyVertex { x: 11, y: -1 },
    HersheyVertex { x: 11, y: -3 },
    HersheyVertex { x: 10, y: -6 },
    HersheyVertex { x: 9, y: -8 },
    HersheyVertex { x: 7, y: -10 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -6, y: -10 },
    HersheyVertex { x: -8, y: -8 },
    HersheyVertex { x: -9, y: -6 },
    HersheyVertex { x: -10, y: -3 },
    HersheyVertex { x: -10, y: 0 },
    HersheyVertex { x: -9, y: 3 },
    HersheyVertex { x: -8, y: 5 },
    HersheyVertex { x: -6, y: 7 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 7, y: 7 },
    HersheyVertex { x: 8, y: 6 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -7 },
    HersheyVertex { x: 5, y: 1 },
    HersheyVertex { x: 5, y: 3 },
    HersheyVertex { x: 6, y: 4 },
];

pub const FUTURAL_GLYPH_32: HersheyGlyph = HersheyGlyph {
    left: -13,
    right: 14,
    vertices: &FUTURAL_GLYPH_32_VERTICES,
};

static FUTURAL_GLYPH_33_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: -8, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 8, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: 2 },
    HersheyVertex { x: 5, y: 2 },
];

pub const FUTURAL_GLYPH_33: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_33_VERTICES,
};

static FUTURAL_GLYPH_34_VERTICES: [HersheyVertex; 23] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 6, y: -10 },
    HersheyVertex { x: 7, y: -8 },
    HersheyVertex { x: 7, y: -6 },
    HersheyVertex { x: 6, y: -4 },
    HersheyVertex { x: 5, y: -3 },
    HersheyVertex { x: 2, y: -2 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -2 },
    HersheyVertex { x: 2, y: -2 },
    HersheyVertex { x: 5, y: -1 },
    HersheyVertex { x: 6, y: 0 },
    HersheyVertex { x: 7, y: 2 },
    HersheyVertex { x: 7, y: 5 },
    HersheyVertex { x: 6, y: 7 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: -7, y: 9 },
];

pub const FUTURAL_GLYPH_34: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 10,
    vertices: &FUTURAL_GLYPH_34_VERTICES,
};

static FUTURAL_GLYPH_35_VERTICES: [HersheyVertex; 18] = [
    HersheyVertex { x: 8, y: -7 },
    HersheyVertex { x: 7, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: -3, y: -11 },
    HersheyVertex { x: -5, y: -9 },
    HersheyVertex { x: -6, y: -7 },
    HersheyVertex { x: -7, y: -4 },
    HersheyVertex { x: -7, y: 1 },
    HersheyVertex { x: -6, y: 4 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 3, y: 9 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 7, y: 6 },
    HersheyVertex { x: 8, y: 4 },
];

pub const FUTURAL_GLYPH_35: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 11,
    vertices: &FUTURAL_GLYPH_35_VERTICES,
};

static FUTURAL_GLYPH_36_VERTICES: [HersheyVertex; 15] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 3, y: -11 },
    HersheyVertex { x: 5, y: -9 },
    HersheyVertex { x: 6, y: -7 },
    HersheyVertex { x: 7, y: -4 },
    HersheyVertex { x: 7, y: 1 },
    HersheyVertex { x: 6, y: 4 },
    HersheyVertex { x: 5, y: 6 },
    HersheyVertex { x: 3, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: -7, y: 9 },
];

pub const FUTURAL_GLYPH_36: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 10,
    vertices: &FUTURAL_GLYPH_36_VERTICES,
};

static FUTURAL_GLYPH_37_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: 7, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: 2, y: -2 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: 9 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_37: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 9,
    vertices: &FUTURAL_GLYPH_37_VERTICES,
};

static FUTURAL_GLYPH_38_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: 7, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: 2, y: -2 },
];

pub const FUTURAL_GLYPH_38: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 8,
    vertices: &FUTURAL_GLYPH_38_VERTICES,
};

static FUTURAL_GLYPH_39_VERTICES: [HersheyVertex; 22] = [
    HersheyVertex { x: 8, y: -7 },
    HersheyVertex { x: 7, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: -3, y: -11 },
    HersheyVertex { x: -5, y: -9 },
    HersheyVertex { x: -6, y: -7 },
    HersheyVertex { x: -7, y: -4 },
    HersheyVertex { x: -7, y: 1 },
    HersheyVertex { x: -6, y: 4 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 3, y: 9 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 7, y: 6 },
    HersheyVertex { x: 8, y: 4 },
    HersheyVertex { x: 8, y: 1 },
    HERSHEY_LIFT,
    HersheyVertex { x: 3, y: 1 },
    HersheyVertex { x: 8, y: 1 },
];

pub const FUTURAL_GLYPH_39: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 11,
    vertices: &FUTURAL_GLYPH_39_VERTICES,
};

static FUTURAL_GLYPH_40_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: 7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -2 },
    HersheyVertex { x: 7, y: -2 },
];

pub const FUTURAL_GLYPH_40: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_40_VERTICES,
};

static FUTURAL_GLYPH_41_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 9 },
];

pub const FUTURAL_GLYPH_41: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_41_VERTICES,
};

static FUTURAL_GLYPH_42_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 4, y: -12 },
    HersheyVertex { x: 4, y: 4 },
    HersheyVertex { x: 3, y: 7 },
    HersheyVertex { x: 2, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -5, y: 7 },
    HersheyVertex { x: -6, y: 4 },
    HersheyVertex { x: -6, y: 2 },
];

pub const FUTURAL_GLYPH_42: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_42_VERTICES,
};

static FUTURAL_GLYPH_43_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: -7, y: 2 },
    HERSHEY_LIFT,
    HersheyVertex { x: -2, y: -3 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_43: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 10,
    vertices: &FUTURAL_GLYPH_43_VERTICES,
};

static FUTURAL_GLYPH_44_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: 9 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_44: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 7,
    vertices: &FUTURAL_GLYPH_44_VERTICES,
};

static FUTURAL_GLYPH_45_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: -8, y: -12 },
    HersheyVertex { x: -8, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -8, y: -12 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 8, y: -12 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 8, y: -12 },
    HersheyVertex { x: 8, y: 9 },
];

pub const FUTURAL_GLYPH_45: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_45_VERTICES,
};

static FUTURAL_GLYPH_46_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_46: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_46_VERTICES,
};

static FUTURAL_GLYPH_47_VERTICES: [HersheyVertex; 21] = [
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -6, y: -9 },
    HersheyVertex { x: -7, y: -7 },
    HersheyVertex { x: -8, y: -4 },
    HersheyVertex { x: -8, y: 1 },
    HersheyVertex { x: -7, y: 4 },
    HersheyVertex { x: -6, y: 6 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 8, y: 1 },
    HersheyVertex { x: 8, y: -4 },
    HersheyVertex { x: 7, y: -7 },
    HersheyVertex { x: 6, y: -9 },
    HersheyVertex { x: 4, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -2, y: -12 },
];

pub const FUTURAL_GLYPH_47: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_47_VERTICES,
};

static FUTURAL_GLYPH_48_VERTICES: [HersheyVertex; 13] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 6, y: -10 },
    HersheyVertex { x: 7, y: -8 },
    HersheyVertex { x: 7, y: -5 },
    HersheyVertex { x: 6, y: -3 },
    HersheyVertex { x: 5, y: -2 },
    HersheyVertex { x: 2, y: -1 },
    HersheyVertex { x: -7, y: -1 },
];

pub const FUTURAL_GLYPH_48: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 10,
    vertices: &FUTURAL_GLYPH_48_VERTICES,
};

static FUTURAL_GLYPH_49_VERTICES: [HersheyVertex; 24] = [
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -4, y: -11 },
    HersheyVertex { x: -6, y: -9 },
    HersheyVertex { x: -7, y: -7 },
    HersheyVertex { x: -8, y: -4 },
    HersheyVertex { x: -8, y: 1 },
    HersheyVertex { x: -7, y: 4 },
    HersheyVertex { x: -6, y: 6 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 8, y: 1 },
    HersheyVertex { x: 8, y: -4 },
    HersheyVertex { x: 7, y: -7 },
    HersheyVertex { x: 6, y: -9 },
    HersheyVertex { x: 4, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -2, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 7, y: 11 },
];

pub const FUTURAL_GLYPH_49: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_49_VERTICES,
};

static FUTURAL_GLYPH_50_VERTICES: [HersheyVertex; 16] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 6, y: -10 },
    HersheyVertex { x: 7, y: -8 },
    HersheyVertex { x: 7, y: -6 },
    HersheyVertex { x: 6, y: -4 },
    HersheyVertex { x: 5, y: -3 },
    HersheyVertex { x: 2, y: -2 },
    HersheyVertex { x: -7, y: -2 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -2 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_50: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 10,
    vertices: &FUTURAL_GLYPH_50_VERTICES,
};

static FUTURAL_GLYPH_51_VERTICES: [HersheyVertex; 20] = [
    HersheyVertex { x: 7, y: -9 },
    HersheyVertex { x: 5, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -5, y: -11 },
    HersheyVertex { x: -7, y: -9 },
    HersheyVertex { x: -7, y: -7 },
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: -5, y: -4 },
    HersheyVertex { x: -3, y: -3 },
    HersheyVertex { x: 3, y: -1 },
    HersheyVertex { x: 5, y: 0 },
    HersheyVertex { x: 6, y: 1 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 7, y: 6 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -5, y: 8 },
    HersheyVertex { x: -7, y: 6 },
];

pub const FUTURAL_GLYPH_51: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_51_VERTICES,
};

static FUTURAL_GLYPH_52_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: -12 },
];

pub const FUTURAL_GLYPH_52: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_52_VERTICES,
};

static FUTURAL_GLYPH_53_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -7, y: 3 },
    HersheyVertex { x: -6, y: 6 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 7, y: -12 },
];

pub const FUTURAL_GLYPH_53: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_53_VERTICES,
};

static FUTURAL_GLYPH_54_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -8, y: -12 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 8, y: -12 },
    HersheyVertex { x: 0, y: 9 },
];

pub const FUTURAL_GLYPH_54: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_54_VERTICES,
};

static FUTURAL_GLYPH_55_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: -10, y: -12 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 10, y: -12 },
    HersheyVertex { x: 5, y: 9 },
];

pub const FUTURAL_GLYPH_55: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_55_VERTICES,
};

static FUTURAL_GLYPH_56_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
];

pub const FUTURAL_GLYPH_56: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_56_VERTICES,
};

static FUTURAL_GLYPH_57_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: -8, y: -12 },
    HersheyVertex { x: 0, y: -2 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 8, y: -12 },
    HersheyVertex { x: 0, y: -2 },
];

pub const FUTURAL_GLYPH_57: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_57_VERTICES,
};

static FUTURAL_GLYPH_58_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: -7, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: -7, y: 9 },
    HersheyVertex { x: 7, y: 9 },
];

pub const FUTURAL_GLYPH_58: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 10,
    vertices: &FUTURAL_GLYPH_58_VERTICES,
};

static FUTURAL_GLYPH_59_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: -3, y: -16 },
    HersheyVertex { x: -3, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -2, y: -16 },
    HersheyVertex { x: -2, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -3, y: -16 },
    HersheyVertex { x: 4, y: -16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -3, y: 16 },
    HersheyVertex { x: 4, y: 16 },
];

pub const FUTURAL_GLYPH_59: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_59_VERTICES,
};

static FUTURAL_GLYPH_60_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: 7, y: 12 },
];

pub const FUTURAL_GLYPH_60: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_60_VERTICES,
};

static FUTURAL_GLYPH_61_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 2, y: -16 },
    HersheyVertex { x: 2, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: 3, y: -16 },
    HersheyVertex { x: 3, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -4, y: -16 },
    HersheyVertex { x: 3, y: -16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -4, y: 16 },
    HersheyVertex { x: 3, y: 16 },
];

pub const FUTURAL_GLYPH_61: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_61_VERTICES,
};

static FUTURAL_GLYPH_62_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -14 },
    HersheyVertex { x: -8, y: 0 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -14 },
    HersheyVertex { x: 8, y: 0 },
];

pub const FUTURAL_GLYPH_62: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_62_VERTICES,
};

static FUTURAL_GLYPH_63_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: -9, y: 16 },
    HersheyVertex { x: 9, y: 16 },
];

pub const FUTURAL_GLYPH_63: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_63_VERTICES,
};

static FUTURAL_GLYPH_64_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 1, y: -7 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -1, y: -3 },
    HersheyVertex { x: 0, y: -2 },
    HersheyVertex { x: 1, y: -3 },
    HersheyVertex { x: 0, y: -4 },
    HersheyVertex { x: -1, y: -3 },
];

pub const FUTURAL_GLYPH_64: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_64_VERTICES,
};

static FUTURAL_GLYPH_65_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_65: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_65_VERTICES,
};

static FUTURAL_GLYPH_66_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: -4, y: -4 },
    HersheyVertex { x: -2, y: -5 },
    HersheyVertex { x: 1, y: -5 },
    HersheyVertex { x: 3, y: -4 },
    HersheyVertex { x: 5, y: -2 },
    HersheyVertex { x: 6, y: 1 },
    HersheyVertex { x: 6, y: 3 },
    HersheyVertex { x: 5, y: 6 },
    HersheyVertex { x: 3, y: 8 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -6, y: 6 },
];

pub const FUTURAL_GLYPH_66: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 9,
    vertices: &FUTURAL_GLYPH_66_VERTICES,
};

static FUTURAL_GLYPH_67_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_67: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_67_VERTICES,
};

static FUTURAL_GLYPH_68_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: 6, y: -12 },
    HersheyVertex { x: 6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_68: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_68_VERTICES,
};

static FUTURAL_GLYPH_69_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: 6, y: 1 },
    HersheyVertex { x: 6, y: -1 },
    HersheyVertex { x: 5, y: -3 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_69: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 9,
    vertices: &FUTURAL_GLYPH_69_VERTICES,
};

static FUTURAL_GLYPH_70_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 5, y: -12 },
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: 1, y: -11 },
    HersheyVertex { x: 0, y: -8 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -3, y: -5 },
    HersheyVertex { x: 4, y: -5 },
];

pub const FUTURAL_GLYPH_70: HersheyGlyph = HersheyGlyph {
    left: -5,
    right: 7,
    vertices: &FUTURAL_GLYPH_70_VERTICES,
};

static FUTURAL_GLYPH_71_VERTICES: [HersheyVertex; 22] = [
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 6, y: 11 },
    HersheyVertex { x: 5, y: 14 },
    HersheyVertex { x: 4, y: 15 },
    HersheyVertex { x: 2, y: 16 },
    HersheyVertex { x: -1, y: 16 },
    HersheyVertex { x: -3, y: 15 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_71: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_71_VERTICES,
};

static FUTURAL_GLYPH_72_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: -5, y: -12 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: -1 },
    HersheyVertex { x: -2, y: -4 },
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: 3, y: -5 },
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 6, y: -1 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_72: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_72_VERTICES,
};

static FUTURAL_GLYPH_73_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: 0, y: -11 },
    HersheyVertex { x: 1, y: -12 },
    HersheyVertex { x: 0, y: -13 },
    HersheyVertex { x: -1, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: 0, y: 9 },
];

pub const FUTURAL_GLYPH_73: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_73_VERTICES,
};

static FUTURAL_GLYPH_74_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 1, y: -11 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 1, y: -13 },
    HersheyVertex { x: 0, y: -12 },
    HERSHEY_LIFT,
    HersheyVertex { x: 1, y: -5 },
    HersheyVertex { x: 1, y: 12 },
    HersheyVertex { x: 0, y: 15 },
    HersheyVertex { x: -2, y: 16 },
    HersheyVertex { x: -4, y: 16 },
];

pub const FUTURAL_GLYPH_74: HersheyGlyph = HersheyGlyph {
    left: -5,
    right: 5,
    vertices: &FUTURAL_GLYPH_74_VERTICES,
};

static FUTURAL_GLYPH_75_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -5, y: -12 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 5, y: -5 },
    HersheyVertex { x: -5, y: 5 },
    HERSHEY_LIFT,
    HersheyVertex { x: -1, y: 1 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_75: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 8,
    vertices: &FUTURAL_GLYPH_75_VERTICES,
};

static FUTURAL_GLYPH_76_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 9 },
];

pub const FUTURAL_GLYPH_76: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_76_VERTICES,
};

static FUTURAL_GLYPH_77_VERTICES: [HersheyVertex; 18] = [
    HersheyVertex { x: -11, y: -5 },
    HersheyVertex { x: -11, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -11, y: -1 },
    HersheyVertex { x: -8, y: -4 },
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: -3, y: -5 },
    HersheyVertex { x: -1, y: -4 },
    HersheyVertex { x: 0, y: -1 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -1 },
    HersheyVertex { x: 3, y: -4 },
    HersheyVertex { x: 5, y: -5 },
    HersheyVertex { x: 8, y: -5 },
    HersheyVertex { x: 10, y: -4 },
    HersheyVertex { x: 11, y: -1 },
    HersheyVertex { x: 11, y: 9 },
];

pub const FUTURAL_GLYPH_77: HersheyGlyph = HersheyGlyph {
    left: -15,
    right: 15,
    vertices: &FUTURAL_GLYPH_77_VERTICES,
};

static FUTURAL_GLYPH_78_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: -1 },
    HersheyVertex { x: -2, y: -4 },
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: 3, y: -5 },
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 6, y: -1 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_78: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_78_VERTICES,
};

static FUTURAL_GLYPH_79_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 7, y: 3 },
    HersheyVertex { x: 7, y: 1 },
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
];

pub const FUTURAL_GLYPH_79: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_79_VERTICES,
};

static FUTURAL_GLYPH_80_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: -6, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: -4, y: -4 },
    HersheyVertex { x: -2, y: -5 },
    HersheyVertex { x: 1, y: -5 },
    HersheyVertex { x: 3, y: -4 },
    HersheyVertex { x: 5, y: -2 },
    HersheyVertex { x: 6, y: 1 },
    HersheyVertex { x: 6, y: 3 },
    HersheyVertex { x: 5, y: 6 },
    HersheyVertex { x: 3, y: 8 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -6, y: 6 },
];

pub const FUTURAL_GLYPH_80: HersheyGlyph = HersheyGlyph {
    left: -10,
    right: 9,
    vertices: &FUTURAL_GLYPH_80_VERTICES,
};

static FUTURAL_GLYPH_81_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 6, y: 16 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 4, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -3, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -6, y: 1 },
    HersheyVertex { x: -6, y: 3 },
    HersheyVertex { x: -5, y: 6 },
    HersheyVertex { x: -3, y: 8 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 4, y: 8 },
    HersheyVertex { x: 6, y: 6 },
];

pub const FUTURAL_GLYPH_81: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_81_VERTICES,
};

static FUTURAL_GLYPH_82_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: -3, y: -5 },
    HersheyVertex { x: -3, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -3, y: 1 },
    HersheyVertex { x: -2, y: -2 },
    HersheyVertex { x: 0, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: 5, y: -5 },
];

pub const FUTURAL_GLYPH_82: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 6,
    vertices: &FUTURAL_GLYPH_82_VERTICES,
};

static FUTURAL_GLYPH_83_VERTICES: [HersheyVertex; 17] = [
    HersheyVertex { x: 6, y: -2 },
    HersheyVertex { x: 5, y: -4 },
    HersheyVertex { x: 2, y: -5 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -4, y: -4 },
    HersheyVertex { x: -5, y: -2 },
    HersheyVertex { x: -4, y: 0 },
    HersheyVertex { x: -2, y: 1 },
    HersheyVertex { x: 3, y: 2 },
    HersheyVertex { x: 5, y: 3 },
    HersheyVertex { x: 6, y: 5 },
    HersheyVertex { x: 6, y: 6 },
    HersheyVertex { x: 5, y: 8 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -5, y: 6 },
];

pub const FUTURAL_GLYPH_83: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 9,
    vertices: &FUTURAL_GLYPH_83_VERTICES,
};

static FUTURAL_GLYPH_84_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 5 },
    HersheyVertex { x: 1, y: 8 },
    HersheyVertex { x: 3, y: 9 },
    HersheyVertex { x: 5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -3, y: -5 },
    HersheyVertex { x: 4, y: -5 },
];

pub const FUTURAL_GLYPH_84: HersheyGlyph = HersheyGlyph {
    left: -5,
    right: 7,
    vertices: &FUTURAL_GLYPH_84_VERTICES,
};

static FUTURAL_GLYPH_85_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: -5, y: 5 },
    HersheyVertex { x: -4, y: 8 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: 3, y: 8 },
    HersheyVertex { x: 6, y: 5 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_85: HersheyGlyph = HersheyGlyph {
    left: -9,
    right: 10,
    vertices: &FUTURAL_GLYPH_85_VERTICES,
};

static FUTURAL_GLYPH_86_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 0, y: 9 },
];

pub const FUTURAL_GLYPH_86: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_86_VERTICES,
};

static FUTURAL_GLYPH_87_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: -8, y: -5 },
    HersheyVertex { x: -4, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: -4, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -5 },
    HersheyVertex { x: 4, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 8, y: -5 },
    HersheyVertex { x: 4, y: 9 },
];

pub const FUTURAL_GLYPH_87: HersheyGlyph = HersheyGlyph {
    left: -11,
    right: 11,
    vertices: &FUTURAL_GLYPH_87_VERTICES,
};

static FUTURAL_GLYPH_88_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: 6, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: -5, y: 9 },
];

pub const FUTURAL_GLYPH_88: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 9,
    vertices: &FUTURAL_GLYPH_88_VERTICES,
};

static FUTURAL_GLYPH_89_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: -6, y: -5 },
    HersheyVertex { x: 0, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: -2, y: 13 },
    HersheyVertex { x: -4, y: 15 },
    HersheyVertex { x: -6, y: 16 },
    HersheyVertex { x: -7, y: 16 },
];

pub const FUTURAL_GLYPH_89: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_89_VERTICES,
};

static FUTURAL_GLYPH_90_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 6, y: -5 },
    HersheyVertex { x: -5, y: 9 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: -5 },
    HersheyVertex { x: 6, y: -5 },
    HERSHEY_LIFT,
    HersheyVertex { x: -5, y: 9 },
    HersheyVertex { x: 6, y: 9 },
];

pub const FUTURAL_GLYPH_90: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 9,
    vertices: &FUTURAL_GLYPH_90_VERTICES,
};

static FUTURAL_GLYPH_91_VERTICES: [HersheyVertex; 39] = [
    HersheyVertex { x: 2, y: -16 },
    HersheyVertex { x: 0, y: -15 },
    HersheyVertex { x: -1, y: -14 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -2, y: -10 },
    HersheyVertex { x: -1, y: -8 },
    HersheyVertex { x: 0, y: -7 },
    HersheyVertex { x: 1, y: -5 },
    HersheyVertex { x: 1, y: -3 },
    HersheyVertex { x: -1, y: -1 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -15 },
    HersheyVertex { x: -1, y: -13 },
    HersheyVertex { x: -1, y: -11 },
    HersheyVertex { x: 0, y: -9 },
    HersheyVertex { x: 1, y: -8 },
    HersheyVertex { x: 2, y: -6 },
    HersheyVertex { x: 2, y: -4 },
    HersheyVertex { x: 1, y: -2 },
    HersheyVertex { x: -3, y: 0 },
    HersheyVertex { x: 1, y: 2 },
    HersheyVertex { x: 2, y: 4 },
    HersheyVertex { x: 2, y: 6 },
    HersheyVertex { x: 1, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: -1, y: 11 },
    HersheyVertex { x: -1, y: 13 },
    HersheyVertex { x: 0, y: 15 },
    HERSHEY_LIFT,
    HersheyVertex { x: -1, y: 1 },
    HersheyVertex { x: 1, y: 3 },
    HersheyVertex { x: 1, y: 5 },
    HersheyVertex { x: 0, y: 7 },
    HersheyVertex { x: -1, y: 8 },
    HersheyVertex { x: -2, y: 10 },
    HersheyVertex { x: -2, y: 12 },
    HersheyVertex { x: -1, y: 14 },
    HersheyVertex { x: 0, y: 15 },
    HersheyVertex { x: 2, y: 16 },
];

pub const FUTURAL_GLYPH_91: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_91_VERTICES,
};

static FUTURAL_GLYPH_92_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -16 },
    HersheyVertex { x: 0, y: 16 },
];

pub const FUTURAL_GLYPH_92: HersheyGlyph = HersheyGlyph {
    left: -4,
    right: 4,
    vertices: &FUTURAL_GLYPH_92_VERTICES,
};

static FUTURAL_GLYPH_93_VERTICES: [HersheyVertex; 39] = [
    HersheyVertex { x: -2, y: -16 },
    HersheyVertex { x: 0, y: -15 },
    HersheyVertex { x: 1, y: -14 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 2, y: -10 },
    HersheyVertex { x: 1, y: -8 },
    HersheyVertex { x: 0, y: -7 },
    HersheyVertex { x: -1, y: -5 },
    HersheyVertex { x: -1, y: -3 },
    HersheyVertex { x: 1, y: -1 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -15 },
    HersheyVertex { x: 1, y: -13 },
    HersheyVertex { x: 1, y: -11 },
    HersheyVertex { x: 0, y: -9 },
    HersheyVertex { x: -1, y: -8 },
    HersheyVertex { x: -2, y: -6 },
    HersheyVertex { x: -2, y: -4 },
    HersheyVertex { x: -1, y: -2 },
    HersheyVertex { x: 3, y: 0 },
    HersheyVertex { x: -1, y: 2 },
    HersheyVertex { x: -2, y: 4 },
    HersheyVertex { x: -2, y: 6 },
    HersheyVertex { x: -1, y: 8 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: 1, y: 11 },
    HersheyVertex { x: 1, y: 13 },
    HersheyVertex { x: 0, y: 15 },
    HERSHEY_LIFT,
    HersheyVertex { x: 1, y: 1 },
    HersheyVertex { x: -1, y: 3 },
    HersheyVertex { x: -1, y: 5 },
    HersheyVertex { x: 0, y: 7 },
    HersheyVertex { x: 1, y: 8 },
    HersheyVertex { x: 2, y: 10 },
    HersheyVertex { x: 2, y: 12 },
    HersheyVertex { x: 1, y: 14 },
    HersheyVertex { x: 0, y: 15 },
    HersheyVertex { x: -2, y: 16 },
];

pub const FUTURAL_GLYPH_93: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &FUTURAL_GLYPH_93_VERTICES,
};

static FUTURAL_GLYPH_94_VERTICES: [HersheyVertex; 23] = [
    HersheyVertex { x: -9, y: 3 },
    HersheyVertex { x: -9, y: 1 },
    HersheyVertex { x: -8, y: -2 },
    HersheyVertex { x: -6, y: -3 },
    HersheyVertex { x: -4, y: -3 },
    HersheyVertex { x: -2, y: -2 },
    HersheyVertex { x: 2, y: 1 },
    HersheyVertex { x: 4, y: 2 },
    HersheyVertex { x: 6, y: 2 },
    HersheyVertex { x: 8, y: 1 },
    HersheyVertex { x: 9, y: -1 },
    HERSHEY_LIFT,
    HersheyVertex { x: -9, y: 1 },
    HersheyVertex { x: -8, y: -1 },
    HersheyVertex { x: -6, y: -2 },
    HersheyVertex { x: -4, y: -2 },
    HersheyVertex { x: -2, y: -1 },
    HersheyVertex { x: 2, y: 2 },
    HersheyVertex { x: 4, y: 3 },
    HersheyVertex { x: 6, y: 3 },
    HersheyVertex { x: 8, y: 2 },
    HersheyVertex { x: 9, y: -1 },
    HersheyVertex { x: 9, y: -3 },
];

pub const FUTURAL_GLYPH_94: HersheyGlyph = HersheyGlyph {
    left: -12,
    right: 12,
    vertices: &FUTURAL_GLYPH_94_VERTICES,
};

static FUTURAL_GLYPH_95_VERTICES: [HersheyVertex; 34] = [
    HersheyVertex { x: -8, y: -12 },
    HersheyVertex { x: -8, y: 9 },
    HersheyVertex { x: -7, y: 9 },
    HersheyVertex { x: -7, y: -12 },
    HersheyVertex { x: -6, y: -12 },
    HersheyVertex { x: -6, y: 9 },
    HersheyVertex { x: -5, y: 9 },
    HersheyVertex { x: -5, y: -12 },
    HersheyVertex { x: -4, y: -12 },
    HersheyVertex { x: -4, y: 9 },
    HersheyVertex { x: -3, y: 9 },
    HersheyVertex { x: -3, y: -12 },
    HersheyVertex { x: -2, y: -12 },
    HersheyVertex { x: -2, y: 9 },
    HersheyVertex { x: -1, y: 9 },
    HersheyVertex { x: -1, y: -12 },
    HersheyVertex { x: 0, y: -12 },
    HersheyVertex { x: 0, y: 9 },
    HersheyVertex { x: 1, y: 9 },
    HersheyVertex { x: 1, y: -12 },
    HersheyVertex { x: 2, y: -12 },
    HersheyVertex { x: 2, y: 9 },
    HersheyVertex { x: 3, y: 9 },
    HersheyVertex { x: 3, y: -12 },
    HersheyVertex { x: 4, y: -12 },
    HersheyVertex { x: 4, y: 9 },
    HersheyVertex { x: 5, y: 9 },
    HersheyVertex { x: 5, y: -12 },
    HersheyVertex { x: 6, y: -12 },
    HersheyVertex { x: 6, y: 9 },
    HersheyVertex { x: 7, y: 9 },
    HersheyVertex { x: 7, y: -12 },
    HersheyVertex { x: 8, y: -12 },
    HersheyVertex { x: 8, y: 9 },
];

pub const FUTURAL_GLYPH_95: HersheyGlyph = HersheyGlyph {
    left: -8,
    right: 8,
    vertices: &FUTURAL_GLYPH_95_VERTICES,
};

pub static FUTURAL_GLYPHS: [HersheyGlyph; 96] = [
    FUTURAL_GLYPH_0,
    FUTURAL_GLYPH_1,
    FUTURAL_GLYPH_2,
    FUTURAL_GLYPH_3,
    FUTURAL_GLYPH_4,
    FUTURAL_GLYPH_5,
    FUTURAL_GLYPH_6,
    FUTURAL_GLYPH_7,
    FUTURAL_GLYPH_8,
    FUTURAL_GLYPH_9,
    FUTURAL_GLYPH_10,
    FUTURAL_GLYPH_11,
    FUTURAL_GLYPH_12,
    FUTURAL_GLYPH_13,
    FUTURAL_GLYPH_14,
    FUTURAL_GLYPH_15,
    FUTURAL_GLYPH_16,
    FUTURAL_GLYPH_17,
    FUTURAL_GLYPH_18,
    FUTURAL_GLYPH_19,
    FUTURAL_GLYPH_20,
    FUTURAL_GLYPH_21,
    FUTURAL_GLYPH_22,
    FUTURAL_GLYPH_23,
    FUTURAL_GLYPH_24,
    FUTURAL_GLYPH_25,
    FUTURAL_GLYPH_26,
    FUTURAL_GLYPH_27,
    FUTURAL_GLYPH_28,
    FUTURAL_GLYPH_29,
    FUTURAL_GLYPH_30,
    FUTURAL_GLYPH_31,
    FUTURAL_GLYPH_32,
    FUTURAL_GLYPH_33,
    FUTURAL_GLYPH_34,
    FUTURAL_GLYPH_35,
    FUTURAL_GLYPH_36,
    FUTURAL_GLYPH_37,
    FUTURAL_GLYPH_38,
    FUTURAL_GLYPH_39,
    FUTURAL_GLYPH_40,
    FUTURAL_GLYPH_41,
    FUTURAL_GLYPH_42,
    FUTURAL_GLYPH_43,
    FUTURAL_GLYPH_44,
    FUTURAL_GLYPH_45,
    FUTURAL_GLYPH_46,
    FUTURAL_GLYPH_47,
    FUTURAL_GLYPH_48,
    FUTURAL_GLYPH_49,
    FUTURAL_GLYPH_50,
    FUTURAL_GLYPH_51,
    FUTURAL_GLYPH_52,
    FUTURAL_GLYPH_53,
    FUTURAL_GLYPH_54,
    FUTURAL_GLYPH_55,
    FUTURAL_GLYPH_56,
    FUTURAL_GLYPH_57,
    FUTURAL_GLYPH_58,
    FUTURAL_GLYPH_59,
    FUTURAL_GLYPH_60,
    FUTURAL_GLYPH_61,
    FUTURAL_GLYPH_62,
    FUTURAL_GLYPH_63,
    FUTURAL_GLYPH_64,
    FUTURAL_GLYPH_65,
    FUTURAL_GLYPH_66,
    FUTURAL_GLYPH_67,
    FUTURAL_GLYPH_68,
    FUTURAL_GLYPH_69,
    FUTURAL_GLYPH_70,
    FUTURAL_GLYPH_71,
    FUTURAL_GLYPH_72,
    FUTURAL_GLYPH_73,
    FUTURAL_GLYPH_74,
    FUTURAL_GLYPH_75,
    FUTURAL_GLYPH_76,
    FUTURAL_GLYPH_77,
    FUTURAL_GLYPH_78,
    FUTURAL_GLYPH_79,
    FUTURAL_GLYPH_80,
    FUTURAL_GLYPH_81,
    FUTURAL_GLYPH_82,
    FUTURAL_GLYPH_83,
    FUTURAL_GLYPH_84,
    FUTURAL_GLYPH_85,
    FUTURAL_GLYPH_86,
    FUTURAL_GLYPH_87,
    FUTURAL_GLYPH_88,
    FUTURAL_GLYPH_89,
    FUTURAL_GLYPH_90,
    FUTURAL_GLYPH_91,
    FUTURAL_GLYPH_92,
    FUTURAL_GLYPH_93,
    FUTURAL_GLYPH_94,
    FUTURAL_GLYPH_95,
];

pub static FONT_FUTURAL: HersheyFont = HersheyFont {
    name: "Futural",
    first_char: b' ',
    glyphs: &FUTURAL_GLYPHS,
    space_advance: 10,
    line_height: 25,
    missing_glyph: &FUTURAL_GLYPH_10,
};