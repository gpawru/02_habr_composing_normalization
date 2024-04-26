pub use codepoint::Codepoint;
use composition::combine_and_write;
use composition::combine_and_write_hangul_vt;
use composition::combine_backwards;
use composition::Combining;
pub use data::{CompositionData, DecompositionData};
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod codepoint;
mod composition;
mod data;
mod slice;

/// последний кодпоинт с декомпозицией (U+2FA1D), его блок - 0x5F4
pub const LAST_DECOMPOSING_CODEPOINT_BLOCK: u16 = (0x2FA1D >> (18 - 11)) as u16;

/// стартер без декомпозиции
pub const MARKER_STARTER: u8 = 0b_000;
/// маркер композиции с предыдущим кодпоинтом (в том числе чамо хангыль)
pub const MARKER_COMBINES_BACKWARDS: u8 = 0b_001;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 0b_010;
/// синглтон
pub const MARKER_SINGLETON: u8 = 0b_011;
/// декомпозиция, вынесенная во внешний блок. сначала стартеры (если есть), потом нестартеры (если есть)
pub const MARKER_EXPANSION: u8 = 0b_100;
/// декомпозиция, вынесенная во внешний блок - прекомпозиция.
/// среди прочего - исключает варианты, где между стартерами может оказаться нестартер
pub const MARKER_EXPANSION_COMBINED_PATCH: u8 = 0b_101;
/// в случае с NF(K)D - декомпозиция присутствует, однако в контексте NF(K)C кодпоинт декомпозиции не имеет
pub const MARKER_EXPANSION_COMBINED_EMPTY: u8 = 0b_110;
/// слог хангыль
pub const MARKER_HANGUL_SYLLABLE: u8 = 0b_111;

/// нормализатор NF(K)C
#[repr(C, align(16))]
pub struct ComposingNormalizer<'a>
{
    /// основные данные
    data: Aligned<'a, u32>,
    /// индекс блока. u8 достаточно, т.к. в NFC последний блок - 0x40, в NFKC - 0x6F (+1 для пустого блока)
    index: Aligned<'a, u16>,
    /// композиции
    compositions: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'a, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
    /// патч декомпозиций
    expansions_patch: Aligned<'a, u32>,
    /// NFC или NFKC
    is_canonical: bool,
}

