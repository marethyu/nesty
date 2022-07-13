#[macro_export]
macro_rules! test_bit {
    ($n:expr, $pos:expr) => {
        ($n & (1 << $pos)) != 0
    };
}

#[macro_export]
macro_rules! modify_bit {
    ($n:expr, $pos:expr, $is_set:expr) => {
        if $is_set {
            $n |= (1 << $pos);
        } else {
            $n &= !(1 << $pos);
        }
    }
}

#[macro_export]
macro_rules! mirror {
    ($base:expr, $addr:expr, $size:expr) => {
        (($addr - $base) & (($size as u16) - 1)) as usize
    }
}
