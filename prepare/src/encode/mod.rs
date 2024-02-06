use std::collections::HashMap;

use unicode_normalization_source::normalization::precomposition::hangul::is_composable_hangul_jamo;
use unicode_normalization_source::properties::Codepoint;
use unicode_normalization_source::{COMBINES_BACKWARDS, NFC, NFKC};

use crate::tables::{COMPOSITION_TABLE_BACKWARDS_INDEX, COMPOSITION_TABLE_INDEX};

/// стартер без декомпозиции
pub const MARKER_STARTER: u64 = 0;
/// стартер-синглтон
pub const MARKER_SINGLETON: u64 = 0b_001;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u64 = 0b_010;
/// 16-битная пара (стартер-нестартер)
pub const MARKER_PAIR: u64 = 0b_011;
/// последовательность стартеров:
///  - первый стартер не комбинируется с предыдущими кодпоинтами
///  - информация о комбинировании записана для последнего стартера последовательности
pub const MARKER_EXPANSION_STARTERS: u64 = 0b_0100;
/// стартер и не-стартеры
///  - стартер не комбинируется с предыдущими кодпоинтами
pub const MARKER_EXPANSION_STARTER_NONSTARTERS: u64 = 0b_101;
/// два стартера + нестартер
///  - первый стартер не комбинируется с предыдущими кодпоинтами
///  - информация о комбинировании записана для второго стартера
pub const MARKER_EXPANSION_TWO_STARTERS_NONSTARTER: u64 = 0b_110;
/// исключения - стартеры, которые декомпозируются в нестартеры
pub const MARKER_EXPANSION_NONSTARTERS_EXCLUSION: u64 = 0b_111;
/// исключения - стартеры, которые комбинируются с предыдущими кодпоинтами
pub const MARKER_EXPANSION_COMBINES_BACKWARDS: u64 = 0b_1000;

#[derive(Debug, Clone)]
pub struct EncodedCodepoint
{
    pub value: u64,
    pub expansion_data: Option<Vec<u32>>,
}

/// закодировать кодпоинт для таблицы данных
/// * expansion_table_position - индекс в таблице расширенных данных, используется если информация
///   о декомпозиции/композиции в закодированные 64 бита
pub fn encode_codepoint(
    codepoint: &Codepoint,
    canonical: bool,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> EncodedCodepoint
{
    let precomposition = match canonical {
        true => &NFC[&codepoint.code],
        false => &NFKC[&codepoint.code],
    };

    let variants = &[
        starter,
        singleton,
        nonstarter,
        starter_nonstarter_pair_16bit,
        starter_nonstarters_sequence,
        starters_sequence,
        two_starters_nonstarter,
        starters_to_nonstarters,
        combines_backwards,
    ];

    let value = variants
        .iter()
        .find_map(|f| f(codepoint, precomposition, expansion_table_position, stats));

    if value.is_none() {
        println!("{:04X} - {}", codepoint.code, codepoint.name);
        dbg!(codepoint);
        dbg!(precomposition);
    }

    value.unwrap()
}

/// обычный стартер без декомпозиции
///
/// 0000 0000  ____ ____    iiii iiii  iiii iiii    ____ ____  ____ ____    ____ ____  ____ ____
///
fn starter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if (starters_map(precomposition) != "")
        || codepoint.is_nonstarter()
        || COMBINES_BACKWARDS.contains_key(&codepoint.code)
    {
        return None;
    }

    *stats.entry("стартер".to_owned()).or_default() += 1;

    let combinations = combination_info(codepoint.code) as u64;

    Some(EncodedCodepoint {
        value: MARKER_STARTER | (combinations << 16),
        expansion_data: None,
    })
}

