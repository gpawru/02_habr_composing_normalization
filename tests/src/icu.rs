use icu_normalizer::DecomposingNormalizer as icu;
use unicode_composing_v1::DecomposingNormalizer as v1;
use unicode_composing_v2::DecomposingNormalizer as v2;

/// сравниваем с результатами нормализации ICU
#[test]
fn icu()
{
    let icu_nfd = icu::new_nfd();
    let icu_nfkd = icu::new_nfkd();

    macro_rules! test {
        ($(($n: ident,  $t: expr)),+) => {
            $(
                let nfd = $n::nfd();
                let nfkd = $n::nfkd();

                for data in crate::data::files() {
                    assert_eq!(
                        nfd.normalize(data.1.as_str()),
                        icu_nfd.normalize(data.1.as_str()),
                        "nfd,  {} - {}",
                        $t,
                        data.0
                    );
                    assert_eq!(
                        nfkd.normalize(data.1.as_str()),
                        icu_nfkd.normalize(data.1.as_str()),
                        "nfkd, {} - {}",
                        $t,
                        data.0
                    );
                }
            )+
        };
    }

    test!((v1, "v1"), (v2, "v2"));
}
