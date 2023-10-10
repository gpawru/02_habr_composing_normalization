pub const WARM_UP_TIME: u64 = 3;
pub const MEASUREMENT_TIME: u64 = 7;

#[macro_export]
macro_rules! group {
    ($dir: expr, $fn: ident, $test: ident, $group: expr,  $name:expr,  $normalizer: expr) => {
        #[inline(never)]
        fn $test(normalizer: &DecomposingNormalizer, source: &str) -> String
        {
            normalizer.normalize(source)
        }

        fn $fn(c: &mut Criterion)
        {
            let mut group = c.benchmark_group($group);
            let normalizer = $normalizer;

            group.warm_up_time(core::time::Duration::from_secs(group::WARM_UP_TIME));
            group.measurement_time(core::time::Duration::from_secs(group::MEASUREMENT_TIME));

            for data in group::read_dir($dir) {
                let text_name = data.0.as_str();
                let text = data.1.as_str();

                group.bench_with_input(
                    criterion::BenchmarkId::new($name, &text_name),
                    &(&normalizer, text),
                    |b, data| b.iter(|| $test(data.0, criterion::black_box(data.1))),
                );
            }

            group.finish();
        }
    };
}

/// прочитать папку с тестовыми текстами
pub fn read_dir(dir: &str) -> Vec<(String, String)>
{
    let dir = std::fs::read_dir(dir).unwrap();

    let mut data: Vec<(String, String)> = vec![];

    for entry in dir {
        let entry = entry.unwrap();

        let path = entry.path();
        let path = path.to_str().unwrap();

        data.push((get_name(path).to_owned(), read(path, 1)));
    }

    data.sort_by(|a, b| a.0.cmp(&b.0));

    data
}

/// прочитать файл n раз
fn read(source: &str, times: usize) -> String
{
    let mut file = std::fs::File::open(source).unwrap();
    let mut buffer = String::new();

    std::io::Read::read_to_string(&mut file, &mut buffer).unwrap();

    let mut result = String::new();

    for _ in 0 ..= times {
        result.push_str(buffer.as_str());
    }

    result
}

/// вырезать из полного пути к файлу его название, без формата
fn get_name(filename: &str) -> &str
{
    let (_, name) = filename.trim_end_matches(".txt").rsplit_once('/').unwrap();

    name
}
