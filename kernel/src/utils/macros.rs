// get lower 8bits
macro_rules! trunc8 {
    ($x:expr) => {
        ($x & 0xFF) as u8
    };
}

// similar to C99's designated initializer
macro_rules! asigned_array {
    ($def:expr; $len:expr; $([$idx:expr] = $val:expr),*) => {{
        let mut tmp = [$def; $len];
        $(tmp[$idx] = $val;)*
        tmp
    }};
}
