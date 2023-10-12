use unicode_normalization_source::{properties::*, UNICODE};

use self::composition::COMPOSES_WITH_LEFT;
use crate::output::stats::CodepointGroups;

pub mod composition;

/// стартер без декомпозиции
pub const MARKER_STARTER: u64 = 0;
/// не-стартер без декомпозиции
pub const MARKER_NON_STARTER: u64 = 1;
/// 16-битная пара
pub const MARKER_PAIR: u64 = 2;
/// синглтон
pub const MARKER_SINGLETON: u64 = 3;
/// декомпозиция, вынесенная во внешний блок
pub const MARKER_EXPANSION: u64 = 4;

/// кодпоинт может быть скомбинирован с предыдущим
pub const MARKER_COMPOSES_WITH_LEFT: u64 = 8;

/// закодировать кодпоинт для таблицы данных
pub fn encode_codepoint(
    codepoint: &Codepoint,
    canonical: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> (u64, Vec<u32>)
{
    let decomposition = match canonical {
        true => &codepoint.canonical_decomposition,
        false => &codepoint.compat_decomposition,
    };

    let value = [
        starter,                  // стартер
        nonstarter,               // не-стартер
        singleton,                // синглтон
        pair16,                   // пара (16 бит)
        triple16,                 // тройка (16 бит)
        pair18,                   // пара (18 бит)
        starter_to_nonstarters,   // стартер в не-стартеры
        nonstarter_decomposition, // не-стартер с декомпозицией в стартер
        triple18,                 // тройка (18 бит)
        long_decomposition,       // декомпозиция > 3 символов
    ]
    .iter()
    .find_map(|f| {
        f(
            codepoint,
            decomposition,
            composes_with_left(codepoint.code, decomposition),
            expansion_position,
            stats,
        )
    });

    match value {
        Some(value) => value,
        None => {
            // не подошёл ни один из вариантов

            let dec_string: String = decomposition
                .iter()
                .map(|c| format!("U+{:04X} [{}] ", *c, u8::from(get_ccc(*c))))
                .collect();

            panic!(
                "\n\nне определили тип хранения кодпоинта: U+{:04X} - {} [CCC={}] -> {}\n\n",
                codepoint.code,
                codepoint.name,
                u8::from(codepoint.ccc),
                dec_string,
            );
        }
    }
}

/// стартер:
///     - CCC = 0
///     - нет декомпозиции
fn starter(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter() || !decomposition.is_empty() {
        return None;
    }

    let value = MARKER_STARTER | compose_marker(composes_with_left);

    if composes_with_left {
        to_stats(
            stats,
            "0. стартеры (с возможной композицией с предыдущим кодпоинтом)",
            codepoint,
            decomposition,
        );
    }

    Some((value, vec![]))
}

/// не-стартер:
///     - ССС > 0
///     - нет декомпозиции
fn nonstarter(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_non_starter() || !decomposition.is_empty() {
        return None;
    }

    let ccc = u64::from(codepoint.ccc);

    let value = MARKER_NON_STARTER | compose_marker(composes_with_left) | (ccc << 8);

    to_stats(
        stats,
        match composes_with_left {
            true => "1.1 не-стартеры (с возможной композицией с предыдущим кодпоинтом)",
            false => "1.2 не-стартеры (невозможна композиция с предыдущим кодпоинтом)",
        },
        codepoint,
        decomposition,
    );
    Some((value, vec![]))
}

/// синглтон:
///     - стартер
///     - декомпозиция из одного стартера
fn singleton(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() != 1
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!("синглтоны не комбинируются с предыдущими кодпоинтами");
    }

    let code = decomposition[0] as u64;

    let value = MARKER_SINGLETON | (code << 32);

    to_stats(stats, "2. синглтоны", codepoint, decomposition);
    Some((value, vec![]))
}

/// пара (16 бит):
///     - стартер
///     - декомпозиция из 2х кодпоинтов
///     - кодпоинты декомпозиции - 16-битные
///     - первый из них - стартер
fn pair16(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() != 2
        || decomposition.iter().any(|&c| c > 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!("пары: первые кодпоинты декомпозиции не комбинируются с предыдущими");
    }

    let c1 = decomposition[0] as u64;
    let c2 = decomposition[1] as u64;
    let c2_ccc = u64::from(get_ccc(decomposition[1]));

    let value = MARKER_PAIR | (c2_ccc << 8) | (c1 << 16) | (c2 << 32);

    to_stats(stats, "3. пары (16 бит)", codepoint, decomposition);
    Some((value, vec![]))
}

/// тройка (16 бит):
///     - стартер
///     - декомпозиция из 3х кодпоинтов
///     - кодпоинты декомпозиции - 16-битные
///     - первый из них - стартер
fn triple16(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    _: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() != 3
        || decomposition.iter().any(|&c| c > 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!("тройки: первые кодпоинты декомпозиции не комбинируются с предыдущими");
    }

    let c1 = decomposition[0] as u64;
    let c2 = decomposition[1] as u64;
    let c3 = decomposition[2] as u64;

    let c2_ccc = u64::from(get_ccc(decomposition[1]));
    let c3_ccc = u64::from(get_ccc(decomposition[2]));

    let value = c1 | (c2 << 16) | (c3 << 32) | (c2_ccc << 48) | (c3_ccc << 56);

    to_stats(stats, "4. тройки (16 бит)", codepoint, decomposition);
    Some((value, vec![]))
}

