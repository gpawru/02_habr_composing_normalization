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
/// количество ведущих согласных
pub const HANGUL_L_COUNT: u32 = 19;
/// количество гласных
pub const HANGUL_V_COUNT: u32 = 21;
/// количество завершающих согласных (-1)
pub const HANGUL_T_COUNT: u32 = 28;
/// количество гласных * количество завершающих согласных
pub const HANGUL_N_COUNT: u32 = 588;
/// количество слогов хангыль в Unicode (-1)
pub const HANGUL_S_COUNT: u32 = 11171;

// /// декомпозция хангыль
// #[inline(never)]
// pub fn decompose_hangul(lvt: u32) -> DecompositionValue
// {
//     let l = lvt / HANGUL_N_COUNT;
//     let v = (lvt % HANGUL_N_COUNT) / HANGUL_T_COUNT;
//     let t = lvt % HANGUL_T_COUNT;

//     let c0 = HANGUL_L_BASE + l;
//     let c1 = HANGUL_V_BASE + v;

//     match t == 0 {
//         true => DecompositionValue::HangulPair(c0, c1),
//         false => DecompositionValue::HangulTriple(c0, c1, HANGUL_T_BASE + t),
//     }
// }
