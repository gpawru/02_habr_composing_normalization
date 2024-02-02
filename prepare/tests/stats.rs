use std::collections::HashMap;

use unicode_normalization_prepare::encode::encode_codepoint;
use unicode_normalization_source::{properties::Codepoint, NFC, NFKC, UNICODE};


/// последний кодпоинт в таблице UNICODE, имеющий декомпозицию 
#[test]
fn last_decomposing_code()
{
    let mut codes: Vec<&u32> = UNICODE.keys().collect();
    codes.sort();

    let mut last = 0;

    for code in codes {
        let codepoint = &UNICODE[code];
        let encoded = encode_codepoint(codepoint, true, 0);

        if encoded.value != 0 {
            last = *code;
        }  
    }

    println!("последний стартер без декомозиций и комбинирования: {:04X}", last)
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
