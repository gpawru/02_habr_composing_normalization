use super::PropertiesError;

/// "зеркальный" символ в двунаправленном тексте, Bidi Mirrored
/// берется из UCD: девятая колонка UnicodeData.txt
///
/// например, круглые скобки.
///
/// см. раздел 4.7 документации,
/// https://www.unicode.org/versions/Unicode15.0.0/ch04.pdf
///
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BidiMirrored(bool);

impl BidiMirrored
{
    /// "зеркальный символ" символ?
    #[inline]
    pub fn is_mirrored(&self) -> bool
    {
        self.0
    }
}

impl TryFrom<&str> for BidiMirrored
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error>
    {
        Ok(match value {
            "Y" => Self(true),
            "N" => Self(false),
            _ => return Err(PropertiesError::UnknownPropertyValue),
        })
    }
}
