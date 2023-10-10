use unicode_normalization_source::{properties::*, UNICODE};

/// проверим, в каких случаях происходит декомпозиция на несколько стартеров, идущих в паре с не-стартерами
fn main()
{
    let unicode = &UNICODE;

    macro_rules! validate {
        ($codepoint: expr, $decomposition: expr, $str: expr) => {
            let mut previous_ccc = CanonicalCombiningClass::NotReordered;

            for element in $decomposition.iter() {
                let ccc = match unicode.get(element) {
                    Some(codepoint) => codepoint.ccc,
                    None => CanonicalCombiningClass::NotReordered,
                };

                if ccc < previous_ccc {
                    println!("{}\nU+{:04X} - {}", $str, $codepoint.code, $codepoint.name);

                    for element in $decomposition.iter() {
                        let ccc = match unicode.get(element) {
                            Some(codepoint) => codepoint.ccc,
                            None => CanonicalCombiningClass::NotReordered,
                        };

                        print!("U+{:04X} ({}) ", element, u8::from(ccc));
                    }

                    println!("\n");
                }

                previous_ccc = ccc;
            }
        };
    }

    let mut keys: Vec<&u32> = unicode.keys().collect();
    keys.sort();

    for code in keys {
        let codepoint = unicode.get(code).unwrap();

        validate!(codepoint, codepoint.decomposition, "как в UCD");
        validate!(
            codepoint,
            codepoint.canonical_decomposition,
            "каноническая декомпозиция"
        );
        validate!(
            codepoint,
            codepoint.compat_decomposition,
            "декомпозиция совместимости"
        );
    }
}

/*

результат:

декомпозиция совместимости
U+3300 - SQUARE APAATO
U+30A2 (0) U+30CF (0) U+309A (8) U+30FC (0) U+30C8 (0)

...

заметим, что встречается только в китайском и арабском, только декомпозиция совместимости

диапазоны:
    - U+3300 ..= U+3356
    - U+FBEA ..= U+FCE0

*/
