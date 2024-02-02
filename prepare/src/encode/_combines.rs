use unicode_normalization_source::{COMPOSITION_EXCLUSIONS, UNICODE};

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMBINES_BACKWARDS: Vec<u32> = combines_backwards();
    pub static ref COMBINES_FORWARDS: Vec<u32> = combines_forwards();
}

/// может ли быть скомбинирован с каким-либо предстоящим кодпоинтом?
fn combines_backwards() -> Vec<u32>
{
    combines(1)
}

/// может ли быть скомбинирован с каким-либо следующим кодпоинтом?
fn combines_forwards() -> Vec<u32>
{
    combines(0)
}

/// может ли кодпоинт быть скомбинирован?
fn combines(i: usize) -> Vec<u32>
{
    let mut map = Vec::new();

    for entry in UNICODE.values() {
        // декомпозиция отсутствует, синглтон или не является канонической
        if (entry.decomposition.len() != 2) || entry.decomposition_tag.is_some() {
            continue;
        }

        if COMPOSITION_EXCLUSIONS.contains(&entry.code) {
            continue;
        }

        map.push(entry.decomposition[i]);
    }

    map
}
