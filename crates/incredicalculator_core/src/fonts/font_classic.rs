//! "Classic" font: the calculator's original hand-made grid font, converted
//! from the old packed-nibble format into Hershey-style glyphs.
//!
//! Old grid units are multiplied by 7 and shifted by -24 in Y so the
//! cap height and baseline line up with Futural at the same text scale.

use crate::fonts::types::HERSHEY_LIFT;
use crate::fonts::{HersheyFont, HersheyGlyph, HersheyVertex};

static CLASSIC_SPACE_VERTICES: [HersheyVertex; 1] = [HERSHEY_LIFT];

const CLASSIC_SPACE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_SPACE_VERTICES,
};

static CLASSIC_HAPPY_VERTICES: [HersheyVertex; 18] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 35, y: 11 },
    HersheyVertex { x: 35, y: -17 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 14, y: 11 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 28, y: 4 },
];

const CLASSIC_HAPPY: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 42,
    vertices: &CLASSIC_HAPPY_VERTICES,
};

static CLASSIC_SAD_VERTICES: [HersheyVertex; 18] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 35, y: 11 },
    HersheyVertex { x: 35, y: -17 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 28, y: 11 },
];

const CLASSIC_SAD: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 42,
    vertices: &CLASSIC_SAD_VERTICES,
};

static CLASSIC_HEART_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
];

const CLASSIC_HEART: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_HEART_VERTICES,
};

static CLASSIC_DIAMOND_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -24 },
];

const CLASSIC_DIAMOND: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_DIAMOND_VERTICES,
};

static CLASSIC_CLUB_VERTICES: [HersheyVertex; 20] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 21, y: -17 },
    HersheyVertex { x: 21, y: -10 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 7, y: -10 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 14, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -10 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 21, y: -10 },
];

const CLASSIC_CLUB: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_CLUB_VERTICES,
};

static CLASSIC_SPADE_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 21, y: -10 },
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: -10 },
    HersheyVertex { x: 14, y: -24 },
];

const CLASSIC_SPADE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_SPADE_VERTICES,
};

static CLASSIC_BULLET_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 7, y: -10 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: -10 },
];

const CLASSIC_BULLET: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_BULLET_VERTICES,
};

static CLASSIC_CIRCLE_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: -10 },
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 7, y: -17 },
];

const CLASSIC_CIRCLE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_CIRCLE_VERTICES,
};

static CLASSIC_TRI_R_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: -24 },
];

const CLASSIC_TRI_R: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_TRI_R_VERTICES,
};

static CLASSIC_TRI_L_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 0, y: -3 },
];

const CLASSIC_TRI_L: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_TRI_L_VERTICES,
};

static CLASSIC_ARROW_UD_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 28, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 28, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_ARROW_UD: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_UD_VERTICES,
};

static CLASSIC_ARROW_U_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 28, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_ARROW_U: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_U_VERTICES,
};

static CLASSIC_ARROW_D_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 28, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_ARROW_D: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_D_VERTICES,
};

static CLASSIC_ARROW_R_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 14, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 14, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_ARROW_R: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_R_VERTICES,
};

static CLASSIC_ARROW_L_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_ARROW_L: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_L_VERTICES,
};

static CLASSIC_ARROW_LR_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 21, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 21, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_ARROW_LR: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_ARROW_LR_VERTICES,
};

static CLASSIC_TRI_U_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: -24 },
];

const CLASSIC_TRI_U: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_TRI_U_VERTICES,
};

static CLASSIC_TRI_D_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_TRI_D: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_TRI_D_VERTICES,
};

static CLASSIC_EXPT_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
];

const CLASSIC_EXPT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_EXPT_VERTICES,
};

static CLASSIC_QUOTE2_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 7, y: -10 },
];

const CLASSIC_QUOTE2: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 14,
    vertices: &CLASSIC_QUOTE2_VERTICES,
};

static CLASSIC_HASH_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 7, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 21, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 28, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 28, y: 4 },
];

const CLASSIC_HASH: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_HASH_VERTICES,
};

static CLASSIC_DOLLAR_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 0, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_DOLLAR: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_DOLLAR_VERTICES,
};

static CLASSIC_PCT_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 11 },
    HERSHEY_LIFT,
];

const CLASSIC_PCT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_PCT_VERTICES,
};

static CLASSIC_AND_VERTICES: [HersheyVertex; 13] = [
    HersheyVertex { x: 21, y: -17 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 28, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 28, y: 4 },
];