// методы нормализации вынесены в макрос в целях оптимизации
macro_rules! normalizer_methods {
    ($normalize_method:ident, $forward:ident, $fast_forward:ident, $first_code_boundary:expr) => {
        #[inline(always)]
        fn $normalize_method(&self, input: &str) -> String
        {
            let mut result = String::with_capacity(input.len());
            let mut buffer = Vec::with_capacity(18);
            let iter = &mut CharsIter::new(input);
            let mut combining = Combining::None;

            loop {
                let entry = match !buffer.is_empty() {
                    true => match self.$forward(iter, &mut combining, &mut result, &mut buffer) {
                        Some(entry) => Some(entry),
                        None => continue,
                    },
                    false => self.$fast_forward(iter, &mut combining, &mut result, &mut buffer),
                };

                match entry {
                    Some((dec_value, code)) => {
                        self.handle_dec_value(
                            dec_value,
                            code,
                            &mut combining,
                            &mut result,
                            &mut buffer,
                        );
                        iter.set_breakpoint();
                    }
                    None => return result,
                }
            }
        }

        /// если буфер не пуст, мы не можем перейти к быстрой проверке.
        /// прочитаем следующий кодпоинт, и если он стартер - скомбинируем буфер
        #[inline(always)]
        fn $forward(
            &self,
            iter: &mut CharsIter,
            combining: &mut Combining,
            result: &mut String,
            buffer: &mut Vec<Codepoint>,
        ) -> Option<(u32, u32)>
        {
            iter.set_breakpoint();

            if !iter.is_empty() {
                let first = unsafe { iter.next_unchecked() };

                if first >= $first_code_boundary {
                    let code = unsafe { iter.next_nonascii_bytes_unchecked(first) };
                    let dec_value = self.get_decomposition_value(code);

                    if dec_value & 1 != 0 {
                        return Some((dec_value, code));
                    }
                }
            }

            combine_and_write(result, buffer, *combining, &self.compositions);
            None
        }

        /// цикл быстрой проверки, является-ли часть строки уже нормализованной
        #[inline(always)]
        fn $fast_forward(
            &self,
            iter: &mut CharsIter,
            combining: &mut Combining,
            result: &mut String,
            buffer: &mut Vec<Codepoint>,
        ) -> Option<(u32, u32)>
        {
            Some(loop {
                // всё прочитали - комбинируем предшествующий текущему отрезку буфер, дописываем остаток
                if iter.is_empty() {
                    write_str(result, iter.ending_slice());
                    return None;
                }

                let first = unsafe { iter.next_unchecked() };

                // текст, состоящий только из ASCII-символов уже NF(K)C нормализован
                // учитывая то, что для NFC и NFKC символы до U+0300 и U+00A0 соответственно также нормализованы,
                // используем не 0x80 в качестве границы, а значение первого байта UTF-8 вышеуказанных символов.
                if first < $first_code_boundary {
                    continue;
                }

                let code = unsafe { iter.next_nonascii_bytes_unchecked(first) };
                let dec_value = self.get_decomposition_value(code);

                // является ли кодпоинт нормализованым? если - "да" или "возможно" (он считается
                // нормализованным, если за ним идёт стартер) - продолжаем быстрый цикл
                if dec_value & 1 == 0 {
                    continue;
                }

                // выходим из быстрого цикла, т.к. мы столкнулись с ситуацией, когда требуется
                // декомпозиция / комбинирование

                // не учитываем однобайтовый вариант, учитываем, что последовательность валидна
                let width: u8 = [2, 2, 3, 4][((first >> 4) & 3) as usize];

                // если у нас есть последовательность кодпоинтов в быстром цикле - комбинируем предшествующие
                // ему кодпоинты буфера, дописываем полученное в результат
                if !iter.at_breakpoint(width as isize) {
                    write_str(result, iter.block_slice(width as isize));

                    // на данный момент мы уже пропустили какой-то кодпоинт (qc = 0) в быстром цикле,
                    // он (предыдущий кодпоинт) может быть только:
                    // - стартером
                    // - слогом хангыль
                    // - парой
                    // - MARKER_EXPANSION_EMPTY | MARKER_EXPANSION_COMBINED_PATCH | MARKER_EXPANSION,
                    //   причём в случае расширений - всегда сначала располагаются стартеры (если есть),
                    //   а затем не стартеры (если есть)

                    let marker = dec_value as u8 >> 1;

                    // поместим в буфер декомпозицию предыдущего кодпоинта в случае, если текущий кодпоинт -
                    // нестартер или декомпозиция (может состоять из нестартеров), и получим ссылку на комбинирование
                    // для крайнего стартера декомпозиции предыдущего кодпоинта

                    if marker == MARKER_NONSTARTER || marker == MARKER_EXPANSION {
                        let previous = u32::from(result.pop().unwrap());
                        *combining = self.buffer_previous(result, buffer, previous);
                    }
                }

                break (dec_value, code);
            })
        }
    };
}

impl<'a> ComposingNormalizer<'a>
{
    normalizer_methods!(normalize_nfc, forward_nfc, fast_forward_nfc, 0xCC);
    normalizer_methods!(normalize_nfkc, forward_nfkc, fast_forward_nfkc, 0xC2);

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

    /// данные о декомпозиции символа
    #[inline(always)]
    fn get_decomposition_value(&self, code: u32) -> u32
    {
        let data_block_base = match code <= self.continuous_block_end {
            true => 0x600 | (((code >> 3) as u16) & !0xF),
            false => {
                let group_index = (code >> 7) as u16;

                // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
                if group_index > LAST_DECOMPOSING_CODEPOINT_BLOCK {
                    return 0;
                };

                self.index[group_index as usize]
            }
        };

        let code_offsets = (code as u16) & 0x7F;
        let data_block_index = data_block_base | (code_offsets >> 3) as u16;
        let index = self.index[data_block_index as usize] | code_offsets & 0x7;

        self.data[index as usize]
    }

