use std::collections::HashMap;

use crate::tables::*;
use unicode_normalization_source::normalization::precomposition::hangul::is_composable_hangul_jamo;
use unicode_normalization_source::properties::Codepoint;
use unicode_normalization_source::*;

/// стартер без декомпозиции
pub const MARKER_STARTER: u64 = 0b_000;
/// пара стартер + нестартер
pub const MARKER_PAIR: u64 = 0b_001;
/// стартер-синглтон
pub const MARKER_SINGLETON: u64 = 0b_010;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u64 = 0b_011;

// 4 маркера объединены в один кейс: 0 или несколько стартеров + 0 или несколько нестартеров
//  - первый стартер не комбинируется с предыдущими кодпоинтами
//  - информация о комбинировании записана для последнего стартера последовательности

/// стартер и не-стартеры
pub const MARKER_EXPANSION_STARTER_NONSTARTERS: u64 = 0b_100;
/// последовательность стартеров
pub const MARKER_EXPANSION_STARTERS: u64 = 0b_0100;
/// два стартера + нестартер
pub const MARKER_EXPANSION_TWO_STARTERS_NONSTARTER: u64 = 0b_100;
/// исключения - стартеры, которые декомпозируются в нестартеры
pub const MARKER_NONSTARTERS_EXCLUSION: u64 = 0b_100;

/// исключения - стартеры, которые комбинируются с предыдущими кодпоинтами
pub const MARKER_COMBINES_BACKWARDS: u64 = 0b_101;

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
    let (precomposition, mut quick_check) = match canonical {
        true => (&NFC[&codepoint.code], QC_NFC[codepoint.code as usize]),
        false => (&NFKC[&codepoint.code], QC_NFKC[codepoint.code as usize]),
    };

    if is_composable_hangul_jamo(codepoint.code) {
        quick_check = 'M';
    }

    let variants = &[
        starter,
        singleton,
        nonstarter,
        starter_nonstarter_pair,
        starter_nonstarters_sequence,
        starters_sequence,
        two_starters_nonstarter,
        starters_to_nonstarters,
        combines_backwards_case,
    ];

    let value = variants.iter().find_map(|f| {
        f(
            codepoint,
            precomposition,
            quick_check,
            expansion_table_position,
            stats,
        )
    });

    value.unwrap()
}

// все битовые представения u64 представлены как big endian
//
// q - маркер быстрой проверки
//   0 - 'Y'/'M' - не участвует или может участвовать в композиции,
//   1 - 'N' - участвует в композиции или кодпоинт хангыль, который может быть скомбинирован с предыдущим
//
// mm - маркер типа записи
//
// ii.. - (16 бит) - информация о комбинациях стартера со следующим кодпоинтом (в большинстве случаев),
//        или с предыдущим кодпоинтом (когда стартер может быть скомбинирован с предыдущим)
//
// ccс. - (8 бит) - ССС нестартера в записи
//
// pp.. - (16 бит) - индекс последовательности кодпоинтов в таблице expansions
// nn.. - (8 бит) - количество кодпоинтов в последовательности в таблице expansions
// tttt - (4 бита) - количество нестартеров в последовательности в таблице expansions

macro_rules! blocking_checks {
    ($($expr: expr),+) => {
        if $($expr ||)+ false {
            return None;
        }
    };
}

macro_rules! combining {
    ($codepoint:expr) => {
        (combination_info($codepoint) as u64)
    };
}

macro_rules! assert_not_combines_backwards {
    ($code: expr) => {
        assert!(!COMBINES_BACKWARDS.contains_key(&($code as u32)));
    };
}

macro_rules! assert_qc {
    ($qc: expr, $($c:expr),+) => {
        assert!([$($c),+].contains(&$qc));
    };
}

macro_rules! to_stats {
    ($stats: ident, $key: expr) => {
        *$stats.entry($key.to_owned()).or_default() += 1;
    };
}

macro_rules! encoded {
    ($marker: expr, $qc: expr, $data: expr, $expansion: expr) => {{
        let qc = match $qc {
            'Y' => 0,
            'N' => 1,
            'M' => 1,
            _ => unreachable!(),
        };

        Some(EncodedCodepoint {
            value: ($marker << 1) | qc | $data,
            expansion_data: $expansion,
        })
    }};
}

