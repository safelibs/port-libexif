pub mod ffi;

use core::ffi::{c_char, c_uchar, c_uint, c_void};
use core::ptr;

use ffi::panic_boundary;
use ffi::types::*;

unsafe extern "C" {
    fn exif_log_new() -> *mut ExifLog;
}

#[used]
static FORCE_EXIF_LOG_SHIM: unsafe extern "C" fn() -> *mut ExifLog = exif_log_new;

const MOTOROLA_NAME: &[u8] = b"Motorola\0";
const INTEL_NAME: &[u8] = b"Intel\0";
const IFD_0_NAME: &[u8] = b"0\0";
const IFD_1_NAME: &[u8] = b"1\0";
const IFD_EXIF_NAME: &[u8] = b"EXIF\0";
const IFD_GPS_NAME: &[u8] = b"GPS\0";
const IFD_INTEROP_NAME: &[u8] = b"Interoperability\0";
const FORMAT_BYTE_NAME: &[u8] = b"Byte\0";
const FORMAT_ASCII_NAME: &[u8] = b"ASCII\0";
const FORMAT_SHORT_NAME: &[u8] = b"Short\0";
const FORMAT_LONG_NAME: &[u8] = b"Long\0";
const FORMAT_RATIONAL_NAME: &[u8] = b"Rational\0";
const FORMAT_SBYTE_NAME: &[u8] = b"SByte\0";
const FORMAT_UNDEFINED_NAME: &[u8] = b"Undefined\0";
const FORMAT_SSHORT_NAME: &[u8] = b"SShort\0";
const FORMAT_SLONG_NAME: &[u8] = b"SLong\0";
const FORMAT_SRATIONAL_NAME: &[u8] = b"SRational\0";
const FORMAT_FLOAT_NAME: &[u8] = b"Float\0";
const FORMAT_DOUBLE_NAME: &[u8] = b"Double\0";
const OPTION_IGNORE_UNKNOWN_TAGS_NAME: &[u8] = b"Ignore unknown tags\0";
const OPTION_FOLLOW_SPECIFICATION_NAME: &[u8] = b"Follow specification\0";
const OPTION_DONT_CHANGE_MAKER_NOTE_NAME: &[u8] = b"Do not change maker note\0";
const OPTION_IGNORE_UNKNOWN_TAGS_DESCRIPTION: &[u8] =
    b"Ignore unknown tags when loading EXIF data.\0";
const OPTION_FOLLOW_SPECIFICATION_DESCRIPTION: &[u8] =
    b"Add, correct and remove entries to get EXIF data that follows the specification.\0";
const OPTION_DONT_CHANGE_MAKER_NOTE_DESCRIPTION: &[u8] = b"When loading and resaving Exif data, save the maker note unmodified. Be aware that the maker note can get corrupted.\0";

fn c_str(bytes: &'static [u8]) -> *const c_char {
    bytes.as_ptr().cast()
}

fn byte_order_name(order: ExifByteOrder) -> *const c_char {
    match order {
        EXIF_BYTE_ORDER_MOTOROLA => c_str(MOTOROLA_NAME),
        EXIF_BYTE_ORDER_INTEL => c_str(INTEL_NAME),
        _ => ptr::null(),
    }
}

fn format_name(format: ExifFormat) -> *const c_char {
    match format {
        EXIF_FORMAT_BYTE => c_str(FORMAT_BYTE_NAME),
        EXIF_FORMAT_ASCII => c_str(FORMAT_ASCII_NAME),
        EXIF_FORMAT_SHORT => c_str(FORMAT_SHORT_NAME),
        EXIF_FORMAT_LONG => c_str(FORMAT_LONG_NAME),
        EXIF_FORMAT_RATIONAL => c_str(FORMAT_RATIONAL_NAME),
        EXIF_FORMAT_SBYTE => c_str(FORMAT_SBYTE_NAME),
        EXIF_FORMAT_UNDEFINED => c_str(FORMAT_UNDEFINED_NAME),
        EXIF_FORMAT_SSHORT => c_str(FORMAT_SSHORT_NAME),
        EXIF_FORMAT_SLONG => c_str(FORMAT_SLONG_NAME),
        EXIF_FORMAT_SRATIONAL => c_str(FORMAT_SRATIONAL_NAME),
        EXIF_FORMAT_FLOAT => c_str(FORMAT_FLOAT_NAME),
        EXIF_FORMAT_DOUBLE => c_str(FORMAT_DOUBLE_NAME),
        _ => ptr::null(),
    }
}

