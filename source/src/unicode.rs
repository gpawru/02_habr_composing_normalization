use std::collections::HashMap;

use crate::properties::*;

lazy_static! {
    /// таблица Unicode
    pub static ref UNICODE: HashMap<u32, Codepoint> = unicode();
}

const DATA: &str = include_str!("./../data/ucd/15.1.0/UnicodeData.txt");

/// разбор UnicodeData.txt из UCD и составление хешмапа свойств символов Unicode
pub fn unicode() -> HashMap<u32, Codepoint>
{
    let mut map: HashMap<u32, Codepoint> = HashMap::new();

    // пригодится, когда встретим диапазоны
    let mut range_start: Option<Codepoint> = None;

    for line in DATA.lines() {
        let props: Vec<&str> = line.split(';').collect();

        // код и название
        let code = u32::from_str_radix(props[0], 16).unwrap();
        let name = props[1].to_owned();

        // начинается Private Use
        if code >= 0xF0000 {
            break;
        }

        // категория и CCC
        let gc = GeneralCategory::try_from(props[2]).unwrap();
        let ccc = CanonicalCombiningClass::try_from(props[3]).unwrap();

        // Bidi класс и Bidi Mirrored
        let bc = BidiClass::try_from(props[4]).unwrap();
        let bidi_mirrored = BidiMirrored::try_from(props[9]).unwrap();

        // декомпозиция и тег декомпозиции
        let decomposition = Decomposition::try_from(props[5]).unwrap();

        // различные numeric значения
        let numeric = NumericType::try_from((props[6], props[7], props[8])).unwrap();

        // связанные символы в другом регистре (если есть)
        let simple_uppercase_mapping = SimpleCaseMapping::try_from(props[12]).unwrap();
        let simple_lowercase_mapping = SimpleCaseMapping::try_from(props[13]).unwrap();
        let simple_titlecase_mapping = SimpleCaseMapping::try_from(props[14]).unwrap();

        // пропускаем колонки 10, 11:
        //
        // * Unicode_1_Name (Obsolete as of 6.2.0)
        // * ISO_Comment (Obsolete as of 5.2.0; Deprecated and Stabilized as of 6.0.0)

        let codepoint = Codepoint {
            code,
            name: name.clone(),
            gc,
            ccc,
            bc,
            numeric,
            bidi_mirrored,
            simple_uppercase_mapping,
            simple_lowercase_mapping,
            simple_titlecase_mapping,
            decomposition_tag: decomposition.tag,
            decomposition: decomposition.codes,
        };

        // различные блоки
        if name.starts_with('<') && (name != "<control>") {
            // что мы можем встретить:
            //
            // U+3400 ..= U+4DBF CJK Ideograph Extension A
            // U+4E00 ..= U+9FFF CJK Ideograph
            // U+AC00 ..= U+D7A3 Hangul Syllable
            // U+D800 ..= U+DB7F Non Private Use High Surrogate
            // U+DB80 ..= U+DBFF Private Use High Surrogate
            // U+DC00 ..= U+DFFF Low Surrogate
            // U+E000 ..= U+F8FF Private Use
            // U+17000 ..= U+187F7 Tangut Ideograph
            // U+18D00 ..= U+18D08 Tangut Ideograph Supplement
            // U+20000 ..= U+2A6DF CJK Ideograph Extension B
            // U+2A700 ..= U+2B739 CJK Ideograph Extension C
            // U+2B740 ..= U+2B81D CJK Ideograph Extension D
            // U+2B820 ..= U+2CEA1 CJK Ideograph Extension E
            // U+2CEB0 ..= U+2EBE0 CJK Ideograph Extension F
            // U+30000 ..= U+3134A CJK Ideograph Extension G
            // U+31350 ..= U+323AF CJK Ideograph Extension H

            // сразу отсекаем Private Use и суррогатные пары
            if name.contains("Private Use") || name.contains("Surrogate") {
                continue;
            }

            // остаются хангыль, тангутский и CJK, добавляем их в таблицу

            if name.ends_with("First>") {
                range_start = Some(codepoint);

                continue;
            }

            if name.ends_with("Last>") && range_start.is_some() {
                let group = range_start.unwrap();
                let group_name = &group.name[1 .. group.name.len() - 8];

                // в данном случае, для нас не важны названия символов
                // при необходимости, их можно получить из UCD - extracted/DerivedName.txt

                for i in group.code ..= code {
                    let mut codepoint = group.clone();

                    codepoint.code = i;
                    codepoint.name = format!("{} - {:X}", group_name, i);

                    map.insert(i, codepoint);
                }

                range_start = None;
            }

            continue;
        }

        map.insert(codepoint.code, codepoint);
    }

    map
}
