use icu_normalizer::ComposingNormalizer;
use unicode_normalization_source::{NormalizationTest, NORMALIZATION_TESTS};

use unicode_composing_v1::ComposingNormalizer as v1;
// use unicode_composing_v1::ComposingNormalizer as v1;
// use unicode_composing_v2::DecomposingNormalizer as v2;

macro_rules! test {
    ($left: expr, $right: expr, $normalizer: expr, $test: expr, $str: expr) => {
        assert_eq!(
            $left,
            $normalizer.normalize(&$right),
            stringify!($str),
            $test.line + 1,
            $test.description
        );
    };
}

#[test]
fn foo()
{
    let normalizer = v1::nfc();

    // 1100 AC00 11A8;1100 AC01;1100 1100 1161 11A8;1100 AC01;1100 1100 1161 11A8; # (ᄀ각; ᄀ각; ᄀ각; ᄀ각; ᄀ각; ) HANGUL CHOSEONG KIYEOK, HANGUL SYLLABLE GA, HANGUL JONGSEONG KIYEOK
    // 0374;02B9;02B9;02B9;02B9; # (ʹ; ʹ; ʹ; ʹ; ʹ; ) GREEK NUMERAL SIGN
    // 0958;0915 093C;0915 093C;0915 093C;0915 093C; # (क़; क◌़; क◌़; क◌़; क◌़; ) DEVANAGARI LETTER QA
    // 212B;00C5;0041 030A;00C5;0041 030A; # (Å; Å; A◌̊; Å; A◌̊; ) ANGSTROM SIGN
    let source = "\u{212b}";

    let result = normalizer.normalize(source);

    for char in result.chars() {
        print!("{:04X} ", u32::from(char));
    }
}

/// тесты NFC нормализации из UCD
#[test]
fn ucd_test_nfc()
{
    // c2 ==  toNFC(c1) ==  toNFC(c2) ==  toNFC(c3)
    // c4 ==  toNFC(c4) ==  toNFC(c5)

    let tests: &Vec<NormalizationTest> = &NORMALIZATION_TESTS;

    macro_rules! test_group {
        ($($normalizer: expr),+) => {
            $(
                let normalizer = $normalizer;

                for t in tests {
                    test!(t.c2, t.c1, normalizer, t, "{} {}: c2 == toNFC(c1)");
                    test!(t.c2, t.c2, normalizer, t, "{} {}: c2 == toNFC(c2)");
                    test!(t.c2, t.c3, normalizer, t, "{} {}: c2 == toNFC(c3)");
                    test!(t.c4, t.c4, normalizer, t, "{} {}: c4 == toNFC(c4)");
                    test!(t.c4, t.c5, normalizer, t, "{} {}: c4 == toNFC(c5)");
                }
            )+
        };
    }

    test_group!(v1::nfc());
}

/// тесты NFKC нормализации из UCD
#[test]
fn ucd_test_nfkc()
{
    // c4 == toNFKC(c1) == toNFKC(c2) == toNFKC(c3) == toNFKC(c4) == toNFKC(c5)

    let tests: &Vec<NormalizationTest> = &NORMALIZATION_TESTS;

    macro_rules! test_group {
        ($($normalizer: expr),+) => {
            $(
            let normalizer = $normalizer;

            for t in tests {
                test!(t.c4, t.c1, normalizer, t, "{} {}: c5 == toNFKD(c1)");
                test!(t.c4, t.c2, normalizer, t, "{} {}: c5 == toNFKD(c2)");
                test!(t.c4, t.c3, normalizer, t, "{} {}: c5 == toNFKD(c3)");
                test!(t.c4, t.c4, normalizer, t, "{} {}: c5 == toNFKD(c4)");
                test!(t.c4, t.c5, normalizer, t, "{} {}: c5 == toNFKD(c5)");
            }
        )+
        };
    }

    test_group!(v1::nfkc());
}
