use codepoint::Codepoint;
use composition::*;
use decomposition::*;
use expansion::*;
use hangul::*;
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod codepoint;
mod composition;
mod data;
mod decomposition;
mod expansion;
mod hangul;
mod macros;
mod slice;
mod utf8;

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
    /// до этого кодпоинта все кодпоинты - стартеры (0xC0/0xA0)
    pub decompositions_start: u32,
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
            decompositions_start: source.decompositions_start,
        }
    }
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
        let mut result = String::with_capacity(input.len());
        let mut buffer: Vec<Codepoint> = Vec::with_capacity(18);
        let mut combining = Combining::None;

        // варианты содержимого буфера:
        //  - пустой
        //  - стартер
        //  - стартер + нестартер(ы)
        //  - нестартер(ы)

        let decompositions_start = self.decompositions_start;

        let iter = &mut CharsIter::new(input);

        loop {
            iter.set_breakpoint();

            let (decomposition, code) = loop {
                if iter.is_empty() {
                    combine_and_write(&mut buffer, &mut result, combining, &self.compositions);
                    write_str!(result, iter.ending_slice());

                    return result;
                }

                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                // текст, состоящий только из ASCII-символов уже NF(KC) нормализован
                if first < 0x80 {
                    continue;
                }

                let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };

                // символы в пределах границы блока символов, не имеющих декомпозиции
                if code < decompositions_start {
                    continue;
                }

                // символ за границами "безопасной зоны". проверяем кейс декомпозиции:
                // если он является обычным стартером без декомпозиции, то продолжаем цикл
                let decomposition = self.decompose(code);

                break (decomposition, code);

                // if !decomposition.is_none() {
                //     // не учитываем однобайтовый символ, т.к. ранее мы их отсекли
                //     let width = match code {
                //         0x00 ..= 0x7F => unreachable!(),
                //         0x80 ..= 0x07FF => 2,
                //         0x0800 ..= 0xFFFF => 3,
                //         0x10000 ..= 0x10FFFF => 4,
                //         _ => unreachable!(),
                //     };

                //     // если мы получили какую-то последовательность символов без декомпозиции:
                //     //  - сливаем буфер предшествующих этому отрезку не-стартеров
                //     //  - сливаем отрезок от брейкпоинта до предыдущего символа

                //     if !iter.at_breakpoint(width) {
                //         flush!(result, buffer);
                //         write_str!(result, iter.block_slice(width));
                //     }

                //     break (decomposition, code);
                // }
            };

            // помним, что стартеры могут быть скомбинированы с предыдущими кодпоинтами в большинстве случаев,
            // и этот кейс вынесен в отдельный блок (как редковстречаемый), чтобы уменьшить количество проверок в цикле

            match decomposition {
                // стартер
                DecompositionValue::None(new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, &self.compositions);

                    buffer.push(Codepoint { code, ccc: 0 });

                    combining = new_combining;
                }
                // нестартер
                DecompositionValue::NonStarter(ccc) => {
                    buffer.push(Codepoint { code, ccc });
                }
                // пара стартер-нестартер
                DecompositionValue::Pair(c0, c1, new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, &self.compositions);

                    buffer.push(c0);
                    buffer.push(c1);

                    combining = new_combining;
                }
                // синглтон
                DecompositionValue::Singleton(code, new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, &self.compositions);

                    buffer.push(Codepoint { code, ccc: 0 });

                    combining = new_combining;
                }
                // вынесенные во внешний блок декомпозиции
                DecompositionValue::Expansion(expansion) => {
                    combining = combine_expansion(
                        &mut buffer,
                        &mut result,
                        code,
                        combining,
                        expansion,
                        &self.compositions,
                        &self.expansions,
                    );
                }
                // гласная или завершающая согласная чамо хангыль
                DecompositionValue::Hangul(vt) => {
                    match buffer.len() {
                        1 => {
                            combine_and_write_hangul_vt(&mut buffer, &mut result, vt);
                        }
                        _ => {
                            combine_and_write(
                                &mut buffer,
                                &mut result,
                                combining,
                                &self.compositions,
                            );
                            write!(result, code);
                        }
                    };

                    combining = Combining::None;
                }
            }
        }
    }

    /// получить декомпозицию символа
    #[inline(always)]
    fn decompose(&self, code: u32) -> DecompositionValue
    {
        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции, не комбинируются ни с чем
        if code > LAST_DECOMPOSITION_CODE {
            return DecompositionValue::None(Combining::None);
        };

        // обычный кодпоинт - получаем данные из таблицы, хангыль - вычисляем алгоритмически
        match is_hangul_vt(code) {
            None => parse_data_value(self.get_decomposition_value(code)),
            Some(value) => DecompositionValue::Hangul(value),
        }
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

                let index = block_offset | code_offset;

                self.data[index]
            }
        }
    }
}
