use unicode_normalization_source::UNICODE;

use crate::encode::encode_codepoint;
use crate::output::stats::CodepointGroups;

/// до этого кодпоинта (включительно) все кодпоинты записаны в таблицу данных последовательно
pub const STARTING_CODEPOINTS_BLOCK: u32 = 0xFFF;
/// последний кодпоинт таблицы с декомпозицией
pub const LAST_DECOMPOSITION_CODE: u32 = 0x2FA1D;
/// количество бит, с помощью которых может быть закодирован индекс блока
pub const BLOCK_BITS: u32 = 7;
/// максимально возможное количество блоков
pub const MAX_BLOCKS: u32 = LAST_DECOMPOSITION_CODE >> BLOCK_BITS;

#[macro_export]
/// получить кодпоинт по содержащему его блоку и смещению
macro_rules! code_for {
    ($block: expr, $offset: expr) => {
        ($block << BLOCK_BITS) + $offset
    };
}

#[macro_export]
macro_rules! block_for {
    ($code: expr) => {
        $code >> BLOCK_BITS
    };
}

/// подготавливаем таблицы NFD, NFKD
pub fn prepare<'a>(canonical: bool) -> (Vec<u32>, Vec<u64>, Vec<u32>, CodepointGroups<'a>)
{
    let unicode = &UNICODE;

    let mut index = [0u32; MAX_BLOCKS as usize + 1];
    let mut data: Vec<u64> = vec![];
    let mut expansions = vec![];

    let mut stats = CodepointGroups::new();

    let mut last_block = 0;

    // заполняем блоки

    for block in 0 ..= MAX_BLOCKS {
        let mut block_data = [0u64; 1 << BLOCK_BITS as usize];
        let mut has_contents = code_for!(block, 0) <= STARTING_CODEPOINTS_BLOCK;

        for offset in 0 .. 1 << BLOCK_BITS {
            let code = code_for!(block, offset);

            let codepoint = unicode.get(&code);

            // если кодпоинт не найден - значит это стартер без декомпозиции
            if codepoint.is_none() {
                block_data[offset as usize] = 0;
                continue;
            }

            let codepoint = codepoint.unwrap();

            let (value, expansion) =
                encode_codepoint(codepoint, canonical, expansions.len(), &mut stats);

            if value > 0 {
                has_contents = true;
            }

            expansions.extend(expansion);

            block_data[offset as usize] = value;
        }

        // если в блоке есть данные - его нужно записать
        // в противном случае - индекс должен ссылаться на блок, состоящий из стартеров без декомпозиции.
        // т.к. в блоке 128 значений, то можно ссылаться на 0 блок, где находятся ASCII (у них нет декомпозиции, все CCC = 0)

        if has_contents {
            index[block as usize] = block_for!(data.len()) as u32;
            data.extend(block_data);
            last_block = block;
        }
    }

    let index = index[0 ..= last_block as usize].to_vec();

    (index, data, expansions, stats)
}
