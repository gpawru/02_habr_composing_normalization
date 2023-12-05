use std::collections::HashMap;

use crate::encode::composition::{CompositionInfo, COMPOSITION_PAIRS};

/// "запеченные" композиции - массив значений и индексы для кодпоинтов
pub fn compositions() -> (Vec<u64>, HashMap<u32, CompositionInfo>)
{
    let mut data = Vec::new();
    let mut indexes = HashMap::new();

    let mut starters: Vec<&u32> = COMPOSITION_PAIRS.keys().collect();
    starters.sort();

    // формат записи:
    // xxxx xxxx  xxxx xxxx    xxyy yyyy  yyyy yyyy    yyyy ____ ____ ____    zzzz zzzz  zzzz zzzz
    // где xx.. - второй кодпоинт, yy.. - результат, zz.. - сжатая информация о композициях результата

    for starter in starters {
        let pairs = COMPOSITION_PAIRS.get(starter).unwrap();

        let mut seconds: Vec<&u32> = pairs.keys().collect();
        seconds.sort();

        indexes.insert(
            *starter,
            CompositionInfo {
                index: data.len() as u16,
                count: seconds.len() as u8,
            },
        );

        for second in seconds {
            // в значении нужно хранить:
            // 1. второй кодпоинт
            // 2. результирующий кодпоинт
            // 3. если полученный кодпоинт может быть скомбинирован - оффсет и количество вариантов, добавим на следующем шаге
            let combined = pairs.get(second).unwrap();
            let value = (*second as u64) | ((*combined as u64) << 18);

            data.push(value);
        }
    }

    for value in data.iter_mut() {
        let codepoint = (*value >> 18) as u32;

        if let Some(info) = indexes.get(&codepoint) {
            *value |= (info.bake() as u64) << 48;
        }
    }

    (data, indexes)
}
