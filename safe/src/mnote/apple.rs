use core::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong};
use core::mem::size_of;
use core::ptr;

use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifEntry, ExifFormat, ExifLog, ExifLogCode, ExifMem, ExifMnoteData,
    ExifMnoteDataMethods, EXIF_BYTE_ORDER_INTEL, EXIF_BYTE_ORDER_MOTOROLA, EXIF_FORMAT_ASCII,
    EXIF_FORMAT_LONG, EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SHORT, EXIF_FORMAT_SLONG,
    EXIF_FORMAT_SRATIONAL, EXIF_FORMAT_SSHORT,
};
use crate::i18n::{empty_message, message};
use crate::mnote::base::{
    check_overflow, generic_mnote_value, invalid_components_message, invalid_format_message,
    tag_description_from_table, tag_name_from_table, tag_title_from_table, write_slice_to_buffer,
    write_str_to_buffer, TagInfo,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::primitives::utils::{
    exif_array_set_byte_order, exif_get_long, exif_get_short, exif_get_slong,
};
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl};

const APPLE_HEADER: &[u8] = b"Apple iOS";
const DOMAIN: &[u8] = b"ExifMnoteDataApple\0";
const DOMAIN_TAG: &[u8] = b"ExifMnoteApplet\0";
const MSG_SHORT: &[u8] = b"Short MakerNote\0";
const MSG_UNRECOGNIZED: &[u8] = b"Unrecognized byte order\0";
const MSG_OVERFLOW: &[u8] = b"Tag size overflow detected (%u vs size %u)\0";
const MSG_COMPONENT_OVERFLOW: &[u8] = b"Tag size overflow detected (components %lu vs size %u)\0";
const MSG_NO_MEMORY: &[u8] = b"Could not allocate %u byte(s)\0";

