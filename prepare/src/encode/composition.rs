use std::collections::HashMap;

use unicode_normalization_source::UNICODE;

use crate::tables::compositions::compositions;

lazy_static! {
    /// комбинируемые кодпоинты
    pub static ref COMBINES_BACKWARDS: Vec<u32> = combines_backwards();
    pub static ref COMBINES_FORWARDS: Vec<u32> = combines_forwards();
    /// пары
    pub static ref COMPOSITION_PAIRS: HashMap<u32, HashMap<u32, u32>> = pairs();
    pub static ref COMPOSITION_REFS: HashMap<u32, CompositionInfo> = composition_refs();
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

        map.push(entry.decomposition[i]);
    }

    map
}

/// хешмап пар для композиции
fn pairs() -> HashMap<u32, HashMap<u32, u32>>
{
    let mut map: HashMap<u32, HashMap<u32, u32>> = HashMap::new();

    for codepoint in UNICODE.values() {
        if codepoint.decomposition.len() != 2 || codepoint.decomposition_tag.is_some() {
            continue;
        }

        let c0 = codepoint.decomposition[0];
        let c1 = codepoint.decomposition[1];

        map.entry(c0)
            .and_modify(|c| {
                c.insert(c1, codepoint.code);
            })
            .or_insert({
                let mut c = HashMap::new();
                c.insert(c1, codepoint.code);
                c
            });
    }

    map
}

fn composition_refs() -> HashMap<u32, CompositionInfo>
{
    let (_, refs) = compositions();
    refs
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
    pub fn bake(&self) -> u16
    {
        assert!(self.index <= 0x7FF);
        assert!(self.count <= 0x1F);

        self.index | ((self.count as u16) << 11)
    }
}
