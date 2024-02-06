use composition::*;
use decomposition::hangul::*;
use decomposition::*;

mod composition;
mod data;
mod decomposition;
mod macros;

/// нормализатор NF(K)C
#[repr(align(32))]
pub struct ComposingNormalizer<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFD последний блок - 0x7E, в NFKD - 0xA6
    index: &'a [u8],
    /// основные данные
    data: &'a [u64],
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: &'a [u32],
    /// композиции
    compositions: &'a [u64],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
}

impl<'a> From<data::CompositionData<'a>> for ComposingNormalizer<'a>
{
    fn from(source: data::CompositionData<'a>) -> Self
    {
        Self {
            index: source.index,
            data: source.data,
            expansions: source.expansions,
            compositions: source.compositions,
            continuous_block_end: source.continuous_block_end,
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

        for char in input.chars() {
            let code = u32::from(char);

            let decomposition = self.decompose(code);

            match decomposition {
                DecompositionValue::None(combining) => {
                    // не имеет декомпозиции, может (но не обязательно) комбинироваться с далее идущими кодпоинтами
                    
                    sort_by_ccc(&mut buffer);

                    buffer.push(Codepoint {
                        code,
                        ccc: 0,
                        combining,
                    });

                    combine_and_flush(&mut result, &mut buffer, self.compositions);
                }
                DecompositionValue::NonStarter(ccc, combining) => {
                    // нужно просто дописать не-стартер в буфер

                    buffer.push(Codepoint {
                        code,
                        ccc,
                        combining,
                    });
                }
                DecompositionValue::Pair(c1, c2) => {
                    // пара из стартера и не-стартера

                    sort_by_ccc(&mut buffer);
                    buffer.push(c1);
                    combine_and_flush(&mut result, &mut buffer, self.compositions);

                    debug_assert_ne!(c2.ccc, 0);
                    debug_assert!(!buffer.is_empty());

                    buffer.push(c2);
                }
                DecompositionValue::Triple(c1, c2, c3) => {
                    // тройка - первый стартер, далее - не-стартеры

                    // нужен дополнительный запрос - посмотреть, может-ли быть скомбинирован стартер,
                    // т.к. в записи эта информация не хранится
                    let c1 = self.get_codepoint_for_starter(c1);

                    debug_assert_eq!(c1.ccc, 0);

                    sort_by_ccc(&mut buffer);
                    buffer.push(c1);
                    combine_and_flush(&mut result, &mut buffer, self.compositions);

                    debug_assert_ne!(c2.ccc, 0);
                    debug_assert_ne!(c3.ccc, 0);
                    debug_assert!(!buffer.is_empty());

                    buffer.push(c2);
                    buffer.push(c3);
                }
                DecompositionValue::Singleton(c1) => {
                    // синглтоны не комбинируются с предыдущими кодпоинтами, но могут комбинироваться со следующими

                    sort_by_ccc(&mut buffer);
                    buffer.push(c1);
                    combine_and_flush(&mut result, &mut buffer, self.compositions);
                }
                // DecompositionValue::HangulPair(c1, c2) => {
                //     sort_by_ccc(&mut buffer);
                //     combine_and_flush(&mut result, &mut buffer, self.compositions);

                //     for codepoint in buffer.iter() {
                //         write!(result, codepoint.code);
                //     }

                //     write!(result, c1, c2);
                // }
                // DecompositionValue::HangulTriple(c1, c2, c3) => {
                //     sort_by_ccc(&mut buffer);
                //     combine_and_flush(&mut result, &mut buffer, self.compositions);

                //     for codepoint in buffer.iter() {
                //         write!(result, codepoint.code);
                //     }

                //     write!(result, c1, c2, c3);
                // }
                DecompositionValue::Expansion(index, count) => {
                    for entry in
                        &self.expansions[(index as usize) .. (index as usize + count as usize)]
                    {
                        let ccc = c32_ccc!(entry);
                        let code = c32_code!(entry);

                        match ccc == 0 {
                            true => {
                                let c1 = self.get_codepoint_for_starter(code);

                                sort_by_ccc(&mut buffer);
                                buffer.push(c1);
                                combine_and_flush(&mut result, &mut buffer, self.compositions);
                            }
                            false => buffer.push(Codepoint {
                                code,
                                ccc,
                                combining: 0,
                            }),
                        }
                    }
                }
            }
        }

        sort_by_ccc(&mut buffer);
        combine_and_flush(&mut result, &mut buffer, self.compositions);

        for codepoint in buffer {
            write!(result, codepoint.code);
        }

        result
    }

    /// получить декомпозицию символа
    #[inline(always)]
    fn decompose(&self, code: u32) -> DecompositionValue
    {
        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции, не комбинируются ни с чем
        if code > LAST_DECOMPOSING_CODEPOINT {
            return DecompositionValue::None(0);
        };

        // let lvt = code.wrapping_sub(HANGUL_S_BASE);

        // if lvt > HANGUL_S_COUNT {
        // return parse_data_value(self.get_decomposition_value(code));
        // };

        // decompose_hangul(lvt)

        parse_data_value(self.get_decomposition_value(code))
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

    /// получаем данные для стартера о его возможной композиции
    /// важно: работает только для стартеров без декомпозиции
    #[inline]
    fn get_codepoint_for_starter(&self, code: u32) -> Codepoint
    {
        let decomposition_value = self.get_decomposition_value(code);

        debug_assert_eq!((decomposition_value as u8) & 0xF7, 0);

        Codepoint {
            ccc: 0,
            code,
            combining: o!(decomposition_value, u16, 1),
        }
    }
}

/// отсортировать кодпоинты по CCC
#[inline(always)]
pub fn sort_by_ccc(buffer: &mut Vec<Codepoint>)
{
    if buffer.len() > 1 {
        buffer.sort_by_key(|c| c.ccc);
    }
}
