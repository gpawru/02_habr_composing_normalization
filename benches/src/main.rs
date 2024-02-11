use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// выведем результаты бенчмарка как CSV
fn main()
{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Использование: {} <имя файла>", args[0]);
        return;
    }

    let file_name = &args[1];

    let absolute_path = match Path::new(file_name).canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return;
        }
    };

    let mut file = match File::open(absolute_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Не удалось открыть файл.");
            return;
        }
    };

    let mut contents = String::new();

    let result = file.read_to_string(&mut contents);

    match result {
        Ok(_) => {
            println!(
                "NFC:\n{}\n\nNFKC:\n{}\n\nDECOMPOSED NFC:\n{}\n\n",
                make_csv(contents.as_str(), "nfc"),
                make_csv(contents.as_str(), "nfkc"),
                make_csv(contents.as_str(), "dec")
            );
        }
        Err(_) => {
            println!("Не удалось прочитать файл.");
        }
    }
}

fn make_csv(source: &str, group: &str) -> String
{
    let source = parse_str(source, group);
    let (languages, variants) = get_rows_cols(&source);

    let mut result = "".to_owned();

    for variant in variants.iter() {
        result.push_str(format!(";{}", variant).as_str());
    }
    result.push('\n');

    for language in languages.iter() {
        let mut s = String::new();

        for variant in variants.iter() {
            let value = source
                .get(&format!("{}/{}/{}", group, variant, language))
                .unwrap_or(&0);

            s.push_str(format!(";{}", value).as_str());
        }

        result.push_str(format!("{}{}\n", language, s).as_str());
    }

    result
}

fn parse_str(source: &str, group: &str) -> HashMap<String, u32>
{
    let mut result = HashMap::new();

    for line in source.lines() {
        if line.is_empty() {
            continue;
        }

        if !line.starts_with(group) {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();

        let name = parts[0];
        let time = parts[4];
        let ttype = parts[5];

        let mut time = time.parse::<f64>().unwrap();

        if ttype == "ms" {
            time *= 1000.0;
        }

        let time = time.trunc() as u32;

        result.insert(name.to_owned(), time);
    }

    result
}

fn get_rows_cols(source: &HashMap<String, u32>) -> (Vec<String>, Vec<String>)
{
    let mut languages = vec![];
    let mut variants = vec![];

    for key in source.keys() {
        let parts: Vec<&str> = key.split('/').collect();

        if !languages.contains(&parts[2].to_owned()) {
            languages.push(parts[2].to_owned());
        };

        if !variants.contains(&parts[1].to_owned()) {
            variants.push(parts[1].to_owned());
        };
    }

    languages.sort();
    variants.sort();

    (languages, variants)
}
