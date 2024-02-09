#[macro_use]
extern crate lazy_static;

pub mod normalization;
pub mod properties;

mod composition_exclusions;
mod normalization_tests;
mod quick_checks;
mod unicode;

pub use normalization_tests::NormalizationTest;
pub use normalization_tests::NORMALIZATION_TESTS;

pub use unicode::UNICODE;

pub use composition_exclusions::is_composition_exclusion;
pub use composition_exclusions::COMPOSITION_EXCLUSIONS;

pub use normalization::NFC;
pub use normalization::NFD;
pub use normalization::NFKC;
pub use normalization::NFKD;

pub use normalization::precomposition::pairs::COMBINES_BACKWARDS;
pub use normalization::precomposition::pairs::COMPOSITION_PAIRS;

pub use quick_checks::QC_NFC;
pub use quick_checks::QC_NFD;
pub use quick_checks::QC_NFKC;
pub use quick_checks::QC_NFKD;
