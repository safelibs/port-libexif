pub mod ffi;

mod i18n;
mod object;
mod parser;
mod primitives;
mod runtime;
mod tables;

use core::ffi::{c_char, c_uchar, c_uint};
use core::ptr;

use ffi::panic_boundary;
use ffi::types::*;

unsafe extern "C" {
    fn exif_log_shim_anchor();
}

#[used]
static FORCE_EXIF_LOG_SHIM: unsafe extern "C" fn() = exif_log_shim_anchor;

fn clear_c_buffer(buffer: *mut c_char, maxlen: c_uint) {
    if !buffer.is_null() && maxlen > 0 {
        unsafe {
            *buffer = 0;
        }
    }
}

fn store_mut_data(buffer: *mut *mut c_uchar, size: *mut c_uint) {
    unsafe {
        if !buffer.is_null() {
            *buffer = ptr::null_mut();
        }
        if !size.is_null() {
            *size = 0;
        }
    }
}

macro_rules! stub_void {
    ($(fn $name:ident($($arg:ident : $ty:ty),*);)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $ty),*) {
                panic_boundary::call_void(|| {
                    let _ = ($($arg),*);
                });
            }
        )+
    };
}

macro_rules! stub_return {
    ($(fn $name:ident($($arg:ident : $ty:ty),*) -> $ret:ty = $default:expr;)+) => {
        $(
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn $name($($arg: $ty),*) -> $ret {
                panic_boundary::call_or($default, || {
                    let _ = ($($arg),*);
                    $default
                })
            }
        )+
    };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| {
        let _ = note;
        store_mut_data(buffer, size);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || {
        let _ = (note, index);
        clear_c_buffer(value, maxlen);
        value
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_entry_get_value(
    entry: *const MnoteCanonEntry,
    subtag: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || {
        let _ = (entry, subtag);
        clear_c_buffer(value, maxlen);
        value
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_olympus_entry_get_value(
    entry: *mut MnoteOlympusEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || {
        let _ = entry;
        clear_c_buffer(value, maxlen);
        value
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_entry_get_value(
    entry: *mut MnotePentaxEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || {
        let _ = entry;
        clear_c_buffer(value, maxlen);
        value
    })
}

stub_void! {
    fn exif_mnote_data_construct(note: *mut ExifMnoteData, mem: *mut ExifMem);
    fn exif_mnote_data_load(note: *mut ExifMnoteData, buffer: *const c_uchar, size: c_uint);
    fn exif_mnote_data_log(note: *mut ExifMnoteData, log: *mut ExifLog);
    fn exif_mnote_data_ref(note: *mut ExifMnoteData);
    fn exif_mnote_data_set_byte_order(note: *mut ExifMnoteData, order: ExifByteOrder);
    fn exif_mnote_data_set_offset(note: *mut ExifMnoteData, offset: c_uint);
    fn exif_mnote_data_unref(note: *mut ExifMnoteData);
}

stub_return! {
    fn exif_mnote_data_canon_new(mem: *mut ExifMem, option: ExifDataOption) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_mnote_data_count(note: *mut ExifMnoteData) -> c_uint = 0;
    fn exif_mnote_data_get_description(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_get_id(note: *mut ExifMnoteData, index: c_uint) -> c_uint = 0;
    fn exif_mnote_data_get_name(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_get_title(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_olympus_new(mem: *mut ExifMem) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_mnote_data_pentax_new(mem: *mut ExifMem) -> *mut ExifMnoteData = ptr::null_mut();
    fn mnote_canon_tag_get_description(tag: MnoteCanonTag) -> *const c_char = ptr::null();
    fn mnote_canon_tag_get_name(tag: MnoteCanonTag) -> *const c_char = ptr::null();
    fn mnote_canon_tag_get_title(tag: MnoteCanonTag) -> *const c_char = ptr::null();
    fn mnote_olympus_tag_get_description(tag: MnoteOlympusTag) -> *const c_char = ptr::null();
    fn mnote_olympus_tag_get_name(tag: MnoteOlympusTag) -> *const c_char = ptr::null();
    fn mnote_olympus_tag_get_title(tag: MnoteOlympusTag) -> *const c_char = ptr::null();
    fn mnote_pentax_tag_get_description(tag: MnotePentaxTag) -> *const c_char = ptr::null();
    fn mnote_pentax_tag_get_name(tag: MnotePentaxTag) -> *const c_char = ptr::null();
    fn mnote_pentax_tag_get_title(tag: MnotePentaxTag) -> *const c_char = ptr::null();
}
