use std::fs;
use std::io::Read;

/// данные на разных языках для тестов
pub fn files() -> Vec<(String, String)>
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

/// прочитать файл
fn read(source: &str) -> String
{
    let mut file = fs::File::open(source).unwrap();
    let mut result = String::new();

    file.read_to_string(&mut result).unwrap();

    result
}

/// вырезать из полного пути к файлу его название, без формата
fn get_name(filename: &str) -> &str
{
    let (_, name) = filename.trim_end_matches(".txt").rsplit_once('/').unwrap();

    name
}