/// синглтон
///
/// 0000 0001  ____ ____    iiii iiii  iiii iiii    xxxx xxxx  xxxx xxxx    xx__ ____  ____ ____
///
fn singleton(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "s" {
        return None;
    }

    if COMBINES_BACKWARDS.contains_key(&precomposition[0].code) {
        panic!("COMBI");
    }

    *stats.entry("синглтон".to_owned()).or_default() += 1;

    // убедимся, что по ошибке кодпоинт и результат декомпозиции не совпадают
    assert_ne!(codepoint.code, precomposition[0].code);
    // синглтон не комбинируется с предыдущими кодпоинтами
    assert!(!COMBINES_BACKWARDS.contains_key(&precomposition[0].code));

    let c0 = precomposition[0].code as u64;
    let c0_combinations = combination_info(c0 as u32) as u64;

    Some(EncodedCodepoint {
        value: MARKER_SINGLETON | (c0_combinations << 16) | (c0 << 32),
        expansion_data: None,
    })
}

/// пара стартер-нестартер (кодпоинты ∊ BMP, 16 бит)
///
/// 0000 0010  cccc cccc    iiii iiii  iiii iiii    xxxx xxxx  xxxx xxxx    yyyy yyyy  yyyy yyyy
///
fn starter_nonstarter_pair_16bit(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "sn" {
        return None;
    }

    if !precomposition.iter().all(|c| c.is_bmp()) {
        return None;
    }

    *stats
        .entry("пара стартер-нестартер".to_owned())
        .or_default() += 1;

    // стартер у пары не комбинируется с предыдущими кодпоинтами
    assert!(!COMBINES_BACKWARDS.contains_key(&precomposition[0].code));

    let c0 = precomposition[0].code as u64;
    let c1 = precomposition[1].code as u64;
    let c1_ccc = precomposition[1].ccc.u8() as u64;
    let c0_combinations = combination_info(precomposition[0].code) as u64;

    Some(EncodedCodepoint {
        value: MARKER_PAIR | (c1_ccc << 8) | (c0_combinations << 16) | (c0 << 32) | (c1 << 48),
        expansion_data: None,
    })
}

/// нестартер без декомпозиции
///
/// 0000 0011  cccc cccc    ____ ____  ____ ____    ____ ____  ____ ____    ____ ____  ____ ____
///
fn nonstarter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "" || codepoint.is_starter() || !precomposition.is_empty() {
        return None;
    }

    *stats.entry("нестартер".to_owned()).or_default() += 1;

    let ccc = codepoint.ccc.u8() as u64;

    Some(EncodedCodepoint {
        value: MARKER_NONSTARTER | (ccc << 8),
        expansion_data: None,
    })
}

/// последовательность стартеров
///
/// 0000 0100  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  ____ ____
///
fn starters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if precomposition.len() < 2 || !precomposition.iter().all(|c| c.is_starter()) {
        return None;
    }

    *stats
        .entry("последовательность стартеров".to_owned())
        .or_default() += 1;

    Some(expansion_entry(
        MARKER_EXPANSION_STARTERS,
        precomposition,
        combination_info(precomposition.last().unwrap().code),
        expansion_table_position,
    ))
}

/// стартер и последовательность нестартеров
///
/// 0000 0101  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  ____ ____
///
fn starter_nonstarters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if precomposition.len() < 2
        || !precomposition[0].is_starter()
        || !precomposition[1 ..].iter().all(|c| c.is_nonstarter())
    {
        return None;
    }

    *stats.entry("стартер + нестартеры".to_owned()).or_default() += 1;

    Some(expansion_entry(
        MARKER_EXPANSION_STARTER_NONSTARTERS,
        precomposition,
        combination_info(precomposition[0].code),
        expansion_table_position,
    ))
}

/// стартер-стартер-нестартер
///
/// 0000 0110  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  ____ ____
///
fn two_starters_nonstarter(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "ssn" {
        return None;
    }

    *stats
        .entry("два стартера + нестартер".to_owned())
        .or_default() += 1;

    Some(expansion_entry(
        MARKER_EXPANSION_TWO_STARTERS_NONSTARTER,
        precomposition,
        combination_info(precomposition[1].code),
        expansion_table_position,
    ))
}

