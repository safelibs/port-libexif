pub mod ffi;

mod i18n;
mod primitives;
mod runtime;
mod tables;

use core::ffi::{c_char, c_uchar, c_uint, c_void};
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

fn store_const_data(buffer: *mut *const c_uchar, size: *mut c_uint) {
    unsafe {
        if !buffer.is_null() {
            *buffer = ptr::null();
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
pub unsafe extern "C" fn exif_data_save_data(
    data: *mut ExifData,
    buffer: *mut *mut c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| {
        let _ = data;
        store_mut_data(buffer, size);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_get_buf(
    loader: *mut ExifLoader,
    buffer: *mut *const c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| {
        let _ = loader;
        store_const_data(buffer, size);
    });
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
pub unsafe extern "C" fn exif_entry_get_value(
    entry: *mut ExifEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        let _ = entry;
        clear_c_buffer(value, maxlen);
        value.cast_const()
    })
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
    fn exif_content_add_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_content_dump(content: *mut ExifContent, indent: c_uint);
    fn exif_content_fix(content: *mut ExifContent);
    fn exif_content_foreach_entry(content: *mut ExifContent, func: ExifContentForeachEntryFunc, user_data: *mut c_void);
    fn exif_content_free(content: *mut ExifContent);
    fn exif_content_log(content: *mut ExifContent, log: *mut ExifLog);
    fn exif_content_ref(content: *mut ExifContent);
    fn exif_content_remove_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_content_unref(content: *mut ExifContent);
    fn exif_data_dump(data: *mut ExifData);
    fn exif_data_fix(data: *mut ExifData);
    fn exif_data_foreach_content(data: *mut ExifData, func: ExifDataForeachContentFunc, user_data: *mut c_void);
    fn exif_data_free(data: *mut ExifData);
    fn exif_data_load_data(data: *mut ExifData, source: *const c_uchar, size: c_uint);
    fn exif_data_log(data: *mut ExifData, log: *mut ExifLog);
    fn exif_data_ref(data: *mut ExifData);
    fn exif_data_set_byte_order(data: *mut ExifData, order: ExifByteOrder);
    fn exif_data_set_data_type(data: *mut ExifData, data_type: ExifDataType);
    fn exif_data_set_option(data: *mut ExifData, option: ExifDataOption);
    fn exif_data_unref(data: *mut ExifData);
    fn exif_data_unset_option(data: *mut ExifData, option: ExifDataOption);
    fn exif_entry_dump(entry: *mut ExifEntry, indent: c_uint);
    fn exif_entry_fix(entry: *mut ExifEntry);
    fn exif_entry_free(entry: *mut ExifEntry);
    fn exif_entry_initialize(entry: *mut ExifEntry, tag: ExifTag);
    fn exif_entry_ref(entry: *mut ExifEntry);
    fn exif_entry_unref(entry: *mut ExifEntry);
    fn exif_loader_log(loader: *mut ExifLoader, log: *mut ExifLog);
    fn exif_loader_ref(loader: *mut ExifLoader);
    fn exif_loader_reset(loader: *mut ExifLoader);
    fn exif_loader_unref(loader: *mut ExifLoader);
    fn exif_loader_write_file(loader: *mut ExifLoader, path: *const c_char);
    fn exif_mnote_data_construct(note: *mut ExifMnoteData, mem: *mut ExifMem);
    fn exif_mnote_data_load(note: *mut ExifMnoteData, buffer: *const c_uchar, size: c_uint);
    fn exif_mnote_data_log(note: *mut ExifMnoteData, log: *mut ExifLog);
    fn exif_mnote_data_ref(note: *mut ExifMnoteData);
    fn exif_mnote_data_set_byte_order(note: *mut ExifMnoteData, order: ExifByteOrder);
    fn exif_mnote_data_set_offset(note: *mut ExifMnoteData, offset: c_uint);
    fn exif_mnote_data_unref(note: *mut ExifMnoteData);
}

stub_return! {
    fn exif_content_get_entry(content: *mut ExifContent, tag: ExifTag) -> *mut ExifEntry = ptr::null_mut();
    fn exif_content_get_ifd(content: *mut ExifContent) -> ExifIfd = EXIF_IFD_COUNT;
    fn exif_content_new() -> *mut ExifContent = ptr::null_mut();
    fn exif_content_new_mem(mem: *mut ExifMem) -> *mut ExifContent = ptr::null_mut();
    fn exif_data_get_byte_order(data: *mut ExifData) -> ExifByteOrder = EXIF_BYTE_ORDER_MOTOROLA;
    fn exif_data_get_data_type(data: *mut ExifData) -> ExifDataType = EXIF_DATA_TYPE_UNKNOWN;
    fn exif_data_get_log(data: *mut ExifData) -> *mut ExifLog = ptr::null_mut();
    fn exif_data_get_mnote_data(data: *mut ExifData) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_data_new() -> *mut ExifData = ptr::null_mut();
    fn exif_data_new_from_data(data: *const c_uchar, size: c_uint) -> *mut ExifData = ptr::null_mut();
    fn exif_data_new_from_file(path: *const c_char) -> *mut ExifData = ptr::null_mut();
    fn exif_data_new_mem(mem: *mut ExifMem) -> *mut ExifData = ptr::null_mut();
    fn exif_entry_new() -> *mut ExifEntry = ptr::null_mut();
    fn exif_entry_new_mem(mem: *mut ExifMem) -> *mut ExifEntry = ptr::null_mut();
    fn exif_loader_get_data(loader: *mut ExifLoader) -> *mut ExifData = ptr::null_mut();
    fn exif_loader_new() -> *mut ExifLoader = ptr::null_mut();
    fn exif_loader_new_mem(mem: *mut ExifMem) -> *mut ExifLoader = ptr::null_mut();
    fn exif_loader_write(loader: *mut ExifLoader, buffer: *mut c_uchar, size: c_uint) -> c_uchar = 0;
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
