use std::collections::HashMap;

/// информация о данных
pub fn print(
    filename: &str,
    index: &[u32],
    data: &[u64],
    expansions: &[u32],
    compositions: &[u64],
    stats: HashMap<String, usize>,
)
{
    println!(
        "\n{}:\n  \
        размер индекса: {}\n  \
        размер блока данных: {}\n  \
        размер дополнительных данных: {}\n  \
        композиции: {}\n  \
        общий размер: {}\n",
        filename,
        index.len(),
        data.len() * 8,
        expansions.len() * 4,
        compositions.len() * 8,
        index.len() + (data.len() * 8) + (expansions.len() * 4) + (compositions.len() * 8)
    );

    println!();

    let mut keys: Vec<&String> = stats.keys().collect();
    keys.sort_by(|a, b| stats[*b].cmp(&stats[*a]));

    for key in keys {
        println!("  {}: {}", key, stats[key]);
    }

    println!();
}
