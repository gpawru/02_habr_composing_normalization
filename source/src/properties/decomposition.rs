use super::PropertiesError;

/// декомпозиция
/// берется из UCD: 5 колонка UnicodeData.txt
#[derive(Debug, Clone)]
pub struct Decomposition
{
    /// декомпозиция
    pub codes: Vec<u32>,
    /// тег декомпозиции
    pub tag: Option<DecompositionTag>,
}

impl TryFrom<&str> for Decomposition
{
    type Error = PropertiesError;

    fn try_from(value: &str) -> Result<Self, Self::Error>
    {
        let (tag_string, decomposition_string) = match value.starts_with('<') {
            true => value.split_once(' ').unwrap(),
            false => ("", value),
        };

        let tag = match !tag_string.is_empty() {
            true => Some(DecompositionTag::try_from(tag_string)?),
            false => None,
        };

        let codes: Vec<u32> = decomposition_string
            .split_whitespace()
            .map(|v| u32::from_str_radix(v, 16).unwrap())
            .collect();

        Ok(Self { codes, tag })
    }
}

/// тег декомпозиции
/// берется из UCD: 5 колонка UnicodeData.txt
/// флаг наличия + 16 вариантов, достаточно 5 бит
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum DecompositionTag
{
    /// вариант шрифта
    Font = 0,
    /// неразрывная версия пробела или дефиса
    NoBreak = 1,
    /// начальная форма представления (арабский)
    Initial = 2,
    /// средняя форма представления (арабский)
    Medial = 3,
    /// конечная форма представления (арабский)
    Final = 4,
    /// изолированная форма представления (арабский)
    Isolated = 5,
    /// окруженная форма
    Circle = 6,
    /// надстрочная форма
    Super = 7,
    /// подстрочная форма
    Sub = 8,
    /// вертикальная форма представления
    Vertical = 9,
    /// совместимый символ широкого формата (или зэнкаку)
    Wide = 10,
    /// совместимый символ узкого формата (или ханкаку)
    Narrow = 11,
    /// малая вариантная форма (совместимость CNS (Chinese National Standard))
    Small = 12,
    /// вариант шрифта в квадрате CJK
    Square = 13,
    /// форма обыкновенной дроби
    Fraction = 14,
    /// неопределенный символ для обеспечения совместимости
    Compat = 15,
}

impl TryFrom<&str> for DecompositionTag
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(abbr: &str) -> Result<Self, Self::Error>
    {
        Ok(match abbr {
            "<font>" => Self::Font,
            "<noBreak>" => Self::NoBreak,
            "<initial>" => Self::Initial,
            "<medial>" => Self::Medial,
            "<final>" => Self::Final,
            "<isolated>" => Self::Isolated,
            "<circle>" => Self::Circle,
            "<super>" => Self::Super,
            "<sub>" => Self::Sub,
            "<vertical>" => Self::Vertical,
            "<wide>" => Self::Wide,
            "<narrow>" => Self::Narrow,
            "<small>" => Self::Small,
            "<square>" => Self::Square,
            "<fraction>" => Self::Fraction,
            "<compat>" => Self::Compat,
            _ => return Err(PropertiesError::UnknownPropertyValue),
        })
    }
}

impl TryFrom<u8> for DecompositionTag
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        match value & 0xe0 == 0 {
            true => Ok(unsafe { core::mem::transmute::<u8, DecompositionTag>(value) }),
            false => Err(PropertiesError::UnknownPropertyValue),
        }
    }
}

impl From<DecompositionTag> for u8
{
    #[inline]
    fn from(value: DecompositionTag) -> Self
    {
        unsafe { core::mem::transmute::<DecompositionTag, u8>(value) }
    }
}

impl core::fmt::Display for DecompositionTag
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result
    {
        let value = match self {
            DecompositionTag::Font => "<font>",
            DecompositionTag::NoBreak => "<noBreak>",
            DecompositionTag::Initial => "<initial>",
            DecompositionTag::Medial => "<medial>",
            DecompositionTag::Final => "<final>",
            DecompositionTag::Isolated => "<isolated>",
            DecompositionTag::Circle => "<circle>",
            DecompositionTag::Super => "<super>",
            DecompositionTag::Sub => "<sub>",
            DecompositionTag::Vertical => "<vertical>",
            DecompositionTag::Wide => "<wide>",
            DecompositionTag::Narrow => "<narrow>",
            DecompositionTag::Small => "<small>",
            DecompositionTag::Square => "<square>",
            DecompositionTag::Fraction => "<fraction>",
            DecompositionTag::Compat => "<compat>",
        };

        write!(f, "{}", value)
    }
}