/// пара (18 бит):
///     - стартер
///     - декомпозиция из 2х кодпоинтов
///     - хотя бы один из кодпоинтов декомпозиции - 18-битный
///     - первый из них - стартер
fn pair18(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() != 2
        || decomposition.iter().all(|&c| c <= 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!("пары за пределами BMP: первые кодпоинты декомпозиции не комбинируются с предыдущими кодпоинтами");
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "5. пары (18 бит)", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

/// стартер с декомпозицией в не-стартеры
///     - стартер
///     - есть декомпозиция, которая состоит из не-стартеров
fn starter_to_nonstarters(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.is_empty()
        || !decomposition.iter().all(|&c| get_ccc(c).is_non_starter())
    {
        return None;
    }

    let value = MARKER_EXPANSION
        | compose_marker(composes_with_left)
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(
        stats,
        match composes_with_left {
            true => "6.1 стартеры в не-стартеры (комбинируются)",
            false => "6.2 стартеры в не-стартеры (не комбинируются)",
        },
        codepoint,
        decomposition,
    );

    Some((value, map_expansion(decomposition)))
}

/// не-стартер с декомпозицией
///     - не-стартер
///     - есть декомпозиция
fn nonstarter_decomposition(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_non_starter() || decomposition.is_empty() {
        return None;
    }

    let value = MARKER_EXPANSION
        | compose_marker(composes_with_left)
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    if !composes_with_left {
        panic!("не-стартеры с декомпозицией: первый кодпоинт декомпозиции всегда может комбинироваться с предыдущим");
    }

    to_stats(
        stats,
        "7. не-стартеры с декомпозицией (комбинируются)",
        codepoint,
        decomposition,
    );
    Some((value, map_expansion(decomposition)))
}

/// тройка (18 бит)
///     - стартер
///     - декомпозиция в 3 кодпоинта
///     - хотя-бы один из них - 18 бит
///     - декомпозиция начинается со стартера
fn triple18(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() != 3
        || decomposition.iter().all(|&c| c <= 0xFFFF)
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!(
            "тройки (18 бит): первый кодпоинт декомпозиции не может комбинироваться с предыдущим"
        );
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "8. тройки (18 бит)", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

/// декомпозиция > 3 символов
///     - стартер
///     - декомпозиция > 3 кодпоинтов
///     - декомпозиция начинается со стартера
fn long_decomposition(
    codepoint: &Codepoint,
    decomposition: &Vec<u32>,
    composes_with_left: bool,
    expansion_position: usize,
    stats: &mut CodepointGroups,
) -> Option<(u64, Vec<u32>)>
{
    if !codepoint.ccc.is_starter()
        || decomposition.len() <= 3
        || !get_ccc(decomposition[0]).is_starter()
    {
        return None;
    }

    if composes_with_left {
        panic!(
            "декомпозиция > 3 кодпоинтов: первый кодпоинт декомпозиции не может комбинироваться с предыдущим"
        );
    }

    let value = MARKER_EXPANSION
        | ((decomposition.len() as u64) << 8)
        | ((expansion_position as u64) << 16);

    to_stats(stats, "9. длинная декомпозиция", codepoint, decomposition);
    Some((value, map_expansion(decomposition)))
}

// ----

/// получаем CCC кодпоинта
fn get_ccc(codepoint: u32) -> CanonicalCombiningClass
{
    match UNICODE.get(&codepoint) {
        Some(codepoint) => codepoint.ccc,
        None => CanonicalCombiningClass::NotReordered,
    }
}

/// преобразовать декомпозицию в вектор значений, состоящих из кодпоинта (младшие биты) и CCC (8 старших бит)
fn map_expansion(decomposition: &[u32]) -> Vec<u32>
{
    decomposition
        .iter()
        .map(|e| e | u32::from(get_ccc(*e)) << 24)
        .collect()
}

/// строка с данными о кодпоинте для статистики
fn info(codepoint: &Codepoint, decomposition: &[u32]) -> String
{
    let dec_string: String = decomposition
        .iter()
        .map(|c| format!("[{}] ", u8::from(get_ccc(*c))))
        .collect();

    format!(
        "U+{:04X} - {} [{}] ({}) {}\n",
        codepoint.code,
        codepoint.name,
        u8::from(codepoint.ccc),
        decomposition.len(),
        dec_string,
    )
}

/// пишем в статистику
fn to_stats<'a>(
    stats: &mut CodepointGroups<'a>,
    group: &'a str,
    codepoint: &Codepoint,
    decomposition: &[u32],
)
{
    stats
        .entry(group)
        .or_insert(vec![])
        .push(info(codepoint, decomposition));
}

/// кодпоинт (или его первый элемент декомпозиции, если она есть) может быть скомбинирован с предыдущим
fn composes_with_left(code: u32, decomposition: &Vec<u32>) -> bool
{
    let code = match decomposition.is_empty() {
        true => code,
        false => decomposition[0],
    };

    COMPOSES_WITH_LEFT.contains(&code)
}

/// флаг комбинирования
fn compose_marker(flag: bool) -> u64
{
    match flag {
        true => MARKER_COMPOSES_WITH_LEFT,
        false => 0,
    }
}
