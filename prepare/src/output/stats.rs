use std::collections::HashMap;

/// информация о данных декомпозиции
pub fn print(
    filename: &str,
    index: &[u32],
    data: &[u64],
    expansions: &[u32],
    dec_starts_at: u32,
    stats: HashMap<String, usize>,
)
{
    println!(
        "\n{}:\n  \
        размер индекса: {}\n  \
        размер блока данных: {}\n  \
        размер дополнительных данных: {}\n  \
        общий размер: {}\n  \
        декомпозиция начинается с 0x{:04X}",
        filename,
        index.len(),
        data.len() * 8,
        expansions.len() * 4,
        index.len() + (data.len() * 8) + (expansions.len() * 4),
        dec_starts_at,
    );

    println!();

    let mut keys: Vec<&String> = stats.keys().collect();
    keys.sort_by(|a, b| stats[*b].cmp(&stats[*a]));

    for key in keys {
        println!("  {}: {}", key, stats[key]);
    }

    println!();
}
