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
/// разбираем u64 на составляющие: o!(исходный u64, тип результата <T>, (опционально: смещение в <T>))
macro_rules! o {
    ($value: expr, $t: ty) => {
        unsafe { *(&$value as *const u64 as *const $t) }
    };
    ($value: expr, $t: ty, $offset: expr) => {
        unsafe { *((&$value as *const u64 as *const $t).add($offset)) }
    };
}
