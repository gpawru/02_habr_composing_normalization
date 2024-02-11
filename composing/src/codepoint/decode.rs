use crate::o;

use crate::codepoint::Codepoint;
use crate::composition::Combining;

/// стартер без декомпозиции (включая комбинируемые чамо хангыль)
const MARKER_STARTER: u8 = 0b_000;
/// пара стартер + нестартер
const MARKER_PAIR: u8 = 0b_001;
/// синглтон
const MARKER_SINGLETON: u8 = 0b_010;
/// нестартер без декомпозиции
const MARKER_NONSTARTER: u8 = 0b_011;

/// 0 или несколько (до 18) стартеров + 0 / 1 / 2 нестартера
const MARKER_EXPANSION: u8 = 0b_100;
/// кодпоинт, который может быть скомбинирован с предыдущим
const MARKER_COMBINES_BACKWARDS: u8 = 0b_101;

#[derive(Debug)]
pub enum DecodedValue
{
    /// стартер без декомпозиции (включая комбинируемые чамо хангыль)
    Starter(Combining),
    /// пара стартер + нестартер
    Pair(Codepoint, Codepoint, Combining),
    /// синглтон
    Singleton(u32, Combining),
    /// нестартер без декомпозиции
    Nonstarter(u8),
    /// последовательность кодпоинтов - в отдельной таблице
    Expansion(Combining, u16, u8, u8),
    /// комбинируется с предыдущим,
    CombinesBackwards(Combining),
}

/// маркер нестартера или расширения с декомпозицией в нестартеры
#[inline(always)]
pub fn is_nonstarters_value(value: u64) -> bool
{
    let marker = (value as u8 >> 1) & 0b_111;

    match marker {
        MARKER_NONSTARTER => true,
        MARKER_EXPANSION => (value as u8 >> 4) & 0b_1111 == (value >> 8) as u8,
        _ => false,
    }
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecodedValue
{
    let marker = (value as u8 >> 1) & 0b_111;

    match marker {
        MARKER_STARTER => parse_starter(value),
        MARKER_PAIR => parse_pair(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_NONSTARTER => parse_nonstarter(value),
        MARKER_EXPANSION => parse_expansion(value),
        MARKER_COMBINES_BACKWARDS => parse_combines_backwards(value),
        _ => unreachable!(),
    }
}

/// стартер без декомпозиции (включая комбинируемые чамо хангыль)
#[inline(always)]
pub fn parse_starter(value: u64) -> DecodedValue
{
    DecodedValue::Starter(o!(value, Combining, 1))
}

/// пара стартер + нестартер
#[inline(always)]
pub fn parse_pair(value: u64) -> DecodedValue
{
    DecodedValue::Pair(
        Codepoint {
            code: ((value >> 12) & 0x3FFFF) as u32,
            ccc: 0,
        },
        Codepoint {
            code: ((value >> 30) & 0x3FFFF) as u32,
            ccc: (value >> 4) as u8,
        },
        o!(value, Combining, 3),
    )
}

/// синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecodedValue
{
    DecodedValue::Singleton(o!(value, u32, 1), o!(value, Combining, 1))
}

/// нестартер без декомпозиции
#[inline(always)]
fn parse_nonstarter(value: u64) -> DecodedValue
{
    DecodedValue::Nonstarter((value >> 4) as u8)
}

/// случай, когда последовательность записана в отдельной таблице
#[inline(always)]
fn parse_expansion(value: u64) -> DecodedValue
{
    DecodedValue::Expansion(
        o!(value, Combining, 2),
        o!(value, u16, 1),
        (value as u8 >> 4) & 0b_1111,
        o!(value, u8, 1),
    )
}

/// комбинируется с предыдущим
#[inline(always)]
fn parse_combines_backwards(value: u64) -> DecodedValue
{
    DecodedValue::CombinesBackwards(o!(value, Combining, 1))
}
