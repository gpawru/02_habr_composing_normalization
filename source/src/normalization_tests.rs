/// тест из UCD
#[derive(Debug)]
pub struct NormalizationTest
{
    pub part: String,
    pub description: String,
    pub line: usize,
    pub c1: String,
    pub c2: String,
    pub c3: String,
    pub c4: String,
    pub c5: String,
}

lazy_static! {
    /// тесты нормализации из UCD
    pub static ref NORMALIZATION_TESTS: Vec<NormalizationTest> = normalization_tests();
}

const DATA: &str = include_str!("./../data/ucd/15.1.0/NormalizationTest.txt");

/// разбор NormalizationTest.txt из UCD
fn normalization_tests() -> Vec<NormalizationTest>
{
    let mut result = vec![];
    let mut part = String::new();

    for (i, line) in DATA.lines().enumerate() {
        if line.starts_with('#') {
            continue;
        }

        if line.starts_with('@') {
            part = line.to_owned();
            continue;
        }

        let (codes, description) = line.split_once('#').unwrap();
        let codes: Vec<&str> = codes.split(';').collect();

        if codes.len() != 6 {
            panic!("{}: некорректное количество полей теста", i);
        }

        macro_rules! codes {
            ($str: expr) => {{
                $str.split_whitespace()
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|v| unsafe {
                        char::from_u32_unchecked(u32::from_str_radix(v, 16).unwrap())
                    })
                    .collect()
            }};
        }

        result.push(NormalizationTest {
            part: part.clone(),
            description: description.to_owned(),
            line: i,
            c1: codes!(codes[0]),
            c2: codes!(codes[1]),
            c3: codes!(codes[2]),
            c4: codes!(codes[3]),
            c5: codes!(codes[4]),
        })
    }

    result
}
