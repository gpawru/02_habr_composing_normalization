use icu_normalizer::ComposingNormalizer;
use unicode_normalization_source::normalization::precomposition::hangul::is_composable_hangul;
use unicode_normalization_source::normalization::precomposition::combine;
use unicode_normalization_source::properties::Codepoint;
use unicode_normalization_source::{NFC, NFKC, UNICODE, COMPOSITION_PAIRS};

#[test]
fn test_nfc()
{
    test_precomposition(true);
}

#[test]
fn test_nfkc()
{
    test_precomposition(false);
}

fn test_precomposition(canonical: bool)
{
    let mut codepoints: Vec<&u32> = UNICODE.keys().collect();
    codepoints.sort();

    let icu_normalizer = ComposingNormalizer::new_nfc();
    let seconds = no_decomposition_list();

    for code in codepoints {
        let codepoint = &UNICODE[code];

        let mut precomposition = match canonical {
            true => NFC[code].clone(),
            false => NFKC[code].clone(),
        };

        if precomposition.is_empty() {
            if !has_compositions(*code) {
                continue;
            }

            precomposition = vec![codepoint.clone()];
        }

        let precomposition_str: String = precomposition.iter().map(|c| c.as_char()).collect();
        println!("{:04X} - {}", codepoint.code, codepoint.name);

        for second in seconds.iter() {
            let source = format!("{}{}", precomposition_str, second.as_char());
            let composed = normalize_precomposed_str(&precomposition, second);
            let icu_composed = icu_normalizer.normalize(source.as_str());

            if composed != icu_composed {
                println!("precomposition:");
                for c in precomposition.iter() {
                    print!("{:04X}({}) ", c.code, c.ccc.u8());
                }
                println!("+ {:04X}({})", second.code, second.ccc.u8());

                println!("my: ");
                for c in composed.chars() {
                    print!("{:04X} ", u32::from(c));
                }
                println!();
                println!("icu: ");
                for c in icu_composed.chars() {
                    print!("{:04X} ", u32::from(c));
                }
                println!();
            }

            assert_eq!(composed, icu_composed);
        }
    }
}

fn normalize_precomposed_str(precomposition: &[Codepoint], next: &Codepoint) -> String
{
    normalize_precomposed(precomposition, next)
        .iter()
        .map(|c| c.as_char())
        .collect()
}

/// тестируем прекомпозицию - конец прекомпозиции может состоять из нескомбинированных кодпоинтов
/// комбинируем со стартером или нестартером (не имеющим декомпозиции)
fn normalize_precomposed(precomposition: &[Codepoint], next: &Codepoint) -> Vec<Codepoint>
{
    assert!(!precomposition.is_empty());
    assert!(next.decomposition.is_empty() || next.decomposition_tag.is_some());

    // находим хвост прекомпозиции
    let mut last_starter = 0;

    for (i, codepoint) in precomposition.iter().enumerate().rev() {
        if codepoint.is_starter() {
            last_starter = i;
            break;
        }
    }

    // не нашли стартер
    // if last_starter == 0 && precomposition[0].is_nonstarter() {
    if precomposition[0].is_nonstarter() {
        let mut result = precomposition.to_vec();
        result.push(next.clone());

        if next.is_nonstarter() {
            result.sort_by(|a, b| a.ccc.u8().cmp(&b.ccc.u8()));
        }

        return result;

        //panic!("не нашли стартер");
    }

    // возможен вариант, когда в предкомпозиции присутствует несколько стартеров - записываем
    // заранее скомбинированную часть в результат
    let mut result = precomposition[.. last_starter].to_vec();

    // последовательность из прекомпозиции и следующего за ней кодпоинта заканчиваются нестартерами - комбинируем

    let mut tail: Vec<Codepoint> = vec![];
    let mut first = precomposition[last_starter].clone();

    let mut nonstarters = precomposition[last_starter + 1 ..].to_vec();

    if next.is_nonstarter() {
        nonstarters.push(next.clone());
    }

    nonstarters.sort_by(|a, b| a.ccc.u8().cmp(&b.ccc.u8()));

    let mut last_ccc = 0;

    for second in nonstarters.iter() {
        if last_ccc == second.ccc.u8() {
            tail.push(second.clone());
            continue;
        }

        if let Some(combined) = combine(first.code, second.code) {
            first = UNICODE[&combined].clone();
            last_ccc = 0;
            continue;
        }

        tail.push(second.clone());
        last_ccc = second.ccc.u8();
    }

    result.push(first);
    result.extend(tail);

    // следующий за прекомпозицией элемент - нестартер? мы его уже скомбинировали
    if next.is_nonstarter() {
        return result;
    }

    // остался стартер, но скомбинированный результат заканчивается нестартером? тогда - просто добавим его к результату
    if result.last().unwrap().is_nonstarter() {
        result.push(next.clone());

        return result;
    }

    // последний элемент скомбинированной прекомпозиции - стартер, и следующий за ней элемент - стартер
    match combine(result.last().unwrap().code, next.code) {
        Some(combined) => {
            let last = result.len() - 1;
            result[last] = UNICODE[&combined].clone();
        }
        None => {
            result.push(next.clone());
        }
    }

    result
}

/// стартеры/нестартеры без декомпозиции - т.е. они могут присутствовать в композиции
fn no_decomposition_list() -> Vec<Codepoint>
{
    let mut list: Vec<Codepoint> = vec![];

    for codepoint in UNICODE.values() {
        if codepoint.decomposition.is_empty() || codepoint.decomposition_tag.is_some() {
            list.push(codepoint.clone());
        }
    }

    list.sort_by_key(|c| c.code);

    list
}

/// может ли кодпоинт быть скомбинирован с чем-либо?
fn has_compositions(code: u32) -> bool
{
    if is_composable_hangul(code) {
        return true;
    }

    COMPOSITION_PAIRS.get(&code).is_some()
}
