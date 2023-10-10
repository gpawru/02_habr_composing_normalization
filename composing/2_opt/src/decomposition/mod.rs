use crate::o;

pub mod hangul;

/// последний кодпоинт с декомпозицией
pub const LAST_DECOMPOSING_CODEPOINT: u32 = 0x2FA1D;

/// стартер без декомпозиции
const MARKER_STARTER: u8 = 0;
/// не-стартер без декомпозиции
const MARKER_NON_STARTER: u8 = 1;
/// 16-битная пара
const MARKER_PAIR: u8 = 2;
/// синглтон
const MARKER_SINGLETON: u8 = 3;
/// декомпозиция, вынесенная во внешний блок
const MARKER_EXPANSION: u8 = 4;

pub enum DecompositionValue
{
    /// стартер, декомпозиция отсутствует
    None,
    /// не-стартер (например, диакретический знак)
    NonStarter(u8),
    /// декомпозиция на 2 кодпоинта, первый - стартер
    Pair(u32, Codepoint),
    /// декомпозиция на 3 кодпоинта, первый - стартер
    Triple(u32, Codepoint, Codepoint),
    /// синглтон (стартер, декомпозирующийся в другой стартер)
    Singleton(u32),
    /// декомпозиция на несколько символов, в параметрах - индекс первого элемента в дополнительной таблице и количество этих элементов
    Expansion(u16, u8),
    /// декомпозиция слога хангыль на 2 чамо. отличие от обычной пары в том, что все символы декомпозиции - стартеры
    ///
    /// если представить декомпозицию в UTF-8, то мы увидим: E1 84 80-92 E1 85 A1-B5. можно увидеть, что кодирование UTF-8
    /// в данном случае будет проще, чем в стандартном случае
    HangulPair(u8, u8),
    /// декомпозиция слога хангыль на 3 чамо, все элементы декомпозиции - стартеры
    ///
    /// аналогично предыдущему случаю, за исключением того, что добавляется ещё один символ - E1 86-87 80-BF
    HangulTriple(u8, u8, u8, u8),
}

impl DecompositionValue
{
    /// декомпозиция отсутствует?
    #[inline(always)]
    pub fn is_none(&self) -> bool
    {
        matches!(self, DecompositionValue::None)
    }
}

/// кодпоинт для декомпозиции
pub struct Codepoint
{
    /// класс комбинирования
    pub ccc: u8,
    /// код символа
    pub code: u32,
}

/// парсим значение из таблицы
#[inline(always)]
pub fn parse_data_value(value: u64) -> DecompositionValue
{
    match value as u8 {
        MARKER_STARTER => DecompositionValue::None,
        MARKER_NON_STARTER => parse_non_starter(value),
        MARKER_PAIR => parse_pair_16bit(value),
        MARKER_SINGLETON => parse_singleton(value),
        MARKER_EXPANSION => parse_expansion(value),
        _ => parse_triple_16bit(value),
    }
}

/// не-стартер без декомпозиции
#[inline(always)]
fn parse_non_starter(value: u64) -> DecompositionValue
{
    DecompositionValue::NonStarter(o!(value, u8, 1))
}

/// синглтон
#[inline(always)]
fn parse_singleton(value: u64) -> DecompositionValue
{
    DecompositionValue::Singleton(o!(value, u32, 1))
}

/// 16-битная пара
#[inline(always)]
fn parse_pair_16bit(value: u64) -> DecompositionValue
{
    DecompositionValue::Pair(
        o!(value, u16, 1) as u32,
        Codepoint {
            ccc: o!(value, u8, 1),
            code: o!(value, u16, 2) as u32,
        },
    )
}

/// 16-битная тройка
#[inline(always)]
fn parse_triple_16bit(value: u64) -> DecompositionValue
{
    DecompositionValue::Triple(
        o!(value, u16) as u32,
        Codepoint {
            code: o!(value, u16, 1) as u32,
            ccc: o!(value, u8, 6),
        },
        Codepoint {
            code: o!(value, u16, 2) as u32,
            ccc: o!(value, u8, 7),
        },
    )
}

/// декомпозиция, вынесенная во внешний блок
#[inline(always)]
fn parse_expansion(value: u64) -> DecompositionValue
{
    DecompositionValue::Expansion(o!(value, u16, 1), o!(value, u8, 1))
}