fn format_size(format: ExifFormat) -> c_uchar {
    match format {
        EXIF_FORMAT_BYTE | EXIF_FORMAT_ASCII | EXIF_FORMAT_SBYTE | EXIF_FORMAT_UNDEFINED => 1,
        EXIF_FORMAT_SHORT | EXIF_FORMAT_SSHORT => 2,
        EXIF_FORMAT_LONG | EXIF_FORMAT_SLONG | EXIF_FORMAT_FLOAT => 4,
        EXIF_FORMAT_RATIONAL | EXIF_FORMAT_SRATIONAL | EXIF_FORMAT_DOUBLE => 8,
        _ => 0,
    }
}

fn ifd_name(ifd: ExifIfd) -> *const c_char {
    match ifd {
        EXIF_IFD_0 => c_str(IFD_0_NAME),
        EXIF_IFD_1 => c_str(IFD_1_NAME),
        EXIF_IFD_EXIF => c_str(IFD_EXIF_NAME),
        EXIF_IFD_GPS => c_str(IFD_GPS_NAME),
        EXIF_IFD_INTEROPERABILITY => c_str(IFD_INTEROP_NAME),
        _ => ptr::null(),
    }
}

fn option_name(option: ExifDataOption) -> *const c_char {
    match option {
        EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS => c_str(OPTION_IGNORE_UNKNOWN_TAGS_NAME),
        EXIF_DATA_OPTION_FOLLOW_SPECIFICATION => c_str(OPTION_FOLLOW_SPECIFICATION_NAME),
        EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE => c_str(OPTION_DONT_CHANGE_MAKER_NOTE_NAME),
        _ => ptr::null(),
    }
}

fn option_description(option: ExifDataOption) -> *const c_char {
    match option {
        EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS => c_str(OPTION_IGNORE_UNKNOWN_TAGS_DESCRIPTION),
        EXIF_DATA_OPTION_FOLLOW_SPECIFICATION => c_str(OPTION_FOLLOW_SPECIFICATION_DESCRIPTION),
        EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE => c_str(OPTION_DONT_CHANGE_MAKER_NOTE_DESCRIPTION),
        _ => ptr::null(),
    }
}

fn is_intel(order: ExifByteOrder) -> bool {
    order == EXIF_BYTE_ORDER_INTEL
}

fn read_bytes<const N: usize>(buffer: *const c_uchar) -> [u8; N] {
    let mut out = [0u8; N];
    if !buffer.is_null() {
        unsafe {
            ptr::copy_nonoverlapping(buffer, out.as_mut_ptr(), N);
        }
    }
    out
}

fn write_bytes(buffer: *mut c_uchar, bytes: &[u8]) {
    if !buffer.is_null() {
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, bytes.len());
        }
    }
}

fn read_u16(buffer: *const c_uchar, order: ExifByteOrder) -> u16 {
    let bytes = read_bytes::<2>(buffer);
    if is_intel(order) {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    }
}

fn read_u32(buffer: *const c_uchar, order: ExifByteOrder) -> u32 {
    let bytes = read_bytes::<4>(buffer);
    if is_intel(order) {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    }
}

fn write_u16(buffer: *mut c_uchar, order: ExifByteOrder, value: u16) {
    let bytes = if is_intel(order) {
        value.to_le_bytes()
    } else {
        value.to_be_bytes()
    };
    write_bytes(buffer, &bytes);
}

