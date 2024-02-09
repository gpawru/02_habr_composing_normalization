use crate::codepoint::Codepoint;
use crate::write;

// в блоке чамо (U+1100..U+11FF) могут быть скомбинированы кодпоинты:
//  - U+1100..=U+1112 (L, ведущие согласные)
//  - U+1161..=U+1176 (V, гласные)
//  - U+11A8..=U+11C3 (T, завершающие согласные)
// все они находятся в пределах диапазона U+1100..=U+11C3 (196 кодпоинтов)

/// количество кодпоинтов между U+1161 и U+11C3, используется для проверки,
/// может ли кодпоинт быть гласной или завершающей согласной чамо
const HANGUL_VT_JAMO_COUNT: u32 = 99;

/// начало блока ведущих согласных чамо
const HANGUL_L_BASE: u32 = 0x1100;
/// количество ведущих согласных
const HANGUL_L_COUNT: u32 = 19;
/// начало блока гласных чамо
const HANGUL_V_BASE: u32 = 0x1161;
/// количество гласных
const HANGUL_V_COUNT: u32 = 21;
/// начало блока завершающих согласных
const HANGUL_T_BASE: u32 = 0x11A8;
/// количество завершающих согласных
const HANGUL_T_COUNT: u32 = 27;
/// количество кодпоинтов на блок LV
const HANGUL_T_BLOCK_SIZE: u32 = HANGUL_T_COUNT + 1;
/// начало блока слогов хангыль
const HANGUL_S_BASE: u32 = 0xAC00;
/// количество слогов хангыль в Unicode
const HANGUL_S_COUNT: u32 = 11172;
/// количество гласных * количество завершающих согласных
const HANGUL_N_COUNT: u32 = 588;

/// кодпоинт хангыль, который может быть скомбинирован с идущим перед ним гласной чамо или слогом LV
#[derive(Debug, PartialEq)]
pub enum HangulVT
{
    /// гласная (V - отступ от начала блока гласных)
    Vowel(u32),
    /// завершающая согласная (T - отступ от начала блока завершающих согласных)
    TrailingConsonant(u32),
}

/// является ли кодпоинт гласной или завершающей согласной чамо хангыль
#[inline(always)]
pub fn is_hangul_vt(code: u32) -> Option<HangulVT>
{
    let v = code.wrapping_sub(HANGUL_V_BASE);

    if v >= HANGUL_VT_JAMO_COUNT {
        return None;
    }

    // if v < HANGUL_VT_JAMO_COUNT {
    if v < HANGUL_V_COUNT {
        return Some(HangulVT::Vowel(v));
    }

    let t = v.wrapping_sub(HANGUL_T_BASE - HANGUL_V_BASE);

    match t < HANGUL_T_COUNT {
        true => Some(HangulVT::TrailingConsonant(t)),
        false => None,
    }
    // }

    // None
}

/// скомбинировать и записать кодпоинт хангыль, предполагается, что в буфере один кодпоинт
#[inline(always)]
pub fn combine_and_write_hangul_vt(code: u32, result: &mut String, vt: HangulVT)
{
    match vt {
        HangulVT::Vowel(v) => {
            let l = code.wrapping_sub(HANGUL_L_BASE);

            match l < HANGUL_L_COUNT {
                true => {
                    let lv = HANGUL_S_BASE + l * HANGUL_N_COUNT + v * HANGUL_T_BLOCK_SIZE;

                    write!(result, lv);
                }
                false => {
                    write!(result, code);
                    write!(result, HANGUL_V_BASE + v);
                }
            }
        }
        HangulVT::TrailingConsonant(t) => {
            let lv = code.wrapping_sub(HANGUL_S_BASE);

            match lv < HANGUL_S_COUNT && lv % HANGUL_T_BLOCK_SIZE == 0 {
                true => {
                    write!(result, code + t + 1);
                }
                false => {
                    write!(result, code);
                    write!(result, HANGUL_T_BASE + t);
                }
            }
        }
    }
}
