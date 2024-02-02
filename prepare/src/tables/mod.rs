mod compositions;
pub use compositions::*;

use unicode_normalization_source::UNICODE;

use crate::encode::encode_codepoint;

/// до этого кодпоинта (включительно) все кодпоинты записаны в таблицу данных последовательно
pub const CONTINUOUS_BLOCK_END: u32 = 0xFFF;
/// последний кодпоинт таблицы с декомпозицией
pub const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;

/// количество бит, с помощью которых может быть закодирован индекс блока
pub const BLOCK_BITS: u32 = 7;
/// максимально возможное количество блоков
pub const MAX_BLOCKS: u32 = LAST_DECOMPOSITION_CODE >> BLOCK_BITS;

/// подготовленные данные для записи
pub struct ComposingNormalizationTables
{
    pub index: Vec<u32>,
    pub data: Vec<u64>,
    pub expansions: Vec<u32>,
}

#[macro_export]
/// получить кодпоинт по содержащему его блоку и смещению
macro_rules! code_for {
    ($block: expr, $offset: expr) => {
        ($block << BLOCK_BITS) + $offset
    };
}

#[macro_export]
/// индекс блока для кодпоинта
macro_rules! block_for {
    ($code: expr) => {
        $code >> BLOCK_BITS
    };
}

/// подготавливаем таблицы NFC, NFKC
pub fn prepare(canonical: bool) -> ComposingNormalizationTables
{
    let mut index = [u32::MAX; MAX_BLOCKS as usize + 1];
    let mut data: Vec<u64> = vec![];
    let mut expansions: Vec<u32> = vec![];

    let mut last_nonempty_block = 0;

    // заполняем блоки
    for block in 0 ..= MAX_BLOCKS {
        let mut block_data = [u64::MAX; 1 << BLOCK_BITS as usize];
        let mut has_contents = code_for!(block, 0) <= CONTINUOUS_BLOCK_END;

        for offset in 0 .. 1 << BLOCK_BITS {
            let code = code_for!(block, offset);

            // если кодпоинт не найден - значит это стартер без декомпозиции
            // стоит заметить, что если кодпоинт участвует в композиции, то он обязательно содержится в таблице
            let codepoint = match UNICODE.get(&code) {
                Some(codepoint) => codepoint,
                None => {
                    block_data[offset as usize] = 0;
                    continue;
                }
            };

            let encoded = encode_codepoint(codepoint, canonical, expansions.len());

            if encoded.value > 0 {
                has_contents = true;
            }

            if let Some(data) = encoded.expansion_data {
                expansions.extend(data);
            }

            block_data[offset as usize] = encoded.value;
        }

        // если в блоке есть данные - его нужно записать, в противном случае -
        // индекс должен ссылаться на блок, состоящий из стартеров без декомпозиции и не участвующих в композиции

        if has_contents {
            index[block as usize] = block_for!(data.len()) as u32;
            data.extend(block_data);
            last_nonempty_block = block;
        }
    }

    // корректируем индекс - индекс для пустых блоков будет указывать на заглушку, находящуюся в конце таблицы
    let index = index[0 ..= last_nonempty_block as usize]
        .iter()
        .map(|v| match *v == u32::MAX {
            true => (data.len() >> BLOCK_BITS) as u32,
            false => *v,
        })
        .collect();

    data.resize(data.len() + (1 << BLOCK_BITS), 0);

    ComposingNormalizationTables {
        index,
        data,
        expansions,
    }
}
