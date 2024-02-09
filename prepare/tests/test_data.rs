use std::io::Read;
use std::{collections::HashMap, fs};

use unicode_normalization_prepare::encode::encode_codepoint;
use unicode_normalization_source::UNICODE;

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
