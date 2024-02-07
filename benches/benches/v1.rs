use criterion::{criterion_group, criterion_main, Criterion};
use unicode_composing_v1::ComposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfc,
    test_nfc,
    "nfc",
    "v1",
    ComposingNormalizer::nfc()
);

// group!(
//     "./../test_data/texts",
//     nfkc,
//     test_nfkc,
//     "nfkc",
//     "v1",
//     ComposingNormalizer::nfkc()
// );

criterion_group!(benches, nfc /* , nfkc */);
criterion_main!(benches);