use crate::decomposition::Codepoint;
use crate::ComposingNormalizer;

#[test]
fn test_combine()
{
    let source = "\u{09C7}\u{09BE}";
    let nfc = ComposingNormalizer::nfc();

    let result = nfc.normalize(source);

    for char in result.chars() {
        print!("{:04X} ", u32::from(char));
    }

    println!()
}

/// композиция кодпоинтов и их запись в результирующую строку
#[inline(always)]
pub fn combine_and_flush(result: &mut String, buffer: &mut Vec<Codepoint>, compositions: &[u64])
{
    let len = buffer.len();

    // в буфере нет кодпоинтов, или один стартер - ничего не делаем
    if len <= 1 {
        return;
    }

    // первый кодпоинт должен быть стартером
    let mut combining = buffer[0].combining;

    macro_rules! result {
        ($code: expr) => {
            // SAFETY: значения получены из таблицы валидных кодпоинтов или таблицы композиций
            result.push(unsafe { char::from_u32_unchecked($code) })
        }
    }

    // если первый кодпоинт не комбинируется с идущими за ним кодпоинтами:
    //  - кодпоинты с первого по предпоследний добавляем в результат
    //  - последний ставим в начало буфера, если он может быть скомбинирован в дальнейшем
    if combining == 0 {
        let last = buffer[len - 1];

        for codepoint in buffer[.. len - 1].iter() {
            result!(codepoint.code);
        }

        buffer.clear();

        match last.combining {
            0 => result!(last.code),
            _ => buffer.push(last),
        }

        return;
    }

    let mut starter = buffer[0].code;
    let mut unwraped_combining = CodepointCombining::from(combining);
    let mut is_final = false;

    // TODO: буфер хвоста нужно переиспользовать, чтобы избежать повторных аллокаций
    let mut tail: Vec<Codepoint> = Vec::with_capacity(len);
    let mut iter = buffer[1 ..].iter();

    //
    let mut recent_skipped_ccc = 0;

    for codepoint in iter.by_ref() {
        let ccc = codepoint.ccc;

        if ccc != 0 {
            if ccc == recent_skipped_ccc {
                tail.push(*codepoint);
                continue;
            }
        } else if recent_skipped_ccc != 0 {
            tail.push(*codepoint);
            break;
        }

        let combined = combine(&unwraped_combining, codepoint.code, compositions);

        debug_assert!(codepoint.ccc >= recent_skipped_ccc);

        match combined {
            CombineResult::Combined(code, new_combining) => {
                starter = code;
                combining = new_combining;
                unwraped_combining = CodepointCombining::from(combining);
            }
            CombineResult::Final(code) => {
                // скомбинировали кодпоинты и оказалось, что больше скомбинировать ничего нельзя

                starter = code;
                is_final = true;

                for codepoint in iter {
                    tail.push(*codepoint);
                }

                break;
            }
            CombineResult::None => {
                tail.push(*codepoint);
                recent_skipped_ccc = ccc;
            }
        }
    }

    buffer.clear();

    // если в результате получили единственный кодпоинт, который может быть скомбинирован с последующими,
    // то сохраняем его в буфер, в противном случае - записываем его в результат
    if !is_final && tail.is_empty() {
        buffer.push(Codepoint {
            ccc: 0,
            code: starter,
            combining,
        });

        return;
    }

    result!(starter);

    // остались нескомбинированные кодпоинты после стартера?
    // в случае, если последний кодпоинт может быть скомбинирован с последующими -
    // сохраняем его в буфере для следующей итерации
    if let Some(last) = tail.pop() {
        for codepoint in tail.iter() {
            result!(codepoint.code);
        }

        match last.combining {
            0 => result!(last.code),
            _ => buffer.push(last),
        }
    }
}

/// результат комбинирования кодпоинтов
#[derive(Debug)]
enum CombineResult
{
    /// кодпоинты скомбинированы, полученный кодпоинт также может быть скомбинирован
    Combined(u32, u16),
    /// кодпоинты скомбинированы, полученный кодпоинт не может быть скомбинирован
    Final(u32),
    /// кодпоинты не комбинируются
    None,
}

/// скомбинировать два кодпоинта
#[inline(always)]
fn combine(combining: &CodepointCombining, second: u32, compositions: &[u64]) -> CombineResult
{
    let first = combining.index as usize;
    let last = first + combining.count as usize;

    for entry in &compositions[first .. last] {
        let entry = *entry;
        let entry_codepoint = entry as u32 & 0x3FFFF;

        // кодпоинты комбинируются
        if entry_codepoint == second {
            let code = (entry >> 18) as u32 & 0x3FFFF;
            let combining = (entry >> 48) as u16;

            return match combining {
                0 => CombineResult::Final(code),
                _ => CombineResult::Combined(code, combining),
            };
        }
    }

    CombineResult::None
}

/// распакованная информация о комбинировании -
/// индекс в таблице комбинаций и количество записанных для кодпоинта вариантов
pub struct CodepointCombining
{
    index: u16,
    count: u16,
}

impl From<u16> for CodepointCombining
{
    fn from(value: u16) -> Self
    {
        Self {
            index: value & 0x7FF,
            count: value >> 11,
        }
    }
}

/// отсортировать кодпоинты по CCC
#[inline(always)]
pub fn sort_by_ccc(buffer: &mut Vec<Codepoint>)
{
    if buffer.len() > 1 {
        buffer.sort_by_key(|c| c.ccc);
    }
}
