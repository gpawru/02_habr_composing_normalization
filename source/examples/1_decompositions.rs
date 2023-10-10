use std::collections::HashMap;
use unicode_normalization_source::{properties::*, UNICODE};

const LAST_NON_PRIVATE_UNICODE_CODEPOINT: u32 = 0xEFFFF;

/// давайте поймём:
/// - какое количество элементов присутствует в декомпозициях символов
/// - где начинаются и где заканчиваются в таблице кодпоинты с декомпозицией
fn main()
{
    let unicode: &HashMap<u32, Codepoint> = &UNICODE;

    let mut ucd_canonical_lengths: HashMap<u8, (u32, u32)> = HashMap::new();
    let mut ucd_compat_lengths: HashMap<u8, (u32, u32)> = HashMap::new();

    let mut canonical_lengths: HashMap<u8, (u32, u32)> = HashMap::new();
    let mut compat_lengths: HashMap<u8, (u32, u32)> = HashMap::new();

    let mut canonical_boundaries = (0, 0);
    let mut compat_boundaries = (0, 0);

    let mut min_first = u32::MAX;

    let mut min_triple_first_byte = 0xFF;

    macro_rules! inc_length {
        ($expr: expr, $dec: expr) => {
            let len = $dec.len() as u8;
            let c = $expr.entry(len).or_insert((0, 0));
            (*c).0 += 1;

            if $dec.iter().any(|&v| v > 0xFFFF) {
                (*c).1 += 1;
            }
        };
    }

    macro_rules! inc_lengths {
        ($can: expr, $comp: expr) => {
            inc_length!(canonical_lengths, $can);
            inc_length!(compat_lengths, $comp);
        };
    }

    macro_rules! boundaries {
        ($code: expr, $bound: ident, $len: expr) => {
            if $len > 0 {
                if $bound.0 == 0 {
                    $bound.0 = $code;
                }
                $bound.1 = $code;
            }
        };
    }

    macro_rules! triple_first_check {
        ($dec: expr) => {
            if $dec.len() == 3 {
                min_triple_first_byte = core::cmp::min($dec[0] & 0xFF, min_triple_first_byte);
            }
        };
    }

    for code in 0 ..= LAST_NON_PRIVATE_UNICODE_CODEPOINT {
        let codepoint = match unicode.get(&code) {
            Some(codepoint) => codepoint,
            None => {
                inc_lengths!([0u32; 0], [0u32; 0]);
                continue;
            }
        };

        match codepoint.decomposition_tag.is_none() {
            true => {
                inc_length!(ucd_canonical_lengths, codepoint.decomposition);
            }
            false => {
                inc_length!(ucd_compat_lengths, codepoint.decomposition);
            }
        };

        inc_lengths!(
            codepoint.canonical_decomposition,
            codepoint.compat_decomposition
        );

        boundaries!(
            code,
            canonical_boundaries,
            codepoint.canonical_decomposition.len() as u8
        );
        boundaries!(
            code,
            compat_boundaries,
            codepoint.compat_decomposition.len() as u8
        );

        if !codepoint.decomposition.is_empty() {
            min_first = std::cmp::min(min_first, codepoint.decomposition[0]);
        }

        triple_first_check!(codepoint.decomposition);
        triple_first_check!(codepoint.canonical_decomposition);
        triple_first_check!(codepoint.compat_decomposition);
    }

    // выводим результаты

    macro_rules! print_lengths {
        ($lengths: ident, $str: expr) => {
            println!("\n{}:", $str);

            let mut keys: Vec<&u8> = $lengths.keys().collect();
            keys.sort();

            for key in keys {
                let counts = $lengths.get(key).unwrap();
                print!("  {} - {}", key, counts.0);
                if counts.1 > 0 {
                    print!(" ({})", counts.1)
                }
                println!();
            }
        };
    }

    print_lengths!(
        ucd_canonical_lengths,
        "каноническая декомпозиция, как хранится в UCD"
    );
    print_lengths!(
        ucd_compat_lengths,
        "декомпозиция совместимости, как хранится в UCD"
    );

    print_lengths!(canonical_lengths, "каноническая декомпозиция");
    print_lengths!(compat_lengths, "декомпозиция совместимости (+каноническая)");

    println!(
        "\nдиапазон канонической декомпозиции: U+{:04X} ..= U+{:X}",
        canonical_boundaries.0, canonical_boundaries.1
    );

    println!(
        "\nдиапазон декомпозиции совместимости: U+{:04X} ..= U+{:X}\n",
        compat_boundaries.0, compat_boundaries.1,
    );

    println!(
        "минимальный первый элемент декомпозиции: U+{:04X}\n",
        min_first
    );

    // полезность этой проверки неочевидна, но она нам пригодится в дальнейшем
    println!(
        "минимальное значение младшего байта первого элемента декомпозиции в 3 кодпоинта: 0x{:02X}\n",
        min_triple_first_byte
    )
}

/*

результат (в скобках - декомпозиции, содержащие 18-битные кодпоинты):

каноническая декомпозиция, как хранится в UCD:
  0 - 143394
  1 - 1035 (112)
  2 - 1026 (26)

декомпозиция совместимости, как хранится в UCD:
  1 - 2648 (6)
  2 - 648
  3 - 411
  4 - 67
  5 - 16
  6 - 3
  7 - 1
  8 - 1
  18 - 1

каноническая декомпозиция:
  0 - 980979
  1 - 1017 (112)
  2 - 779 (17)
  3 - 229 (9)
  4 - 36

декомпозиция совместимости (+каноническая):
  0 - 977183
  1 - 3639 (118)
  2 - 1388 (17)
  3 - 688 (9)
  4 - 109
  5 - 16
  6 - 14
  7 - 1
  8 - 1
  18 - 1

диапазон канонической декомпозиции: U+00C0 ..= U+2FA1D

диапазон декомпозиции совместимости: U+00A0 ..= U+2FA1D

минимальный первый элемент декомпозиции: U+0020

минимальное значение младшего байта первого элемента декомпозиции в 3 кодпоинта: 0x14

*/
