use std::collections::HashMap;

use unicode_normalization_source::{
    properties::{CanonicalCombiningClass, Codepoint},
    UNICODE,
};

#[test]
fn test_nfc()
{
    let pairs = pairs();
}

/// хешмап пар для композиции
pub fn pairs() -> HashMap<u32, HashMap<u32, u32>>
{
    let unicode = &UNICODE;

    let mut map: HashMap<u32, HashMap<u32, u32>> = HashMap::new();

    let mut i = 0;

    for codepoint in unicode.values() {
        if codepoint.decomposition.len() != 2 || codepoint.decomposition_tag.is_some() {
            continue;
        }

        let c0 = codepoint.decomposition[0];
        let c1 = codepoint.decomposition[1];

        i += 1;

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

    println!("{}", i);

    map
}
