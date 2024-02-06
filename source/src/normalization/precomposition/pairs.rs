use std::collections::HashMap;

use crate::{properties::Codepoint, COMPOSITION_EXCLUSIONS, UNICODE};

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMPOSITION_PAIRS: HashMap<u32, HashMap<u32, Codepoint>> = pairs();
    /// кодпоинты, комбинируемые с предыдущими
    pub static ref COMBINES_BACKWARDS: HashMap<u32, HashMap<u32, Codepoint>> = combines_backwards();
}

/// хешмап пар для композиции
fn pairs() -> HashMap<u32, HashMap<u32, Codepoint>>
{
    let mut map: HashMap<u32, HashMap<u32, Codepoint>> = HashMap::new();

    for codepoint in UNICODE.values() {
        if codepoint.decomposition.len() != 2 || codepoint.decomposition_tag.is_some() {
            continue;
        }

        if COMPOSITION_EXCLUSIONS.contains(&codepoint.code) {
            continue;
        }

        if codepoint.is_nonstarter() {
            continue;
        }

        let c0 = &UNICODE[&codepoint.decomposition[0]];
        let c1 = &UNICODE[&codepoint.decomposition[1]];

        if c0.is_nonstarter() && c1.is_nonstarter() {
            continue;
        }

        map.entry(c0.code)
            .and_modify(|c| {
                c.insert(c1.code, codepoint.clone());
            })
            .or_insert({
                let mut c = HashMap::new();
                c.insert(c1.code, codepoint.clone());
                c
            });
    }

    map
}

/// может ли быть скомбинирован с каким-либо предстоящим кодпоинтом?
fn combines_backwards() -> HashMap<u32, HashMap<u32, Codepoint>>
{
    let mut map: HashMap<u32, HashMap<u32, Codepoint>> = HashMap::new();

    for entry in UNICODE.values() {
        // декомпозиция отсутствует, синглтон или не является канонической
        if (entry.decomposition.len() != 2) || entry.decomposition_tag.is_some() {
            continue;
        }

        if COMPOSITION_EXCLUSIONS.contains(&entry.code) {
            continue;
        }

        let c0 = entry.decomposition[0];
        let c1 = entry.decomposition[1];

        if !map.contains_key(&c1) {
            map.insert(c1, HashMap::new());
        }

        map.entry(c1)
            .or_insert(HashMap::new())
            .insert(c0, entry.clone());
    }

    map
}
