/// структура хранимых данных для нормализации
pub struct CompositionData<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFC последний блок - 0x40, в NFKC - 0x6F (+1 для пустого блока)
    pub index: &'a [u8],
    /// основные данные
    pub data: &'a [u64],
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: &'a [u32],
    /// композиции
    pub compositions: &'a [u64],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
}

/// данные для NFC-нормализации
pub fn nfc<'a>() -> CompositionData<'a>
{
    include!("./../../../data/nfc.rs.txt")
}

/// данные для NFKC-нормализации
pub fn nfkc<'a>() -> CompositionData<'a>
{
    include!("./../../../data/nfkc.rs.txt")
}
