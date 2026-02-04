#![no_std]

use core::{cmp, fmt};
use rgb::*;
use glam::IVec2;
use crate::platform::Shape;

pub const LIFT: u8 = 0xFF;
// the f_ means font_
pub const F_SPACE: &[u8] = &[ LIFT ]; // Character 0x00 (often NUL, using as space here)
pub const F_HAPPY: &[u8] = &[ 0x10, 0x01, 0x05, 0x16, 0x46, 0x55, 0x51, 0x40, 0x10, LIFT, 0x22, LIFT, 0x32, LIFT, 0x14, 0x25, 0x35, 0x44 ]; // 0x01
pub const F_SAD: &[u8] = &[ 0x10, 0x01, 0x05, 0x16, 0x46, 0x55, 0x51, 0x40, 0x10, LIFT, 0x22, LIFT, 0x32, LIFT, 0x15, 0x24, 0x34, 0x45 ]; // 0x02
pub const F_HEART: &[u8] = &[ 0x10, 0x21, 0x30, 0x41, 0x43, 0x26, 0x03, 0x01, 0x10 ]; // 0x03
pub const F_DIAMOND: &[u8] = &[ 0x20, 0x43, 0x26, 0x03, 0x20 ]; // 0x04
pub const F_CLUB: &[u8] = &[ 0x20, 0x31, 0x32, 0x23, 0x12, 0x11, 0x20, LIFT, 0x12, 0x03, 0x04, 0x15, 0x24, 0x16, 0x36, 0x24, 0x35, 0x44, 0x43, 0x32 ]; // 0x05
pub const F_SPADE: &[u8] = &[ 0x20, 0x32, 0x43, 0x44, 0x35, 0x24, 0x36, 0x16, 0x24, 0x15, 0x04, 0x03, 0x12, 0x20 ]; // 0x06
pub const F_BULLET: &[u8] = &[ 0x12, 0x23, 0x14, 0x03, 0x12 ]; // 0x07
// 0x08 backspace - using space for now
pub const F_CIRCLE: &[u8] = &[ 0x11, 0x02, 0x03, 0x14, 0x24, 0x33, 0x32, 0x21, 0x11 ]; // 0x09 TAB - using circle
// 0x0A line feed - handled in code
// 0x0B VT - using space
// 0x0C FF - using space
// 0x0D CR - handled in code (often ignored if LF present)
// 0x0E SO - using space
// 0x0F SI - using space
pub const F_TRI_R: &[u8] = &[ 0x00, 0x06, 0x33, 0x00 ]; // 0x10
pub const F_TRI_L: &[u8] = &[ 0x03, 0x30, 0x36, 0x03 ]; // 0x11
pub const F_ARROW_UD: &[u8] = &[ 0x20, 0x02, LIFT, 0x20, 0x42, LIFT, 0x26, 0x04, LIFT, 0x26, 0x44, LIFT, 0x20, 0x26 ]; // 0x12
// 0x13 XON - using space
// 0x14 DC4 - using space
// 0x15 NAK - using space
// 0x16 SYN - using space
// 0x17 ETB - using space
pub const F_ARROW_U: &[u8] = &[ 0x20, 0x02, LIFT, 0x20, 0x42, LIFT, 0x20, 0x26 ]; // 0x18 CAN
pub const F_ARROW_D: &[u8] = &[ 0x26, 0x04, LIFT, 0x26, 0x44, LIFT, 0x20, 0x26 ]; // 0x19 EM
pub const F_ARROW_R: &[u8] = &[ 0x43, 0x21, LIFT, 0x43, 0x25, LIFT, 0x03, 0x43 ]; // 0x1A SUB
pub const F_ARROW_L: &[u8] = &[ 0x03, 0x21, LIFT, 0x03, 0x25, LIFT, 0x03, 0x43 ]; // 0x1B ESC
// 0x1C FS - using space
pub const F_ARROW_LR: &[u8] = &[ 0x03, 0x11, LIFT, 0x03, 0x15, LIFT, 0x43, 0x31, LIFT, 0x43, 0x35, LIFT, 0x03, 0x43 ]; // 0x1D GS
pub const F_TRI_U: &[u8] = &[ 0x20, 0x46, 0x06, 0x20 ]; // 0x1E RS
pub const F_TRI_D: &[u8] = &[ 0x26, 0x00, 0x40, 0x26 ]; // 0x1F US
// ASCII character definitions (0x20 - 0x7E)
pub const F_EXPT: &[u8] = &[ 0x00, 0x04, LIFT, 0x06, LIFT ]; // 0x21 !
pub const F_QUOTE2: &[u8] = &[ 0x00, 0x02, LIFT, 0x10, 0x12 ]; // 0x22 "
pub const F_HASH: &[u8] = &[ 0x10, 0x16, LIFT, 0x30, 0x36, LIFT, 0x02, 0x42, LIFT, 0x04, 0x44 ]; // 0x23 #
pub const F_DOLLAR: &[u8] = &[ 0x41, 0x11, 0x02, 0x13, 0x33, 0x44, 0x35, 0x05, LIFT, 0x20, 0x26 ]; // 0x24 $
pub const F_PCT: &[u8] = &[ 0x21, 0x05, LIFT, 0x01, LIFT, 0x25, LIFT ]; // 0x25 %
pub const F_AND: &[u8] = &[ 0x31, 0x20, 0x10, 0x01, 0x02, 0x46, LIFT, 0x13, 0x04, 0x05, 0x16, 0x26, 0x44 ]; // 0x26 &
pub const F_QUOTE1: &[u8] = &[ 0x00, 0x02 ]; // 0x27 '
pub const F_LPAREN: &[u8] = &[ 0x10, 0x01, 0x05, 0x16 ]; // 0x28 (
pub const F_RPAREN: &[u8] = &[ 0x00, 0x11, 0x15, 0x06 ]; // 0x29 )
pub const F_STAR: &[u8] = &[ 0x01, 0x45, LIFT, 0x21, 0x25, LIFT, 0x41, 0x05, LIFT, 0x03, 0x43 ]; // 0x2A *
pub const F_PLUS: &[u8] = &[ 0x21, 0x25, LIFT, 0x03, 0x43 ]; // 0x2B +
pub const F_COMMA: &[u8] = &[ 0x16, LIFT, 0x16, 0x07 ]; // 0x2C ,
pub const F_DASH: &[u8] = &[ 0x03, 0x43 ]; // 0x2D -
pub const F_DOT: &[u8] = &[ 0x06, LIFT ]; // 0x2E .
pub const F_SLASH: &[u8] = &[ 0x40, 0x06 ]; // 0x2F /
pub const F_0: &[u8] = &[ 0x10, 0x01, 0x05, 0x16, 0x36, 0x45, 0x41, 0x30, 0x10 ]; // 0x30 0
pub const F_1: &[u8] = &[ 0x03, 0x20, 0x26, LIFT, 0x06, 0x46 ]; // 0x31 1
pub const F_2: &[u8] = &[ 0x01, 0x10, 0x30, 0x41, 0x42, 0x06, 0x46 ]; // 0x32 2
pub const F_3: &[u8] = &[ 0x01, 0x10, 0x30, 0x41, 0x42, 0x33, 0x23, LIFT, 0x33, 0x44, 0x45, 0x36, 0x16, 0x05 ]; // 0x33 3
pub const F_4: &[u8] = &[ 0x36, 0x30, 0x03, 0x04, 0x44 ]; // 0x34 4
pub const F_5: &[u8] = &[ 0x40, 0x00, 0x03, 0x33, 0x44, 0x45, 0x36, 0x16, 0x05 ]; // 0x35 5
pub const F_6: &[u8] = &[ 0x30, 0x20, 0x02, 0x05, 0x16, 0x36, 0x45, 0x44, 0x33, 0x13, 0x04 ]; // 0x36 6
pub const F_7: &[u8] = &[ 0x00, 0x40, 0x16 ]; // 0x37 7
pub const F_8: &[u8] = &[ 0x13, 0x02, 0x01, 0x10, 0x30, 0x41, 0x42, 0x33, 0x44, 0x45, 0x36, 0x16, 0x05, 0x04, 0x13, 0x33 ]; // 0x38 8
pub const F_9: &[u8] = &[ 0x42, 0x33, 0x13, 0x02, 0x01, 0x10, 0x30, 0x41, 0x44, 0x26, 0x16 ]; // 0x39 9
pub const F_COLON: &[u8] = &[ 0x02, LIFT, 0x04, LIFT ]; // 0x3A :
pub const F_SEMI: &[u8] = &[ 0x13, LIFT, 0x16, LIFT, 0x16, 0x07 ]; // 0x3B ;
pub const F_LESS: &[u8] = &[ 0x21, 0x03, 0x25 ]; // 0x3C <
pub const F_EQUAL: &[u8] = &[ 0x02, 0x42, LIFT, 0x04, 0x44 ]; // 0x3D =
pub const F_GREAT: &[u8] = &[ 0x01, 0x23, 0x05 ]; // 0x3E >
pub const F_QUESTION: &[u8] = &[ 0x01, 0x10, 0x30, 0x41, 0x23, 0x24, LIFT, 0x26, LIFT ]; // 0x3F ?
pub const F_AT: &[u8] = &[ 0x33, 0x22, 0x13, 0x14, 0x25, 0x35, 0x32, 0x21, 0x11, 0x02, 0x05, 0x16, 0x36 ]; // 0x40 @
pub const F_A: &[u8] = &[ 0x06, 0x20, 0x46, LIFT, 0x13, 0x33 ]; // 0x41 A
pub const F_B: &[u8] = &[ 0x23, 0x32, 0x31, 0x20, 0x00, 0x06, 0x36, 0x45, 0x44, 0x33, 0x03 ]; // 0x42 B
pub const F_C: &[u8] = &[ 0x41, 0x30, 0x10, 0x01, 0x05, 0x16, 0x36, 0x45 ]; // 0x43 C
pub const F_D: &[u8] = &[ 0x00, 0x06, 0x26, 0x44, 0x42, 0x20, 0x00 ]; // 0x44 D
pub const F_E: &[u8] = &[ 0x40, 0x00, 0x06, 0x46, LIFT, 0x03, 0x33 ]; // 0x45 E
pub const F_F: &[u8] = &[ 0x40, 0x00, 0x06, LIFT, 0x03, 0x33 ]; // 0x46 F
pub const F_G: &[u8] = &[ 0x41, 0x30, 0x10, 0x01, 0x05, 0x16, 0x36, 0x45, 0x43, 0x23 ]; // 0x47 G
pub const F_H: &[u8] = &[ 0x00, 0x06, LIFT, 0x40, 0x46, LIFT, 0x03, 0x43 ]; // 0x48 H
pub const F_I: &[u8] = &[ 0x10, 0x30, LIFT, 0x16, 0x36, LIFT, 0x20, 0x26 ]; // 0x49 I
pub const F_J: &[u8] = &[ 0x30, 0x40, 0x45, 0x36, 0x16, 0x05 ]; // 0x4A J
pub const F_K: &[u8] = &[ 0x00, 0x06, LIFT, 0x30, 0x03, 0x13, 0x46 ]; // 0x4B K
pub const F_L: &[u8] = &[ 0x00, 0x06, 0x46 ]; // 0x4C L
pub const F_M: &[u8] = &[ 0x06, 0x00, 0x24, 0x40, 0x46 ]; // 0x4D M
pub const F_N: &[u8] = &[ 0x06, 0x00, 0x46, 0x40 ]; // 0x4E N
pub const F_O: &[u8] = &[ 0x10, 0x01, 0x05, 0x16, 0x36, 0x45, 0x41, 0x30, 0x10 ]; // 0x4F O
pub const F_P: &[u8] = &[ 0x06, 0x00, 0x30, 0x41, 0x42, 0x33, 0x03 ]; // 0x50 P
pub const F_Q: &[u8] = &[ 0x10, 0x01, 0x05, 0x16, 0x36, 0x45, 0x41, 0x30, 0x10, LIFT, 0x24, 0x46 ]; // 0x51 Q
pub const F_R: &[u8] = &[ 0x06, 0x00, 0x30, 0x41, 0x42, 0x33, 0x03, LIFT, 0x13, 0x46 ]; // 0x52 R
pub const F_S: &[u8] = &[ 0x40, 0x30, 0x10, 0x01, 0x02, 0x13, 0x33, 0x44, 0x45, 0x36, 0x16, 0x06 ]; // 0x53 S
pub const F_T: &[u8] = &[ 0x00, 0x40, LIFT, 0x20, 0x26 ]; // 0x54 T
pub const F_U: &[u8] = &[ 0x00, 0x05, 0x16, 0x36, 0x45, 0x40 ]; // 0x55 U
pub const F_V: &[u8] = &[ 0x00, 0x26, 0x40 ]; // 0x56 V
pub const F_W: &[u8] = &[ 0x00, 0x06, 0x23, 0x46, 0x40 ]; // 0x57 W
pub const F_X: &[u8] = &[ 0x00, 0x46, LIFT, 0x40, 0x06 ]; // 0x58 X
pub const F_Y: &[u8] = &[ 0x00, 0x23, 0x26, LIFT, 0x40, 0x23 ]; // 0x59 Y
pub const F_Z: &[u8] = &[ 0x00, 0x40, 0x06, 0x46 ]; // 0x5A Z
pub const F_LBRACK: &[u8] = &[ 0x20, 0x00, 0x06, 0x26 ]; // 0x5B [
pub const F_BSLASH: &[u8] = &[ 0x00, 0x46 ]; // 0x5C ''
pub const F_RBRACK: &[u8] = &[ 0x00, 0x20, 0x26, 0x06 ]; // 0x5D ]
pub const F_CARAT: &[u8] = &[ 0x02, 0x11, 0x22 ]; // 0x5E ^
pub const F_UNDER: &[u8] = &[ 0x06, 0x46 ]; // 0x5F _
pub const F_ACUTE: &[u8] = &[ 0x00, 0x22 ]; // 0x60 `
pub const F_A_LOWER: &[u8] = &[ 0x35, 0x26, 0x16, 0x05, 0x04, 0x13, 0x33, 0x36 ]; // 0x61 a
pub const F_B_LOWER: &[u8] = &[ 0x00, 0x06, 0x26, 0x35, 0x34, 0x23, 0x03 ]; // 0x62 b
pub const F_C_LOWER: &[u8] = &[ 0x33, 0x13, 0x04, 0x05, 0x16, 0x36 ]; // 0x63 c
pub const F_D_LOWER: &[u8] = &[ 0x33, 0x13, 0x04, 0x05, 0x16, 0x36, 0x30 ]; // 0x64 d
pub const F_E_LOWER: &[u8] = &[ 0x05, 0x34, 0x23, 0x13, 0x04, 0x05, 0x16, 0x36 ]; // 0x65 e
pub const F_F_LOWER: &[u8] = &[ 0x30, 0x20, 0x11, 0x16, LIFT, 0x03, 0x23 ]; // 0x66 f
pub const F_G_LOWER: &[u8] = &[ 0x36, 0x16, 0x05, 0x04, 0x13, 0x33, 0x37, 0x28, 0x08 ]; // 0x67 g
pub const F_H_LOWER: &[u8] = &[ 0x00, 0x06, LIFT, 0x03, 0x23, 0x34, 0x36 ]; // 0x68 h
pub const F_I_LOWER: &[u8] = &[ 0x01, LIFT, 0x03, 0x06 ]; // 0x69 i
pub const F_J_LOWER: &[u8] = &[ 0x21, LIFT, 0x23, 0x27, 0x18, 0x08 ]; // 0x6A j
pub const F_K_LOWER: &[u8] = &[ 0x00, 0x06, LIFT, 0x22, 0x04, 0x14, 0x36 ]; // 0x6B k
pub const F_L_LOWER: &[u8] = &[ 0x00, 0x06 ]; // 0x6C l
pub const F_M_LOWER: &[u8] = &[ 0x06, 0x03, 0x13, 0x24, LIFT, 0x25, 0x23, 0x33, 0x44, 0x46 ]; // 0x6D m
pub const F_N_LOWER: &[u8] = &[ 0x03, 0x06, LIFT, 0x04, 0x13, 0x23, 0x34, 0x36 ]; // 0x6E n
pub const F_O_LOWER: &[u8] = &[ 0x13, 0x04, 0x05, 0x16, 0x26, 0x35, 0x34, 0x23, 0x13 ]; // 0x6F o
pub const F_P_LOWER: &[u8] = &[ 0x08, 0x03, 0x23, 0x34, 0x35, 0x26, 0x16, 0x05 ]; // 0x70 p
pub const F_Q_LOWER: &[u8] = &[ 0x35, 0x26, 0x16, 0x05, 0x04, 0x13, 0x23, 0x34, LIFT, 0x33, 0x38, 0x47 ]; // 0x71 q
pub const F_R_LOWER: &[u8] = &[ 0x03, 0x06, LIFT, 0x04, 0x13, 0x23, 0x34 ]; // 0x72 r
pub const F_S_LOWER: &[u8] = &[ 0x33, 0x13, 0x04, 0x35, 0x26, 0x06 ]; // 0x73 s
pub const F_T_LOWER: &[u8] = &[ 0x11, 0x15, 0x26, LIFT, 0x03, 0x23 ]; // 0x74 t
pub const F_U_LOWER: &[u8] = &[ 0x03, 0x05, 0x16, 0x26, 0x35, LIFT, 0x33, 0x36 ]; // 0x75 u
pub const F_V_LOWER: &[u8] = &[ 0x03, 0x16, 0x26, 0x33 ]; // 0x76 v
pub const F_W_LOWER: &[u8] = &[ 0x03, 0x05, 0x16, 0x25, 0x24, LIFT, 0x25, 0x36, 0x45, 0x43 ]; // 0x77 w
pub const F_X_LOWER: &[u8] = &[ 0x03, 0x36, LIFT, 0x33, 0x06 ]; // 0x78 x
pub const F_Y_LOWER: &[u8] = &[ 0x03, 0x05, 0x16, 0x26, 0x35, LIFT, 0x33, 0x37, 0x28, 0x18, 0x07 ]; // 0x79 y
pub const F_Z_LOWER: &[u8] = &[ 0x03, 0x33, 0x06, 0x36 ]; // 0x7A z
pub const F_LBRACE: &[u8] = &[ 0x20, 0x11, 0x13, 0x04, 0x15, 0x17, 0x28 ]; // 0x7B {
pub const F_BAR: &[u8] = &[ 0x00, 0x08 ]; // 0x7C |
pub const F_RBRACE: &[u8] = &[ 0x00, 0x11, 0x13, 0x24, 0x15, 0x17, 0x08 ]; // 0x7D ]
pub const F_TILDE: &[u8] = &[ 0x10, 0x21, 0x12, 0x01, 0x10 ]; // 0x7E ~
pub const F_DEL: &[u8] = &[ LIFT ]; // 0x7F DEL

