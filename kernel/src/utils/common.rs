// get lower 8bits
macro_rules! trunc8 {
    ($x:expr) => {
        ($x & 0xFF) as u8
    };
}

// similar to C99's designated initializer
macro_rules! assigned_array {
    ($def:expr; $len:expr; $([$idx:expr] = $val:expr),*) => {{
        let mut tmp = [$def; $len];
        $(tmp[$idx] = $val;)*
        tmp
    }};
}

//------------------------------------------------------------------------------

pub fn fill(dst: &mut [u8], value: u8) {
    for x in dst.into_iter() {
        *x = value;
    }
}
