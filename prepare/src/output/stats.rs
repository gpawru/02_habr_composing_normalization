/// информация о данных декомпозиции
pub fn print(filename: &str, index: &[u32], data: &[u64], expansions: &[u32], dec_starts_at: u32)
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
}
