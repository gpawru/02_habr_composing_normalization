/// кодпоинт
#[derive(Debug, Clone, Copy)]
pub struct Codepoint
{
    /// класс комбинирования
    pub ccc: u8,
    /// код символа
    pub code: u32,
}

impl Codepoint
{
    /// в виде char. проверка не нужна, т.к. имеем априори валидный кодпоинт Unicode
    #[inline(always)]
    pub fn char(&self) -> char
    {
        unsafe { char::from_u32_unchecked(self.code) }
    }

    /// из "сжатого" u32, где CCC хранится в старших битах
    #[inline(always)]
    pub fn from_compressed(value: u32) -> Self
    {
        Self {
            ccc: (value >> 24) as u8,
            code: value & 0x3FFFF,
        }
    }
}
