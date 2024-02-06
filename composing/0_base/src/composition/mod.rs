use crate::codepoint::Codepoint;
use crate::write;

pub use combine::*;
mod combine;

/// композиция кодпоинтов и их запись
/// предполагается, что буффер не содержит несколько стартеров, стартер может стоять только в начале последовательности
#[inline(always)]
pub fn combine_and_write(
    buffer: &mut Vec<Codepoint>,
    result: &mut String,
    mut combining: Combining,
    compositions_table: &[u64],
)
{
    match buffer.len() {
        0 => return,
        1 => {
            result.push(buffer[0].char());
            buffer.clear();

            return;
        }
        _ => (),
    };

    // если первый кодпоинт не комбинируется со следующими, то это может означать также то, что он может являться нестартером
    // в любом случае, порядок действий в таком случае один - отсортировать кодпоинты по CCC и записать

    if combining.is_none() {
        buffer.sort_by_key(|c| c.ccc);

        for c in buffer.iter() {
            result.push(c.char());
        }

        buffer.clear();
        return;
    }

    // остался только основной вариант - стартер, за которым следуют нестартеры

    let mut starter = buffer[0].code;
    let nonstarters = &mut buffer[1 ..];

    let mut tail: Vec<u32> = vec![];
    let mut recent_skipped_ccc = 0;

    if nonstarters.len() > 1 {
        nonstarters.sort_by_key(|c| c.ccc);
    }

    let mut iter = nonstarters.iter();

    for nonstarter in iter.by_ref() {
        let ccc = nonstarter.ccc;

        if ccc == recent_skipped_ccc {
            tail.push(nonstarter.code);
            continue;
        }

        let combined = combine(combining, nonstarter.code, compositions_table);

        match combined {
            CombineResult::Combined(new_starter, new_combining) => {
                starter = new_starter;
                combining = new_combining;
            }
            CombineResult::Final(new_starter) => {
                starter = new_starter;

                for codepoint in iter {
                    tail.push(codepoint.code);
                }

                break;
            }
            CombineResult::None => {
                tail.push(nonstarter.code);
                recent_skipped_ccc = ccc;
            }
        }
    }

    buffer.clear();

    write!(result, starter);

    for codepoint in tail {
        write!(result, codepoint);
    }
}
