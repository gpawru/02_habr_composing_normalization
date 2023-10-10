#[macro_export]
/// отсортировать кодпоинты буфера по CCC, дописать их в строку результата, затем очистить буфер
macro_rules! flush {
    ($result: expr, $buffer: expr) => {
        if !$buffer.is_empty() {
            if $buffer.len() > 1 {
                $buffer.sort_by_key(|c| c.ccc);
            }

            for codepoint in $buffer.iter() {
                write!($result, codepoint.code);
            }

            $buffer.clear();
        }
    };
}

#[macro_export]
/// записать кодпоинт
macro_rules! write {
    ($result: expr, $($code: expr),+) => {{
        $(
            $result.push(unsafe { char::from_u32_unchecked($code) });
        )+
    }};
}

#[macro_export]
/// записать UTF-8 блок
macro_rules! write_str {
    ($result: expr, $block: expr) => {
        $result.push_str(unsafe { core::str::from_utf8_unchecked($block) })
    };
}

// в expansions кодпоинт и его CCC записаны в 32-битном формате - старший байт - CCC, остальные - код

#[macro_export]
/// прочитать CCC из старшего байта u32
macro_rules! c32_ccc {
    ($entry: expr) => {
        unsafe { *(($entry as *const u32 as *const u8).add(3)) }
    };
}

#[macro_export]
/// прочитать кодпоинт из u32, где старший байт - CCC
macro_rules! c32_code {
    ($entry: expr) => {
        $entry & 0x3FFFF
    };
}

#[macro_export]
/// разбираем u64 на составляющие: o!(исходный u64, тип результата <T>, (опционально: смещение в <T>))
macro_rules! o {
    ($value: expr, $t: ty) => {
        unsafe { *(&$value as *const u64 as *const $t) }
    };
    ($value: expr, $t: ty, $offset: expr) => {
        unsafe { *((&$value as *const u64 as *const $t).add($offset)) }
    };
}
