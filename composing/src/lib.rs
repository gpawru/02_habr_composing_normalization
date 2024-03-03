pub use codepoint::Codepoint;
use composition::combine_and_write;
use composition::combine_and_write_hangul_vt;
use composition::combine_backwards;
use composition::Combining;
pub use data::CompositionData;
use slice::aligned::Aligned;
use slice::iter::CharsIter;

mod codepoint;
mod composition;
mod data;
mod slice;
mod utf8;

/// последний кодпоинт с декомпозицией (U+2FA1D), его блок - 0x5F4
pub const LAST_DECOMPOSING_CODEPOINT_BLOCK: u16 = 0x5F4;

/// стартер без декомпозиции
pub const MARKER_STARTER: u8 = 0b_000;
/// нестартер без декомпозиции
pub const MARKER_NONSTARTER: u8 = 0b_001;
/// стартер-синглтон
pub const MARKER_SINGLETON: u8 = 0b_010;
/// - стартер и нестартеры
/// - последовательность стартеров
/// - два стартера + нестартер
/// - исключения - стартеры, которые декомпозируются в нестартеры
pub const MARKER_EXPANSION_0: u8 = 0b_100;
pub const MARKER_EXPANSION_1: u8 = 0b_101;
pub const MARKER_EXPANSION_2: u8 = 0b_110;
/// исключения - стартеры, которые комбинируются с предыдущими кодпоинтами
pub const MARKER_COMBINES_BACKWARDS: u8 = 0b_011;

/// нормализатор NF(K)C
#[repr(C, align(16))]
pub struct ComposingNormalizer<'a>
{
    /// основные данные
    data: Aligned<'a, u32>,
    /// индекс блока. u8 достаточно, т.к. в NFC последний блок - 0x40, в NFKC - 0x6F (+1 для пустого блока)
    index: Aligned<'a, u8>,
    /// композиции
    compositions: Aligned<'a, u64>,
    /// данные кодпоинтов, которые не вписываются в основную часть
    expansions: Aligned<'a, u32>,
    /// с U+0000 и до этого кодпоинта включительно блоки в data идут последовательно
    continuous_block_end: u32,
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
                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                if first >= $first_code_boundary {
                    let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };
                    let dec_value = self.get_dec_value(code);

                    if dec_value as u8 != 0 {
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
                    combine_and_write(result, buffer, *combining, &self.compositions);
                    write_str(result, iter.ending_slice());

                    return None;
                }

                let first = unsafe { utf8::char_first_byte_unchecked(iter) };

                // текст, состоящий только из ASCII-символов уже NF(K)C нормализован
                // учитывая то, что для NFC и NFKC символы до U+0300 и U+00A0 соответственно также нормализованы,
                // используем не 0x80 в качестве границы, а значение первого байта UTF-8 вышеуказанных символов.
                if first < $first_code_boundary {
                    continue;
                }

                let code = unsafe { utf8::char_nonascii_bytes_unchecked(iter, first) };
                let dec_value = self.get_dec_value(code);

                // является ли кодпоинт нормализованым? если - "да" или "возможно" (он считается
                // нормализованным, если за ним идёт стартер) - продолжаем быстрый цикл
                if dec_value as u8 & 1 == 0 {
                    continue;
                }

                // выходим из быстрого цикла, т.к. мы столкнулись с ситуацией, когда требуется
                // декомпозиция / комбинирование

                let width = utf8::get_utf8_sequence_width(first) as isize;

