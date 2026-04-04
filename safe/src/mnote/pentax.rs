use core::ffi::{c_char, c_int};
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifData, ExifEntry, ExifMem, ExifMnoteData, MnotePentaxEntry, MnotePentaxTag,
};

unsafe extern "C" {
    fn safe_helper_exif_mnote_data_pentax_identify(
        data: *const ExifData,
        entry: *const ExifEntry,
    ) -> c_int;
    fn safe_helper_exif_mnote_data_pentax_new(mem: *mut ExifMem) -> *mut ExifMnoteData;
    fn safe_helper_mnote_pentax_entry_get_value(
        entry: *mut MnotePentaxEntry,
        value: *mut c_char,
        maxlen: core::ffi::c_uint,
    ) -> *mut c_char;
    fn safe_helper_mnote_pentax_tag_get_description(tag: MnotePentaxTag) -> *const c_char;
    fn safe_helper_mnote_pentax_tag_get_name(tag: MnotePentaxTag) -> *const c_char;
    fn safe_helper_mnote_pentax_tag_get_title(tag: MnotePentaxTag) -> *const c_char;
}

pub(crate) unsafe fn identify_impl(data: *const ExifData, entry: *const ExifEntry) -> c_int {
    unsafe { safe_helper_exif_mnote_data_pentax_identify(data, entry) }
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    unsafe { safe_helper_exif_mnote_data_pentax_new(mem) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_pentax_new(mem: *mut ExifMem) -> *mut ExifMnoteData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { new_impl(mem) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_entry_get_value(
    entry: *mut MnotePentaxEntry,
    value: *mut c_char,
    maxlen: core::ffi::c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        safe_helper_mnote_pentax_entry_get_value(entry, value, maxlen)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_description(
    tag: MnotePentaxTag,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        safe_helper_mnote_pentax_tag_get_description(tag)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_name(tag: MnotePentaxTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe { safe_helper_mnote_pentax_tag_get_name(tag) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_title(tag: MnotePentaxTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        safe_helper_mnote_pentax_tag_get_title(tag)
    })
}
