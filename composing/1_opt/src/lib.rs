use codepoint::decode::{is_nonstarters_value, parse_data_value, DecodedValue};
use codepoint::Codepoint;
use composition::*;
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod codepoint;
mod composition;
mod data;
mod macros;
mod slice;
mod utf8;

/// последний кодпоинт таблицы с декомпозицией
const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;

/// нормализатор NF(K)C
#[repr(align(128))]
pub struct ComposingNormalizer<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFC последний блок - 0x40, в NFKC - 0x6F (+1 для пустого блока)
    pub index: Aligned<'a, u8>,
    /// основные данные
    pub data: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: Aligned<'a, u32>,
    /// композиции
    pub compositions: Aligned<'a, u64>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
}

impl<'a> From<data::CompositionData<'a>> for ComposingNormalizer<'a>
{
    fn from(source: data::CompositionData<'a>) -> Self
    {
        Self {
            index: Aligned::from(source.index),
            data: Aligned::from(source.data),
            expansions: Aligned::from(source.expansions),
            compositions: Aligned::from(source.compositions),
            continuous_block_end: source.continuous_block_end,
        }
    }
}

macro_rules! normalization_method {
    ($method: ident, $bound: expr, $bound_first_byte: expr) => {
        #[inline(never)]
        pub fn $method(&self, input: &str) -> String
        {
            let mut result = String::with_capacity(input.len());
            let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);
            let mut combining = Combining::None;

            macro_rules! combine {
                () => {
                    combine_and_write(&mut buffer, &mut result, combining, &self.compositions);
                };
            }
            macro_rules! to_buffer {
                ($codepoint: expr) => {
                    buffer.push($codepoint);
                };
                ($code: expr, $ccc: expr) => {
                    buffer.push(Codepoint {
                        ccc: $ccc,
                        code: $code,
                    })
                };
            }

            let iter = &mut CharsIter::new(input);

            loop {
                iter.set_breakpoint();

                let (data_value, code) = loop {
                    // всё прочитали - комбинируем предшествующий текущему отрезку буфер, дописываем остаток
                    if iter.is_empty() {
                        combine_and_write(&mut buffer, &mut result, combining, &self.compositions);
                        write_str!(result, iter.ending_slice());

                        return result;
                    }

                    let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                    // текст, состоящий только из ASCII-символов уже NF(KC) нормализован
                    // учитывая то, что для NFC и NFKC символы до U+0300 и U+00A0 соответственно также нормализованы,
                    // используем не 0x80 в качестве границы, а значение первого байта UTF-8 вышеуказанных символов.
                    if first < $bound_first_byte || first >> 6 == 0b_10 {
                        continue;
                    }

                    let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

                    if code > LAST_DECOMPOSITION_CODE {
                        continue;
                    }

                    let data_value = self.get_data_value(code);

                    // является ли кодпоинт нормализованым? если - "да" или "возможно" (он считается
                    // нормализованным, если за ним идёт стартер) - продолжаем быстрый цикл
                    if data_value as u8 & 1 == 0 {
                        continue;
                    }

                    // выходим из быстрого цикла, т.к. мы столкнулись с ситуацией, когда требуется
                    // декомпозиция / комбинирование

                    // не учитываем однобайтовый символ, т.к. ранее мы их отсекли
                    let width = match code {
                        0x00 ..= 0x7F => unreachable!(),
                        0x80 ..= 0x07FF => 2,
                        0x0800 ..= 0xFFFF => 3,
                        0x10000 ..= 0x10FFFF => 4,
                        _ => unreachable!(),
                    };

                    // если у нас есть последовательность кодпоинтов в быстром цикле - комбинируем предшествующие
                    // ему кодпоинты буфера, дописываем полученное в результат
                    if !iter.at_breakpoint(width) {
                        combine!();
                        write_str!(result, iter.block_slice(width));

                        // в случае, если получили нестартер, нам потребуется предшествующий ему кодпоинт.
                        // он может быть только стартером / парой / частным случаем расширения -
                        // стартер + несколько нестартеров
                        // (смотрим, какие метки быстрых проверок используются в каких типах кодирования таблицы)
                        if is_nonstarters_value(data_value) {
                            let previous = u32::from(result.pop().unwrap());
                            combining = self.buffer_previous(&mut buffer, previous);
                        }
                    }

                    break (data_value, code);
                };

                let decoded = parse_data_value(data_value);

                match decoded {
                    DecodedValue::Pair(starter, nonstarter, new_combining) => {
                        combine!();
                        to_buffer!(starter);
                        to_buffer!(nonstarter);
                        combining = new_combining;
                    }
                    DecodedValue::Nonstarter(ccc) => {
                        to_buffer!(code, ccc);
                    }
                    DecodedValue::Singleton(code, new_combining) => {
                        combine!();
                        to_buffer!(code, 0);
                        combining = new_combining;
                    }
                    DecodedValue::Starter(_) => {
                        combine!();
                        // в этот блок попадают только стартеры - чамо хангыль
                        combine_and_write_hangul_vt(&mut result, code);
                        combining = Combining::None;
                    }

                    DecodedValue::Expansion(new_combining, index, ns_len, len) => {
                        let index = index as usize;
                        let st_len = len - ns_len;
                        let buffer_from = st_len.saturating_sub(1) as usize;

                        let slice = &self.expansions[index .. index + len as usize];

                        if st_len != 0 {
                            combine!();
                            combining = new_combining;

                            if st_len > 1 {
                                slice[.. buffer_from]
                                    .iter()
                                    .for_each(|c| write!(result, *c));
                            }
                        }

                        slice[buffer_from ..].iter().for_each(|c| {
                            to_buffer!(Codepoint::from_compressed(*c));
                        });
                    }
                    DecodedValue::CombinesBackwards(backwards_combining) => {
                        combining = combine_backwards(
                            &mut buffer,
                            &mut result,
                            code,
                            combining,
                            backwards_combining,
                            &self.compositions,
                        );
                    }
                }
            }
        }
    };
}