    /// записать в буфер декомпозицию (точнее, прекомпозицию) последнего полученного кодпоинта
    /// для комбинирования с нестартером. буфер пуст.
    #[inline(never)]
    fn buffer_previous(
        &self,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
        code: u32,
    ) -> Combining
    {
        let dec_value = self.get_decomposition_value(code);
        let marker = (dec_value as u8) >> 1;

        // MARKER_SINGLETON | MARKER_COMBINES_BACKWARDS никогда здесь не окажутся,
        // т.к. их бит проверки = 1, и они не получаются в результате комбинирования

        match marker {
            MARKER_STARTER | MARKER_HANGUL_SYLLABLE => {
                buffer.push(Codepoint::from_code(code));

                Combining::from((dec_value >> 16) as u16)
            }
            MARKER_EXPANSION_COMBINED_PATCH => {
                // декомпозиция в NF(K)D и NF(K)C отличается

                self.buffer_previous_expansion_patch(dec_value, result, buffer)
            }
            MARKER_EXPANSION_COMBINED_EMPTY => {
                // в NF(K)D этот кодпоинт имеет декомпозицию, в NF(K)C это обычный стартер
                buffer.push(Codepoint::from_code(code));

                Combining::from(self.expansions[(dec_value >> 18) as usize] as u16)
            }
            MARKER_EXPANSION => {
                // сначала стартеры, потом нестартеры (если есть)
                // т.к. кодпоинт взят из результата, сюда никоим образом не может попасть декомпозиция без стартера
                // если в начале несколько стартеров - они не комбинируются обратно

                self.buffer_previous_expansion(dec_value, result, buffer)
            }
            _ => {
                let starter = ((dec_value as u16) >> 1) as u32;
                let nonstarter = dec_value >> 16;
                let nonstarter_ccc = (self.get_decomposition_value(nonstarter) >> 8) as u8;

                buffer.push(Codepoint::from_code(starter));
                buffer.push(Codepoint::from_code_and_ccc(nonstarter, nonstarter_ccc));

                Combining::from((self.get_decomposition_value(starter) >> 16) as u16)
            }
        }
    }

