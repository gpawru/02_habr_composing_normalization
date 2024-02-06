use std::io::Write;
use std::{collections::HashMap, fs::File};

use crate::tables::{COMPOSITION_TABLE_DATA, CONTINUOUS_BLOCK_END};

use self::format::format_num_vec;

mod format;
mod stats;

/// длина строки в файле с подготовленными данными
const FORMAT_STRING_LENGTH: usize = 120;

/// пишем данные о декомпозиции
pub fn write(canonical: bool, file: &mut File /* stats_file: &mut File */)
{
    let mut stats = HashMap::new();
    let tables = crate::tables::prepare(canonical, &mut stats);

    let (name, _) = match canonical {
        true => ("NFC", 0xC0),
        false => ("NFKC", 0xA0),
    };

    let dec_starts_at = 0;

    let output = format!(
        "CompositionData {{\n  \
            index: &[{}  ],\n  \
            data: &[{}  ],\n  \
            expansions: &[{}  ],\n  \
            compositions: &[{} ],\n  \
            continuous_block_end: 0x{:04X},\n  \
            dec_starts_at: 0x{:04X},\n\
        }}\n",
        format_num_vec(tables.index.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(tables.data.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(tables.expansions.as_slice(), FORMAT_STRING_LENGTH),
        format_num_vec(COMPOSITION_TABLE_DATA.as_slice(), FORMAT_STRING_LENGTH),
        CONTINUOUS_BLOCK_END,
        dec_starts_at
    );

    write!(file, "{}", output).unwrap();

    stats::print(
        name,
        tables.index.as_slice(),
        tables.data.as_slice(),
        tables.expansions.as_slice(),
        dec_starts_at,
        stats
    );
    println!(
        "  размер блока композиций: {}",
        COMPOSITION_TABLE_DATA.len() * 8
    );
    
}
