use super::PropertiesError;

/// класс направления текста (bidi class)
/// берется из UCD: четвертая колонка UnicodeData.txt
/// 23 варианта, укладывается в 5 бит
///
/// группы классов:
///     strong (L, R, AL) - сильный тип направления - символы, которые имеют явно заданное направление
///     weak (EN, ES, ET, AN, CS, NSM, BN) - слабый тип направления - символы, направление которых зависит от контекста
///     neutral (B, S, WS, ON) - нейтральные типы - символы, не имеющие определенного направления и не влияющие на направление письма
///     explicit (LRE, LRO, RLE, RLO, PDF, LRI, RLI, FSI, PDI) - явные типы форматирования - символы, используемые для явного управления направлением текста
///
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum BidiClass
{
    /// L - любой сильный LTR-символ
    LeftToRight = 1,
    /// R - любой сильный (не арабский) RTL-символ
    RightToLeft = 2,
    /// AL - любой сильный (арабский) RTL-символ
    ArabicLetter = 3,

    /// EN - любая цифра ASCII или восточно-арабская индийская цифра
    EuropeanNumber = 4,
    /// ES - знаки плюса и минуса
    EuropeanSeparator = 5,
    /// ET - терминатор в контексте числового формата, включает символы валюты
    EuropeanTerminator = 6,
    /// AN - любая арабско-индийская цифра
    ArabicNumber = 7,
    /// CS - запятые, двоеточия и слеши
    CommonSeparator = 8,
    /// NSM - не занимающий места символ, не оказывающий влияния на направление текста
    NonspacingMark = 9,
    /// BN - большинство символов форматирования, управляющие коды или недопустимые символы
    BoundaryNeutral = 10,

    /// B - различные символы новой строки, которые разделяют абзацы
    ParagraphSeparator = 12,
    /// S - различные управляющие коды, связанные с сегментами текста
    SegmentSeparator = 13,
    /// WS - пробельные символы, такие как пробелы и табуляции
    Whitespace = 14,
    /// ON - большинство других символов и знаков пунктуации, которые не имеют специального влияния на направление письма
    OtherNeutral = 15,

    /// LRE - U+202A - символ вставки слева направо (LR embedding control)
    LeftToRightEmbedding = 16,
    /// LRO - U+202D - символ переопределения слева направо (LR override control)
    LeftToRightOverride = 17,
    /// RLE - U+202B - символ вставки справа налево (RL embedding control)
    RightToLeftEmbedding = 18,
    /// RLO - U+202E - символ переопределения справа налево (RL override control)
    RightToLeftOverride = 19,
    /// PDF - U+202C - символ окончания направляющего форматирования (pop directional format)
    PopDirectionalFormat = 20,
    /// LRI - U+2066 - символ изоляции слева направо (LR isolate control)
    LeftToRightIsolate = 21,
    /// RLI - U+2067 - символ изоляции справа налево (RL isolate control)
    RightToLeftIsolate = 22,
    /// FSI - U+2068 - символ первой сильной изоляции (first strong isolate control)
    FirstStrongIsolate = 23,
    /// PDI - U+2069 - символ окончания изоляции направления (pop directional isolate)
    PopDirectionalIsolate = 24,
}

impl BidiClass
{
    /// является-ли сильным типом направления
    #[inline]
    pub fn is_strong(&self) -> bool
    {
        u8::from(*self) < 4
    }

    /// является-ли слабым типом направления
    #[inline]
    pub fn is_weak(&self) -> bool
    {
        let value = u8::from(*self);

        value & 0b_1111_1100 == 0b_0000_0100 || value & 0b_1111_1100 == 0b_0000_1000
    }

    /// является-ли нейтральным типом
    #[inline]
    pub fn is_neutral(&self) -> bool
    {
        u8::from(*self) & 0b_1111_1100 == 0b_0000_1100
    }

    /// является-ли явным типом
    #[inline]
    pub fn is_explicit(&self) -> bool
    {
        u8::from(*self) & 0b_1111_0000 == 0b_0001_0000
    }
}

impl TryFrom<&str> for BidiClass
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(abbr: &str) -> Result<Self, Self::Error>
    {
        Ok(match abbr {
            "L" => Self::LeftToRight,
            "R" => Self::RightToLeft,
            "AL" => Self::ArabicLetter,
            "EN" => Self::EuropeanNumber,
            "ES" => Self::EuropeanSeparator,
            "ET" => Self::EuropeanTerminator,
            "AN" => Self::ArabicNumber,
            "CS" => Self::CommonSeparator,
            "NSM" => Self::NonspacingMark,
            "BN" => Self::BoundaryNeutral,
            "B" => Self::ParagraphSeparator,
            "S" => Self::SegmentSeparator,
            "WS" => Self::Whitespace,
            "ON" => Self::OtherNeutral,
            "LRE" => Self::LeftToRightEmbedding,
            "LRO" => Self::LeftToRightOverride,
            "RLE" => Self::RightToLeftEmbedding,
            "RLO" => Self::RightToLeftOverride,
            "PDF" => Self::PopDirectionalFormat,
            "LRI" => Self::LeftToRightIsolate,
            "RLI" => Self::RightToLeftIsolate,
            "FSI" => Self::FirstStrongIsolate,
            "PDI" => Self::PopDirectionalIsolate,
            _ => return Err(PropertiesError::UnknownPropertyValue),
        })
    }
}

impl TryFrom<u8> for BidiClass
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        if value == 0 || value == 11 || value > 24 {
            return Err(PropertiesError::UnknownPropertyValue);
        }

        Ok(unsafe { core::mem::transmute::<u8, BidiClass>(value) })
    }
}

impl From<BidiClass> for u8
{
    #[inline]
    fn from(value: BidiClass) -> Self
    {
        unsafe { core::mem::transmute::<BidiClass, u8>(value) }
    }
}
