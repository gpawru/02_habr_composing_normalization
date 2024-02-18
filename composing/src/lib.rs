use codepoint::decode::{is_nonstarters_value, parse_data_value, DecodedValue};
use codepoint::Codepoint;
use composition::*;
pub use data::CompositionData;
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
    index: Aligned<'a, u8>,
    /// основные данные
    data: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'a, u32>,
    /// композиции
    compositions: Aligned<'a, u64>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    /// NFC или NFKC
    is_canonical: bool,
}

// методы нормализации вынесены в макрос в целях оптимизации
macro_rules! normalizer_methods {
    ($normalize_method: ident, $fast_forward_method: ident, $fast_forward_first_code_boundary: expr) => {
        #[inline(always)]
        fn $normalize_method(&self, input: &str) -> String
        {
            let mut result = String::with_capacity(input.len());
            let mut buffer = Vec::with_capacity(18);
            let iter = &mut CharsIter::new(input);
            let mut combining = Combining::None;

            loop {
                iter.set_breakpoint();

                match self.$fast_forward_method(iter, &mut combining, &mut result, &mut buffer) {
                    Some((data_value, code)) => self.decode_codepoint(
                        data_value,
                        code,
                        &mut combining,
                        &mut result,
                        &mut buffer,
                    ),
                    None => return result,
                };
            }
        }

        /// цикл быстрой проверки, является-ли часть строки уже нормализованной
        #[inline(always)]
        fn $fast_forward_method(
            &self,
            iter: &mut CharsIter,
            combining: &mut Combining,
            result: &mut String,
            buffer: &mut Vec<Codepoint>,
        ) -> Option<(u64, u32)>
        {
            macro_rules! combine {
                () => {
                    combine_and_write(buffer, result, *combining, &self.compositions);
                };
            }

            Some(loop {
                // всё прочитали - комбинируем предшествующий текущему отрезку буфер, дописываем остаток
                if iter.is_empty() {
                    combine!();
                    write_str!(result, iter.ending_slice());

                    return None;
                }

                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                // текст, состоящий только из ASCII-символов уже NF(K)C нормализован
                // учитывая то, что для NFC и NFKC символы до U+0300 и U+00A0 соответственно также нормализованы,
                // используем не 0x80 в качестве границы, а значение первого байта UTF-8 вышеуказанных символов.
                if first < $fast_forward_first_code_boundary {
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
                let width = match first {
                    0x00 ..= 0xBF => unreachable!(),
                    0xC0 ..= 0xDF => 2,
                    0xE0 ..= 0xEF => 3,
                    0xF0 ..= 0xF7 => 4,
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
                        *combining = self.buffer_previous(buffer, previous);
                    }
                }

                break (data_value, code);
            })
        }
    };
}

impl<'a> ComposingNormalizer<'a>
{
    normalizer_methods!(normalize_nfc, fast_forward_nfc, 0xCC);
    normalizer_methods!(normalize_nfkc, fast_forward_nfkc, 0xC2);

    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String
    {
        match self.is_canonical() {
            true => self.normalize_nfc(input),
            false => self.normalize_nfkc(input),
        }
    }

    /// NFC или NFKC нормализация?
    #[inline(never)]
    fn is_canonical(&self) -> bool
    {
        self.is_canonical
    }

    /// данные о декомпозиции / композиции кодпоинта
    #[inline(never)]
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
    #[inline(always)]
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

    /// кодпоинт участвует в декомпозиции - комбинируем текущий буфер (кроме случая с нестартером),
    /// пишем в буфер декомпозицию кодпоинта или комбинируем сразу (хангыль, комбинирование с предыдущим)
    #[inline(never)]
    fn decode_codepoint(
        &self,
        data_value: u64,
        code: u32,
        combining: &mut Combining,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        macro_rules! combine {
            () => {
                combine_and_write(buffer, result, *combining, &self.compositions);
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

        let decoded = parse_data_value(data_value);

        match decoded {
            DecodedValue::Pair(starter, nonstarter, new_combining) => {
                combine!();
                to_buffer!(starter);
                to_buffer!(nonstarter);
                *combining = new_combining;
            }
            DecodedValue::Nonstarter(ccc) => {
                to_buffer!(code, ccc);
            }
            DecodedValue::Singleton(code, new_combining) => {
                combine!();
                to_buffer!(code, 0);
                *combining = new_combining;
            }
            DecodedValue::Starter(_) => {
                combine!();
                // в этот блок попадают только стартеры - чамо хангыль
                combine_and_write_hangul_vt(result, code);
                *combining = Combining::None;
            }

            DecodedValue::Expansion(new_combining, index, ns_len, len) => {
                let index = index as usize;
                let st_len = len - ns_len;
                let buffer_from = st_len.saturating_sub(1) as usize;

                let slice = &self.expansions[index .. index + len as usize];

                if st_len != 0 {
                    combine!();
                    *combining = new_combining;

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
                *combining = combine_backwards(
                    buffer,
                    result,
                    code,
                    *combining,
                    backwards_combining,
                    &self.compositions,
                );
            }
        }
    }

    /// NFC-нормализатор
    pub fn nfc() -> Self
    {
        Self::from_baked(data::nfc(), true)
    }

    /// NFKC-нормализатор
    pub fn nfkc() -> Self
    {
        Self::from_baked(data::nfkc(), false)
    }

    /// заранее подготовленные данные
    pub fn from_baked(source: data::CompositionData<'a>, is_canonical: bool) -> Self
    {
        Self {
            index: Aligned::from(source.index),
            data: Aligned::from(source.data),
            expansions: Aligned::from(source.expansions),
            compositions: Aligned::from(source.compositions),
            continuous_block_end: source.continuous_block_end,
            is_canonical,
        }
    }
}