const TAGS: [TagInfo; 7] = [
    TagInfo {
        tag: 0x000a,
        name: Some(message(b"HDR\0")),
        title: Some(message(b"HDR Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0003,
        name: Some(message(b"RUNTIME\0")),
        title: Some(message(b"Runtime\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0009,
        name: Some(message(b"ACCELERATION_VECTOR\0")),
        title: Some(message(b"Acceleration Vector\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000a,
        name: Some(message(b"HDR\0")),
        title: Some(message(b"HDR\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000b,
        name: Some(message(b"BURST_UUID\0")),
        title: Some(message(b"Burst UUID\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0011,
        name: Some(message(b"MEDIA_GROUP_UUID\0")),
        title: Some(message(b"Media Group UUID\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0015,
        name: Some(message(b"IMAGE_UNIQUE_ID\0")),
        title: Some(message(b"Image Unique ID\0")),
        description: Some(empty_message()),
    },
];

#[repr(C)]
struct MnoteAppleEntry {
    tag: c_int,
    format: ExifFormat,
    components: c_ulong,
    data: *mut c_uchar,
    size: c_uint,
    order: ExifByteOrder,
}

#[repr(C)]
struct ExifMnoteDataApple {
    parent: ExifMnoteData,
    order: ExifByteOrder,
    offset: c_uint,
    entries: *mut MnoteAppleEntry,
    count: c_uint,
}

unsafe extern "C" {
    fn exif_log(
        log: *mut ExifLog,
        code: ExifLogCode,
        domain: *const c_char,
        format: *const c_char,
        ...
    );
}

#[inline]
unsafe fn apple_note(note: *mut ExifMnoteData) -> *mut ExifMnoteDataApple {
    note.cast()
}

unsafe fn log_message(
    note: *mut ExifMnoteDataApple,
    code: ExifLogCode,
    domain: &[u8],
    format: &[u8],
) {
    unsafe {
        exif_log(
            (*note).parent.log,
            code,
            domain.as_ptr().cast(),
            format.as_ptr().cast(),
        )
    };
}

unsafe fn log_no_memory(note: *mut ExifMnoteDataApple, size: usize) {
    unsafe {
        exif_log(
            (*note).parent.log,
            2,
            DOMAIN.as_ptr().cast(),
            MSG_NO_MEMORY.as_ptr().cast(),
            size as c_uint,
        );
    }
}

unsafe extern "C" fn exif_mnote_data_apple_free(note: *mut ExifMnoteData) {
    let note = unsafe { apple_note(note) };
    if note.is_null() {
        return;
    }

    if !unsafe { (*note).entries }.is_null() {
        for index in 0..unsafe { (*note).count as usize } {
            let entry = unsafe { (*note).entries.add(index) };
            if !unsafe { (*entry).data }.is_null() {
                unsafe {
                    exif_mem_free_impl((*note).parent.mem, (*entry).data.cast());
                    (*entry).data = ptr::null_mut();
                }
            }
        }
        unsafe {
            exif_mem_free_impl((*note).parent.mem, (*note).entries.cast());
            (*note).entries = ptr::null_mut();
            (*note).count = 0;
        }
    }
}

unsafe extern "C" fn exif_mnote_data_apple_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    buffer_size: c_uint,
) {
    let note = unsafe { apple_note(note) };
    let buffer_size = buffer_size as usize;

    if note.is_null() || buffer.is_null() || buffer_size < 22 {
        if !note.is_null() {
            unsafe { log_message(note, 3, DOMAIN, MSG_SHORT) };
        }
        return;
    }

    let mut offset = unsafe { (*note).offset as usize } + 6;
    if check_overflow(offset, buffer_size, 16) {
        unsafe { log_message(note, 3, DOMAIN, MSG_SHORT) };
        return;
    }

    let marker = unsafe { std::slice::from_raw_parts(buffer.add(offset + 12), 2) };
    unsafe {
        (*note).order = if marker == b"MM" {
            EXIF_BYTE_ORDER_MOTOROLA
        } else if marker == b"II" {
            EXIF_BYTE_ORDER_INTEL
        } else {
            log_message(note, 3, DOMAIN, MSG_UNRECOGNIZED);
            return;
        };
    }

    let tag_count = unsafe { exif_get_short(buffer.add(offset + 14), (*note).order) as usize };
    let min_size = unsafe { (*note).offset as usize }
        .saturating_add(6)
        .saturating_add(16)
        .saturating_add(tag_count.saturating_mul(12))
        .saturating_add(4);
    if buffer_size < min_size {
        unsafe { log_message(note, 3, DOMAIN, MSG_SHORT) };
        return;
    }

    offset += 16;

    unsafe { exif_mnote_data_apple_free(ptr::addr_of_mut!((*note).parent)) };

    let entry_size = size_of::<MnoteAppleEntry>().saturating_mul(tag_count);
    let entries = unsafe { exif_mem_alloc_impl((*note).parent.mem, entry_size as u32) }
        .cast::<MnoteAppleEntry>();
    if entries.is_null() {
        unsafe { log_no_memory(note, entry_size) };
        return;
    }

    unsafe { (*note).entries = entries };

    for index in 0..tag_count {
        if offset.saturating_add(12) > buffer_size {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN_TAG.as_ptr().cast(),
                    MSG_OVERFLOW.as_ptr().cast(),
                    offset as c_uint + 12,
                    buffer_size as c_uint,
                );
            }
            break;
        }

        let entry = unsafe { (*note).entries.add(index) };
        unsafe {
            (*entry).tag = exif_get_short(buffer.add(offset), (*note).order) as c_int;
            (*entry).format = exif_get_short(buffer.add(offset + 2), (*note).order) as ExifFormat;
            (*entry).components = exif_get_long(buffer.add(offset + 4), (*note).order) as c_ulong;
            (*entry).order = (*note).order;
        }

        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if unsafe { (*entry).components } != 0
            && (buffer_size as u128 / unsafe { (*entry).components as u128 }) < format_size as u128
        {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN_TAG.as_ptr().cast(),
                    MSG_COMPONENT_OVERFLOW.as_ptr().cast(),
                    (*entry).components,
                    buffer_size as c_uint,
                );
            }
            break;
        }

        let data_size = match format_size.checked_mul(unsafe { (*entry).components as usize }) {
            Some(size) => size,
            None => break,
        };
        if data_size > 65_536 || data_size > buffer_size {
            break;
        }

        let data_offset = if data_size > 4 {
            unsafe {
                (*note).offset as usize
                    + exif_get_long(buffer.add(offset + 8), (*note).order) as usize
            }
        } else {
            offset + 8
        };

        if data_offset > buffer_size {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN_TAG.as_ptr().cast(),
                    MSG_OVERFLOW.as_ptr().cast(),
                    data_offset as c_uint,
                    buffer_size as c_uint,
                );
            }
            offset += 12;
            continue;
        }

        offset += 12;

        let data =
            unsafe { exif_mem_alloc_impl((*note).parent.mem, data_size as u32) }.cast::<c_uchar>();
        if data.is_null() {
            unsafe { log_no_memory(note, data_size) };
            continue;
        }
        unsafe { (*entry).data = data };

        if data_offset.saturating_add(data_size) > buffer_size {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN_TAG.as_ptr().cast(),
                    MSG_OVERFLOW.as_ptr().cast(),
                    (data_offset + data_size) as c_uint,
                    buffer_size as c_uint,
                );
            }
            continue;
        }

        unsafe {
            ptr::copy_nonoverlapping(buffer.add(data_offset), (*entry).data, data_size);
            (*entry).size = data_size as c_uint;
        }
    }

    unsafe { (*note).count = tag_count as c_uint };
}

unsafe extern "C" fn exif_mnote_data_apple_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    let note = unsafe { apple_note(note) };
    if !note.is_null() {
        unsafe { (*note).offset = offset };
    }
}

unsafe extern "C" fn exif_mnote_data_apple_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    let note = unsafe { apple_note(note) };
    if note.is_null() || unsafe { (*note).order } == order {
        return;
    }

    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { (*note).entries.add(index) };
        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if unsafe { (*entry).components } != 0
            && unsafe { (*entry).size as usize / (*entry).components as usize } < format_size
        {
            continue;
        }

        unsafe {
            exif_array_set_byte_order(
                (*entry).format,
                (*entry).data,
                (*entry).components as c_uint,
                (*entry).order,
                order,
            );
            (*entry).order = order;
        }
    }

    unsafe { (*note).order = order };
}

