use crate::decomposition::Codepoint;
use crate::write;
use crate::ComposingNormalizer;

use combine::{combine, CodepointCombining, CombineResult};

mod combine;

#[test]
fn test_combine()
{
    // 1100 AC00 11A8;1100 AC01;1100 1100 1161 11A8;1100 AC01;1100 1100 1161 11A8; # (ᄀ각; ᄀ각; ᄀ각; ᄀ각; ᄀ각; ) HANGUL CHOSEONG KIYEOK, HANGUL SYLLABLE GA, HANGUL JONGSEONG KIYEOK

    // 1100 AC00 11A8
    // 1100 AC01

    let source = "\u{1100}\u{AC00}\u{11A8}";
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

    // если первый кодпоинт не комбинируется с идущими за ним кодпоинтами:
    //  - кодпоинты с первого по предпоследний добавляем в результат
    //  - последний ставим в начало буфера, если он может быть скомбинирован в дальнейшем
    if combining == 0 {
        let last = buffer[len - 1];

        for codepoint in buffer[.. len - 1].iter() {
            write!(result, codepoint.code);
        }

        buffer.clear();

        match last.combining {
            0 => write!(result, last.code),
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

    write!(result, starter);

    // остались нескомбинированные кодпоинты после стартера?
    flush_tail(&mut tail, buffer, result);
}

/// записать оставшиеся после комбинирования символы
/// в случае, если последний кодпоинт может быть скомбинирован с последующими -
/// сохраняем его в буфере для следующей итерации
#[inline(always)]
fn flush_tail(tail: &mut Vec<Codepoint>, buffer: &mut Vec<Codepoint>, result: &mut String)
{
    if let Some(last) = tail.pop() {
        for codepoint in tail.iter() {
            write!(result, codepoint.code);
        }

        match last.combining {
            0 => write!(result, last.code),
            _ => buffer.push(last),
        }
    }
}
