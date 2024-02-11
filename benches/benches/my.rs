use criterion::{criterion_group, criterion_main, Criterion};
use unicode_composing::ComposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfc,
    test_nfc,
    "nfc",
    "my",
    ComposingNormalizer::nfc()
);

group!(
    "./../test_data/texts",
    nfkc,
    test_nfkc,
    "nfkc",
    "my",
    ComposingNormalizer::nfkc()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "my",
    ComposingNormalizer::nfc()
);

criterion_group!(benches, nfc, nfkc, dec);
criterion_main!(benches);

