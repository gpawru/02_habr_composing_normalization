use std::collections::HashMap;

use crate::{properties::Codepoint, COMPOSITION_EXCLUSIONS, NFD, NFKD, UNICODE};

use pairs::COMPOSITION_PAIRS;

/// композиция хангыль
pub mod hangul;

/// комбинирование кодпоинтов (пары)
pub mod pairs;

/// прекомпозиция - в некоторых случаях делать полную декомпозицию, а затем каноническую композицию - избыточно
pub fn precompose(code: u32, canonical: bool) -> Vec<Codepoint>
{
    let decomposition_map: &HashMap<u32, Vec<Codepoint>> = match canonical {
        true => &NFD,
        false => &NFKD,
    };

    let mut decomposition = match decomposition_map.get(&code) {
        Some(decomposition) => decomposition.clone(),
        None => return vec![],
    };

    if decomposition.len() <= 1 {
        return decomposition;
    }

    let mut start = 0;

    loop {
        if start >= decomposition.len() - 1 {
            break;
        }

        if decomposition[start].is_nonstarter() {
            start += 1;
            continue;
        }

        let mut end = start + 1;

        loop {
            if end >= decomposition.len() - 1 {
                break;
            }

            if decomposition[end].is_starter() {
                break;
            }

            end += 1;
        }

        let mut first = decomposition[start].clone();
        let mut tail: Vec<Codepoint> = vec![];

        // не делаем композицию подпоследовательности, если она находится в конце и заканчивается нестартером
        // однако есть частный случай - все нестартеры подпоследовательности имеют минимальный CCC

        if (end + 1 == decomposition.len()) && decomposition[end].is_nonstarter() {
            let tail = get_min_variant(&decomposition[start ..]);
            decomposition = decomposition[.. start].to_vec();
            decomposition.extend(tail);

            break;
        }

        let mut last_starter = first.clone();
        let mut last_ccc = 255;
        let mut move_to_next = 1;

        for codepoint in decomposition[start + 1 ..= end].iter() {
            if codepoint.is_starter() && !tail.is_empty() {
                end -= 1;
                break;
            }

            if codepoint.ccc.u8() == last_ccc && last_starter.code == first.code {
                tail.push(codepoint.clone());
                continue;
            }

            if codepoint.is_starter() {
                move_to_next = 0;
            }

            match combine(first.code, codepoint.code) {
                Some(combined) => first = UNICODE[&combined].clone(),
                None => {
                    last_starter = first.clone();
                    last_ccc = codepoint.ccc.u8();
                    tail.push(codepoint.clone())
                }
            }
        }

        let mut new_dec = decomposition[0 .. start].to_vec();
        start = start + tail.len() + move_to_next;

        new_dec.push(first);
        new_dec.extend(tail);
        new_dec.extend(decomposition[end + 1 ..].to_vec());

        decomposition = new_dec;
    }

    // пересобрали кодпоинт обратно в себя?
    match decomposition.len() == 1 && code == decomposition[0].code {
        true => vec![],
        false => decomposition,
    }
}

pub fn combine(a: u32, b: u32) -> Option<u32>
{
    match hangul::compose_hangul(a, b) {
        Some(c) => Some(c),
        None => {
            let pairs = match COMPOSITION_PAIRS.get(&a) {
                Some(pairs) => pairs.clone(),
                None => HashMap::new(),
            };

            match pairs.get(&b) {
                Some(c) => Some(c.code),
                None => None,
            }
        }
    }
}

/// последовательность из стартера и нескольких нестартеров. можно-ли упростить?
fn get_min_variant(decomposition: &[Codepoint]) -> Vec<Codepoint>
{
    assert!(decomposition[0].is_starter());
    assert!(decomposition[1 ..].iter().all(|c| c.is_nonstarter()));

    let mut first = decomposition[0].clone();
    let nonstarters = &decomposition[1 ..];

    let mut pos = 0;

    for nonstarter in nonstarters.iter() {
        if COMPOSITION_EXCLUSIONS.contains(&first.code) {
            break;
        }

        match min_ccc(first.code) {
            Some(min_ccc) if nonstarter.ccc.u8() == min_ccc => (),
            _ => break,
        };

        match combine(first.code, nonstarter.code) {
            Some(combined) => {
                first = UNICODE[&combined].clone();
            }
            None => {
                break;
            }
        }

        pos += 1;
    }

    let mut result = vec![first];
    result.extend(nonstarters[pos ..].to_vec());

    result
}

/// минимальный ccc следующего кодпоинта, с которым можно скомбинировать текущий
fn min_ccc(code: u32) -> Option<u8>
{
    let mut min_ccc = 255;

    match COMPOSITION_PAIRS.get(&code) {
        Some(pairs) => {
            for c in pairs.keys() {
                let ccc = u8::from(UNICODE[c].ccc);

                if ccc == 0 {
                    continue;
                }

                if ccc < min_ccc {
                    min_ccc = ccc;
                }
            }

            Some(min_ccc)
        }
        None => None,
    }
}
