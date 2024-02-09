use crate::codepoint::Codepoint;
use crate::hangul::HangulVT;
use crate::o;
use crate::Combining;
use crate::Expansion;

/// последний кодпоинт таблицы с декомпозицией
pub const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;

/// стартер без декомпозиции
pub const MARKER_STARTER: u8 = 0;
/// пара стартер-нестартер
pub const MARKER_PAIR: u8 = 0b_001;
/// стартер-синглтон
pub const MARKER_SINGLETON: u8 = 0b_010;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 0b_011;

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
    Singleton(u32, Combining),
    /// последовательность кодпоинтов из отдельной таблицы
    Expansion(Expansion),
    /// кодпоинт хангыль чамо, который может быть скомбинирован с ранее идущим кодпоинтом
    Hangul(HangulVT),
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecompositionValue
{
    match value as u8 & 0b_111 {
        MARKER_STARTER => parse_starter(value),
        MARKER_PAIR => parse_pair(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_NONSTARTER => parse_nonstarter(value),
        _ => parse_expansion(value),
    }
}

/// стартер без декомпозиции
#[inline(always)]
pub fn parse_starter(value: u64) -> DecompositionValue
{
    DecompositionValue::None(parse_starter_value(value))
}

/// стартер без декомпозиции
#[inline(always)]
pub fn parse_starter_value(value: u64) -> Combining
{
    o!(value, Combining, 1)
}

/// нестартер без декомпозиции (например, диакретический знак)
#[inline(always)]
fn parse_nonstarter(value: u64) -> DecompositionValue
{
    DecompositionValue::NonStarter((value >> 4) as u8)
}

/// пара стартер-нестартер
#[inline(always)]
pub fn parse_pair(value: u64) -> DecompositionValue
{
    DecompositionValue::Pair(
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

/// стартер-синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecompositionValue
{
    DecompositionValue::Singleton(o!(value, u32, 1), o!(value, Combining, 1))
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
