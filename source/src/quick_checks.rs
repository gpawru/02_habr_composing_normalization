const QC_TABLE_SIZE: usize = 0x2FA1D + 1;

lazy_static! {
    pub static ref QC_NFD: Vec<char> = nfd_qc();
    pub static ref QC_NFC: Vec<char> = nfc_qc();
    pub static ref QC_NFKD: Vec<char> = nfkd_qc();
    pub static ref QC_NFKC: Vec<char> = nfkc_qc();
}

const DATA: &str = include_str!("./../data/ucd/15.1.0/DerivedNormalizationProps.txt");

/// быстрые проверки NFD, Y/N
fn nfd_qc() -> Vec<char>
{
    let mut table = vec!['Y'; QC_TABLE_SIZE];

    fill_table("NFD_Quick_Check=No", 'N', &mut table);

    table
}

/// быстрые проверки NFC Y/N/M
fn nfc_qc() -> Vec<char>
{
    let mut table = vec!['Y'; QC_TABLE_SIZE];

    fill_table("NFC_Quick_Check=No", 'N', &mut table);
    fill_table("NFC_Quick_Check=Maybe", 'M', &mut table);

    table
}

/// быстрые проверки NFKD Y/N
fn nfkd_qc() -> Vec<char>
{
    let mut table = vec!['Y'; QC_TABLE_SIZE];
    fill_table("NFKD_Quick_Check=No", 'N', &mut table);

    table
}

/// быстрые проверки NFKC Y/N/M
fn nfkc_qc() -> Vec<char>
{
    let mut table = vec!['Y'; QC_TABLE_SIZE];

    fill_table("NFKC_Quick_Check=No", 'N', &mut table);
    fill_table("NFKC_Quick_Check=Maybe", 'M', &mut table);
    table
}

fn fill_table(block: &str, value: char, table: &mut Vec<char>)
{
    let lines = DATA
        .lines()
        .skip_while(|&line| !line.contains(block))
        .skip(2);

    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            break;
        }

        let (codepoints, _) = line.split_once(';').unwrap();

        if codepoints.contains("..") {
            let (from, to) = codepoints.trim().split_once("..").unwrap();
            let from = usize::from_str_radix(from, 16).unwrap();
            let to = usize::from_str_radix(to, 16).unwrap();

            for code in from ..= to {
                table[code] = value;
            }
        } else {
            let code = usize::from_str_radix(codepoints.trim(), 16).unwrap();

            table[code] = value;
        }
    }
}
