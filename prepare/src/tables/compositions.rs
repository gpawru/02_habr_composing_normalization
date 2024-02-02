use std::collections::HashMap;

use unicode_normalization_source::COMPOSITION_PAIRS;

lazy_static! {
    static ref COMPOSITION_TABLE: (Vec<u64>, HashMap<u32, CompositionInfo>) = compositions();
    pub static ref COMPOSITION_TABLE_DATA: &'static Vec<u64> = &self::COMPOSITION_TABLE.0;
    pub static ref COMPOSITION_TABLE_INDEX: &'static HashMap<u32, CompositionInfo> =
        &COMPOSITION_TABLE.1;
}

/// "запеченные" композиции - массив значений и индексы для кодпоинтов
///
/// формат записи в таблице:
/// xxxx xxxx  xxxx xxxx    xxyy yyyy  yyyy yyyy    yyyy ____ ____ ____    iiii iiii  iiii iiii
/// где:
///     xx.. - второй кодпоинт
///     yy.. - результат комбинирования
///     ii.. - сжатая информация о композициях результата (см. CompositionInfo)
fn compositions() -> (Vec<u64>, HashMap<u32, CompositionInfo>)
{
    let mut data = Vec::new();
    let mut indexes = HashMap::new();

    let mut starters: Vec<&u32> = COMPOSITION_PAIRS.keys().collect();
    starters.sort();

    // таблица записей для каждого комбинируемого стартера - кодпоинт, с которым он комбинируется, результат
    for starter in starters {
        let pairs = &COMPOSITION_PAIRS[starter];

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
            let value = (*second as u64) | ((combined.code as u64) << 18);

            data.push(value);
        }
    }

    // для каждой записанной комбинируемой пары записываем дополнительную информацию - ссылку на варианты комбинирования
    // получаемого кодпоинта и количество вариантов
    for value in data.iter_mut() {
        let codepoint = (*value >> 18) as u32;

        if let Some(info) = indexes.get(&codepoint) {
            *value |= (info.bake() as u64) << 48;
        }
    }

    (data, indexes)
}

/// информация о хранимых композициях для стартера
#[derive(Default)]
pub struct CompositionInfo
{
    /// индекс первого элемента в таблице композиций
    pub index: u16,
    /// количество композиций для стартера
    pub count: u8,
}

impl CompositionInfo
{
    /// информация о хранимых композициях в сжатом виде:
    ///   [zzzzz] [zzz zzzz zzzz]
    ///    5 бит      11 бит
    ///       \          \---------- индекс в таблице пар
    ///        \-------------------- количество пар
    pub fn bake(&self) -> u16
    {
        assert!(self.index <= 0x7FF);
        assert!(self.count <= 0x1F);

        self.index | ((self.count as u16) << 11)
    }
}
