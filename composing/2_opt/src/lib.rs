use decomposition::hangul::*;
use decomposition::*;
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod data;
mod decomposition;
mod macros;
mod slice;
mod utf8;

/// нормализатор NF(K)D
#[repr(align(32))]
pub struct DecomposingNormalizer<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: Aligned<'a, u8>,
    /// основные данные
    data: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'a, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    /// первый кодпоинт в таблице, имеющий декомпозицию / не-стартер (для NFD - U+00C0, для NFKD - U+00A0)
    dec_starts_at: u32,
}

impl<'a> From<data::DecompositionData<'a>> for DecomposingNormalizer<'a>
{
    fn from(source: data::DecompositionData<'a>) -> Self
    {
        Self {
            index: Aligned::from(source.index),
            data: Aligned::from(source.data),
            expansions: Aligned::from(source.expansions),
            continuous_block_end: source.continuous_block_end,
            dec_starts_at: source.dec_starts_at,
        }
    }
}

impl<'a> DecomposingNormalizer<'a>
{
    /// NFD-нормализатор
    pub fn nfd() -> Self
    {
        Self::from(data::nfd())
    }

    /// NFKD-нормализатор
    pub fn nfkd() -> Self
    {
        Self::from(data::nfkd())
    }

    /// нормализация строки
    /// исходная строка должна являться well-formed UTF-8 строкой
    #[inline(never)]
    pub fn normalize(&self, input: &str) -> String
    {
        let mut result = String::with_capacity(input.len());
        let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);

        let dec_starts_at = self.dec_starts_at;

        let iter = &mut CharsIter::new(input);

        loop {
            iter.set_breakpoint();

            // цикл для отрезка текста из символов, не имеющих декомпозиций

            let (decomposition, code) = loop {
                if iter.is_empty() {
                    flush!(result, buffer);
                    write_str!(result, iter.ending_slice());

                    return result;
                }

                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                // все ASCII-символы не имеют декомпозиции
                if first < 0x80 {
                    continue;
                }

                let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

                // символы в пределах границы блока символов, не имеющих декомпозиции
                if code < dec_starts_at {
                    continue;
                }

                // символ за границами "безопасной зоны". проверяем кейс декомпозиции:
                // если он является обычным стартером без декомпозиции, то продолжаем цикл
                let decomposition = self.decompose(code);

                if !decomposition.is_none() {
                    // не учитываем однобайтовый символ, т.к. ранее мы их отсекли
                    let width = match code {
                        0x00 ..= 0x7F => unreachable!(),
                        0x80 ..= 0x07FF => 2,
                        0x0800 ..= 0xFFFF => 3,
                        0x10000 ..= 0x10FFFF => 4,
                        _ => unreachable!(),
                    };

                    // если мы получили какую-то последовательность символов без декомпозиции:
                    //  - сливаем буфер предшествующих этому отрезку не-стартеров
                    //  - сливаем отрезок от брейкпоинта до предыдущего символа

                    if !iter.at_breakpoint(width) {
                        flush!(result, buffer);
                        write_str!(result, iter.block_slice(width));
                    }

                    break (decomposition, code);
                }
            };

            // у символа есть декомпозиция: стартеры пишем в результат, не-стартеры - в буфер,
            // если после не-стартера встречаем стартер - сортируем не-стартеры, записываем, сбрасываем буфер

            match decomposition {
                DecompositionValue::NonStarter(ccc) => buffer.push(Codepoint { code, ccc }),
                DecompositionValue::Pair(c1, c2) => {
                    flush!(result, buffer);
                    write!(result, c1);

                    match c2.ccc == 0 {
                        true => write!(result, c2.code),
                        false => buffer.push(c2),
                    }
                }
                DecompositionValue::Triple(c1, c2, c3) => {
                    flush!(result, buffer);
                    write!(result, c1);

                    if c3.ccc == 0 {
                        write!(result, c2.code, c3.code);
                    } else {
                        match c2.ccc == 0 {
                            true => write!(result, c2.code),
                            false => buffer.push(c2),
                        }
                        buffer.push(c3);
                    }
                }
                DecompositionValue::Singleton(c1) => {
                    flush!(result, buffer);
                    write!(result, c1);
                }
                DecompositionValue::HangulPair(c1, c2) => {
                    flush!(result, buffer);
                    write_str!(result, &[0xE1, 0x84, c1, 0xE1, 0x85, c2]);
                }
                DecompositionValue::HangulTriple(c1, c2, c3, c4) => {
                    flush!(result, buffer);
                    write_str!(result, &[0xE1, 0x84, c1, 0xE1, 0x85, c2, 0xE1, c3, c4]);
                }
                DecompositionValue::Expansion(index, count) => {
                    for entry in
                        &self.expansions[(index as usize) .. (index as usize + count as usize)]
                    {
                        let ccc = c32_ccc!(entry);
                        let code = c32_code!(entry);

                        match ccc == 0 {
                            true => {
                                flush!(result, buffer);
                                write!(result, code);
                            }
                            false => buffer.push(Codepoint { code, ccc }),
                        }
                    }
                }
                DecompositionValue::None => unreachable!(),
            }
        }
    }

    /// получить декомпозицию символа
    #[inline(always)]
    fn decompose(&self, code: u32) -> DecompositionValue
    {
        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
        if code > LAST_DECOMPOSING_CODEPOINT {
            return DecompositionValue::None;
        };

        let lvt = code.wrapping_sub(HANGUL_S_BASE);

        if lvt > HANGUL_S_COUNT {
            return parse_data_value(self.get_decomposition_value(code));
        };

        decompose_hangul(lvt)
    }

    /// данные о декомпозиции символа
    #[inline(always)]
    fn get_decomposition_value(&self, code: u32) -> u64
    {
        match code <= self.continuous_block_end {
            true => self.data[code as usize],
            false => {
                let block_index = (code >> 7) as usize;
                let block = self.index[block_index] as usize;

                let block_offset = block << 7;
                let code_offset = ((code as u8) & 0x7F) as usize;

                let index = block_offset + code_offset;

                self.data[index]
            }
        }
    }
}