impl<'a> ComposingNormalizer<'a>
{
    /// NFC-нормализатор
    pub fn nfc() -> Self
    {
        Self::from(data::nfc())
    }

    /// NFKC-нормализатор
    pub fn nfkc() -> Self
    {
        Self::from(data::nfkc())
    }
    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String
    {
        match self.get_data_value(0xA0) {
            0 => self.normalize_nfc(input),
            _ => self.normalize_nfkc(input),
        }
    }

    normalization_method!(normalize_nfc, 0x0300, 0xCC);
    normalization_method!(normalize_nfkc, 0x00A0, 0xC2);

    /// данные о декомпозиции / композиции кодпоинта
    #[inline(always)]
    fn get_data_value(&self, code: u32) -> u64
    {
        if code <= self.continuous_block_end {
            self.data[code as usize]
        } else {
            let block_index = (code >> 7) as usize;
            let block = self.index[block_index] as usize;

            let block_offset = block << 7;
            let code_offset = ((code as u8) & 0x7F) as usize;

            let index = block_offset | code_offset;

            self.data[index]
        }
    }

    /// записать в буфер декомпозицию (точнее, прекомпозицию) последнего полученного кодпоинта
    /// для комбинирования с нестартером
    #[inline]
    fn buffer_previous(&self, buffer: &mut Vec<Codepoint>, code: u32) -> Combining
    {
        let data_value = self.get_data_value(code);

        // стартер / пара / стартер + нестартеры
        match parse_data_value(data_value) {
            DecodedValue::Starter(combining) => {
                buffer.push(Codepoint { code, ccc: 0 });
                combining
            }
            DecodedValue::Pair(starter, nonstarter, combining) => {
                buffer.push(starter);
                buffer.push(nonstarter);

                combining
            }
            DecodedValue::Expansion(combining, index, _, len) => {
                let index = index as usize;
                let len = len as usize;

                buffer.push(Codepoint {
                    ccc: 0,
                    code: self.expansions[index],
                });

                self.expansions[index + 1 .. index + len]
                    .iter()
                    .for_each(|c| buffer.push(Codepoint::from_compressed(*c)));

                combining
            }
            _ => unreachable!(),
        }
    }
}
