use unicode_normalization_source::UNICODE;

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMPOSES_WITH_LEFT: Vec<u32> = composes_with_left();
}

/// может ли быть скомбинирован с каким-либо предстоящим кодпоинтом?
fn composes_with_left() -> Vec<u32>
{
    let unicode = &UNICODE;
    let mut map = Vec::new();

    for entry in unicode.values() {
        // декомпозиция отсутствует, синглтон или не является канонической
        if (entry.decomposition.len() != 2) || entry.decomposition_tag.is_some() {
            continue;
        }

        map.push(entry.decomposition[1]);
    }

    map
}
