use std::collections::HashMap;

use unicode_normalization_source::UNICODE;

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMPOSES_WITH_LEFT: Vec<u32> = composes_with_left();
    pub static ref COMPOSES_WITH_RIGHT: Vec<u32> = composes_with_right();
    /// пары
    pub static ref COMPOSITIONS: HashMap<(u32, u32), u32> = compositions();
}

/// может ли быть скомбинирован с каким-либо предстоящим кодпоинтом?
fn composes_with_left() -> Vec<u32>
{
    composes(1)
}

// может ли быть скомбинирован с каким-либо следующим кодпоинтом?
fn composes_with_right() -> Vec<u32>
{
    composes(0)
}

fn composes(i: usize) -> Vec<u32>
{
    let mut map = Vec::new();

    for entry in UNICODE.values() {
        // декомпозиция отсутствует, синглтон или не является канонической
        if (entry.decomposition.len() != 2) || entry.decomposition_tag.is_some() {
            continue;
        }

        map.push(entry.decomposition[i]);
    }

    map
}

/// каноническая композиция
fn compositions() -> HashMap<(u32, u32), u32>
{
    let mut map = HashMap::new();

    for entry in UNICODE.values() {
        if entry.decomposition.len() == 2 && entry.decomposition_tag.is_none() {
            let key = (entry.decomposition[0], entry.decomposition[1]);

            map.insert(key, entry.code);
        }
    }

    map
}