const CLASSIC_AND: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_AND_VERTICES,
};

static CLASSIC_QUOTE1_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: -10 },
];

const CLASSIC_QUOTE1: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_QUOTE1_VERTICES,
};

static CLASSIC_LPAREN_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
];

const CLASSIC_LPAREN: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 14,
    vertices: &CLASSIC_LPAREN_VERTICES,
};

static CLASSIC_RPAREN_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_RPAREN: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 14,
    vertices: &CLASSIC_RPAREN_VERTICES,
};

static CLASSIC_STAR_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 28, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 14, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_STAR: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_STAR_VERTICES,
};

static CLASSIC_PLUS_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 14, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_PLUS: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_PLUS_VERTICES,
};

static CLASSIC_COMMA_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 7, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 25 },
];

const CLASSIC_COMMA: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 14,
    vertices: &CLASSIC_COMMA_VERTICES,
};

static CLASSIC_DASH_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_DASH: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_DASH_VERTICES,
};

static CLASSIC_DOT_VERTICES: [HersheyVertex; 2] = [HersheyVertex { x: 0, y: 18 }, HERSHEY_LIFT];

const CLASSIC_DOT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_DOT_VERTICES,
};

static CLASSIC_SLASH_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_SLASH: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_SLASH_VERTICES,
};

static CLASSIC_0_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
];

const CLASSIC_0: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_0_VERTICES,
};

static CLASSIC_1_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_1: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_1_VERTICES,
};

static CLASSIC_2_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_2: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_2_VERTICES,
};

static CLASSIC_3_VERTICES: [HersheyVertex; 14] = [
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
];

const CLASSIC_3: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_3_VERTICES,
};

static CLASSIC_4_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 28, y: 4 },
];

const CLASSIC_4: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_4_VERTICES,
};

static CLASSIC_5_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
];

const CLASSIC_5: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_5_VERTICES,
};

static CLASSIC_6_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
];

const CLASSIC_6: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_6_VERTICES,
};

static CLASSIC_7_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 7, y: 18 },
];

const CLASSIC_7: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_7_VERTICES,
};

static CLASSIC_8_VERTICES: [HersheyVertex; 16] = [
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
];

const CLASSIC_8: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_8_VERTICES,
};

static CLASSIC_9_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 7, y: 18 },
];

const CLASSIC_9: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_9_VERTICES,
};

static CLASSIC_COLON_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HERSHEY_LIFT,
];

const CLASSIC_COLON: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_COLON_VERTICES,
};

static CLASSIC_SEMI_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 7, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 25 },
];

const CLASSIC_SEMI: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 14,
    vertices: &CLASSIC_SEMI_VERTICES,
};

static CLASSIC_LESS_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: 11 },
];

const CLASSIC_LESS: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_LESS_VERTICES,
};

static CLASSIC_EQUAL_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 28, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 28, y: 4 },
];

const CLASSIC_EQUAL: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_EQUAL_VERTICES,
};

static CLASSIC_GREAT_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 0, y: 11 },
];

const CLASSIC_GREAT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_GREAT_VERTICES,
};

static CLASSIC_QUESTION_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 14, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 18 },
    HERSHEY_LIFT,
];

const CLASSIC_QUESTION: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_QUESTION_VERTICES,
};

static CLASSIC_AT_VERTICES: [HersheyVertex; 13] = [
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 14, y: -10 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 14, y: 11 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 21, y: -10 },
    HersheyVertex { x: 14, y: -17 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_AT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_AT_VERTICES,
};

static CLASSIC_A_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 28, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
];

const CLASSIC_A: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_A_VERTICES,
};

static CLASSIC_B_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: -10 },
    HersheyVertex { x: 21, y: -17 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: -3 },
];

const CLASSIC_B: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_B_VERTICES,
};

static CLASSIC_C_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
];

const CLASSIC_C: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_C_VERTICES,
};

static CLASSIC_D_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -24 },
];

const CLASSIC_D: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_D_VERTICES,
};

static CLASSIC_E_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: -3 },
];

const CLASSIC_E: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_E_VERTICES,
};

static CLASSIC_F_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: -3 },
];

const CLASSIC_F: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_F_VERTICES,
};

static CLASSIC_G_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -3 },
    HersheyVertex { x: 14, y: -3 },
];

const CLASSIC_G: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_G_VERTICES,
};

