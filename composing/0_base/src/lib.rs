use codepoint::Codepoint;
use composition::*;
use decomposition::*;
use expansion::*;
use hangul::*;

mod codepoint;
mod composition;
mod data;
mod decomposition;
mod expansion;
mod hangul;
mod macros;

/// нормализатор NF(K)C
#[repr(align(32))]
pub struct ComposingNormalizer<'a>
{
    /// индекс блока. u8 достаточно, т.к. в NFC последний блок - 0x40, в NFKC - 0x6F (+1 для пустого блока)
    pub index: &'a [u8],
    /// основные данные
    pub data: &'a [u64],
    /// данные кодпоинтов, которые не вписываются в основную часть
    pub expansions: &'a [u32],
    /// композиции
    pub compositions: &'a [u64],
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    pub continuous_block_end: u32,
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
        let mut combining = Combining::None;

        // варианты содержимого буфера:
        //  - пустой
        //  - стартер
        //  - стартер + нестартер(ы)
        //  - нестартер(ы)

        for char in input.chars() {
            let code = u32::from(char);

            let decomposition = self.decompose(code);

            // помним, что стартеры могут быть скомбинированы с предыдущими кодпоинтами в большинстве случаев,
            // и этот кейс вынесен в отдельный блок (как редковстречаемый), чтобы уменьшить количество проверок в цикле

            match decomposition {
                // стартер
                DecompositionValue::None(new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, self.compositions);

                    buffer.push(Codepoint { code, ccc: 0 });

                    combining = new_combining;
                }
                // нестартер
                DecompositionValue::NonStarter(ccc) => {
                    buffer.push(Codepoint { code, ccc });
                }
                // пара стартер-нестартер
                DecompositionValue::Pair(c0, c1, new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, self.compositions);

                    buffer.push(c0);
                    buffer.push(c1);

                    combining = new_combining;
                }
                // синглтон
                DecompositionValue::Singleton(c0, new_combining) => {
                    combine_and_write(&mut buffer, &mut result, combining, self.compositions);

                    buffer.push(c0);

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
                        self.compositions,
                        self.expansions,
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
                                self.compositions,
                            );
                            write!(result, code);
                        }
                    };

                    combining = Combining::None;
                }
            }
        }

        combine_and_write(&mut buffer, &mut result, combining, self.compositions);

        result
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
