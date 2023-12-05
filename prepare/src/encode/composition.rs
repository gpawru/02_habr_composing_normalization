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

        if is_composition_exception(entry.code) {
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

        if is_composition_exception(codepoint.code) {
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

/// исключения композиции
pub fn is_composition_exception(code: u32) -> bool
{
    let exceptions = [
        // (1) Script Specifics
        0x0958, // DEVANAGARI LETTER QA
        0x0959, // DEVANAGARI LETTER KHHA
        0x095A, // DEVANAGARI LETTER GHHA
        0x095B, // DEVANAGARI LETTER ZA
        0x095C, // DEVANAGARI LETTER DDDHA
        0x095D, // DEVANAGARI LETTER RHA
        0x095E, // DEVANAGARI LETTER FA
        0x095F, // DEVANAGARI LETTER YYA
        0x09DC, // BENGALI LETTER RRA
        0x09DD, // BENGALI LETTER RHA
        0x09DF, // BENGALI LETTER YYA
        0x0A33, // GURMUKHI LETTER LLA
        0x0A36, // GURMUKHI LETTER SHA
        0x0A59, // GURMUKHI LETTER KHHA
        0x0A5A, // GURMUKHI LETTER GHHA
        0x0A5B, // GURMUKHI LETTER ZA
        0x0A5E, // GURMUKHI LETTER FA
        0x0B5C, // ORIYA LETTER RRA
        0x0B5D, // ORIYA LETTER RHA
        0x0F43, // TIBETAN LETTER GHA
        0x0F4D, // TIBETAN LETTER DDHA
        0x0F52, // TIBETAN LETTER DHA
        0x0F57, // TIBETAN LETTER BHA
        0x0F5C, // TIBETAN LETTER DZHA
        0x0F69, // TIBETAN LETTER KSSA
        0x0F76, // TIBETAN VOWEL SIGN VOCALIC R
        0x0F78, // TIBETAN VOWEL SIGN VOCALIC L
        0x0F93, // TIBETAN SUBJOINED LETTER GHA
        0x0F9D, // TIBETAN SUBJOINED LETTER DDHA
        0x0FA2, // TIBETAN SUBJOINED LETTER DHA
        0x0FA7, // TIBETAN SUBJOINED LETTER BHA
        0x0FAC, // TIBETAN SUBJOINED LETTER DZHA
        0x0FB9, // TIBETAN SUBJOINED LETTER KSSA
        0xFB1D, // HEBREW LETTER YOD WITH HIRIQ
        0xFB1F, // HEBREW LIGATURE YIDDISH YOD YOD PATAH
        0xFB2A, // HEBREW LETTER SHIN WITH SHIN DOT
        0xFB2B, // HEBREW LETTER SHIN WITH SIN DOT
        0xFB2C, // HEBREW LETTER SHIN WITH DAGESH AND SHIN DOT
        0xFB2D, // HEBREW LETTER SHIN WITH DAGESH AND SIN DOT
        0xFB2E, // HEBREW LETTER ALEF WITH PATAH
        0xFB2F, // HEBREW LETTER ALEF WITH QAMATS
        0xFB30, // HEBREW LETTER ALEF WITH MAPIQ
        0xFB31, // HEBREW LETTER BET WITH DAGESH
        0xFB32, // HEBREW LETTER GIMEL WITH DAGESH
        0xFB33, // HEBREW LETTER DALET WITH DAGESH
        0xFB34, // HEBREW LETTER HE WITH MAPIQ
        0xFB35, // HEBREW LETTER VAV WITH DAGESH
        0xFB36, // HEBREW LETTER ZAYIN WITH DAGESH
        0xFB38, // HEBREW LETTER TET WITH DAGESH
        0xFB39, // HEBREW LETTER YOD WITH DAGESH
        0xFB3A, // HEBREW LETTER FINAL KAF WITH DAGESH
        0xFB3B, // HEBREW LETTER KAF WITH DAGESH
        0xFB3C, // HEBREW LETTER LAMED WITH DAGESH
        0xFB3E, // HEBREW LETTER MEM WITH DAGESH
        0xFB40, // HEBREW LETTER NUN WITH DAGESH
        0xFB41, // HEBREW LETTER SAMEKH WITH DAGESH
        0xFB43, // HEBREW LETTER FINAL PE WITH DAGESH
        0xFB44, // HEBREW LETTER PE WITH DAGESH
        0xFB46, // HEBREW LETTER TSADI WITH DAGESH
        0xFB47, // HEBREW LETTER QOF WITH DAGESH
        0xFB48, // HEBREW LETTER RESH WITH DAGESH
        0xFB49, // HEBREW LETTER SHIN WITH DAGESH
        0xFB4A, // HEBREW LETTER TAV WITH DAGESH
        0xFB4B, // HEBREW LETTER VAV WITH HOLAM
        0xFB4C, // HEBREW LETTER BET WITH RAFE
        0xFB4D, // HEBREW LETTER KAF WITH RAFE
        0xFB4E, // HEBREW LETTER PE WITH RAFE
        // (2) Post Composition Version precomposed characters
        0x2ADC,  //  FORKING
        0x1D15E, //  MUSICAL SYMBOL HALF NOTE
        0x1D15F, //  MUSICAL SYMBOL QUARTER NOTE
        0x1D160, //  MUSICAL SYMBOL EIGHTH NOTE
        0x1D161, //  MUSICAL SYMBOL SIXTEENTH NOTE
        0x1D162, //  MUSICAL SYMBOL THIRTY-SECOND NOTE
        0x1D163, //  MUSICAL SYMBOL SIXTY-FOURTH NOTE
        0x1D164, //  MUSICAL SYMBOL ONE HUNDRED TWENTY-EIGHTH NOTE
        0x1D1BB, //  MUSICAL SYMBOL MINIMA
        0x1D1BC, //  MUSICAL SYMBOL MINIMA BLACK
        0x1D1BD, //  MUSICAL SYMBOL SEMIMINIMA WHITE
        0x1D1BE, //  MUSICAL SYMBOL SEMIMINIMA BLACK
        0x1D1BF, //  MUSICAL SYMBOL FUSA WHITE
        0x1D1C0, //  MUSICAL SYMBOL FUSA BLACK
    ];

    exceptions.contains(&code)
}
