#[macro_export]
macro_rules! test_bit {
    ($n:expr, $pos:expr) => {
        ($n & (1 << $pos)) != 0
    };
}

#[macro_export]
macro_rules! modify_bit {
    ($n:expr, $pos:expr, $is_set:expr) => {
        $n = ($n & !(1 << $pos)) | (($is_set as u8) << $pos);
    }
}
