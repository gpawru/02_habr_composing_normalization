/// распакованная информация о комбинировании -
/// индекс в таблице комбинаций и количество записанных для кодпоинта вариантов
pub struct CodepointCombining
{
    index: u16,
    count: u16,
}

impl From<u16> for CodepointCombining
{
    fn from(value: u16) -> Self
    {
        Self {
            index: value & 0x7FF,
            count: value >> 11,
        }
    }
}

/// результат комбинирования кодпоинтов
pub enum CombineResult
{
    /// кодпоинты скомбинированы, полученный кодпоинт также может быть скомбинирован
    Combined(u32, u16),
    /// кодпоинты скомбинированы, полученный кодпоинт не может быть скомбинирован
    Final(u32),
    /// кодпоинты не комбинируются
    None,
}

/// скомбинировать два кодпоинта
#[inline(always)]
pub fn combine(combining: &CodepointCombining, second: u32, compositions: &[u64]) -> CombineResult
{
    let first = combining.index as usize;
    let last = first + combining.count as usize;

    for entry in &compositions[first .. last] {
        let entry = *entry;
        let entry_codepoint = entry as u32 & 0x3FFFF;

        // кодпоинты комбинируются
        if entry_codepoint == second {
            let code = (entry >> 18) as u32 & 0x3FFFF;
            let combining = (entry >> 48) as u16;

            return match combining {
                0 => CombineResult::Final(code),
                _ => CombineResult::Combined(code, combining),
            };
        }
    }

    CombineResult::None
}