macro_rules! expansion_entry {
    ($marker: expr, $fast: expr, $combining: expr, $precomposition: expr, $e_index: expr) => {{
        let e_index = $e_index as u64;
        let e_len = $precomposition.len() as u64;

        assert!((e_index < 0xFFFF) && (e_len < 0xFF));

        if !$precomposition.is_empty() {
            let c0 = $precomposition[0].code;
            let c0_ccc = $precomposition[0].ccc;

            assert!(!is_composable_hangul_jamo(c0));

            if c0_ccc.is_starter() {
                assert_not_combines_backwards!(c0);
            }
        }

        let n_len = $precomposition.iter().filter(|c| c.is_nonstarter()).count() as u64;

        assert!(n_len < 0b_1111);

        encoded!(
            $marker,
            $fast,
            ($combining << 32) | (e_index << 16) | (e_len << 8) | (n_len << 4),
            Some(prepare_expansion_data($precomposition))
        )
    }};
}

/// обычный стартер без декомпозиции
///
/// ____ ____  ____ ____    ____ ____  ____ ____    iiii iiii  iiii iiii    ____ ____  ____ mmmq
///
fn starter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    // стартер, нет декомпозиции, не комбинируется с предыдущим

    blocking_checks!(
        !precomposition.is_empty(),
        codepoint.is_nonstarter(),
        combines_backwards(codepoint.code)
    );

    // Y - если за стартером следует стартер, то кодпоинт не изменится
    // M - чамо хангыль V / T - нужно попробовать скомбинировать с предыдущим кодпоинтом
    assert_qc!(qc, 'Y', 'M');

    to_stats!(stats, "стартер");

    encoded!(MARKER_STARTER, qc, combining!(codepoint.code) << 16, None)
}

/// пара стартер + нестартер
///
/// iiii iiii  iiii iiii    yyyy yyyy  yyyy yyyy    yyxx xxxx  xxxx xxxx    xxxx cccc  cccc mmmq
///
fn starter_nonstarter_pair(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    // пара: стартер + нестартер, не является исключением композиции

    blocking_checks!(
        starters_map(precomposition) != "sn",
        is_exclusion(codepoint.code)
    );

    // Y - пара будет скомбинирована обратно, если за ней следет стартер
    // N - при комбинировании будет получен другой символ
    assert_qc!(qc, 'Y', 'N');

    to_stats!(stats, "пара - стартер + нестартер");

    let c0 = precomposition[0].code as u64;
    let c1 = precomposition[1].code as u64;
    let c1_ccc = precomposition[1].ccc.u8() as u64;

    assert_not_combines_backwards!(c0);

    encoded!(
        MARKER_PAIR,
        qc,
        (c1_ccc << 4) | (c0 << 12) | (c1 << 30) | (combining!(c0) << 48),
        None
    )
}

/// синглтон
///
/// ____ ____  ____ __xx    xxxx xxxx  xxxx xxxx    iiii iiii  iiii iiii    ____ ____  ____ mmmq
///
fn singleton(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(starters_map(precomposition) != "s");

    // N - синглтон - это всегда замена одного кодпоинта другим
    assert_qc!(qc, 'N');

    to_stats!(stats, "синглтон");

    let c0 = precomposition[0].code as u64;

    assert_not_combines_backwards!(c0);
    assert_ne!(codepoint.code as u64, c0);

    encoded!(
        MARKER_SINGLETON,
        qc,
        (combining!(c0) << 16) | (c0 << 32),
        None
    )
}

/// нестартер без декомпозиции
///
/// ____ ____  ____ ____    ____ ____  ____ ____    ____ ____  ____ ____    ____ cccc  cccc mmmq
///
fn nonstarter(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(!precomposition.is_empty(), codepoint.is_starter());

    // M, Y - нестартеры делятся на те, которые могут быть скомбинированы, и на те, которые не могут
    // тем не менее, будем рассматривать любой нестартер как N, т.к. мы в данном случае не используем
    // непосредственно алгоритм валидации NF(K)C, а его часть
    assert_qc!(qc, 'Y', 'M');

    to_stats!(stats, "нестартер");

    let ccc = codepoint.ccc.u8() as u64;

    encoded!(MARKER_NONSTARTER, 'N', (ccc << 4), None)
}

