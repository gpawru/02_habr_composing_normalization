use std::collections::HashMap;

use unicode_normalization_prepare::encode::encode_codepoint;
use unicode_normalization_source::{
    properties::Codepoint, COMPOSITION_EXCLUSIONS, COMPOSITION_PAIRS, NFC, NFKC, UNICODE,
};

/// последний кодпоинт в таблице UNICODE, имеющий декомпозицию
#[test]
fn last_decomposing_code()
{
    let mut codes: Vec<&u32> = UNICODE.keys().collect();
    codes.sort();

    let mut last = 0;
    let mut stats = HashMap::new();

    for code in codes {
        let codepoint = &UNICODE[code];
        let encoded = encode_codepoint(codepoint, true, 0, &mut stats);

        if encoded.value != 0 {
            last = *code;
        }
    }

    println!(
        "последний стартер без декомозиций и комбинирования: {:04X}",
        last
    )
}

#[test]
fn nfc_stats()
{
    let stats = stats(&NFC);

    let mut keys: Vec<&String> = stats.keys().collect();
    keys.sort();

    for key in keys {
        println!("{}: {}", key, stats[key]);
    }
}

#[test]
fn nfkc_stats()
{
    let stats = stats(&NFKC);

    let mut keys: Vec<&String> = stats.keys().collect();
    keys.sort();

    for key in keys {
        println!("{}: {}", key, stats[key]);
    }
}

/// статистика прекомпозиций
fn stats(precomposed: &HashMap<u32, Vec<Codepoint>>) -> HashMap<String, usize>
{
    let mut codepoints: Vec<u32> = precomposed.keys().map(|c| *c).collect();
    codepoints.sort();

    let mut stats: HashMap<String, usize> = HashMap::new();

    for code in codepoints.iter() {
        let precomposition = &precomposed[code];

        let starters_map: String = precomposition
            .iter()
            .map(|c| match c.is_starter() {
                true => 's',
                false => 'n',
            })
            .collect();

        if starters_map.is_empty() {
            continue;
        }

        stats
            .entry(starters_map)
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    stats
}

/// стартеры, комбинируемые с предыдущими кодпоинтами (пробуем найти закономерности для оптимизаций)
#[test]
fn starters_combining_with_previous()
{
    let mut unicode: Vec<&Codepoint> = UNICODE.values().collect();
    unicode.sort_by(|a, b| a.code.cmp(&b.code));
    let mut last = 0;

    for codepoint in unicode {
        if codepoint.decomposition.len() != 2
            || codepoint.decomposition_tag.is_some()
            || COMPOSITION_EXCLUSIONS.contains(&codepoint.code)
            || UNICODE[&codepoint.decomposition[1]].is_nonstarter()
        {
            continue;
        }

        let c0 = &UNICODE[&codepoint.decomposition[0]];
        let c1 = &UNICODE[&codepoint.decomposition[1]];

        if last == c1.code {
            continue;
        }
        last = c1.code;

        println!("{:#04X} - {}", c1.code, c1.name);
        assert!(c1.decomposition.is_empty());
        assert!(COMPOSITION_PAIRS.get(&c1.code).is_none());

        // первый элемент декомпозиции - может-ли он быть получен в результате комбинирования?
        if c0.decomposition.len() == 2 && c0.decomposition_tag.is_none() {
            println!(" - {:04X} - {}", c0.code, c0.name);
        }
    }

    // выводы:
    //  - все комбинируемые с ранее идущими кодпоинтами стартеры не имеют декомпозиции
    //  - их немного (можно захардкодить), находятся в довольно определяемых диапазонах
    //  - они не комбинируются с идущими за ними кодпоинтами
    //  - только в случае с U+0CD5 предыдущий стартер (U+0CCA) может быть получен в результате комбинирования
    //    (из 2х стартеров)
}