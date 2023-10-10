#[macro_use]
extern crate lazy_static;

mod normalization_tests;
pub mod properties;
mod unicode;

pub use normalization_tests::NormalizationTest;
pub use normalization_tests::NORMALIZATION_TESTS;

pub use unicode::UNICODE;
