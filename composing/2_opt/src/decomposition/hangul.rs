#![allow(dead_code)]

use super::DecompositionValue;

/// начало блока слогов хангыль
pub const HANGUL_S_BASE: u32 = 0xAC00;
/// начало блока ведущих согласных чамо
pub const HANGUL_L_BASE: u32 = 0x1100;
/// начало блока гласных чамо
pub const HANGUL_V_BASE: u32 = 0x1161;
/// начало блока завершающих согласных (на 1 меньше, см. спецификацию)
pub const HANGUL_T_BASE: u32 = 0x11A7;
/// количество ведущих согласных (может быть полезно знать, по пока не используется)
pub const HANGUL_L_COUNT: u32 = 19;
/// количество гласных (может быть полезно знать, но пока не используется)
pub const HANGUL_V_COUNT: u32 = 21;
/// количество завершающих согласных (-1)
pub const HANGUL_T_COUNT: u32 = 28;
/// количество гласных * количество завершающих согласных
pub const HANGUL_N_COUNT: u32 = 588;
/// количество слогов хангыль в Unicode (-1)
pub const HANGUL_S_COUNT: u32 = 11171;

/// декомпозция хангыль
#[inline(never)]
pub fn decompose_hangul(lvt: u32) -> DecompositionValue
{
    let l = (lvt / HANGUL_N_COUNT) as u8;
    let v = ((lvt % HANGUL_N_COUNT) / HANGUL_T_COUNT) as u8;
    let t = (lvt % HANGUL_T_COUNT) as u8;

    let c0 = 0x80 + l;
    let c1 = 0xA1 + v;

    if t == 0 {
        return DecompositionValue::HangulPair(c0, c1);
    }

    let c2 = 0x86 | ((0x07 + t) >> 5);
    let c3 = 0x80 | ((0xA7 + t) & 0x3F);

    DecompositionValue::HangulTriple(c0, c1, c2, c3)
}
