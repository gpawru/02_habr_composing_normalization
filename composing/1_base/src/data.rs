/// структура хранимых данных для нормализации
pub struct DecompositionData<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    pub index: &'a [u8],
    /// основные данные
    pub data: &'a [u64],
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: &'a [u32],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
    /// первый кодпоинт в таблице, имеющий декомпозицию / не-стартер (для NFD - U+00C0, для NFKD - U+00A0)
    pub dec_starts_at: u32,
}

/// данные для NFD-нормализации
pub fn nfd<'a>() -> DecompositionData<'a>
{
    include!("./../../../data/nfd.rs.txt")
}

/// данные для NFKD-нормализации
pub fn nfkd<'a>() -> DecompositionData<'a>
{
    include!("./../../../data/nfkd.rs.txt")
}
