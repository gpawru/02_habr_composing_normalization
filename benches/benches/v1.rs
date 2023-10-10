use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing_v1::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "base",
    DecomposingNormalizer::nfd()
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "base",
    DecomposingNormalizer::nfkd()
);

criterion_group!(benches, nfd, nfkd);
criterion_main!(benches);
