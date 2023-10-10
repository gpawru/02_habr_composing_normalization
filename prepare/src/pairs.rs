use std::collections::HashMap;

use unicode_normalization_source::UNICODE;

#[test]
fn test_nfc()
{
    nfc();
}

pub fn nfc()
{
    let unicode = &UNICODE;

    let mut count = 0;

    let mut map: HashMap<u64, u32> = HashMap::new();

    for (_, codepoint) in unicode.iter() {
        if codepoint.decomposition.len() == 2 && codepoint.decomposition_tag.is_none() {
            let c0 = codepoint.decomposition[0];
            let c1 = codepoint.decomposition[1];

            let key = ((c0 as u64) << 32) | c1 as u64;

            if map.contains_key(&key) {
                panic!("{:X} {}", codepoint.code, codepoint.name);
            }

            let value = c1;

            if map.values().any(|x| *x == c1) {
                let iter = map.values().filter(|x| **x == c1);

                println!("{:X} {}", c1, iter.count());
            }

            map.insert(key, value);

            count += 1;
        }
    }

    println!("{}", count);
}
