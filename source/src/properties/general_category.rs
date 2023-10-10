use super::PropertiesError;

/// основная категория символа (General Category, GC)
/// берется из UCD: вторая колонка UnicodeData.txt
/// всего 31 вариант, что укладывается в 5 бит
/// варианты отсортированы таким образом, чтобы было проще применять побитовые операции для получения общей категории
///
/// общие категории:
///     LC (Lu, Ll, Lt) - буквы, имеющие регистр
///     L (Lu, Ll, Lt, Lm, Lo) - буквы
///     M (Mn, Mc, Me) - комбинирующие символы
///     N (Nd, Nl, No) - цифры, числовые символы
///     P (Pc, Pd, Ps, Pe, Pi, Pf, Po) - знаки препинания
///     S (Sm, Sc, Sk, So) - различные символы (математические, валюты и т.д.)
///     Z (Zs, Zl, Zp) - разделители
///     C (Cc, Cf, Cs, Co, Cn) - системные символы
///
#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum GeneralCategory
{
    /// Cn - место под символ зарезервировано или не назначено, или же элемент не является символом.
    /// дефолтный вариант при отсутствии записи о символе в UCD
    Unassigned = 0, // 0b_0000_0000

    /// Lu - прописная буква
    UppercaseLetter = 1, // 0b_0000_0001
    /// Ll - строчная буква
    LowercaseLetter = 2, // 0b_0000_0010
    /// Lt - диграфический символ, первая часть - заглавная буква
    TitlecaseLetter = 3, // 0b_0000_0011

    /// Lm - буква-модификатор
    ModifierLetter = 4, // 0b_0000_0100
    /// Lo - прочие буквы, включая слоги и иероглифы
    OtherLetter = 5, // 0b_0000_0101

    /// Mn - неразрывный комбинирующий маркер (не занимающий пространства)
    NonspacingMark = 6, // 0b_0000_0110
    /// Mc - комбинирующий маркер, занимающий пространство
    SpacingMark = 7, // 0b_0000_0111
    /// Me - охватывающий комбинирующий маркер
    EnclosingMark = 8, // 0b_0000_1000

    /// Nd - десятичная цифра
    DecimalNumber = 9, // 0b_0000_1001
    /// Nl - буквоподобный числовой символ
    LetterNumber = 10, // 0b_0000_1010
    /// No - прочие числовые символы
    OtherNumber = 11, // 0b_0000_1011

    /// Zs - разделитель-пробел
    SpaceSeparator = 12, // 0b_0000_1100
    /// Zl - разделитель строки
    LineSeparator = 13, // 0b_0000_1101
    /// Zp - разделитель параграфов
    ParagraphSeparator = 14, // 0b_0000_1110

    /// Cc - управляющий символ, относится к C0 или C1
    Control = 16, // 0b_0001_0000
    /// Cf - управляющий символ форматирования
    Format = 17, // 0b_0001_0001
    /// Cs - символ-суррогат
    Surrogate = 18, // 0b_0001_0010
    /// Co - символ для приватного использования
    PrivateUse = 19, // 0b_0001_0011

    /// Pc - объединяющяя пунктуация, например _
    ConnectorPunctuation = 20, // 0b_0001_0100
    /// Pd - тире или дефис как знак препинания
    DashPunctuation = 21, // 0b_0001_0101
    /// Ps - открывающий знак пунктуации (из пары)
    OpenPunctuation = 22, // 0b_0001_0110
    /// Pe - закрывающий знак пунктуации (из пары)
    ClosePunctuation = 23, // 0b_0001_0111
    /// Pi - начальный знак цитаты
    InitialPunctuation = 24, // 0b_0001_1000
    /// Pf - конечный знак цитаты
    FinalPunctuation = 25, // 0b_0001_1001
    /// Po - знак препинания другого типа
    OtherPunctuation = 26, // 0b_0001_1010

    /// Sm - математический символ
    MathSymbol = 28, // 0b_0001_1100
    /// Sc - символ валюты
    CurrencySymbol = 29, // 0b_0001_1101
    /// Sk - символ модификатора, не похожий на букву
    ModifierSymbol = 30, // 0b_0001_1110
    /// So - прочие символы
    OtherSymbol = 31, // 0b_0001_1111
}

