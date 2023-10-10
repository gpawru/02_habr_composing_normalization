use unicode_normalization_source::UNICODE;

/// декомпозиции с символами за пределами BMP
fn main()
{
    let unicode = &UNICODE;

    let mut keys: Vec<&u32> = unicode.keys().collect();
    keys.sort();

    println!("\nкодпоинты, имеющие в своей декомпозиции символы, выходящие за пределы BMP:\n");

    for code in keys {
        let codepoint = unicode.get(code).unwrap();

        if codepoint.decomposition.is_empty() {
            continue;
        }

        let mut record = vec![];

        macro_rules! has_nonbmp {
            ($dec: expr,  $mark: expr) => {
                for element in $dec.iter() {
                    if *element > 0xFFFF && !record.contains(&$mark) {
                        record.push($mark);
                    }
                }
            };
        }

        has_nonbmp!(codepoint.decomposition, 'U');
        has_nonbmp!(codepoint.canonical_decomposition, 'C');
        has_nonbmp!(codepoint.compat_decomposition, 'K');

        if record.is_empty() {
            continue;
        }

        let map: String = record.iter().collect();

        println!("U+{:04X} ({}) - {}", codepoint.code, map, codepoint.name);
    }
}
