use super::PropertiesError;

/// числовое значение (Numeric Type, Numeric Value)
/// берется из UCD: 6, 7, 8 колонки UnicodeData.txt
///
/// кроме значений, присутствующих в UnicodeData, имеет смысл обратить внимание на значения этого свойства в CJK
/// см. https://www.unicode.org/versions/Unicode15.0.0/ch04.pdf, глава 4.6, раздел Ideographic Numeric Values
///
#[derive(Debug, Clone)]
pub enum NumericType
{
    /// не является числовым значением
    None,
    /// десятичное, от 0 до 9
    Decimal(u8),
    /// цифра, от 0 до 9
    Digit(u8),
    /// числовое (например, дробь)
    Numeric(String),
}

impl NumericType
{
    pub fn is_some(&self) -> bool
    {
        !matches!(self, NumericType::None)
    }

    pub fn is_none(&self) -> bool
    {
        matches!(self, NumericType::None)
    }
}

impl TryFrom<(&str, &str, &str)> for NumericType
{
    type Error = PropertiesError;

    fn try_from(v: (&str, &str, &str)) -> Result<Self, Self::Error>
    {
        let mask = u8::from(!v.0.is_empty())
            | u8::from(!v.1.is_empty()) << 1
            | u8::from(!v.2.is_empty()) << 2;

        let value = match mask {
            0b111 => Self::Decimal(v.0.parse()?),
            0b110 => Self::Digit(v.1.parse()?),
            0b100 => Self::Numeric(v.2.to_owned()),
            0b000 => Self::None,
            _ => return Err(PropertiesError::UnknownPropertyValue),
        };

        Ok(value)
    }
}
