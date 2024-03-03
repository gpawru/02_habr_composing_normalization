use crate::slice::iter::CharsIter;

/// получаем количество байт в последовательности UTF-8
#[inline(always)]
pub fn get_utf8_sequence_width(first: u8) -> u8
{
    match first {
        0 ..= 0x7F => 1,
        0xC2 ..= 0xDF => 2,
        0xE0 ..= 0xEF => 3,
        0xF0 ..= 0xF4 => 4,
        _ => 0,
    }
}

/// читаем первый байт UTF-8 последовательности без проверок
#[inline(always)]
pub unsafe fn char_first_byte_unchecked(iter: &mut CharsIter) -> u8
{
    *unsafe { iter.next_unchecked() }
}

/// читаем 2, 3, 4 байты последовательности UTF-8 и получаем код символа
#[inline(always)]
pub unsafe fn char_nonascii_bytes_unchecked(iter: &mut CharsIter, x: u8) -> u32
{
    // убираем старшие биты для случая с 2-байтовой последовательностью
    let init = utf8_first_byte(x, 2);
    let y = unsafe { *iter.next_unchecked() };
    let mut code = utf8_acc_cont_byte(init, y);

    // 3 байта
    if x >= 0xE0 {
        // 5й бит в диапазоне 0xE0 ..= 0xEF = 0, так что init здесь можно использовать
        let z = unsafe { *iter.next_unchecked() };
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        code = init << 12 | y_z;

        // 4 байта
        if x >= 0xF0 {
            let w = unsafe { *iter.next_unchecked() };
            // от init нам нужно 3 бита, сдвигаем их и комбинируем оставшуюся часть
            code = (init & 0x07) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }

    code
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
