use criterion::{criterion_group, criterion_main, Criterion};
use unicode_composing_v0::ComposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfc,
    test_nfc,
    "nfc",
    "base",
    ComposingNormalizer::nfc()
);

group!(
    "./../test_data/texts",
    nfkc,
    test_nfkc,
    "nfkc",
    "base",
    ComposingNormalizer::nfkc()
);

criterion_group!(benches, nfc, nfkc);
criterion_main!(benches);