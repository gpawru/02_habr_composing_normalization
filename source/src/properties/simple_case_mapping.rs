use super::PropertiesError;

/// Simple Uppercase/Lowercase/Titlecase Mapping
/// берется из UCD: 12, 13, 14 колонки UnicodeData.txt
///
/// соответствующая символу прописная/строчная/заглавная буква, один символ
///
/// более детально - https://www.unicode.org/reports/tr44/#Casemapping
///
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SimpleCaseMapping
{
    None,
    Some(u32),
}

impl SimpleCaseMapping {}

impl TryFrom<&str> for SimpleCaseMapping
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error>
    {
        Ok(match value.is_empty() {
            true => Self::None,
            false => match u32::from_str_radix(value, 16) {
                Ok(value) => Self::Some(value),
                Err(_) => return Err(PropertiesError::UnknownPropertyValue),
            },
        })
    }
}
