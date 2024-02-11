/// информация о комбинировании стартера с идущими следом кодпоинтами
#[derive(Debug, Clone, Copy)]
pub struct Combining(u16);

impl Combining
{
    /// некомбинируемый стартер
    #[allow(non_upper_case_globals)]
    pub const None: Self = Self(0);

    /// стартер не комбинируется?
    #[inline(always)]
    pub fn is_none(&self) -> bool
    {
        self.0 == 0
    }

    /// индекс в таблице комбинирования
    #[inline(always)]
    pub fn index(&self) -> u16
    {
        self.0 & 0x7FF
    }

    /// количество вариантов
    #[inline(always)]
    pub fn count(&self) -> u16
    {
        self.0 >> 11
    }
}

impl From<u16> for Combining
{
    fn from(value: u16) -> Self
    {
        Self(value)
    }
}

/// результат комбинирования кодпоинтов
pub enum CombineResult
{
    /// кодпоинты скомбинированы, полученный кодпоинт также может быть скомбинирован
    Combined(u32, Combining),
    /// кодпоинты скомбинированы, полученный кодпоинт не может быть скомбинирован
    Final(u32),
    /// кодпоинты не комбинируются
    None,
}

/// скомбинировать два кодпоинта
#[inline(always)]
pub fn combine(combining: Combining, second: u32, compositions_table: &[u64]) -> CombineResult
{
    let first = combining.index();
    let last = first + combining.count();

    for entry in &compositions_table[first as usize .. last as usize] {
        let entry = *entry;

        let entry_codepoint = entry as u32 & 0x3FFFF;

        // кодпоинты комбинируются
        if entry_codepoint == second {
            let code = (entry >> 18) as u32 & 0x3FFFF;
            let combining = (entry >> 48) as u16;

            return match combining {
                0 => CombineResult::Final(code),
                _ => CombineResult::Combined(code, Combining::from(combining)),
            };
        }
    }

    CombineResult::None
}