static CLASSIC_H_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 28, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_H: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_H_VERTICES,
};

static CLASSIC_I_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_I: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_I_VERTICES,
};

static CLASSIC_J_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
];

const CLASSIC_J: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_J_VERTICES,
};

static CLASSIC_K_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_K: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_K_VERTICES,
};

static CLASSIC_L_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_L: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_L_VERTICES,
};

static CLASSIC_M_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_M: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_M_VERTICES,
};

static CLASSIC_N_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 28, y: -24 },
];

const CLASSIC_N: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_N_VERTICES,
};

static CLASSIC_O_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
];

const CLASSIC_O: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_O_VERTICES,
};

static CLASSIC_P_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: -3 },
];

const CLASSIC_P: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_P_VERTICES,
};

static CLASSIC_Q_VERTICES: [HersheyVertex; 12] = [
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_Q: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_Q_VERTICES,
};

static CLASSIC_R_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 28, y: -17 },
    HersheyVertex { x: 28, y: -10 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_R: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_R_VERTICES,
};

static CLASSIC_S_VERTICES: [HersheyVertex; 12] = [
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 7, y: -24 },
    HersheyVertex { x: 0, y: -17 },
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_S: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_S_VERTICES,
};

static CLASSIC_T_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: -24 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_T: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_T_VERTICES,
};

static CLASSIC_U_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -24 },
];

const CLASSIC_U: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_U_VERTICES,
};

static CLASSIC_V_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 28, y: -24 },
];

const CLASSIC_V: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_V_VERTICES,
};

static CLASSIC_W_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 28, y: -24 },
];

const CLASSIC_W: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_W_VERTICES,
};

static CLASSIC_X_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_X: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_X_VERTICES,
};

static CLASSIC_Y_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 14, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 14, y: -3 },
];

const CLASSIC_Y: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_Y_VERTICES,
};

static CLASSIC_Z_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_Z: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_Z_VERTICES,
};

static CLASSIC_LBRACK_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: 18 },
];

const CLASSIC_LBRACK: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_LBRACK_VERTICES,
};

static CLASSIC_BSLASH_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_BSLASH: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_BSLASH_VERTICES,
};

static CLASSIC_RBRACK_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_RBRACK: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_RBRACK_VERTICES,
};

static CLASSIC_CARAT_VERTICES: [HersheyVertex; 3] = [
    HersheyVertex { x: 0, y: -10 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 14, y: -10 },
];

const CLASSIC_CARAT: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_CARAT_VERTICES,
};

static CLASSIC_UNDER_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_UNDER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_UNDER_VERTICES,
};

static CLASSIC_ACUTE_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 14, y: -10 },
];

const CLASSIC_ACUTE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_ACUTE_VERTICES,
};

static CLASSIC_A_LOWER_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_A_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_A_LOWER_VERTICES,
};

static CLASSIC_B_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 0, y: -3 },
];

const CLASSIC_B_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_B_LOWER_VERTICES,
};

static CLASSIC_C_LOWER_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_C_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_C_LOWER_VERTICES,
};

static CLASSIC_D_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 21, y: -24 },
];

const CLASSIC_D_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_D_LOWER_VERTICES,
};

static CLASSIC_E_LOWER_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_E_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_E_LOWER_VERTICES,
};

static CLASSIC_F_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 21, y: -24 },
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 7, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -3 },
];

const CLASSIC_F_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_F_LOWER_VERTICES,
};

static CLASSIC_G_LOWER_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: 25 },
    HersheyVertex { x: 14, y: 32 },
    HersheyVertex { x: 0, y: 32 },
];

const CLASSIC_G_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_G_LOWER_VERTICES,
};

static CLASSIC_H_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_H_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_H_LOWER_VERTICES,
};

static CLASSIC_I_LOWER_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_I_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_I_LOWER_VERTICES,
};

static CLASSIC_J_LOWER_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 14, y: -17 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 14, y: 25 },
    HersheyVertex { x: 7, y: 32 },
    HersheyVertex { x: 0, y: 32 },
];

const CLASSIC_J_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_J_LOWER_VERTICES,
};

static CLASSIC_K_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: -10 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: 4 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_K_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_K_LOWER_VERTICES,
};

static CLASSIC_L_LOWER_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_L_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_L_LOWER_VERTICES,
};

static CLASSIC_M_LOWER_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 14, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 11 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 18 },
];

const CLASSIC_M_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_M_LOWER_VERTICES,
};

