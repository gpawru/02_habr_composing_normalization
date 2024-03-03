use crate::codepoint::Codepoint;
use crate::write_char;

pub use combine::*;
mod combine;

pub use hangul::combine_and_write_hangul_vt;
mod hangul;

/// композиция кодпоинтов и их запись
/// предполагается, что буффер не содержит несколько стартеров, стартер может стоять только в начале последовательности
#[inline(always)]
pub fn combine_and_write(
    result: &mut String,
    buffer: &mut Vec<Codepoint>,
    mut combining: Combining,
    compositions_table: &[u64],
)
{
    match buffer.len() {
        0 => return,
        1 => {
            result.push(char::from(buffer[0]));
            buffer.clear();

            return;
        }
        _ => (),
    };

    // если первый кодпоинт не комбинируется со следующими, то это может означать также то, что он может являться нестартером
    // в любом случае, порядок действий в таком случае один - отсортировать кодпоинты по CCC и записать

    if combining.is_none() {
        buffer.sort_by_key(|c| c.ccc());
        buffer.iter().for_each(|c| result.push(char::from(*c)));
        buffer.clear();

        return;
    }

    // остался только основной вариант - стартер, за которым следуют нестартеры

    let mut starter = buffer[0].code();
    let nonstarters = &mut buffer[1 ..];

    let mut tail = String::new();
    let mut recent_skipped_ccc = 0;

    if nonstarters.len() > 1 {
        nonstarters.sort_by_key(|c| c.ccc());
    }

    let mut iter = nonstarters.iter();

    for nonstarter in iter.by_ref() {
        let ccc = nonstarter.ccc();

        if ccc == recent_skipped_ccc {
            tail.push(char::from(*nonstarter));
            continue;
        }

        let combined = combine(combining, nonstarter.code(), compositions_table);

        match combined {
            CombineResult::Combined(new_starter, new_combining) => {
                starter = new_starter;
                combining = new_combining;
            }
            CombineResult::Final(new_starter) => {
                starter = new_starter;
                iter.for_each(|c| tail.push(char::from(*c)));

                break;
            }
            CombineResult::None => {
                tail.push(char::from(*nonstarter));
                recent_skipped_ccc = ccc;
            }
        }
    }

    buffer.clear();

    write_char(result, starter);
    result.push_str(&tail);
}

/// скомбинировать с предыдущим
#[inline(always)]
pub fn combine_backwards(
    buffer: &mut Vec<Codepoint>,
    result: &mut String,
    code: u32,
    mut combining: Combining,
    backwards_combining: Combining,
    compositions: &[u64],
) -> Combining
{
    combine_and_write(result, buffer, combining, compositions);

    combining = Combining::None;

    match result.pop() {
        Some(previous) => {
            let previous = u32::from(previous);

            match combine(backwards_combining, previous, compositions) {
                CombineResult::Combined(code, new_combining) => {
                    buffer.push(Codepoint::from_code(code));
                    combining = new_combining;
                }
                CombineResult::Final(code) => {
                    write_char(result, code);
                }
                CombineResult::None => {
                    write_char(result, previous);
                    write_char(result, code);
                }
            }
        }
        None => {
            write_char(result, code);
        }
    }

    combining
}
