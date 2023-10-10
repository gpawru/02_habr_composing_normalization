use std::collections::HashMap;
use unicode_normalization_source::{properties::*, UNICODE};

/// какие есть исключения в декомпозициях?
fn main()
{
    let unicode: &HashMap<u32, Codepoint> = &UNICODE;

    let mut st_ucd: HashMap<u32, bool> = HashMap::new();
    let mut st_canonical: HashMap<u32, bool> = HashMap::new();
    let mut st_compat: HashMap<u32, bool> = HashMap::new();
    let mut ns = vec![];

    for codepoint in unicode.values() {
        // нас не интересуют кодпоинты без декомпозиции
        if codepoint.decomposition.is_empty() {
            continue;
        }

        if codepoint.ccc.is_non_starter() && !codepoint.decomposition.is_empty() {
            ns.push(codepoint.code);
            continue;
        }

        macro_rules! fill_starters_to_nonstarters {
            ($dec: expr, $result: expr) => {
                if !$dec.is_empty() {
                    let first_ccc = match unicode.get(&$dec[0]) {
                        Some(codepoint) => codepoint.ccc,
                        None => CanonicalCombiningClass::NotReordered,
                    };

                    // стартер с декомпозицией в не-стартер
                    if codepoint.ccc.is_starter() && first_ccc.is_non_starter() {
                        $result.insert(codepoint.code, true);
                    }
                }
            };
        }

        fill_starters_to_nonstarters!(codepoint.decomposition, st_ucd);
        fill_starters_to_nonstarters!(codepoint.canonical_decomposition, st_canonical);
        fill_starters_to_nonstarters!(codepoint.compat_decomposition, st_compat);
    }

    macro_rules! print_st {
        ($dec: expr, $str: expr) => {
            let mut keys: Vec<&u32> = $dec.keys().collect();
            keys.sort();

            println!("\n{}:\n", $str);

            for code in keys {
                let codepoint = unicode.get(code).unwrap();

                println!("U+{:04X} - {}", codepoint.code, codepoint.name);
            }
        };
    }

    println!("\nстартеры с декомпозицией в не-стартеры:");

    print_st!(st_ucd, "UCD");
    print_st!(st_canonical, "каноническая декомпозиция");
    print_st!(st_compat, "декомпозиция совместимости");

    println!("\nне-стартеры с декомпозицией:\n");

    ns.sort();

    for code in ns {
        let codepoint = unicode.get(&code).unwrap();
        print!("U+{:04X} - {}\n  ", codepoint.code, codepoint.name);

        for element in codepoint.decomposition.iter() {
            let codepoint = unicode.get(element).unwrap();
            print!("U+{:04X} ({}) ", codepoint.code, u8::from(codepoint.ccc));
        }
        println!("\n");
    }

    println!();
}

/*

результат:

стартеры с декомпозицией в не-стартеры:

UCD:

U+0F73 - TIBETAN VOWEL SIGN II
U+0F75 - TIBETAN VOWEL SIGN UU
U+0F81 - TIBETAN VOWEL SIGN REVERSED II
U+FF9E - HALFWIDTH KATAKANA VOICED SOUND MARK
U+FF9F - HALFWIDTH KATAKANA SEMI-VOICED SOUND MARK

каноническая декомпозиция:

U+0F73 - TIBETAN VOWEL SIGN II
U+0F75 - TIBETAN VOWEL SIGN UU
U+0F81 - TIBETAN VOWEL SIGN REVERSED II

декомпозиция совместимости:

U+0F73 - TIBETAN VOWEL SIGN II
U+0F75 - TIBETAN VOWEL SIGN UU
U+0F81 - TIBETAN VOWEL SIGN REVERSED II
U+FF9E - HALFWIDTH KATAKANA VOICED SOUND MARK
U+FF9F - HALFWIDTH KATAKANA SEMI-VOICED SOUND MARK

не-стартеры с декомпозицией:

U+0340 - COMBINING GRAVE TONE MARK
  U+0300 (230)

U+0341 - COMBINING ACUTE TONE MARK
  U+0301 (230)

U+0343 - COMBINING GREEK KORONIS
  U+0313 (230)

U+0344 - COMBINING GREEK DIALYTIKA TONOS
  U+0308 (230) U+0301 (230)

*/