pub const FONT_CHARS: [&[u8]; 0x80] = [
        F_SPACE,    F_HAPPY,    F_SAD,      F_HEART,      // 00-03
        F_DIAMOND,  F_CLUB,     F_SPADE,    F_BULLET,     // 04-07
        F_SPACE,    F_CIRCLE,   F_SPACE,    F_SPACE,      // 08-0B (BS, TAB)
        F_SPACE,    F_SPACE,    F_SPACE,    F_SPACE,      // 0C-0F
        F_TRI_R,    F_TRI_L,    F_ARROW_UD, F_SPACE,      // 10-13
        F_SPACE,    F_SPACE,    F_SPACE,    F_SPACE,      // 14-17
        F_ARROW_U,  F_ARROW_D,  F_ARROW_R,  F_ARROW_L,    // 18-1B (CAN,EM,SUB,ESC)
        F_SPACE,    F_ARROW_LR, F_TRI_U,    F_TRI_D,      // 1C-1F
        // ASCII (0x20 - 0x7F)
        F_SPACE,  F_EXPT,   F_QUOTE2, F_HASH,      // 20-23
        F_DOLLAR, F_PCT,    F_AND,    F_QUOTE1,     // 24-27
        F_LPAREN, F_RPAREN, F_STAR,   F_PLUS,       // 28-2B
        F_COMMA,  F_DASH,   F_DOT,    F_SLASH,      // 2C-2F
        F_0,      F_1,      F_2,      F_3,          // 30-33
        F_4,      F_5,      F_6,      F_7,          // 34-37
        F_8,      F_9,      F_COLON,  F_SEMI,       // 38-3B
        F_LESS,   F_EQUAL,  F_GREAT,  F_QUESTION,   // 3C-3F
        F_AT,     F_A,      F_B,      F_C,          // 40-43
        F_D,      F_E,      F_F,      F_G,          // 44-47
        F_H,      F_I,      F_J,      F_K,          // 48-4B
        F_L,      F_M,      F_N,      F_O,          // 4C-4F
        F_P,      F_Q,      F_R,      F_S,          // 50-53
        F_T,      F_U,      F_V,      F_W,          // 54-57
        F_X,      F_Y,      F_Z,      F_LBRACK,     // 58-5B
        F_BSLASH, F_RBRACK, F_CARAT,  F_UNDER,      // 5C-5F
        F_ACUTE,  F_A_LOWER,      F_B_LOWER,      F_C_LOWER,          // 60-63
        F_D_LOWER,      F_E_LOWER,      F_F_LOWER,      F_G_LOWER,          // 64-67
        F_H_LOWER,      F_I_LOWER,      F_J_LOWER,      F_K_LOWER,          // 68-6B
        F_L_LOWER,      F_M_LOWER,      F_N_LOWER,      F_O_LOWER,          // 6C-6F
        F_P_LOWER,      F_Q_LOWER,      F_R_LOWER,      F_S_LOWER,          // 70-73
        F_T_LOWER,      F_U_LOWER,      F_V_LOWER,      F_W_LOWER,          // 74-77
        F_X_LOWER,      F_Y_LOWER,      F_Z_LOWER,      F_LBRACE,     // 78-7B
        F_BAR,    F_RBRACE, F_TILDE,  F_DEL,        // 7C-7F
];