    /// кодпоинт участвует в декомпозиции - комбинируем текущий буфер (кроме случая с нестартером),
    /// пишем в буфер декомпозицию кодпоинта или комбинируем сразу (хангыль, комбинирование с предыдущим)
    #[inline(always)]
    fn handle_dec_value(
        &self,
        dec_value: u32,
        code: u32,
        combining: &mut Combining,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        let marker = (dec_value as u8) >> 1;

        // MARKER_STARTER | MARKER_HANGUL_SYLLABLE | MARKER_EXPANSION_COMBINED_EMPTY - бит быстрой проверки = 0,
        // что означает, что в этот блок эти варианты просто не попадут

        match marker {
            MARKER_NONSTARTER => {
                buffer.push(Codepoint::from_code_and_ccc(code, (dec_value >> 8) as u8))
            }
            MARKER_SINGLETON => {
                combine_and_write(result, buffer, *combining, &self.compositions);

                let code = dec_value >> 8;
                buffer.push(Codepoint::from_code(code));

                // в данном случае значение комбинирования нужно получить ещё раз заглянув в таблицу,
                // кодпоинт, который мы рассматриваем, может оказаться только стартером
                *combining = Combining::from((self.get_decomposition_value(code) >> 16) as u16);
            }
            MARKER_EXPANSION => {
                // сначала стартеры (если есть), потом нестартеры (если есть)
                // здесь же - частный случай декомпозиции нестартеров
                // не может являться синглтоном
                // если в начале несколько стартеров - они не комбинируются обратно

                self.handle_expansion(dec_value, combining, result, buffer);
            }
            MARKER_EXPANSION_COMBINED_PATCH => {
                // декомпозиция в NF(K)D и в NF(K)C отличается - декомпозиция собирается в синглтон или
                // комбинируются первые кодпоинты декомпозиции

                combine_and_write(result, buffer, *combining, &self.compositions);

                self.handle_expansion_patch(dec_value, combining, result, buffer);
            }
            MARKER_COMBINES_BACKWARDS => {
                // стартер, комбинируемый с предыдущим стартером или чамо хангыль (комбинируемый с предыдущим L/LV)

                combine_and_write(result, buffer, *combining, &self.compositions);

                if !combine_and_write_hangul_vt(result, code, combining) {
                    let backwards_combining = Combining::from((dec_value >> 16) as u16);

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
            _ => {
                // пара. для кодпоинта пары не хранится отдельно информация о комбинировании -
                // она получается при последовательном комбинировании стартера декомпозиции с нестартерами

                combine_and_write(result, buffer, *combining, &self.compositions);

                let starter = ((dec_value as u16) >> 1) as u32;
                let nonstarter = dec_value >> 16;
                let nonstarter_ccc = (self.get_decomposition_value(nonstarter) >> 8) as u8;

                buffer.push(Codepoint::from_code(starter));
                buffer.push(Codepoint::from_code_and_ccc(nonstarter, nonstarter_ccc));

                *combining = Combining::from((self.get_decomposition_value(starter) >> 16) as u16);
            }
        }
    }

    /// NFC-нормализатор
    pub fn new_nfc() -> Self
    {
        Self::from_baked(
            data::nfd(),
            data::compositions(),
            data::nfc_expansions(),
            true,
        )
    }

    /// NFKC-нормализатор
    pub fn new_nfkc() -> Self
    {
        Self::from_baked(
            data::nfkd(),
            data::compositions(),
            data::nfkc_expansions(),
            false,
        )
    }

    /// заранее подготовленные данные
    pub fn from_baked(
        decomposition_data: data::DecompositionData,
        compositions: data::CompositionData,
        expansions_patch: data::ExpansionsPatch,
        is_canonical: bool,
    ) -> Self
    {
        Self {
            index: Aligned::from(decomposition_data.index),
            data: Aligned::from(decomposition_data.data),
            expansions: Aligned::from(decomposition_data.expansions),
            compositions: Aligned::from(compositions.compositions),
            continuous_block_end: decomposition_data.continuous_block_end,
            expansions_patch: Aligned::from(expansions_patch.expansions),
            is_canonical,
        }
    }

    /// декомпозиция предыдущего кодпоинта, всегда начинается со стартера
    #[inline(never)]
    fn buffer_previous_expansion(
        &self,
        dec_value: u32,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    ) -> Combining
    {
        let last_starter = (dec_value >> 8) & 0x1F;
        let count = (dec_value >> 13) & 0x1F;
        let index = dec_value >> 18;

        let expansions = &self.expansions[index as usize .. (index + count) as usize];

        expansions[.. last_starter as usize]
            .iter()
            .for_each(|&entry| write_char(result, entry >> 8));

        expansions[last_starter as usize ..]
            .iter()
            .for_each(|&entry| buffer.push(Codepoint::from_baked(entry)));

        Combining::from((self.get_decomposition_value(buffer[0].code()) >> 16) as u16)
    }

    /// декомпозиция предыдущего кодпоинта, чья декомпозиция отличается от NFD (сделана прекомпозиция)
    #[inline(never)]
    fn buffer_previous_expansion_patch(
        &self,
        dec_value: u32,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    ) -> Combining
    {
        let index = dec_value >> 18;
        let info = self.expansions[index as usize];

        let last_starter = info & 0x7;
        let count = (info >> 3) & 0x7;
        let index = (info >> 6) & 0x3FF;

        let expansions = &self.expansions_patch[index as usize .. (index + count) as usize];

        expansions[.. last_starter as usize]
            .iter()
            .for_each(|&entry| write_char(result, entry >> 8));

        expansions[last_starter as usize ..]
            .iter()
            .for_each(|&entry| buffer.push(Codepoint::from_baked(entry)));

        Combining::from((info >> 16) as u16)
    }

    /// декомпозиция текущего кодпоинта, основной случай
    #[inline(never)]
    fn handle_expansion(
        &self,
        dec_value: u32,
        combining: &mut Combining,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        let last_starter = (dec_value >> 8) & 0x1F;
        let count = (dec_value >> 13) & 0x1F;
        let index = dec_value >> 18;

        let expansions = &self.expansions[index as usize .. (index + count) as usize];

        // если декомпозиция начинается со стартера, то предварительно комбинируем и пишем буфер
        if expansions[0] as u8 == 0 {
            combine_and_write(result, buffer, *combining, &self.compositions);

            expansions[.. last_starter as usize]
                .iter()
                .for_each(|&entry| write_char(result, entry >> 8));

            let last_starter = expansions[last_starter as usize] >> 8;

            *combining = Combining::from((self.get_decomposition_value(last_starter) >> 16) as u16);
        }

        expansions[last_starter as usize ..]
            .iter()
            .for_each(|&entry| buffer.push(Codepoint::from_baked(entry)));
    }

    /// декомпозиция текущего кодпоинта, чья декомпозиция отличается от NFD (сделана прекомпозиция)
    #[inline(never)]
    fn handle_expansion_patch(
        &self,
        dec_value: u32,
        combining: &mut Combining,
        result: &mut String,
        buffer: &mut Vec<Codepoint>,
    )
    {
        let index = dec_value >> 18;
        let info = self.expansions[index as usize];

        let last_starter = info & 0x7;
        let count = (info >> 3) & 0x7;
        let index = (info >> 6) & 0x3FF;

        let expansions = &self.expansions_patch[index as usize .. (index + count) as usize];

        expansions[.. last_starter as usize]
            .iter()
            .for_each(|&entry| write_char(result, entry >> 8));

        expansions[last_starter as usize ..]
            .iter()
            .for_each(|&entry| buffer.push(Codepoint::from_baked(entry)));

        *combining = Combining::from((info >> 16) as u16);
    }
}

/// дописать символ(по коду) в результат
#[inline(always)]
fn write_char(result: &mut String, code: u32)
{
    result.push(unsafe { char::from_u32_unchecked(code) });
}

/// дописать уже нормализованный кусок исходной строки в UTF-8 результат
#[inline(always)]
fn write_str(result: &mut String, string: &[u8])
{
    result.push_str(unsafe { core::str::from_utf8_unchecked(string) });
}
