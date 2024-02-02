/// начало блока слогов хангыль
pub const HANGUL_S_BASE: u32 = 0xAC00;
/// начало блока ведущих согласных чамо
pub const HANGUL_L_BASE: u32 = 0x1100;
/// начало блока гласных чамо
pub const HANGUL_V_BASE: u32 = 0x1161;
/// начало блока завершающих согласных
pub const HANGUL_T_BASE: u32 = 0x11A8;
/// количество ведущих согласных
pub const HANGUL_L_COUNT: u32 = 19;
/// количество гласных
pub const HANGUL_V_COUNT: u32 = 21;
/// количество завершающих согласных (на 1 больше)
pub const HANGUL_T_COUNT: u32 = 28;
/// количество гласных * количество завершающих согласных
pub const HANGUL_N_COUNT: u32 = 588;
/// количество слогов хангыль в Unicode
pub const HANGUL_S_COUNT: u32 = 11172;

// последовательность (предполагаются 2 стартера) - чамо хангыль
pub fn compose_hangul(first: u32, second: u32) -> Option<u32>
{
    let l = first.wrapping_sub(HANGUL_L_BASE);
    // кейс L, V

    // кодпоинт является ведущей согласной чамо
    if l < HANGUL_L_COUNT {
        let v = second.wrapping_sub(HANGUL_V_BASE);

        // второй кодпоинт - гласная
        if v < HANGUL_V_COUNT {
            let code = HANGUL_S_BASE + l * HANGUL_N_COUNT + v * HANGUL_T_COUNT;
            return Some(code);
        }
    }

    // кейс LV, T

    let lv = first.wrapping_sub(HANGUL_S_BASE);

    // первый кодпоинт - слог хангыль LV
    if lv < HANGUL_S_COUNT && lv % HANGUL_T_COUNT == 0 {
        let t = second.wrapping_sub(HANGUL_T_BASE);

        // второй кодпоинт - завершающая согласная
        if t < HANGUL_T_COUNT - 1 {
            let code = first + t + 1;
            return Some(code);
        }
    }

    return None;
}

// относится ли кодпоинт к хангыль и может ли быть скомбинирован?
pub fn is_composable_hangul(code: u32) -> bool
{
    let l = code.wrapping_sub(HANGUL_L_BASE);
    let lv = code.wrapping_sub(HANGUL_S_BASE);

    // кодпоинт является ведущей согласной чамо / первый кодпоинт - слог хангыль LV
    (l < HANGUL_L_COUNT) || (lv < HANGUL_S_COUNT && lv % HANGUL_T_COUNT == 0)
}

// является ли кодпоинт гласной или завершающей согласной чамо?
pub fn is_composable_hangul_jamo(code: u32) -> bool
{
    let v = code.wrapping_sub(HANGUL_V_BASE);
    let t = code.wrapping_sub(HANGUL_T_BASE);

    (v < HANGUL_V_COUNT) || (t < HANGUL_T_COUNT - 1)
}
