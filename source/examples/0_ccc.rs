use std::cmp::{max, min};
use std::collections::HashMap;
use unicode_normalization_source::{properties::*, UNICODE};

/// в каких границах находятся не-стартеры?
/// сколько стартеров, сколько не-стартеров?
fn main()
{
    let mut from = u32::MAX;
    let mut to = 0;

    let mut starters = 0;
    let mut non_starters = 0;

    let unicode: &HashMap<u32, Codepoint> = &UNICODE;

    for codepoint in unicode.values() {
        if codepoint.ccc.is_starter() {
            starters += 1;
            continue;
        }

        non_starters += 1;

        from = min(from, codepoint.code);
        to = max(to, codepoint.code);
    }

    println!(
        "\nне-стартеры находятся в пределах диапазона: U+{:04X} ..= U+{:04X}\n",
        from, to
    );

    println!("стартеров (записанных в UnicodeData.txt): {}, не-стартеров: {}\n", starters, non_starters);
}

/*

результат:

не-стартеры находятся в пределах диапазона: U+0300 ..= U+1E94A

стартеров: 148329, не-стартеров: 922

*/
