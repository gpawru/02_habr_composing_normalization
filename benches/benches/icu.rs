use criterion::{criterion_group, criterion_main, Criterion};
use icu_normalizer::DecomposingNormalizer;

mod group;

group!(
    "./../test_data/texts",
    nfd,
    test_nfd,
    "nfd",
    "icu",
    DecomposingNormalizer::try_new_nfd_unstable(&icu_testdata::unstable()).unwrap()
);

group!(
    "./../test_data/texts",
    nfkd,
    test_nfkd,
    "nfkd",
    "icu",
    DecomposingNormalizer::try_new_nfkd_unstable(&icu_testdata::unstable()).unwrap()
);

group!(
    "./../test_data/texts_decomposed",
    dec,
    test_dec,
    "dec",
    "icu",
    DecomposingNormalizer::try_new_nfd_unstable(&icu_testdata::unstable()).unwrap()
);

criterion_group!(benches, nfd, nfkd, dec);
criterion_main!(benches);
