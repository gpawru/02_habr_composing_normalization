use std::fmt::UpperHex;

/// представить массив чисел в текстовом виде
pub fn format_num_vec<T: UpperHex + Into<u64> + Copy>(input: &[T], boundary: usize) -> String
{
    let mut output = String::new();

    let mut cur_len = boundary;

    for &e in input {
        let e_str = match e.into() == 0 {
            true => "0, ".to_owned(),
            false => format!("0x{:X}, ", e),
        };

        match cur_len + e_str.len() > boundary {
            true => {
                output.push_str("\n    ");
                cur_len = e_str.len();
            }
            false => {
                cur_len += e_str.len();
            }
        };

        output.push_str(e_str.as_str());
    }
    output.push('\n');

    output
}
