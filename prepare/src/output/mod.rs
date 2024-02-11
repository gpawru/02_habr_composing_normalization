use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::tables::{COMPOSITION_TABLE_DATA, CONTINUOUS_BLOCK_END};

use self::format::format_num_vec;

mod format;
mod stats;

/// длина строки в файле с подготовленными данными
const FORMAT_STRING_LENGTH: usize = 120;

/// пишем данные о декомпозиции
pub fn write(canonical: bool, file: &mut File)
{
    let mut stats = HashMap::new();
    let tables = crate::tables::prepare(canonical, &mut stats);

    let name = match canonical {
        true => "NFC",
        false => "NFKC",
    };

    let output = format!(
        "CompositionData {{\n  \
            index: &[{}  ],\n  \
            data: &[{}  ],\n  \
            expansions: &[{}  ],\n  \
            compositions: &[{} ],\n  \
            continuous_block_end: 0x{:04X},\n\
        }}\n",
        format_num_vec(tables.index.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(tables.data.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(tables.expansions.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(COMPOSITION_TABLE_DATA.as_slice(), FORMAT_STRING_LENGTH),
        CONTINUOUS_BLOCK_END,
    );

    write!(file, "{}", output).unwrap();

    stats::print(
        name,
        tables.index.as_slice(),
        tables.data.as_slice(),
        tables.expansions.as_slice(),
        COMPOSITION_TABLE_DATA.as_slice(),
        stats,
    );
}
