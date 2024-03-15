/// структура хранимых данных для нормализации
pub struct DecompositionData<'a>
{
    /// индекс блока
    pub index: &'a [u16],
    /// основные данные
    pub data: &'a [u32],
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: &'a [u32],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
}

/// данные для комбинирования кодпоинтов
pub struct CompositionData<'a>
{
    pub compositions: &'a [u64],
}

/// замена декомпозиций для NF(K)C
pub struct ExpansionsPatch<'a>
{
    pub expansions: &'a [u32],
}

/// данные для NFD-нормализации
pub fn nfd<'a>() -> DecompositionData<'a>
{
    include!("./../../data/nfd.txt")
}

/// данные для NFKD-нормализации
pub fn nfkd<'a>() -> DecompositionData<'a>
{
    include!("./../../data/nfkd.txt")
}

/// данные комбинирования для NF(K)C-нормализации
pub fn compositions<'a>() -> CompositionData<'a>
{
    include!("./../../data/compositions.txt")
}

/// замена части декомпозиций для NFC
pub fn nfc_expansions<'a>() -> ExpansionsPatch<'a>
{
    include!("./../../data/nfc.txt")
}

/// замена части декомпозиций для NFKC
pub fn nfkc_expansions<'a>() -> ExpansionsPatch<'a>
{
    include!("./../../data/nfkc.txt")
}
