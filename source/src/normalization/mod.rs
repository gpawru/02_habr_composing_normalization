use std::collections::HashMap;

use crate::properties::Codepoint;

pub mod composition;
pub mod decomposition;
pub mod precomposition;

// в файле UnicodeData.txt хранится декомпозиция в сжатом виде, т.е. элементы декомпозиции
// могут также иметь свою декомпозицию.
//  * NFD, NFKD: получаем развернутую версию декомпозиции
//  * NFC, NFKC: получаем "пересобранную" версию - если кодпоинт может быть скомбинирован единственно возможным
//    способом - зачем в таком случае сначала делать декомпозицию, а потом собирать обратно? (пример - синглтоны)

lazy_static! {
    /// таблица декомпозиций NFD
    pub static ref NFD: HashMap<u32, Vec<Codepoint>> = decomposition::nfd();

    /// таблица декомпозиций NFKD
    pub static ref NFKD: HashMap<u32, Vec<Codepoint>> = decomposition::nfkd();

    /// таблица композиций NFC
    pub static ref NFC: HashMap<u32, Vec<Codepoint>> = composition::nfc();

    /// таблица композиций NFKC
    pub static ref NFKC: HashMap<u32, Vec<Codepoint>> = composition::nfkc();
}
