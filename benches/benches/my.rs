use criterion::{criterion_group, criterion_main, Criterion};
use unicode_composing::ComposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfc,
    test_nfc,
    "nfc",
    "my",
    ComposingNormalizer::new_nfc()
);

group!(
    "./../test_data/texts",
    nfkc,
    test_nfkc,
    "nfkc",
    "my",
    ComposingNormalizer::new_nfkc()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "my",
    ComposingNormalizer::new_nfc()
);

criterion_group!(benches, nfc, nfkc, dec);
criterion_main!(benches);
