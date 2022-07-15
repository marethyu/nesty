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
macro_rules! toggle_bit {
    ($n:expr, $pos:expr) => {
        $n ^= (1 << $pos);
    }
}

#[macro_export]
macro_rules! reverse_byte {
    ($n:expr) => {
        $n = ($n & 0b11110000) >> 4 | ($n & 0b00001111) << 4;
        $n = ($n & 0b11001100) >> 2 | ($n & 0b00110011) << 2;
        $n = ($n & 0b10101010) >> 1 | ($n & 0b01010101) << 1;
    }
}

#[macro_export]
macro_rules! mirror {
    ($base:expr, $addr:expr, $size:expr) => {
        (($addr - $base) & (($size as u16) - 1)) as usize
    }
}
