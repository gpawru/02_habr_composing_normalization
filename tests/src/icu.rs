use icu_normalizer::ComposingNormalizer as icu;
use unicode_composing::ComposingNormalizer as my;

/// сравниваем с результатами нормализации ICU
#[test]
fn icu()
{
    let icu_nfc = icu::new_nfc();
    let icu_nfkc = icu::new_nfkc();

    macro_rules! test {
        ($(($n: ident,  $t: expr)),+) => {
            $(
                let nfc = $n::nfc();
                let nfkc = $n::nfkc();

                for data in crate::data::files() {
                    assert_eq!(
                        nfc.normalize(data.1.as_str()),
                        icu_nfc.normalize(data.1.as_str()),
                        "nfc,  {} - {}",
                        $t,
                        data.0
                    );
                    assert_eq!(
                        nfkc.normalize(data.1.as_str()),
                        icu_nfkc.normalize(data.1.as_str()),
                        "nfkc, {} - {}",
                        $t,
                        data.0
                    );
                }
            )+
        };
    }

    test!((my, "my"));
}
