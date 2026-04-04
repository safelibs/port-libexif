use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifData, ExifDataOption, ExifEntry, ExifMem, ExifMnoteData, MnoteCanonEntry, MnoteCanonTag,
};

unsafe extern "C" {
    fn safe_helper_exif_mnote_data_canon_identify(
        data: *const ExifData,
        entry: *const ExifEntry,
    ) -> c_int;
    fn safe_helper_exif_mnote_data_canon_new(
        mem: *mut ExifMem,
        option: ExifDataOption,
    ) -> *mut ExifMnoteData;
    fn safe_helper_mnote_canon_entry_get_value(
        entry: *const MnoteCanonEntry,
        subtag: c_uint,
        value: *mut c_char,
        maxlen: c_uint,
    ) -> *mut c_char;
    fn safe_helper_mnote_canon_tag_get_description(tag: MnoteCanonTag) -> *const c_char;
    fn safe_helper_mnote_canon_tag_get_name(tag: MnoteCanonTag) -> *const c_char;
    fn safe_helper_mnote_canon_tag_get_title(tag: MnoteCanonTag) -> *const c_char;
}

pub(crate) unsafe fn identify_impl(data: *const ExifData, entry: *const ExifEntry) -> c_int {
    unsafe { safe_helper_exif_mnote_data_canon_identify(data, entry) }
}

pub(crate) unsafe fn new_impl(
    mem: *mut ExifMem,
    option: ExifDataOption,
) -> *mut ExifMnoteData {
    unsafe { safe_helper_exif_mnote_data_canon_new(mem, option) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_canon_new(
    mem: *mut ExifMem,
    option: ExifDataOption,
) -> *mut ExifMnoteData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { new_impl(mem, option) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_entry_get_value(
    entry: *const MnoteCanonEntry,
    subtag: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        safe_helper_mnote_canon_entry_get_value(entry, subtag, value, maxlen)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_description(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        safe_helper_mnote_canon_tag_get_description(tag)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_name(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe { safe_helper_mnote_canon_tag_get_name(tag) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_title(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe { safe_helper_mnote_canon_tag_get_title(tag) })
}
