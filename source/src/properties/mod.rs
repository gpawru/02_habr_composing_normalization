mod bidi_class;
mod bidi_mirrored;
mod canonical_combining_class;
mod decomposition;
mod general_category;
mod numeric_type;
mod simple_case_mapping;

pub use bidi_class::BidiClass;
pub use bidi_mirrored::BidiMirrored;
pub use canonical_combining_class::CanonicalCombiningClass;
pub use decomposition::Decomposition;
pub use decomposition::DecompositionTag;
pub use general_category::GeneralCategory;
pub use numeric_type::NumericType;
pub use simple_case_mapping::SimpleCaseMapping;

/// Кодпоинт Unicode
/// источник - UCD, UnicodeData.txt
#[derive(Debug, Clone)]
pub struct Codepoint
{
    /// код символа
    pub code: u32,
    /// название
    pub name: String,
    /// категория символа (general category)
    pub gc: GeneralCategory,
    /// класс канонического комбинирования (canonical combining class)
    pub ccc: CanonicalCombiningClass,
    /// класс направления (bidi class)
    pub bc: BidiClass,
    /// числовой тип
    pub numeric: NumericType,
    /// "зеркальный" символ двунаправленого текста (bidi mirrored)
    pub bidi_mirrored: BidiMirrored,
    /// соответствующая прописная буква
    pub simple_uppercase_mapping: SimpleCaseMapping,
    /// соответствующая строчная буква
    pub simple_lowercase_mapping: SimpleCaseMapping,
    /// соответствующая заглавная буква
    pub simple_titlecase_mapping: SimpleCaseMapping,
    /// тег декомпозиции
    pub decomposition_tag: Option<DecompositionTag>,
    /// декомпозиция
    pub decomposition: Vec<u32>,
}

impl Codepoint
{
    // стартер?
    pub fn is_starter(&self) -> bool
    {
        self.ccc.is_starter()
    }

    // нестартер?
    pub fn is_nonstarter(&self) -> bool
    {
        self.ccc.is_nonstarter()
    }

    // как символ
    pub fn as_char(&self) -> char
    {
        char::from_u32(self.code).unwrap()
    }

    /// ASCII (Basic Latin)?
    pub fn is_ascii(&self) -> bool
    {
        self.code <= 0x7F
    }

    /// Basic Multilingual Plane?
    pub fn is_bmp(&self) -> bool
    {
        self.code <= 0xFFFF
    }
}

#[derive(Debug, PartialEq)]
pub enum PropertiesError
{
    UnknownPropertyValue,
}

impl From<core::num::ParseIntError> for PropertiesError
{
    fn from(_: core::num::ParseIntError) -> Self
    {
        Self::UnknownPropertyValue
    }
}
