use std::io::Read;
use std::{collections::HashMap, fs};

use unicode_normalization_prepare::encode::encode_codepoint;
use unicode_normalization_source::UNICODE;

// что не так с чешским?
#[test]
fn test_czech()
{
    let files = files();
    let (_, text) = files.iter().find(|(name, _)| name == "czech").unwrap();
    let mut stats = HashMap::new();

    let mut chars: HashMap<u32, (usize, u64)> = HashMap::new();

    for char in text.chars() {
    
        let code = u32::from(char);
        let codepoint = &UNICODE.get(&code).unwrap();
        let encoded = encode_codepoint(codepoint, true, 0, &mut stats);

        chars.entry(code).and_modify(|c| c.0 += 1).or_insert((0, encoded.value));
    }

    let mut keys: Vec<&u32> = chars.keys().collect();
    keys.sort();

    for code in keys {
        let count = chars[code].0;
        let value = chars[code].1;

        if value & 1 != 0 {
            panic!();
        }

        let marker = (value as u8 >> 1) & 0b_111;

        if marker != 1 {
            continue;
        }

        println!("{} - {:04X} - {}", char::from_u32(*code).unwrap(), code, count);
    }    
}

/// поймём, в чём причина просадок для определенных языков
#[test]
fn test_languages()
{
    let files = files();

    for (language, data) in files.iter() {
        println!("{}:", language);
        let stats = language_stats(data, true);

        let mut keys: Vec<&String> = stats.keys().collect();
        keys.sort_by_key(|key| stats[*key]);

        for key in keys {
            println!("  {}: {}", key, stats[key]);
        }

        println!()
    }
}

/// пройдемся по всем кодпоинтам и соберём статистику
fn language_stats(data: &String, canonical: bool) -> HashMap<String, usize>
{
    let mut stats = HashMap::new();
    for char in data.chars() {
        let code = u32::from(char);
        let codepoint = &UNICODE.get(&code);

        if codepoint.is_none() {
            *stats.entry("стартер".to_owned()).or_default() += 1;
            continue;
        }

        let _ = encode_codepoint(codepoint.unwrap(), canonical, 0, &mut stats);
    }
    stats
}

/// данные на разных языках для тестов
fn files() -> Vec<(String, String)>
{
    let dir = fs::read_dir("./../test_data/texts").unwrap();

    let mut data = vec![];

    for entry in dir {
        let entry = entry.unwrap();

        let path = entry.path();
        let path = path.to_str().unwrap();

        data.push((get_name(path).to_owned(), read(path)));
    }

    data.sort_by(|a, b| a.0.cmp(&b.0));

    data
}

/// вырезать из полного пути к файлу его название, без формата
fn get_name(filename: &str) -> &str
{
    let (_, name) = filename.trim_end_matches(".txt").rsplit_once('/').unwrap();

    name
}

/// прочитать файл
fn read(source: &str) -> String
{
    let mut file = fs::File::open(source).unwrap();
    let mut result = String::new();

    file.read_to_string(&mut result).unwrap();

    result
}
