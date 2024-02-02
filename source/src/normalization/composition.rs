use std::collections::HashMap;

use crate::normalization::precomposition::precompose;
use crate::properties::Codepoint;
use crate::UNICODE;

/// прекомпозиция NFC
pub fn nfc() -> HashMap<u32, Vec<Codepoint>>
{
    UNICODE
        .iter()
        .map(|(code, codepoint)| (*code, precompose(codepoint.code, true)))
        .collect()
}

/// прекомпозиция NFKC
pub fn nfkc() -> HashMap<u32, Vec<Codepoint>>
{
    UNICODE
        .iter()
        .map(|(code, codepoint)| (*code, precompose(codepoint.code, false)))
        .collect()
}
