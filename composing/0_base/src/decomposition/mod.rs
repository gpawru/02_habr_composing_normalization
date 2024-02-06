use crate::codepoint::Codepoint;
use crate::hangul::HangulVT;
use crate::o;
use crate::Combining;
use crate::Expansion;

/// последний кодпоинт таблицы с декомпозицией
pub const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;

/// стартер без декомпозиции
const MARKER_STARTER: u8 = 0;
/// стартер-синглтон
const MARKER_SINGLETON: u8 = 0b_001;
/// нестартер без декомпозиции
const MARKER_NONSTARTER: u8 = 0b_010;
/// 16-битная пара (стартер-нестартер)
const MARKER_PAIR: u8 = 0b_011;

#[derive(Debug)]
pub enum DecompositionValue
{
    /// стартер без декомпозиции
    None(Combining),
    /// нестартер без декомпозиции (например, диакретический знак)
    NonStarter(u8),
    /// 16-битная пара (стартер-нестартер)
    Pair(Codepoint, Codepoint, Combining),
    /// синглтон (стартер, декомпозирующийся в другой стартер)
    Singleton(Codepoint, Combining),
    /// последовательность кодпоинтов из отдельной таблицы
    Expansion(Expansion),
    /// кодпоинт хангыль чамо, который может быть скомбинирован с ранее идущим кодпоинтом
    Hangul(HangulVT),
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecompositionValue
{
    match value as u8 {
        MARKER_STARTER => parse_starter(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_NONSTARTER => parse_nonstarter(value),
        MARKER_PAIR => parse_pair(value),
        _ => parse_expansion(value),
    }
}

/// стартер без декомпозиции
#[inline(always)]
fn parse_starter(value: u64) -> DecompositionValue
{
    DecompositionValue::None(o!(value, Combining, 1))
}

/// нестартер без декомпозиции (например, диакретический знак)
#[inline(always)]
fn parse_nonstarter(value: u64) -> DecompositionValue
{
    DecompositionValue::NonStarter(o!(value, u8, 1))
}

/// 16-битная пара (стартер-нестартер)
#[inline(always)]
fn parse_pair(value: u64) -> DecompositionValue
{
    DecompositionValue::Pair(
        Codepoint {
            code: o!(value, u16, 2) as u32,
            ccc: 0,
        },
        Codepoint {
            code: o!(value, u16, 3) as u32,
            ccc: o!(value, u8, 1),
        },
        o!(value, Combining, 1),
    )
}

/// стартер-синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecompositionValue
{
    DecompositionValue::Singleton(
        Codepoint {
            code: o!(value, u32, 1),
            ccc: 0,
        },
        o!(value, Combining, 1),
    )
}

/// случаи, когда последовательность записана в отдельной таблице
#[inline(always)]
fn parse_expansion(value: u64) -> DecompositionValue
{
    DecompositionValue::Expansion(Expansion {
        marker: o!(value, u8, 0),
        len: o!(value, u8, 6),
        index: o!(value, u16, 2),
        combining: o!(value, Combining, 1),
    })
}
