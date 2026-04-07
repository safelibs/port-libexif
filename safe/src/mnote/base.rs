use core::ffi::{c_char, c_int, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;
use std::fmt::Write;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifFormat, ExifLog, ExifMem, ExifMnoteData, ExifMnoteDataPriv,
    EXIF_FORMAT_ASCII, EXIF_FORMAT_LONG, EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SHORT,
    EXIF_FORMAT_SLONG, EXIF_FORMAT_SRATIONAL, EXIF_FORMAT_SSHORT,
};
use crate::i18n::{empty_message, gettext, Message};
use crate::primitives::format::{exif_format_get_name_impl, exif_format_get_size_impl};
use crate::primitives::utils::{
    exif_get_long, exif_get_rational, exif_get_short, exif_get_slong, exif_get_srational,
    exif_get_sshort,
};
use crate::runtime::log::{exif_log_ref_impl, exif_log_unref_impl};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_ref_impl, exif_mem_unref_impl,
};

#[repr(C)]
struct MnoteDataPrivate {
    ref_count: u32,
}

#[inline]
unsafe fn mnote_private(note: *mut ExifMnoteData) -> *mut MnoteDataPrivate {
    unsafe { (*note).priv_.cast::<MnoteDataPrivate>() }
}

pub(crate) unsafe fn exif_mnote_data_free_impl(note: *mut ExifMnoteData) {
    if note.is_null() {
        return;
    }

    let mem = unsafe { (*note).mem };
    if !unsafe { (*note).priv_ }.is_null() {
        unsafe {
            if let Some(free_fn) = (*note).methods.free {
                free_fn(note);
            }
            exif_mem_free_impl(mem, (*note).priv_.cast());
            (*note).priv_ = ptr::null_mut::<ExifMnoteDataPriv>();
        }
    }

    unsafe {
        exif_log_unref_impl((*note).log);
        exif_mem_free_impl(mem, note.cast());
        exif_mem_unref_impl(mem);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_construct(note: *mut ExifMnoteData, mem: *mut ExifMem) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || mem.is_null() || !(*note).priv_.is_null() {
            return;
        }

        let private = exif_mem_alloc_impl(mem, size_of::<MnoteDataPrivate>() as u32)
            .cast::<MnoteDataPrivate>();
        if private.is_null() {
            return;
        }

        (*private).ref_count = 1;
        (*note).priv_ = private.cast();
        (*note).mem = mem;
        exif_mem_ref_impl(mem);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_ref(note: *mut ExifMnoteData) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || (*note).priv_.is_null() {
            return;
        }

        let private = &mut *mnote_private(note);
        private.ref_count = private.ref_count.wrapping_add(1);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_unref(note: *mut ExifMnoteData) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || (*note).priv_.is_null() {
            return;
        }

        let private = &mut *mnote_private(note);
        if private.ref_count > 0 {
            private.ref_count -= 1;
        }
        if private.ref_count == 0 {
            exif_mnote_data_free_impl(note);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    size: c_uint,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(load_fn) = (*note).methods.load {
            load_fn(note, buffer, size);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(save_fn) = (*note).methods.save {
            save_fn(note, buffer, size);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_log(note: *mut ExifMnoteData, log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }

        exif_log_unref_impl((*note).log);
        (*note).log = log;
        exif_log_ref_impl(log);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(setter) = (*note).methods.set_byte_order {
            setter(note, order);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(setter) = (*note).methods.set_offset {
            setter(note, offset);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_count(note: *mut ExifMnoteData) -> c_uint {
    panic_boundary::call_or(0, || unsafe {
        if note.is_null() {
            return 0;
        }
        (*note).methods.count.map_or(0, |count_fn| count_fn(note))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_id(note: *mut ExifMnoteData, index: c_uint) -> c_uint {
    panic_boundary::call_or(0, || unsafe {
        if note.is_null() {
            return 0;
        }
        (*note)
            .methods
            .get_id
            .map_or(0, |get_id_fn| get_id_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_name
            .map_or(ptr::null(), |get_name_fn| get_name_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_title
            .map_or(ptr::null(), |get_title_fn| get_title_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_description
            .map_or(ptr::null(), |get_description_fn| {
                get_description_fn(note, index)
            })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        if note.is_null() {
            return ptr::null_mut();
        }
        (*note)
            .methods
            .get_value
            .map_or(ptr::null_mut(), |get_value_fn| {
                get_value_fn(note, index, value, maxlen)
            })
    })
}

#[derive(Clone, Copy)]
pub(crate) struct TagInfo {
    pub tag: c_int,
    pub name: Option<Message>,
    pub title: Option<Message>,
    pub description: Option<Message>,
}

pub(crate) fn find_tag_info(table: &[TagInfo], tag: c_int) -> Option<TagInfo> {
    table.iter().copied().find(|entry| entry.tag == tag)
}

pub(crate) fn tag_name_from_table(table: &[TagInfo], tag: c_int) -> *const c_char {
    find_tag_info(table, tag)
        .and_then(|entry| entry.name)
        .map_or(ptr::null(), Message::as_ptr)
}

pub(crate) fn tag_title_from_table(table: &[TagInfo], tag: c_int) -> *const c_char {
    find_tag_info(table, tag)
        .and_then(|entry| entry.title)
        .map_or(ptr::null(), gettext)
}

pub(crate) fn tag_description_from_table(table: &[TagInfo], tag: c_int) -> *const c_char {
    find_tag_info(table, tag).map_or(ptr::null(), |entry| {
        entry
            .description
            .map_or_else(|| gettext(empty_message()), gettext_or_empty)
    })
}

fn gettext_or_empty(message: Message) -> *const c_char {
    if message.is_empty() {
        gettext(empty_message())
    } else {
        gettext(message)
    }
}

pub(crate) fn check_overflow(offset: usize, datasize: usize, structsize: usize) -> bool {
    offset >= datasize || structsize > datasize || offset > datasize.saturating_sub(structsize)
}

pub(crate) unsafe fn zero_buffer(buffer: *mut c_char, maxlen: c_uint) -> bool {
    if buffer.is_null() || maxlen == 0 {
        return false;
    }

    unsafe { ptr::write_bytes(buffer.cast::<u8>(), 0, maxlen as usize) };
    true
}

pub(crate) unsafe fn write_slice_to_buffer(
    buffer: *mut c_char,
    maxlen: c_uint,
    bytes: &[u8],
) -> *mut c_char {
    if !unsafe { zero_buffer(buffer, maxlen) } {
        return ptr::null_mut();
    }

    let copy_len = bytes.len().min(maxlen.saturating_sub(1) as usize);
    if copy_len > 0 {
        unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.cast::<u8>(), copy_len) };
    }
    buffer
}

pub(crate) unsafe fn write_str_to_buffer(
    buffer: *mut c_char,
    maxlen: c_uint,
    value: &str,
) -> *mut c_char {
    unsafe { write_slice_to_buffer(buffer, maxlen, value.as_bytes()) }
}

pub(crate) fn invalid_format_message(actual: ExifFormat, expected: &[ExifFormat]) -> String {
    let actual_name = c_string_or_unknown(exif_format_get_name_impl(actual));
    let expected_names = expected
        .iter()
        .map(|format| c_string_or_unknown(exif_format_get_name_impl(*format)))
        .collect::<Vec<_>>();

    match expected_names.as_slice() {
        [expected_name] => {
            format!("Invalid format '{actual_name}', expected '{expected_name}'.")
        }
        [first, second] => {
            format!("Invalid format '{actual_name}', expected '{first}' or '{second}'.")
        }
        _ => format!("Invalid format '{actual_name}'."),
    }
}

pub(crate) fn invalid_components_message(components: u64, expected: &[u64]) -> String {
    match expected {
        [expected] => format!("Invalid number of components ({components}, expected {expected})."),
        [first, second] => {
            format!("Invalid number of components ({components}, expected {first} or {second}).")
        }
        _ => format!("Invalid number of components ({components})."),
    }
}

fn c_string_or_unknown(value: *const c_char) -> String {
    if value.is_null() {
        return "Unknown".to_owned();
    }

    unsafe { std::ffi::CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned()
}

pub(crate) unsafe fn generic_mnote_value(
    format: ExifFormat,
    components: u64,
    data: *const c_uchar,
    size: usize,
    order: ExifByteOrder,
    buffer: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    if data.is_null() && size != 0 {
        return ptr::null_mut();
    }
    if !unsafe { zero_buffer(buffer, maxlen) } {
        return ptr::null_mut();
    }

    let mut rendered = String::new();
    let mut data = data;
    let mut remaining = size;

    match format {
        EXIF_FORMAT_ASCII => {
            let bytes = if data.is_null() {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(data, size) }
            };
            let end = bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(bytes.len());
            return unsafe { write_slice_to_buffer(buffer, maxlen, &bytes[..end]) };
        }
        EXIF_FORMAT_SHORT => {
            for _ in 0..components {
                if remaining < 2 {
                    break;
                }
                let _ = write!(&mut rendered, "{} ", unsafe { exif_get_short(data, order) });
                data = unsafe { data.add(2) };
                remaining -= 2;
            }
        }
        EXIF_FORMAT_SSHORT => {
            for _ in 0..components {
                if remaining < 2 {
                    break;
                }
                let _ = write!(&mut rendered, "{} ", unsafe {
                    exif_get_sshort(data, order)
                });
                data = unsafe { data.add(2) };
                remaining -= 2;
            }
        }
        EXIF_FORMAT_LONG => {
            for _ in 0..components {
                if remaining < 4 {
                    break;
                }
                let _ = write!(&mut rendered, "{} ", unsafe { exif_get_long(data, order) });
                data = unsafe { data.add(4) };
                remaining -= 4;
            }
        }
        EXIF_FORMAT_SLONG => {
            for _ in 0..components {
                if remaining < 4 {
                    break;
                }
                let _ = write!(&mut rendered, "{} ", unsafe { exif_get_slong(data, order) });
                data = unsafe { data.add(4) };
                remaining -= 4;
            }
        }
        EXIF_FORMAT_RATIONAL => {
            if components > 0
                && remaining >= exif_format_get_size_impl(EXIF_FORMAT_RATIONAL) as usize
            {
                let value = unsafe { exif_get_rational(data, order) };
                if value.denominator == 0 {
                    rendered.push_str("Unknown");
                } else {
                    let _ = write!(
                        &mut rendered,
                        "{:.4}",
                        value.numerator as f64 / value.denominator as f64
                    );
                }
            }
        }
        EXIF_FORMAT_SRATIONAL => {
            if components > 0
                && remaining >= exif_format_get_size_impl(EXIF_FORMAT_SRATIONAL) as usize
            {
                let value = unsafe { exif_get_srational(data, order) };
                if value.denominator == 0 {
                    rendered.push_str("Unknown");
                } else {
                    let _ = write!(
                        &mut rendered,
                        "{:.4}",
                        value.numerator as f64 / value.denominator as f64
                    );
                }
            }
        }
        _ => {
            let _ = write!(&mut rendered, "{} bytes unknown data", size);
        }
    }

    unsafe { write_str_to_buffer(buffer, maxlen, &rendered) }
}