/// исключение - стартеры с декомпозицией в нестартеры
///
/// 0000 0111  ____ ____    ____ ____  ____ ____    pppp pppp  pppp pppp    nnnn nnnn  ____ ____
///
fn starters_to_nonstarters(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if precomposition.is_empty() || !precomposition.iter().all(|c| c.is_nonstarter()) {
        return None;
    }

    *stats.entry("стартер в нестартеры".to_owned()).or_default() += 1;

    Some(expansion_entry(
        MARKER_EXPANSION_NONSTARTERS_EXCLUSION,
        precomposition,
        0,
        expansion_table_position,
    ))
}

/// исключение - стартеры, комбинируемые с предыдущим кодпоинтом
///
/// 0000 1000  ____ ____    ____ ____  ____ ____    pppp pppp  pppp pppp    nnnn nnnn  ____ ____
///
fn combines_backwards(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    if codepoint.is_nonstarter() || !COMBINES_BACKWARDS.contains_key(&codepoint.code) {
        return None;
    }

    assert_eq!(combination_info(codepoint.code), 0);
    assert!(precomposition.is_empty());

    *stats
        .entry("комбинируется с предыдущим".to_owned())
        .or_default() += 1;

    let combination_info = combination_backwards_info(codepoint.code);

    assert_ne!(combination_info, 0);

    Some(expansion_entry(
        MARKER_EXPANSION_COMBINES_BACKWARDS,
        precomposition,
        combination_info,
        expansion_table_position,
    ))
}

// запись с данными о кодпоинтах, хранящиеся в дополнительной таблице
fn expansion_entry(
    marker: u64,
    precomposition: &[Codepoint],
    combination_info: u16,
    expansion_table_position: usize,
) -> EncodedCodepoint
{
    if !precomposition.is_empty() && precomposition[0].is_starter() {
        // первый стартер последовательности не может быть скомбинирован с предыдущими кодпоинтами
        assert!(!COMBINES_BACKWARDS.contains_key(&precomposition[0].code));
        assert!(!is_composable_hangul_jamo(precomposition[0].code));
    }

    let expansion_info = expansion_info(expansion_table_position, precomposition.len());

    EncodedCodepoint {
        value: marker | ((combination_info as u64) << 16) | ((expansion_info as u64) << 32),
        expansion_data: Some(prepare_expansion_data(precomposition)),
    }
}

/// строка, описывающая прекомпозицию, состоящая из символов s и n, где s - стартер, n - нестартер
fn starters_map(precomposition: &Vec<Codepoint>) -> String
{
    precomposition
        .iter()
        .map(|c| match c.is_starter() {
            true => 's',
            false => 'n',
        })
        .collect()
}

/// информация о записи в таблице комбинирования для кодпоинта, записанная как u16
/// 0, если кодпоинт не комбинируется с идущими за ним
fn combination_info(code: u32) -> u16
{
    match COMPOSITION_TABLE_INDEX.get(&code) {
        Some(info) => info.bake(),
        None => 0,
    }
}

/// информация о комбинировании с предыдущим кодпоинтом
fn combination_backwards_info(code: u32) -> u16
{
    match COMPOSITION_TABLE_BACKWARDS_INDEX.get(&code) {
        Some(info) => info.bake(),
        None => 0,
    }
}

/// информация о данных в расширенной части
fn expansion_info(index: usize, len: usize) -> u32
{
    // хватит ли нам u32?
    assert!(index < 0xFFFF);
    assert!(len < 0xFF);

    index as u32 | ((len as u32) << 16)
}

/// последовательность кодпоинтов (u32), в старших 8 битах записан CCC
fn prepare_expansion_data(codepoints: &[Codepoint]) -> Vec<u32>
{
    codepoints
        .iter()
        .map(|c| c.code | ((c.ccc.u8() as u32) << 24))
        .collect()
}
