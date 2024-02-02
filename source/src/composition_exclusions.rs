lazy_static! {
    /// исключения композиции
    pub static ref COMPOSITION_EXCLUSIONS: Vec<u32> = composition_exclusions();
}

const DATA: &str = include_str!("./../data/ucd/15.1.0/CompositionExclusions.txt");

/// разбор CompositionExclusions.txt из UCD
/// исключения композиции не могут быть вычислены, этот список составляется консорциумом Unicode в ручном режиме
pub fn composition_exclusions() -> Vec<u32>
{
    let mut exclusions = vec![];

    for line in DATA.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (code, _) = line.split_once('#').unwrap();
        let code = u32::from_str_radix(code.trim(), 16).unwrap();

        exclusions.push(code);
    }

    exclusions
}

/// является ли кодпоинт исключением композиции?
pub fn is_composition_exclusion(code: u32) -> bool
{
    COMPOSITION_EXCLUSIONS.contains(&code)
}
