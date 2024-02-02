use std::collections::HashMap;

use crate::properties::Codepoint;
use crate::UNICODE;

/// полная декомпозиция NFD
pub fn nfd() -> HashMap<u32, Vec<Codepoint>>
{
    UNICODE
        .iter()
        .map(|(code, codepoint)| (*code, decompose_entry(codepoint, true)))
        .collect()
}

/// полная декомпозиция NFKD
pub fn nfkd() -> HashMap<u32, Vec<Codepoint>>
{
    UNICODE
        .iter()
        .map(|(code, codepoint)| (*code, decompose_entry(codepoint, false)))
        .collect()
}

/// построить развернутую декомпозицию символа
fn decompose_entry(codepoint: &Codepoint, canonical: bool) -> Vec<Codepoint>
{
    let mut result: Vec<Codepoint> = vec![];

    // хотим получить каноническую декомпозицию, у элемента - декомопозиция совместимости
    if canonical && codepoint.decomposition_tag.is_some() {
        return result;
    }

    // проходим по всем элементам декомпозиции
    for code in codepoint.decomposition.iter() {
        let codepoint = &UNICODE[code];

        // получаем декомпозицию элемента (если она есть)
        let codepoint_decomposition = decompose_entry(codepoint, canonical);

        match codepoint_decomposition.is_empty() {
            true => {
                result.push(codepoint.clone());
            }
            false => {
                result.extend(codepoint_decomposition);
            }
        }
    }

    result
}