unsafe extern "C" fn exif_mnote_data_apple_count(note: *mut ExifMnoteData) -> c_uint {
    let note = unsafe { apple_note(note) };
    if note.is_null() {
        0
    } else {
        unsafe { (*note).count }
    }
}

unsafe extern "C" fn exif_mnote_data_apple_get_id(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> c_uint {
    let note = unsafe { apple_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return 0;
    }

    unsafe { (*(*note).entries.add(index as usize)).tag as c_uint }
}

unsafe extern "C" fn exif_mnote_data_apple_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { apple_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null();
    }

    tag_name_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

unsafe extern "C" fn exif_mnote_data_apple_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { apple_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null();
    }

    tag_title_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

unsafe extern "C" fn exif_mnote_data_apple_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { apple_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null();
    }

    tag_description_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

fn copy_ascii_value(
    data: *const c_uchar,
    size: usize,
    buffer: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    if data.is_null() {
        return ptr::null_mut();
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, size) };
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    unsafe { write_slice_to_buffer(buffer, maxlen, &bytes[..end]) }
}

fn mnote_apple_entry_get_value_impl(
    entry: *const MnoteAppleEntry,
    buffer: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    if entry.is_null() {
        return ptr::null_mut();
    }

    let entry = unsafe { &*entry };
    let size = entry.size as usize;

    match entry.tag {
        0x000a => {
            if size < 4 {
                return ptr::null_mut();
            }
            if entry.format != EXIF_FORMAT_SLONG {
                let message = invalid_format_message(entry.format, &[EXIF_FORMAT_SLONG]);
                return unsafe { write_str_to_buffer(buffer, maxlen, &message) };
            }
            if entry.components != 1 {
                let message = invalid_components_message(entry.components as u64, &[1]);
                return unsafe { write_str_to_buffer(buffer, maxlen, &message) };
            }
            let value = unsafe { exif_get_slong(entry.data, entry.order) };
            unsafe { write_str_to_buffer(buffer, maxlen, &value.to_string()) }
        }
        0x0015 | 0x000b | 0x0011 => {
            if entry.format != EXIF_FORMAT_ASCII {
                let message = invalid_format_message(entry.format, &[EXIF_FORMAT_ASCII]);
                return unsafe { write_str_to_buffer(buffer, maxlen, &message) };
            }
            copy_ascii_value(entry.data, size, buffer, maxlen)
        }
        _ => match entry.format {
            EXIF_FORMAT_ASCII => copy_ascii_value(entry.data, size, buffer, maxlen),
            EXIF_FORMAT_SHORT | EXIF_FORMAT_SSHORT | EXIF_FORMAT_LONG | EXIF_FORMAT_SLONG => unsafe {
                generic_mnote_value(
                    entry.format,
                    entry.components as u64,
                    entry.data,
                    size,
                    entry.order,
                    buffer,
                    maxlen,
                )
            },
            EXIF_FORMAT_RATIONAL => {
                if size < exif_format_get_size_impl(EXIF_FORMAT_RATIONAL) as usize
                    || entry.components < 1
                {
                    return ptr::null_mut();
                }
                let value =
                    unsafe { crate::primitives::utils::exif_get_rational(entry.data, entry.order) };
                if value.denominator == 0 {
                    return unsafe { write_str_to_buffer(buffer, maxlen, "") };
                }
                unsafe {
                    write_str_to_buffer(
                        buffer,
                        maxlen,
                        &format!("{:.4}", value.numerator as f64 / value.denominator as f64),
                    )
                }
            }
            EXIF_FORMAT_SRATIONAL => {
                if size < exif_format_get_size_impl(EXIF_FORMAT_SRATIONAL) as usize
                    || entry.components < 1
                {
                    return ptr::null_mut();
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_srational(entry.data, entry.order)
                };
                if value.denominator == 0 {
                    return unsafe { write_str_to_buffer(buffer, maxlen, "") };
                }
                unsafe {
                    write_str_to_buffer(
                        buffer,
                        maxlen,
                        &format!("{:.4}", value.numerator as f64 / value.denominator as f64),
                    )
                }
            }
            _ => unsafe {
                write_str_to_buffer(
                    buffer,
                    maxlen,
                    &format!("{} bytes unknown data", entry.size),
                )
            },
        },
    }
}