                // если у нас есть последовательность кодпоинтов в быстром цикле - комбинируем предшествующие
                // ему кодпоинты буфера, дописываем полученное в результат
                if !iter.at_breakpoint(width) {
                    combine_and_write(result, buffer, *combining, &self.compositions);
                    write_str(result, iter.block_slice(width));

                    // в случае, если получили нестартер, нам потребуется предшествующий ему кодпоинт.
                    // он может быть только стартером / парой / частным случаем расширения -
                    // стартер + несколько нестартеров (смотрим, какие метки быстрых проверок используются
                    // в вариантах кодирования декомпозиции кодпоинтов при запекании)

                    let marker = dec_value as u8 >> 1;
                    if marker == MARKER_NONSTARTER || marker & 0b100 != 0 {
                        let previous = u32::from(result.pop().unwrap());
                        *combining = self.buffer_previous(buffer, previous);
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

    /// данные о декомпозиции / композиции кодпоинта
    #[inline(always)]
    fn get_dec_value(&self, code: u32) -> u32
    {
        if code <= self.continuous_block_end {
            return self.data[code as usize];
        }

        let block_index = (code >> 7) as u16;

        // все кодпоинты, следующие за U+2FA1D не имеют декомпозиции
        if block_index > LAST_DECOMPOSING_CODEPOINT_BLOCK {
            return 0;
        };

        let block_index = (code >> 7) as usize;
        let block = self.index[block_index] as usize;

        let block_offset = block << 7;
        let code_offset = ((code as u8) & 0x7F) as usize;

        let index = block_offset | code_offset;

        self.data[index]
    }

    /// записать в буфер декомпозицию (точнее, прекомпозицию) последнего полученного кодпоинта
    /// для комбинирования с нестартером
    #[inline(always)]
    fn buffer_previous(&self, buffer: &mut Vec<Codepoint>, code: u32) -> Combining
    {
        let dec_value = self.get_dec_value(code);
        let marker = (dec_value as u8) >> 1;

        // стартер / пара / стартер + нестартеры
        match marker {
            MARKER_STARTER => {
                buffer.push(Codepoint::from_code(code));

                Combining::from((dec_value >> 16) as u16)
            }
            MARKER_EXPANSION_0 | MARKER_EXPANSION_1 | MARKER_EXPANSION_2 => {
                let len = (dec_value >> 8) as u8;
                let index = (dec_value >> 16) as usize;

                let starter = Codepoint::from_baked(self.expansions[index]);

                buffer.push(starter);

                self.expansions[index + 1 .. index + len as usize]
                    .iter()
                    .for_each(|c| buffer.push(Codepoint::from_baked(*c)));

                Combining::from((self.get_dec_value(starter.code()) >> 16) as u16)
            }
            _ => {
                let starter = ((dec_value as u16) >> 1) as u32;
                let nonstarter = dec_value >> 16;
                let nonstarter_ccc = (self.get_dec_value(nonstarter) >> 8) as u8;

                buffer.push(Codepoint::from_code(starter));
                buffer.push(Codepoint::from_code_and_ccc(nonstarter, nonstarter_ccc));

                Combining::from((self.get_dec_value(starter) >> 16) as u16)
            }
        }
    }

    /// кодпоинт участвует в декомпозиции - комбинируем текущий буфер (кроме случая с нестартером),
    /// пишем в буфер декомпозицию кодпоинта или комбинируем сразу (хангыль, комбинирование с предыдущим)
    #[inline(never)]
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

        match marker {
            MARKER_NONSTARTER => {
                buffer.push(Codepoint::from_code_and_ccc(code, (dec_value >> 8) as u8))
            }
            MARKER_STARTER => {
                combine_and_write(result, buffer, *combining, &self.compositions);

                // в этот блок попадают только стартеры - чамо хангыль
                combine_and_write_hangul_vt(result, code);
                
                *combining = Combining::None;
            }
            MARKER_SINGLETON => {
                combine_and_write(result, buffer, *combining, &self.compositions);

                let code = dec_value >> 8;
                buffer.push(Codepoint::from_code(code));

                // в данном случае значение комбинирования нужно получить ещё раз заглянув в таблицу,
                // кодпоинт, который мы рассматриваем, может оказаться только стартером
                *combining = Combining::from((self.get_dec_value(code) >> 16) as u16);
            }
            MARKER_EXPANSION_0 | MARKER_EXPANSION_1 | MARKER_EXPANSION_2 => {
                let ns_len = marker & 0b11;
                let len = (dec_value >> 8) as u8;
                let index = (dec_value >> 16) as usize;
                let st_len = len - ns_len;

                let buffer_from = st_len.saturating_sub(1) as usize;
                let slice = &self.expansions[index .. index + len as usize];

                if st_len != 0 {
                    combine_and_write(result, buffer, *combining, &self.compositions);

                    *combining = Combining::from(
                        (self.get_dec_value(slice[(st_len - 1) as usize] >> 8) >> 16) as u16,
                    );

                    if st_len > 1 {
                        slice[.. buffer_from]
                            .iter()
                            .for_each(|c| write_char(result, *c >> 8));
                    }
                }

                slice[buffer_from ..].iter().for_each(|c| {
                    buffer.push(Codepoint::from_baked(*c));
                });
            }
            MARKER_COMBINES_BACKWARDS => {
                combine_and_write(result, buffer, *combining, &self.compositions);

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
            _ => {
                combine_and_write(result, buffer, *combining, &self.compositions);

                let starter = ((dec_value as u16) >> 1) as u32;
                let nonstarter = dec_value >> 16;
                let nonstarter_ccc = (self.get_dec_value(nonstarter) >> 8) as u8;

                buffer.push(Codepoint::from_code(starter));
                buffer.push(Codepoint::from_code_and_ccc(nonstarter, nonstarter_ccc));

                *combining = Combining::from((self.get_dec_value(starter) >> 16) as u16);
            }
        }
    }

    /// NFC-нормализатор
    pub fn new_nfc() -> Self
    {
        Self::from_baked(data::nfc(), true)
    }

    /// NFKC-нормализатор
    pub fn new_nfkc() -> Self
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