static CLASSIC_N_LOWER_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_N_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_N_LOWER_VERTICES,
};

static CLASSIC_O_LOWER_VERTICES: [HersheyVertex; 9] = [
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 7, y: -3 },
];

const CLASSIC_O_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_O_LOWER_VERTICES,
};

static CLASSIC_P_LOWER_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: 32 },
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: 4 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
];

const CLASSIC_P_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_P_LOWER_VERTICES,
};

static CLASSIC_Q_LOWER_VERTICES: [HersheyVertex; 12] = [
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: 32 },
    HersheyVertex { x: 28, y: 25 },
];

const CLASSIC_Q_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_Q_LOWER_VERTICES,
};

static CLASSIC_R_LOWER_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 14, y: -3 },
    HersheyVertex { x: 21, y: 4 },
];

const CLASSIC_R_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_R_LOWER_VERTICES,
};

static CLASSIC_S_LOWER_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 21, y: 11 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_S_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_S_LOWER_VERTICES,
};

static CLASSIC_T_LOWER_VERTICES: [HersheyVertex; 6] = [
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 14, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 14, y: -3 },
];

const CLASSIC_T_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_T_LOWER_VERTICES,
};

static CLASSIC_U_LOWER_VERTICES: [HersheyVertex; 8] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 21, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_U_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_U_LOWER_VERTICES,
};

static CLASSIC_V_LOWER_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 21, y: -3 },
];

const CLASSIC_V_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_V_LOWER_VERTICES,
};

static CLASSIC_W_LOWER_VERTICES: [HersheyVertex; 10] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 11 },
    HersheyVertex { x: 14, y: 4 },
    HERSHEY_LIFT,
    HersheyVertex { x: 14, y: 11 },
    HersheyVertex { x: 21, y: 18 },
    HersheyVertex { x: 28, y: 11 },
    HersheyVertex { x: 28, y: -3 },
];

const CLASSIC_W_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 35,
    vertices: &CLASSIC_W_LOWER_VERTICES,
};

static CLASSIC_X_LOWER_VERTICES: [HersheyVertex; 5] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: 18 },
];

const CLASSIC_X_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_X_LOWER_VERTICES,
};

static CLASSIC_Y_LOWER_VERTICES: [HersheyVertex; 11] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 0, y: 11 },
    HersheyVertex { x: 7, y: 18 },
    HersheyVertex { x: 14, y: 18 },
    HersheyVertex { x: 21, y: 11 },
    HERSHEY_LIFT,
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 21, y: 25 },
    HersheyVertex { x: 14, y: 32 },
    HersheyVertex { x: 7, y: 32 },
    HersheyVertex { x: 0, y: 25 },
];

const CLASSIC_Y_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_Y_LOWER_VERTICES,
};

static CLASSIC_Z_LOWER_VERTICES: [HersheyVertex; 4] = [
    HersheyVertex { x: 0, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 21, y: 18 },
];

const CLASSIC_Z_LOWER: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 28,
    vertices: &CLASSIC_Z_LOWER_VERTICES,
};

static CLASSIC_LBRACE_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 14, y: -24 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 7, y: 25 },
    HersheyVertex { x: 14, y: 32 },
];

const CLASSIC_LBRACE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_LBRACE_VERTICES,
};

static CLASSIC_BAR_VERTICES: [HersheyVertex; 2] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 0, y: 32 },
];

const CLASSIC_BAR: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_BAR_VERTICES,
};

static CLASSIC_RBRACE_VERTICES: [HersheyVertex; 7] = [
    HersheyVertex { x: 0, y: -24 },
    HersheyVertex { x: 7, y: -17 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 14, y: 4 },
    HersheyVertex { x: 7, y: 11 },
    HersheyVertex { x: 7, y: 25 },
    HersheyVertex { x: 0, y: 32 },
];

const CLASSIC_RBRACE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 21,
    vertices: &CLASSIC_RBRACE_VERTICES,
};

