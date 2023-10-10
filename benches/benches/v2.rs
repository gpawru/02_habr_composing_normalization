use criterion::{criterion_group, criterion_main, Criterion};
use unicode_decomposing_v2::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "opt",
    DecomposingNormalizer::nfd()
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "opt",
    DecomposingNormalizer::nfkd()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "opt",
    DecomposingNormalizer::nfd()
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
