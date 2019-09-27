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

pub fn fill<T: Copy>(dst: &mut [T], value: T) {
    for x in dst.into_iter() {
        *x = value;
    }
}

pub fn bytes_from_ref<'a, T>(r: &'a T) -> &'a [u8] {
    unsafe { core::slice::from_raw_parts(r as *const T as *const u8, core::mem::size_of::<T>()) }
}
pub fn mut_bytes_from_ref<'a, T>(r: &'a mut T) -> &'a mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(r as *mut T as *mut u8, core::mem::size_of::<T>()) }
}