/// последовательность стартеров
///
/// ____ ____  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  0000 mmmq
///
fn starters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(
        precomposition.len() < 2,
        !precomposition.iter().all(|c| c.is_starter())
    );

    // N - декомпозиция происходит всегда
    assert_qc!(qc, 'N');

    to_stats!(stats, "последовательности стартеров");

    let cl = precomposition.last().unwrap().code;

    expansion_entry!(
        MARKER_EXPANSION_STARTERS,
        qc,
        combining!(cl),
        precomposition,
        expansion_table_position
    )
}

/// стартер и последовательность нестартеров
///
/// ____ ____  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  tttt mmmq
///
fn starter_nonstarters_sequence(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(
        precomposition.len() < 2,
        !precomposition[0].is_starter(),
        !precomposition[1 ..].iter().all(|c| c.is_nonstarter())
    );

    // Y, N - аналогично паре
    assert_qc!(qc, 'Y', 'N');

    to_stats!(stats, "стартер + нестартеры");

    let c0 = precomposition[0].code;

    expansion_entry!(
        MARKER_EXPANSION_STARTER_NONSTARTERS,
        qc,
        combining!(c0),
        precomposition,
        expansion_table_position
    )
}

/// стартер + стартер + нестартер
///
/// ____ ____  ____ ____    iiii iiii  iiii iiii    pppp pppp  pppp pppp    nnnn nnnn  tttt mmmq
///
fn two_starters_nonstarter(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(starters_map(precomposition) != "ssn");

    // N - только в NFKC - декомпозиция происходит всегда
    assert_qc!(qc, 'N');

    to_stats!(stats, "два стартера + нестартер");

    let c1 = precomposition[1].code;

    expansion_entry!(
        MARKER_EXPANSION_TWO_STARTERS_NONSTARTER,
        qc,
        combining!(c1),
        precomposition,
        expansion_table_position
    )
}

/// исключение - стартеры с декомпозицией в нестартеры
///
/// ____ ____  ____ ____    ____ ____  ____ ____    pppp pppp  pppp pppp    nnnn nnnn  tttt mmmq
///
fn starters_to_nonstarters(
    _: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    expansion_table_position: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(
        precomposition.is_empty(),
        !precomposition.iter().all(|c| c.is_nonstarter())
    );

    // N - т.к. кодпоинт всегда декомпозируется и не собирается обратно
    assert_qc!(qc, 'N');

    to_stats!(stats, "декомпозиция в нестартеры");

    expansion_entry!(
        MARKER_NONSTARTERS_EXCLUSION,
        qc,
        0,
        precomposition,
        expansion_table_position
    )
}

/// исключение - стартеры, комбинируемые с предыдущим кодпоинтом
///
/// ____ ____  ____ ____    ____ ____  ____ ____    iiii iiii  iiii iiii    ____ ____  ____ mmmq
///
fn combines_backwards_case(
    codepoint: &Codepoint,
    precomposition: &Vec<Codepoint>,
    qc: char,
    _: usize,
    stats: &mut HashMap<String, usize>,
) -> Option<EncodedCodepoint>
{
    blocking_checks!(
        codepoint.is_nonstarter(),
        !combines_backwards(codepoint.code)
    );

    assert_eq!(combining!(codepoint.code), 0);
    assert!(precomposition.is_empty());

    // M - может быть скомбинирован с предыдущим или оставлен как есть
    assert_qc!(qc, 'M');

    to_stats!(stats, "комбинируется с предыдущим");

    let combines = combination_backwards_info(codepoint.code) as u64;

    encoded!(MARKER_COMBINES_BACKWARDS, qc, (combines << 16), None)
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
fn combination_info<T: Into<u64>>(code: T) -> u16
{
    match COMPOSITION_TABLE_INDEX.get(&(code.into() as u32)) {
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

/// последовательность кодпоинтов (u32), в старших 8 битах записан CCC
fn prepare_expansion_data(codepoints: &[Codepoint]) -> Vec<u32>
{
    codepoints
        .iter()
        .map(|c| c.code | ((c.ccc.u8() as u32) << 24))
        .collect()
}

/// кодпоинт может быть скомбинирован с предыдущим
fn combines_backwards<T: Into<u64>>(code: T) -> bool
{
    COMBINES_BACKWARDS.contains_key(&(code.into() as u32))
}

/// исключение композиции
fn is_exclusion<T: Into<u64>>(code: T) -> bool
{
    COMPOSITION_EXCLUSIONS.contains(&(code.into() as u32))
}