static CLASSIC_TILDE_VERTICES: [HersheyVertex; 37] = [
    HersheyVertex { x: 0, y: 18 },
    HersheyVertex { x: 0, y: 4 },
    HersheyVertex { x: 7, y: -3 },
    HersheyVertex { x: 21, y: -3 },
    HersheyVertex { x: 28, y: 4 },
    HersheyVertex { x: 28, y: 18 },
    HersheyVertex { x: 21, y: 25 },
    HersheyVertex { x: 7, y: 25 },
    HersheyVertex { x: 0, y: 18 },
    HERSHEY_LIFT,
    HersheyVertex { x: 35, y: 25 },
    HersheyVertex { x: 35, y: -10 },
    HersheyVertex { x: 49, y: -24 },
    HersheyVertex { x: 77, y: -24 },
    HersheyVertex { x: 77, y: -10 },
    HersheyVertex { x: 98, y: -10 },
    HersheyVertex { x: 105, y: -3 },
    HERSHEY_LIFT,
    HersheyVertex { x: 98, y: 25 },
    HersheyVertex { x: 77, y: 25 },
    HersheyVertex { x: 77, y: 39 },
    HersheyVertex { x: 49, y: 39 },
    HersheyVertex { x: 35, y: 25 },
    HERSHEY_LIFT,
    HersheyVertex { x: 77, y: -10 },
    HersheyVertex { x: 56, y: -10 },
    HERSHEY_LIFT,
    HersheyVertex { x: 77, y: 25 },
    HersheyVertex { x: 56, y: 25 },
    HERSHEY_LIFT,
    HersheyVertex { x: 98, y: 25 },
    HersheyVertex { x: 105, y: 18 },
    HersheyVertex { x: 105, y: 11 },
    HersheyVertex { x: 77, y: 11 },
    HersheyVertex { x: 77, y: 4 },
    HersheyVertex { x: 105, y: 4 },
    HersheyVertex { x: 105, y: -3 },
];

const CLASSIC_TILDE: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 112,
    vertices: &CLASSIC_TILDE_VERTICES,
};

static CLASSIC_DEL_VERTICES: [HersheyVertex; 1] = [HERSHEY_LIFT];

const CLASSIC_DEL: HersheyGlyph = HersheyGlyph {
    left: -7,
    right: 7,
    vertices: &CLASSIC_DEL_VERTICES,
};

