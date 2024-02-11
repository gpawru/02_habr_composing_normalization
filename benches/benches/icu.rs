use criterion::{criterion_group, criterion_main, Criterion};
use icu_normalizer::ComposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfc,
    test_nfc,
    "nfc",
    "icu",
    ComposingNormalizer::new_nfc()
);

group!(
    "./../test_data/texts",
    nfkc,
    test_nfkc,
    "nfkc",
    "icu",
    ComposingNormalizer::new_nfkc()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "icu",
    ComposingNormalizer::new_nfc()
);

criterion_group!(benches, nfc, nfkc, dec);
criterion_main!(benches);
