use unicode_normalization_source::{NormalizationTest, NORMALIZATION_TESTS};

use unicode_composing_v1::DecomposingNormalizer as v1;
// use unicode_composing_v2::DecomposingNormalizer as v2;

macro_rules! test {
    ($left: expr, $right: expr, $normalizer: expr, $test: expr, $str: expr) => {
        assert_eq!(
            $left,
            $normalizer.normalize(&$right),
            stringify!($str),
            $test.line,
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

    test_group!(v1::nfd() /*, v2::nfc() */);
}

// /// тесты NFKC нормализации из UCD
// #[test]
// fn ucd_test_nfkc()
// {
//     // c4 == toNFKC(c1) == toNFKC(c2) == toNFKC(c3) == toNFKC(c4) == toNFKC(c5)

//     let tests: &Vec<NormalizationTest> = &NORMALIZATION_TESTS;

//     macro_rules! test_group {
//         ($($normalizer: expr),+) => {
//             $(
//             let normalizer = $normalizer;

//             for t in tests {
//                 test!(t.c5, t.c1, normalizer, t, "{} {}: c5 == toNFKD(c1)");
//                 test!(t.c5, t.c2, normalizer, t, "{} {}: c5 == toNFKD(c2)");
//                 test!(t.c5, t.c3, normalizer, t, "{} {}: c5 == toNFKD(c3)");
//                 test!(t.c5, t.c4, normalizer, t, "{} {}: c5 == toNFKD(c4)");
//                 test!(t.c5, t.c5, normalizer, t, "{} {}: c5 == toNFKD(c5)");
//             }
//         )+
//         };
//     }

//     test_group!(v1::nfkd(), v2::nfkd());
// }
