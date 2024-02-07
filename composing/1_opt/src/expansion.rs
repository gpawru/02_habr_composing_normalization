use crate::codepoint::Codepoint;
use crate::combine;
use crate::combine_and_write;
use crate::write;
use crate::CombineResult;
use crate::Combining;

/// последовательность стартеров:
///  - информация о комбинировании записана для последнего стартера последовательности
const MARKER_EXPANSION_STARTERS: u8 = 0b_100;
/// стартер и не-стартеры
const MARKER_EXPANSION_STARTER_NONSTARTERS: u8 = 0b_101;
/// два стартера + нестартер
///  - информация о комбинировании записана для второго стартера
const MARKER_EXPANSION_TWO_STARTERS_NONSTARTER: u8 = 0b_110;
/// исключения - стартеры, которые декомпозируются в нестартеры
const MARKER_EXPANSION_NONSTARTERS_EXCLUSION: u8 = 0b_111;
/// исключения - стартеры, которые комбинируются с предыдущими кодпоинтами
const MARKER_EXPANSION_COMBINES_BACKWARDS: u8 = 0b_1000;

/// информация о данных, вынесенных в отдельный блок
#[derive(Debug)]
pub struct Expansion
{
    /// маркер расширения
    pub marker: u8,
    /// количество кодпоинтов
    pub len: u8,
    /// индекс внешней таблицы
    pub index: u16,
    /// комбинирование последнего стартера последовательности
    pub combining: Combining,
}

/// частные случаи декомпозиции:
///  - из последовательности стартеров
///  - > 2 кодпоинтов или состоящая из кодпоинтов за пределами BMP
///  - 2 стартера + нестартер
///  - нестартеры
#[inline(never)]
pub fn combine_expansion(
    buffer: &mut Vec<Codepoint>,
    result: &mut String,
    code: u32,
    combining: Combining,
    expansion: Expansion,
    compositions_table: &[u64],
    expansions_table: &[u32],
) -> Combining
{
    let start = expansion.index as usize;
    let end = start + expansion.len as usize;

    let mut new_combining = expansion.combining;

    match expansion.marker {
        // стартеры
        MARKER_EXPANSION_STARTERS => {
            combine_and_write(buffer, result, combining, compositions_table);

            for codepoint in expansions_table[start .. end - 1].iter() {
                write!(result, *codepoint);
            }

            buffer.push(Codepoint::from_compressed(expansions_table[end - 1]));
        }
        // стартер + нестартеры
        MARKER_EXPANSION_STARTER_NONSTARTERS => {
            combine_and_write(buffer, result, combining, compositions_table);

            for codepoint in expansions_table[start .. end].iter() {
                buffer.push(Codepoint::from_compressed(*codepoint));
            }
        }
        // стартер + стартер + нестартер
        MARKER_EXPANSION_TWO_STARTERS_NONSTARTER => {
            combine_and_write(buffer, result, combining, compositions_table);

            write!(result, expansions_table[start]);

            for codepoint in expansions_table[start + 1 .. end].iter() {
                buffer.push(Codepoint::from_compressed(*codepoint));
            }
        }
        // нестартеры
        MARKER_EXPANSION_NONSTARTERS_EXCLUSION => {
            for codepoint in expansions_table[start .. end].iter() {
                buffer.push(Codepoint::from_compressed(*codepoint));
            }

            new_combining = combining;
        }
        // комбинируются с предыдущим
        MARKER_EXPANSION_COMBINES_BACKWARDS => {
            combine_and_write(buffer, result, combining, compositions_table);

            match result.pop() {
                None => {
                    write!(result, code);
                }
                Some(char) => {
                    let previous = u32::from(char);

                    match combine(expansion.combining, previous, &compositions_table) {
                        CombineResult::None => {
                            write!(result, previous, code);
                        }
                        CombineResult::Final(code) => {
                            write!(result, code);
                        }
                        CombineResult::Combined(code, combining) => {
                            buffer.push(Codepoint { code, ccc: 0 });

                            new_combining = combining
                        }
                    }
                }
            }
        }
        _ => unreachable!(),
    }

    new_combining
}