pub fn get_char_def(code: u8) -> &'static[u8] {
    const FONT_CHAR_MIN: u8 = 0x00;
    const FONT_CHAR_MAX: u8 = 0x7F;
    if code >= FONT_CHAR_MIN && code <= FONT_CHAR_MAX {
        FONT_CHARS[(code - FONT_CHAR_MIN) as usize]
    } else {
        // Return definition for space
        FONT_CHARS[(0x20 - FONT_CHAR_MIN) as usize]
    }
}

pub fn draw_text(platform: &mut dyn crate::platform::IcPlatform, text: &str, x: f32, y: f32, scale: f32, color: RGB8) {
    if scale <= 0.0 {
        return
    }
    let mut current_x = x;
    let mut current_y = y;
    const CHAR_HEIGHT: u8 = 9;
    let advance_y: f32 = CHAR_HEIGHT as f32 * scale;
    for c in text.bytes() {
        if c == b'\n' {
            current_x = x;
            current_y += advance_y;
            continue;
        }
        if c == b'\r' {
            current_x = x;
            continue;
        }
        let fontchar: &'static [u8] = get_char_def(c);
        // start calcualting width as we go thru points so we know how
        // much to advance the x coordinate for the next character
        // todo
        let mut last_point_valid: bool = false;
        let mut last_sx: f32 = 0.0;
        let mut last_sy: f32 = 0.0;
        let mut pt_idx: usize = 0;
        let mut max_x: u8 = 0;
        for pt in fontchar.iter() {
            let p = *pt;
            if p == LIFT {
                last_point_valid = false;
            } else {
                // Decode grid coordinates (high nibble X, low nibble Y)
                let fx: u8 = (p >> 4) & 0x0F;
                let fy: u8 = p & 0x0F;
                // Scale coords to screen pixels relative to character origin
                let sx: f32 = current_x + fx as f32 * scale;
                let sy: f32 = current_y + fy as f32 * scale;
                if last_point_valid {
                    platform.draw_shape(Shape {
                        start: IVec2 { x: last_sx as i32, y: last_sy as i32 },
                        end: IVec2 { x: sx as i32, y: sy as i32 },
                        color: color
                    });
                } else {
                     // This is the first point after a LIFT or the start of the character data.
                    // Check if it's a standalone point (i.e., the next item is LIFT or end of data)
                    if pt_idx + 1 >= fontchar.len() || fontchar[pt_idx + 1] == LIFT {
                        // DOT!
                        //platform.draw_shape(sx - (scale * 0.5), sy - (scale * 0.5), sx + (scale * 0.5), sy + (scale * 0.5));
                        platform.draw_shape(Shape {
                            start: IVec2 { x: (sx - (scale * 0.5)) as i32, y: (sy - (scale * 0.5)) as i32 },
                            end: IVec2 { x: (sx + (scale * 0.5)) as i32, y: (sy + (scale * 0.5)) as i32 },
                            color: color
                        });
                    }
                    // If it's the start of a line segment (next point is not LIFT/end),
                    // we don't draw the point explicitly here. The olivec_line call
                    // in the *next* iteration will draw the line *starting* from this point.
                }
                // Update the last point for the next potential line segment
                last_sx = sx;
                last_sy = sy;
                last_point_valid = true;
                max_x = cmp::max(fx, max_x);
            }
            pt_idx += 1;
        }
        // override char width for some characters
        let char_width = match c {
            b' ' => { scale * 2.0 }
            _ => { max_x as f32 + 1.0 }
        };
        current_x += (char_width + 1.0) * scale;
    }
}

