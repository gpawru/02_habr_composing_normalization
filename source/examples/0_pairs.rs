use std::collections::HashMap;
use unicode_normalization_source::{properties::*, UNICODE};

/// какие стартеры комбинируются? см. стабильные кодпоинты
/// https://www.unicode.org/reports/tr15/#Stable_Code_Points
fn main()
{
    let unicode: &HashMap<u32, Codepoint> = &UNICODE;

    let mut combining_start: Vec<u32> = Vec::new();
    let mut combining_end: Vec<u32> = Vec::new();

    for codepoint in unicode.values() {
        if codepoint.decomposition.len() == 2 && codepoint.decomposition_tag.is_none() {
            let c0 = codepoint.decomposition[0];
            let c1 = codepoint.decomposition[1];

            if !combining_start.contains(&c0) {
                combining_start.push(c0);
            }

            if !combining_end.contains(&c1) {
                combining_end.push(c1);
            }
        }
    }

    combining_start.sort();
    combining_end.sort();

    println!("starting");
    for c in combining_start.iter() {
        print!("{:#04X} ", c);
    }
    println!();

    println!("ending");
    for c in combining_end.iter() {
        print!("{:#04X} ", c);
    }
    println!();

    // кодпоинты при полной декомпозиции

    for codepoint in unicode.values() {
        if codepoint.canonical_decomposition.len() >= 2 {
            let c0 = codepoint.canonical_decomposition[0];
            let c1 = *codepoint.canonical_decomposition.last().unwrap();

            if combining_end.contains(&c0) && c0 != 0x308 {
                panic!("{:#04X} {:#04X}", c0, codepoint.code);
            }

            if combining_start.contains(&c1) && c1 != 0x308 {
                panic!("{:#04X} {:#04X}", c1, codepoint.code);
            }
        }
    }
}
