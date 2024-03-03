use crate::write_char;

// в блоке чамо (U+1100..U+11FF) могут быть скомбинированы кодпоинты:
//  - U+1100..=U+1112 (L, ведущие согласные)
//  - U+1161..=U+1176 (V, гласные)
//  - U+11A8..=U+11C3 (T, завершающие согласные)
// все они находятся в пределах диапазона U+1100..=U+11C3 (196 кодпоинтов)

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

/// скомбинировать чамо хангыль (V / T) с предыдущим кодпоинтом
#[inline(always)]
pub fn combine_and_write_hangul_vt(result: &mut String, jamo: u32)
{
    let code = match result.pop() {
        Some(code) => u32::from(code),
        None => {
            write_char(result, jamo);
            return;
        }
    };

    match get_vt(jamo) {
        HangulVT::Vowel(v) => {
            let l = code.wrapping_sub(HANGUL_L_BASE);

            match l < HANGUL_L_COUNT {
                true => {
                    let lv = HANGUL_S_BASE + l * HANGUL_N_COUNT + v * HANGUL_T_BLOCK_SIZE;
                    write_char(result, lv);
                }
                false => {
                    write_char(result, code);
                    write_char(result, HANGUL_V_BASE + v);
                }
            }
        }
        HangulVT::TrailingConsonant(t) => {
            let lv = code.wrapping_sub(HANGUL_S_BASE);

            match lv < HANGUL_S_COUNT && lv % HANGUL_T_BLOCK_SIZE == 0 {
                true => {
                    write_char(result, code + t + 1);
                }
                false => {
                    write_char(result, code);
                    write_char(result, HANGUL_T_BASE + t);
                }
            }
        }
    }
}

/// мы знаем, что кодпоинт является гласной или завершающей согласной чамо хангыль, получаем значения
#[inline(always)]
fn get_vt(code: u32) -> HangulVT
{
    let v = code.wrapping_sub(HANGUL_V_BASE);

    match v < HANGUL_V_COUNT {
        true => HangulVT::Vowel(v),
        false => {
            let t = v.wrapping_sub(HANGUL_T_BASE - HANGUL_V_BASE);
            HangulVT::TrailingConsonant(t)
        }
    }
}
