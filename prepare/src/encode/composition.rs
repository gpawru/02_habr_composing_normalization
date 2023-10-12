use std::collections::HashMap;

use unicode_normalization_source::UNICODE;

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMPOSES_WITH_LEFT: Vec<u32> = composes_with_left();
    /// пары
    pub static ref COMPOSITIONS: HashMap<u64, u32> = compositions();
}

/// может ли быть скомбинирован с каким-либо предстоящим кодпоинтом?
fn composes_with_left() -> Vec<u32>
{
    let mut map = Vec::new();

    for entry in UNICODE.values() {
        // декомпозиция отсутствует, синглтон или не является канонической
        if (entry.decomposition.len() != 2) || entry.decomposition_tag.is_some() {
            continue;
        }

        map.push(entry.decomposition[1]);
    }

    map
}

/// каноническая композиция
fn compositions() -> HashMap<u64, u32>
{
    let mut map = HashMap::new();

    for entry in UNICODE.values() {
        if entry.decomposition.len() == 2 && entry.decomposition_tag.is_none() {
            let key = (entry.decomposition[0] as u64) << 32 | entry.decomposition[1] as u64;

            map.insert(key, entry.code);
        }
    }

    map
}