impl GeneralCategory
{
    /// относится-ли категория к буквам с регистром (LC)
    #[inline]
    pub fn is_cased_letter(&self) -> bool
    {
        !self.is_unassigned() && u8::from(*self) < 4
    }

    /// относится-ли категория к буквам (L)
    #[inline]
    pub fn is_letter(&self) -> bool
    {
        !self.is_unassigned() && u8::from(*self) < 6
    }

    /// относится-ли категория к комбинирующим символам (M)
    #[inline]
    pub fn is_combining_mark(&self) -> bool
    {
        let value = u8::from(*self);

        value & 0b_1111_1110 == 0b_0000_0110 || value == 0b_0000_1000
    }

    /// относится-ли категория к цифрам и числовым символам (N)
    #[inline]
    pub fn is_numeric(&self) -> bool
    {
        let value = u8::from(*self);

        value & 0b_1111_1100 == 0b_0000_1000 && value != 0b_0000_1000
    }

    /// относится-ли категория к разделителям (Z)
    #[inline]
    pub fn is_separator(&self) -> bool
    {
        u8::from(*self) & 0b_1111_1100 == 0b_0000_1100
    }

    /// относится-ли категория к управляющим символам (или не назначена) (C)
    #[inline]
    pub fn is_control(&self) -> bool
    {
        self.is_unassigned() || u8::from(*self) & 0b_1111_1100 == 0b_0001_0000
    }

    /// категория не назначена (Cn)
    #[inline]
    pub fn is_unassigned(&self) -> bool
    {
        u8::from(*self) == 0
    }

    /// относится-ли категория к пунктуации (P)
    #[inline]
    pub fn is_punctuation(&self) -> bool
    {
        let masked = u8::from(*self) & 0b_1111_1100;
        masked == 0b_0001_0100 || masked == 0b_0001_1000
    }

    /// относится-ли категория к символам (S)
    #[inline]
    pub fn is_symbol(&self) -> bool
    {
        u8::from(*self) & 0b_1111_1100 == 0b_0001_1100
    }
}

impl TryFrom<&str> for GeneralCategory
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(abbr: &str) -> Result<Self, Self::Error>
    {
        Ok(match abbr {
            "Cn" | "" => Self::Unassigned,
            "Lu" => Self::UppercaseLetter,
            "Ll" => Self::LowercaseLetter,
            "Lt" => Self::TitlecaseLetter,
            "Lm" => Self::ModifierLetter,
            "Lo" => Self::OtherLetter,
            "Mn" => Self::NonspacingMark,
            "Mc" => Self::SpacingMark,
            "Me" => Self::EnclosingMark,
            "Nd" => Self::DecimalNumber,
            "Nl" => Self::LetterNumber,
            "No" => Self::OtherNumber,
            "Zs" => Self::SpaceSeparator,
            "Zl" => Self::LineSeparator,
            "Zp" => Self::ParagraphSeparator,
            "Cc" => Self::Control,
            "Cf" => Self::Format,
            "Cs" => Self::Surrogate,
            "Co" => Self::PrivateUse,
            "Pc" => Self::ConnectorPunctuation,
            "Pd" => Self::DashPunctuation,
            "Ps" => Self::OpenPunctuation,
            "Pe" => Self::ClosePunctuation,
            "Pi" => Self::InitialPunctuation,
            "Pf" => Self::FinalPunctuation,
            "Po" => Self::OtherPunctuation,
            "Sm" => Self::MathSymbol,
            "Sc" => Self::CurrencySymbol,
            "Sk" => Self::ModifierSymbol,
            "So" => Self::OtherSymbol,
            _ => return Err(PropertiesError::UnknownPropertyValue),
        })
    }
}

impl TryFrom<u8> for GeneralCategory
{
    type Error = PropertiesError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error>
    {
        if value == 15 || value == 27 || value > 31 {
            return Err(PropertiesError::UnknownPropertyValue);
        }

        Ok(unsafe { core::mem::transmute::<u8, GeneralCategory>(value) })
    }
}

impl From<GeneralCategory> for u8
{
    #[inline]
    fn from(value: GeneralCategory) -> Self
    {
        unsafe { *(&value as *const GeneralCategory as *const u8) }
    }
}
