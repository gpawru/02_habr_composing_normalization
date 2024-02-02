use unicode_normalization_source::normalization::precomposition::hangul::is_composable_hangul_jamo;
use unicode_normalization_source::properties::Codepoint;
use unicode_normalization_source::{COMBINES_BACKWARDS, NFC, NFKC};

use crate::tables::COMPOSITION_TABLE_INDEX;

/// стартер без декомпозиции / синглтон
pub const MARKER_STARTER: u64 = 0;

/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u64 = 0b_001;

/// 16-битная пара (стартер-нестартер)
pub const MARKER_PAIR: u64 = 0b_010;

/// последовательность стартеров:
///  - первый стартер не комбинируется с предыдущими кодпоинтами
///  - информация о комбинировании записана для последнего стартера последовательности
pub const MARKER_EXPANSION_STARTERS: u64 = 0b_011;

/// стартер и не-стартеры
///  - стартер не комбинируется с предыдущими кодпоинтами
pub const MARKER_EXPANSION_STARTER_NONSTARTERS: u64 = 0b_100;

/// два стартера + нестартер
///  - первый стартер не комбинируется с предыдущими кодпоинтами
///  - информация о комбинировании записана для второго стартера
pub const MARKER_EXPANSION_TWO_STARTERS_NONSTARTER: u64 = 0b_101;

/// исключения - стартеры, которые декомпозируются в нестартеры
pub const MARKER_EXPANSION_NONSTARTERS_EXCLUSION: u64 = 0b_110;

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
) -> EncodedCodepoint
{
    let precomposition = match canonical {
        true => &NFC[&codepoint.code],
        false => &NFKC[&codepoint.code],
    };

    let variants = &[
        starter,
        nonstarter,
        starter_nonstarter_pair_16bit,
        singleton,
        starter_nonstarters_sequence,
        starters_sequence,
        two_starters_nonstarter,
        starters_to_nonstarters,
    ];

    let value = variants
        .iter()
        .find_map(|f| f(codepoint, precomposition, expansion_table_position));

    match value {
        Some(value) => value,
        None => {
            println!(
                "{:04X}: {} - {}",
                codepoint.code,
                starters_map(precomposition),
                codepoint.name
            );
            EncodedCodepoint {
                value: 0,
                expansion_data: None,
            }
            // unreachable!();
        }
    }
}

/// обычный стартер без декомпозиции
///
/// 0000 0000  ____ ____    iiii iiii  iiii iiii    ____ ____  ____ ____    ____ ____  ____ ____
///
fn starter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "" {
        return None;
    }

    let combinations = combination_index(codepoint.code) as u64;

    Some(EncodedCodepoint {
        value: MARKER_STARTER | (combinations << 16),
        expansion_data: None,
    })
}

/// нестартер без декомпозиции
///
/// 0000 0001  cccc cccc    ____ ____  ____ ____    ____ ____  ____ ____    ____ ____  ____ ____
///
fn nonstarter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "n" {
        return None;
    }

    if !precomposition.is_empty() {
        return None;
    }

    let ccc = codepoint.ccc.u8() as u64;

    Some(EncodedCodepoint {
        value: MARKER_NONSTARTER | (ccc << 8),
        expansion_data: None,
    })
}

/// пара стартер-нестартер (кодпоинты ∊ BMP, 16 бит)
///
/// 0000 0002  cccc cccc    iiii iiii  iiii iiii    xxxx xxxx  xxxx xxxx    yyyy yyyy  yyyy yyyy
///
fn starter_nonstarter_pair_16bit(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    _: usize,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "sn" {
        return None;
    }

    if !precomposition.iter().all(|c| c.is_bmp()) {
        return None;
    }

    let c0 = precomposition[0].code as u64;
    let c1 = precomposition[1].code as u64;
    let c1_ccc = precomposition[1].ccc.u8() as u64;
    let c0_combinations = combination_index(precomposition[0].code) as u64;

    Some(EncodedCodepoint {
        value: MARKER_PAIR | (c1_ccc << 8) | (c0_combinations << 16) | (c0 << 32) | (c1 << 48),
        expansion_data: None,
    })
}