static CLASSIC_GLYPHS: [HersheyGlyph; 128] = [
    CLASSIC_SPACE,    // 0x00
    CLASSIC_HAPPY,    // 0x01
    CLASSIC_SAD,      // 0x02
    CLASSIC_HEART,    // 0x03
    CLASSIC_DIAMOND,  // 0x04
    CLASSIC_CLUB,     // 0x05
    CLASSIC_SPADE,    // 0x06
    CLASSIC_BULLET,   // 0x07
    CLASSIC_SPACE,    // 0x08
    CLASSIC_CIRCLE,   // 0x09
    CLASSIC_SPACE,    // 0x0A
    CLASSIC_SPACE,    // 0x0B
    CLASSIC_SPACE,    // 0x0C
    CLASSIC_SPACE,    // 0x0D
    CLASSIC_SPACE,    // 0x0E
    CLASSIC_SPACE,    // 0x0F
    CLASSIC_TRI_R,    // 0x10
    CLASSIC_TRI_L,    // 0x11
    CLASSIC_ARROW_UD, // 0x12
    CLASSIC_SPACE,    // 0x13
    CLASSIC_SPACE,    // 0x14
    CLASSIC_SPACE,    // 0x15
    CLASSIC_SPACE,    // 0x16
    CLASSIC_SPACE,    // 0x17
    CLASSIC_ARROW_U,  // 0x18
    CLASSIC_ARROW_D,  // 0x19
    CLASSIC_ARROW_R,  // 0x1A
    CLASSIC_ARROW_L,  // 0x1B
    CLASSIC_SPACE,    // 0x1C
    CLASSIC_ARROW_LR, // 0x1D
    CLASSIC_TRI_U,    // 0x1E
    CLASSIC_TRI_D,    // 0x1F
    CLASSIC_SPACE,    // 0x20
    CLASSIC_EXPT,     // 0x21
    CLASSIC_QUOTE2,   // 0x22
    CLASSIC_HASH,     // 0x23
    CLASSIC_DOLLAR,   // 0x24
    CLASSIC_PCT,      // 0x25
    CLASSIC_AND,      // 0x26
    CLASSIC_QUOTE1,   // 0x27
    CLASSIC_LPAREN,   // 0x28
    CLASSIC_RPAREN,   // 0x29
    CLASSIC_STAR,     // 0x2A
    CLASSIC_PLUS,     // 0x2B
    CLASSIC_COMMA,    // 0x2C
    CLASSIC_DASH,     // 0x2D
    CLASSIC_DOT,      // 0x2E
    CLASSIC_SLASH,    // 0x2F
    CLASSIC_0,        // 0x30
    CLASSIC_1,        // 0x31
    CLASSIC_2,        // 0x32
    CLASSIC_3,        // 0x33
    CLASSIC_4,        // 0x34
    CLASSIC_5,        // 0x35
    CLASSIC_6,        // 0x36
    CLASSIC_7,        // 0x37
    CLASSIC_8,        // 0x38
    CLASSIC_9,        // 0x39
    CLASSIC_COLON,    // 0x3A
    CLASSIC_SEMI,     // 0x3B
    CLASSIC_LESS,     // 0x3C
    CLASSIC_EQUAL,    // 0x3D
    CLASSIC_GREAT,    // 0x3E
    CLASSIC_QUESTION, // 0x3F
    CLASSIC_AT,       // 0x40
    CLASSIC_A,        // 0x41
    CLASSIC_B,        // 0x42
    CLASSIC_C,        // 0x43
    CLASSIC_D,        // 0x44
    CLASSIC_E,        // 0x45
    CLASSIC_F,        // 0x46
    CLASSIC_G,        // 0x47
    CLASSIC_H,        // 0x48
    CLASSIC_I,        // 0x49
    CLASSIC_J,        // 0x4A
    CLASSIC_K,        // 0x4B
    CLASSIC_L,        // 0x4C
    CLASSIC_M,        // 0x4D
    CLASSIC_N,        // 0x4E
    CLASSIC_O,        // 0x4F
    CLASSIC_P,        // 0x50
    CLASSIC_Q,        // 0x51
    CLASSIC_R,        // 0x52
    CLASSIC_S,        // 0x53
    CLASSIC_T,        // 0x54
    CLASSIC_U,        // 0x55
    CLASSIC_V,        // 0x56
    CLASSIC_W,        // 0x57
    CLASSIC_X,        // 0x58
    CLASSIC_Y,        // 0x59
    CLASSIC_Z,        // 0x5A
    CLASSIC_LBRACK,   // 0x5B
    CLASSIC_BSLASH,   // 0x5C
    CLASSIC_RBRACK,   // 0x5D
    CLASSIC_CARAT,    // 0x5E
    CLASSIC_UNDER,    // 0x5F
    CLASSIC_ACUTE,    // 0x60
    CLASSIC_A_LOWER,  // 0x61
    CLASSIC_B_LOWER,  // 0x62
    CLASSIC_C_LOWER,  // 0x63
    CLASSIC_D_LOWER,  // 0x64
    CLASSIC_E_LOWER,  // 0x65
    CLASSIC_F_LOWER,  // 0x66
    CLASSIC_G_LOWER,  // 0x67
    CLASSIC_H_LOWER,  // 0x68
    CLASSIC_I_LOWER,  // 0x69
    CLASSIC_J_LOWER,  // 0x6A
    CLASSIC_K_LOWER,  // 0x6B
    CLASSIC_L_LOWER,  // 0x6C
    CLASSIC_M_LOWER,  // 0x6D
    CLASSIC_N_LOWER,  // 0x6E
    CLASSIC_O_LOWER,  // 0x6F
    CLASSIC_P_LOWER,  // 0x70
    CLASSIC_Q_LOWER,  // 0x71
    CLASSIC_R_LOWER,  // 0x72
    CLASSIC_S_LOWER,  // 0x73
    CLASSIC_T_LOWER,  // 0x74
    CLASSIC_U_LOWER,  // 0x75
    CLASSIC_V_LOWER,  // 0x76
    CLASSIC_W_LOWER,  // 0x77
    CLASSIC_X_LOWER,  // 0x78
    CLASSIC_Y_LOWER,  // 0x79
    CLASSIC_Z_LOWER,  // 0x7A
    CLASSIC_LBRACE,   // 0x7B
    CLASSIC_BAR,      // 0x7C
    CLASSIC_RBRACE,   // 0x7D
    CLASSIC_TILDE,    // 0x7E
    CLASSIC_DEL,      // 0x7F
];

pub static FONT_CLASSIC: HersheyFont = HersheyFont {
    name: "Classic",
    first_char: 0x00,
    glyphs: &CLASSIC_GLYPHS,
    space_advance: 35,
    line_height: 50,
    placeholder_glyph: &CLASSIC_STAR,
    units_per_em: 20.0,
};
