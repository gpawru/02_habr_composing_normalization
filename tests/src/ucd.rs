use unicode_composing::ComposingNormalizer as my;
use unicode_data::{NormalizationTest, NORMALIZATION_TESTS};

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

    test_group!(my::new_nfc());
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

    test_group!(my::new_nfkc());
}