unsafe extern "C" fn exif_mnote_data_apple_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    buffer: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    let note = unsafe { apple_note(note) };
    if note.is_null() || buffer.is_null() || unsafe { (*note).count <= index } {
        return ptr::null_mut();
    }

    mnote_apple_entry_get_value_impl(
        unsafe { (*note).entries.add(index as usize) },
        buffer,
        maxlen,
    )
}

pub(crate) unsafe fn identify_impl(_data: *const ExifData, entry: *const ExifEntry) -> c_int {
    if entry.is_null() || unsafe { (*entry).size as usize } < APPLE_HEADER.len() + 1 {
        return 0;
    }

    let data = unsafe { std::slice::from_raw_parts((*entry).data, APPLE_HEADER.len()) };
    (data == APPLE_HEADER) as c_int
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let note = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifMnoteDataApple>() as u32) }
        .cast::<ExifMnoteDataApple>();
    if note.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        crate::mnote::base::exif_mnote_data_construct(ptr::addr_of_mut!((*note).parent), mem)
    };
    unsafe {
        (*note).parent.methods = ExifMnoteDataMethods {
            free: Some(exif_mnote_data_apple_free),
            save: None,
            load: Some(exif_mnote_data_apple_load),
            set_offset: Some(exif_mnote_data_apple_set_offset),
            set_byte_order: Some(exif_mnote_data_apple_set_byte_order),
            count: Some(exif_mnote_data_apple_count),
            get_id: Some(exif_mnote_data_apple_get_id),
            get_name: Some(exif_mnote_data_apple_get_name),
            get_title: Some(exif_mnote_data_apple_get_title),
            get_description: Some(exif_mnote_data_apple_get_description),
            get_value: Some(exif_mnote_data_apple_get_value),
        };
    }

    unsafe { ptr::addr_of_mut!((*note).parent) }
}
