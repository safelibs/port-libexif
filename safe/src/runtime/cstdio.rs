use core::ffi::{c_char, c_int};

use std::ffi::CString;

unsafe extern "C" {
    fn printf(format: *const c_char, ...) -> c_int;
}

pub(crate) fn print_line(line: &str) {
    let Ok(c_line) = CString::new(line) else {
        return;
    };

    unsafe {
        printf(b"%s\n\0".as_ptr().cast(), c_line.as_ptr());
    }
}