/// синглтон
///
/// 0000 0000  ____ ____    iiii iiii  iiii iiii    xxxx xxxx  xxxx xxxx    xx__ ____  ____ ____
///
fn singleton(_: &Codepoint, precomposition: &Vec<Codepoint>, _: usize) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "s" {
        return None;
    }

    let c0 = precomposition[0].code as u64;
    let c0_combinations = combination_index(c0 as u32) as u64;

    Some(EncodedCodepoint {
        value: MARKER_STARTER | (c0_combinations << 16) | (c0 << 32),
        expansion_data: None,
    })
}

/// стартер и последовательность нестартеров
///
/// 0000 0011  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    ____ ____  ____ ____
///
fn starter_nonstarters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
) -> Option<EncodedCodepoint>
{
    if precomposition.len() < 2
        || !precomposition[0].is_starter()
        || !precomposition[1 ..].iter().all(|c| c.is_nonstarter())
    {
        return None;
    }

    Some(expansion_entry(
        MARKER_EXPANSION_STARTER_NONSTARTERS,
        precomposition,
        combination_index(precomposition[0].code),
        expansion_table_position,
    ))
}

/// последовательность стартеров
///
/// 0000 0100  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    ____ ____  ____ ____
///
fn starters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
) -> Option<EncodedCodepoint>
{
    if precomposition.len() < 2 || !precomposition.iter().all(|c| c.is_starter()) {
        return None;
    }

    Some(expansion_entry(
        MARKER_EXPANSION_STARTERS,
        precomposition,
        combination_index(precomposition.last().unwrap().code),
        expansion_table_position,
    ))
}

/// стартер-стартер-нестартер
///
/// 0000 0101  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    ____ ____  ____ ____
///
fn two_starters_nonstarter(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
) -> Option<EncodedCodepoint>
{
    if starters_map(precomposition) != "ssn" {
        return None;
    }

    Some(expansion_entry(
        MARKER_EXPANSION_TWO_STARTERS_NONSTARTER,
        precomposition,
        combination_index(precomposition[1].code),
        expansion_table_position,
    ))
}

/// исключение - стартеры с декомпозицией в нестартеры
///
/// 0000 0110  ____ ____    ____ ____  ____ ____    pppp pppp  pppp pppp    ____ ____  ____ ____
///
fn starters_to_nonstarters(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    expansion_table_position: usize,
) -> Option<EncodedCodepoint>
{
    if !precomposition.iter().all(|c| c.is_nonstarter()) {
        return None;
    }

    Some(expansion_entry(
        MARKER_EXPANSION_NONSTARTERS_EXCLUSION,
        precomposition,
        0,
        expansion_table_position,
    ))
}

// запись с данными о кодпоинтах, хранящиеся в дополнительной таблице
fn expansion_entry(
    marker: u64,
    precomposition: &[Codepoint],
    combinations_index: u16,
    expansion_table_position: usize,
) -> EncodedCodepoint
{
    if precomposition[0].is_starter() {
        // первый стартер последовательности не может быть скомбинирован с предыдущими кодпоинтами
        assert!(!COMBINES_BACKWARDS.contains(&precomposition[0].code));
        assert!(!is_composable_hangul_jamo(precomposition[0].code));
    }

    // хватит ли нам u16 для записи позиции?
    assert!(expansion_table_position < u16::MAX as usize);

    EncodedCodepoint {
        value: marker
            | ((combinations_index as u64) << 16)
            | ((expansion_table_position as u64) << 32),
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
fn combination_index(code: u32) -> u16
{
    match COMPOSITION_TABLE_INDEX.get(&code) {
        Some(info) => info.bake(),
        None => 0,
    }
}

/// последовательность кодпоинтов (u32), в старших 8 битах записан CCC
fn prepare_expansion_data(codepoints: &[Codepoint]) -> Vec<u32>
{
    codepoints
        .iter()
        .map(|c| c.code | ((c.ccc.u8() as u32) << 24))
        .collect()
}
