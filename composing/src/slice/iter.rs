use core::marker::PhantomData;
use core::slice::from_raw_parts;

#[repr(align(16))]
pub struct CharsIter<'a>
{
    ptr: *const u8,
    end: *const u8,
    breakpoint: *const u8,
    _marker: PhantomData<&'a u8>,
}

impl<'a> CharsIter<'a>
{
    #[inline(always)]
    pub fn new(str: &'a str) -> Self
    {
        unsafe {
            let length = str.len() as isize;
            let ptr = str.as_ptr();
            let end = ptr.offset(length);

            Self {
                ptr,
                breakpoint: ptr,
                end,
                _marker: PhantomData,
            }
        }
    }

    /// запомнить текущую позицию
    #[inline(always)]
    pub fn set_breakpoint(&mut self)
    {
        self.breakpoint = self.ptr;
    }

    /// указатель на запомненной позиции?
    #[inline(always)]
    pub fn at_breakpoint(&mut self, offset: isize) -> bool
    {
        unsafe { self.ptr.offset_from(self.breakpoint) - offset == 0 }
    }

    /// данные закончились?
    #[inline(always)]
    pub fn is_empty(&self) -> bool
    {
        unsafe { self.end.offset_from(self.ptr) == 0 }
    }

    /// прочитать байт без проверки длины оставшихся данных
    #[inline(always)]
    pub unsafe fn next_unchecked(&mut self) -> u8
    {
        let old = self.ptr;
        self.ptr = unsafe { self.ptr.add(1) };
        *old
    }

    /// если мы знаем, что последующие байты - 2, 3, 4 байты UTF-8 - читаем их без проверок
    #[inline(always)]
    pub unsafe fn next_nonascii_bytes_unchecked(&mut self, x: u8) -> u32
    {
        // убираем старшие биты для случая с 2-байтовой последовательностью
        let init = utf8_first_byte(x, 2);
        let y = unsafe { self.next_unchecked() };
        let mut code = utf8_acc_cont_byte(init, y);

        // 3 байта
        if x >= 0xE0 {
            // 5й бит в диапазоне 0xE0 ..= 0xEF = 0, так что init здесь можно использовать
            let z = unsafe { self.next_unchecked() };
            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            code = init << 12 | y_z;

            // 4 байта
            if x >= 0xF0 {
                let w = unsafe { self.next_unchecked() };
                // от init нам нужно 3 бита, сдвигаем их и комбинируем оставшуюся часть
                code = (init & 0x07) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }

        code
    }

    /// конечный участок слайса от запомненной позиции
    #[inline]
    pub fn ending_slice(&self) -> &[u8]
    {
        unsafe {
            let length = self.end.offset_from(self.breakpoint) as usize;
            from_raw_parts(self.breakpoint, length)
        }
    }

    /// слайс от запомненной позиции до текущего указателя, минус поправка
    #[inline]
    pub fn block_slice(&self, offset: isize) -> &[u8]
    {
        unsafe {
            let length = (self.ptr.offset_from(self.breakpoint) - offset) as usize;
            from_raw_parts(self.breakpoint, length)
        }
    }
}

/// маска, использующаяся для получения битов значения первого байта UTF-8
const FIRST_BYTE_VALUE_MASK: u8 = 0x7F;
/// маска, исключащая 2 старших бита в 2, 3, 4 байтах последовательности UTF-8
const CONT_MASK: u8 = 0x3F;

/// убираем старшие биты первого байта UTF-8 последовательности
#[inline(always)]
fn utf8_first_byte(byte: u8, width: u32) -> u32
{
    (byte & (FIRST_BYTE_VALUE_MASK >> width)) as u32
}

/// убираем 2 старших бита у следующего байта последовательности и комбинируем с предыдущим значением
#[inline(always)]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32
{
    (ch << 6) | (byte & CONT_MASK) as u32
}