pub fn draw_text_f(platform: &mut dyn crate::platform::IcPlatform, arg: fmt::Arguments, x: f32, y: f32, scale: f32, color: RGB8) {
    let mut buf = [0u8; 128];
    draw_text(
        platform, 
        format_no_std::show(&mut buf, arg).unwrap(),
        x,
        y,
        scale,
        color
    );
}

pub fn text_to_pos(text: &str, x: f32, scale: f32, cursor: usize) -> f32 {
    if scale <= 0.0 || cursor == 0 {
        return x;
    }
    let mut current_x = x;
    const CHAR_HEIGHT: u8 = 9;
    let mut counter = 0;
    for c in text.bytes() {
        if c == b'\n' {
            current_x = x;
            continue;
        }
        if c == b'\r' {
            current_x = x;
            continue;
        }
        let fontchar: &'static [u8] = get_char_def(c);
        // start calcualting width as we go thru points so we know how
        // much to advance the x coordinate for the next character
        // todo
        let mut max_x: u8 = 0;
        for pt in fontchar.iter() {
            let p = *pt;
            if p != LIFT {
                let fx: u8 = (p >> 4) & 0x0F;
                max_x = cmp::max(fx, max_x);
            }
        }
        // override char width for some characters
        let char_width = match c {
            b' ' => { 9 }
            _ => { max_x + 1 }
        };
        current_x += (char_width as f32 + 1.0) * scale;
        counter += 1;
        if counter >= cursor {
            return current_x;
        }
    }
    current_x
}