fn write_u32(buffer: *mut c_uchar, order: ExifByteOrder, value: u32) {
    let bytes = if is_intel(order) {
        value.to_le_bytes()
    } else {
        value.to_be_bytes()
    };
    write_bytes(buffer, &bytes);
}

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
pub unsafe extern "C" fn exif_byte_order_get_name(order: ExifByteOrder) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || byte_order_name(order))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_format_get_name(format: ExifFormat) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || format_name(format))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_format_get_size(format: ExifFormat) -> c_uchar {
    panic_boundary::call_or(0, || format_size(format))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_ifd_get_name(ifd: ExifIfd) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || ifd_name(ifd))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_option_get_name(option: ExifDataOption) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || option_name(option))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_option_get_description(option: ExifDataOption) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || option_description(option))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_short(buffer: *const c_uchar, order: ExifByteOrder) -> ExifShort {
    panic_boundary::call_or(0, || read_u16(buffer, order))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_sshort(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifSShort {
    panic_boundary::call_or(0, || read_u16(buffer, order) as ExifSShort)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_long(buffer: *const c_uchar, order: ExifByteOrder) -> ExifLong {
    panic_boundary::call_or(0, || read_u32(buffer, order))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_slong(buffer: *const c_uchar, order: ExifByteOrder) -> ExifSLong {
    panic_boundary::call_or(0, || read_u32(buffer, order) as ExifSLong)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_rational(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifRational {
    panic_boundary::call_or(ExifRational::default(), || ExifRational {
        numerator: read_u32(buffer, order),
        denominator: read_u32(unsafe { buffer.add(4) }, order),
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_srational(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifSRational {
    panic_boundary::call_or(ExifSRational::default(), || ExifSRational {
        numerator: read_u32(buffer, order) as ExifSLong,
        denominator: read_u32(unsafe { buffer.add(4) }, order) as ExifSLong,
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_short(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifShort,
) {
    panic_boundary::call_void(|| write_u16(buffer, order, value));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_sshort(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSShort,
) {
    panic_boundary::call_void(|| write_u16(buffer, order, value as u16));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_long(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifLong,
) {
    panic_boundary::call_void(|| write_u32(buffer, order, value));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_slong(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSLong,
) {
    panic_boundary::call_void(|| write_u32(buffer, order, value as u32));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_rational(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifRational,
) {
    panic_boundary::call_void(|| {
        write_u32(buffer, order, value.numerator);
        write_u32(unsafe { buffer.add(4) }, order, value.denominator);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_srational(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSRational,
) {
    panic_boundary::call_void(|| {
        write_u32(buffer, order, value.numerator as u32);
        write_u32(unsafe { buffer.add(4) }, order, value.denominator as u32);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_array_set_byte_order(
    format: ExifFormat,
    buffer: *mut c_uchar,
    count: c_uint,
    original_order: ExifByteOrder,
    new_order: ExifByteOrder,
) {
    panic_boundary::call_void(|| {
        let _ = (format, buffer, count, original_order, new_order);
    });
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
    fn exif_mem_free(mem: *mut ExifMem, ptr_: *mut c_void);
    fn exif_mem_ref(mem: *mut ExifMem);
    fn exif_mem_unref(mem: *mut ExifMem);
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
    fn exif_mem_alloc(mem: *mut ExifMem, size: ExifLong) -> *mut c_void = ptr::null_mut();
    fn exif_mem_new(alloc: ExifMemAllocFunc, realloc: ExifMemReallocFunc, free: ExifMemFreeFunc) -> *mut ExifMem = ptr::null_mut();
    fn exif_mem_new_default() -> *mut ExifMem = ptr::null_mut();
    fn exif_mem_realloc(mem: *mut ExifMem, ptr_: *mut c_void, size: ExifLong) -> *mut c_void = ptr::null_mut();
    fn exif_mnote_data_canon_new(mem: *mut ExifMem, option: ExifDataOption) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_mnote_data_count(note: *mut ExifMnoteData) -> c_uint = 0;
    fn exif_mnote_data_get_description(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_get_id(note: *mut ExifMnoteData, index: c_uint) -> c_uint = 0;
    fn exif_mnote_data_get_name(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_get_title(note: *mut ExifMnoteData, index: c_uint) -> *const c_char = ptr::null();
    fn exif_mnote_data_olympus_new(mem: *mut ExifMem) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_mnote_data_pentax_new(mem: *mut ExifMem) -> *mut ExifMnoteData = ptr::null_mut();
    fn exif_tag_from_name(name: *const c_char) -> ExifTag = 0;
    fn exif_tag_get_description(tag: ExifTag) -> *const c_char = ptr::null();
    fn exif_tag_get_description_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char = ptr::null();
    fn exif_tag_get_name(tag: ExifTag) -> *const c_char = ptr::null();
    fn exif_tag_get_name_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char = ptr::null();
    fn exif_tag_get_support_level_in_ifd(tag: ExifTag, ifd: ExifIfd, data_type: ExifDataType) -> ExifSupportLevel = EXIF_SUPPORT_LEVEL_UNKNOWN;
    fn exif_tag_get_title(tag: ExifTag) -> *const c_char = ptr::null();
    fn exif_tag_get_title_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char = ptr::null();
    fn exif_tag_table_count() -> c_uint = 0;
    fn exif_tag_table_get_name(index: c_uint) -> *const c_char = ptr::null();
    fn exif_tag_table_get_tag(index: c_uint) -> ExifTag = 0;
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
