use core::ffi::{c_char, c_int, c_uchar, c_uint, c_ulong};
use core::mem::size_of;
use core::ptr;

use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifEntry, ExifFormat, ExifLog, ExifLogCode, ExifMem, ExifMnoteData,
    ExifMnoteDataMethods, EXIF_BYTE_ORDER_INTEL, EXIF_FORMAT_ASCII, EXIF_FORMAT_LONG,
    EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SHORT, EXIF_FORMAT_SLONG, EXIF_FORMAT_SRATIONAL,
    EXIF_FORMAT_SSHORT, EXIF_FORMAT_UNDEFINED,
};
use crate::i18n::{empty_message, message};
use crate::mnote::base::{
    check_overflow, invalid_components_message, invalid_format_message, tag_description_from_table,
    tag_name_from_table, tag_title_from_table, write_slice_to_buffer, write_str_to_buffer, TagInfo,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::primitives::utils::{
    exif_array_set_byte_order, exif_get_long, exif_get_rational, exif_get_short, exif_get_slong,
    exif_get_srational, exif_get_sshort, exif_set_long, exif_set_short,
};
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_realloc_impl};

const TAGS: [TagInfo; 31] = [
    TagInfo {
        tag: 0x0000,
        name: Some(message(b"Version\0")),
        title: Some(message(b"Maker Note Version\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0010,
        name: Some(message(b"SerialNumber\0")),
        title: Some(message(b"Serial Number\0")),
        description: Some(message(
            b"This number is unique and based on the date of manufacture.\0",
        )),
    },
    TagInfo {
        tag: 0x1000,
        name: Some(message(b"Quality\0")),
        title: Some(message(b"Quality\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1001,
        name: Some(message(b"Sharpness\0")),
        title: Some(message(b"Sharpness\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1002,
        name: Some(message(b"WhiteBalance\0")),
        title: Some(message(b"White Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1003,
        name: Some(message(b"ChromaticitySaturation\0")),
        title: Some(message(b"Chromaticity Saturation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1004,
        name: Some(message(b"Contrast\0")),
        title: Some(message(b"Contrast\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1010,
        name: Some(message(b"FlashMode\0")),
        title: Some(message(b"Flash Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1011,
        name: Some(message(b"FlashStrength\0")),
        title: Some(message(b"Flash Firing Strength Compensation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1020,
        name: Some(message(b"MacroMode\0")),
        title: Some(message(b"Macro Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1021,
        name: Some(message(b"FocusingMode\0")),
        title: Some(message(b"Focusing Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1023,
        name: Some(message(b"FocusPoint\0")),
        title: Some(message(b"Focus Point\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1030,
        name: Some(message(b"SlowSynchro\0")),
        title: Some(message(b"Slow Synchro Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1031,
        name: Some(message(b"PictureMode\0")),
        title: Some(message(b"Picture Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1100,
        name: Some(message(b"ContinuousTaking\0")),
        title: Some(message(b"Continuous Taking\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1101,
        name: Some(message(b"ContinuousSequence\0")),
        title: Some(message(b"Continuous Sequence Number\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1210,
        name: Some(message(b"FinePixColor\0")),
        title: Some(message(b"FinePix Color\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1300,
        name: Some(message(b"BlurCheck\0")),
        title: Some(message(b"Blur Check\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1301,
        name: Some(message(b"AutoFocusCheck\0")),
        title: Some(message(b"Auto Focus Check\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1302,
        name: Some(message(b"AutoExposureCheck\0")),
        title: Some(message(b"Auto Exposure Check\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1400,
        name: Some(message(b"DynamicRange\0")),
        title: Some(message(b"Dynamic Range\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1401,
        name: Some(message(b"FilmMode\0")),
        title: Some(message(b"Film Simulation Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1402,
        name: Some(message(b"DRangeMode\0")),
        title: Some(message(b"Dynamic Range Wide Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1403,
        name: Some(message(b"DevDRangeMode\0")),
        title: Some(message(b"Development Dynamic Range Wide Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1404,
        name: Some(message(b"MinFocalLen\0")),
        title: Some(message(b"Minimum Focal Length\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1405,
        name: Some(message(b"MaxFocalLen\0")),
        title: Some(message(b"Maximum Focal Length\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1406,
        name: Some(message(b"MaxApertAtMinFoc\0")),
        title: Some(message(b"Maximum Aperture at Minimum Focal\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1407,
        name: Some(message(b"MaxApertAtMaxFoc\0")),
        title: Some(message(b"Maximum Aperture at Maximum Focal\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8000,
        name: Some(message(b"FileSource\0")),
        title: Some(message(b"File Source\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8002,
        name: Some(message(b"OrderNumber\0")),
        title: Some(message(b"Order Number\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8003,
        name: Some(message(b"FrameNumber\0")),
        title: Some(message(b"Frame Number\0")),
        description: Some(empty_message()),
    },
];

const DOMAIN: &[u8] = b"ExifMnoteDataFuji\0";
const MSG_SHORT: &[u8] = b"Short MakerNote\0";
const MSG_TOO_MANY: &[u8] = b"Too much tags (%d) in Fuji MakerNote\0";
const MSG_OVERFLOW: &[u8] = b"Tag size overflow detected (%u * %lu)\0";
const MSG_DATA_OVERFLOW: &[u8] = b"Tag data past end of buffer (%u >= %u)\0";
const MSG_NO_MEMORY: &[u8] = b"Could not allocate %u byte(s)\0";

#[derive(Clone, Copy)]
struct FujiValueInfo {
    tag: c_int,
    value: u16,
    text: &'static str,
}

const FUJI_VALUES: &[FujiValueInfo] = &[
    FujiValueInfo {
        tag: 0x1001,
        value: 1,
        text: "Softest",
    },
    FujiValueInfo {
        tag: 0x1001,
        value: 2,
        text: "Soft",
    },
    FujiValueInfo {
        tag: 0x1001,
        value: 3,
        text: "Normal",
    },
    FujiValueInfo {
        tag: 0x1001,
        value: 4,
        text: "Hard",
    },
    FujiValueInfo {
        tag: 0x1001,
        value: 5,
        text: "Hardest",
    },
    FujiValueInfo {
        tag: 0x1002,
        value: 0,
        text: "Auto",
    },
    FujiValueInfo {
        tag: 0x1002,
        value: 0x100,
        text: "Daylight",
    },
    FujiValueInfo {
        tag: 0x1002,
        value: 0x200,
        text: "Cloudy",
    },
    FujiValueInfo {
        tag: 0x1003,
        value: 0,
        text: "Standard",
    },
    FujiValueInfo {
        tag: 0x1003,
        value: 0x100,
        text: "High",
    },
    FujiValueInfo {
        tag: 0x1010,
        value: 0,
        text: "Auto",
    },
    FujiValueInfo {
        tag: 0x1010,
        value: 1,
        text: "On",
    },
    FujiValueInfo {
        tag: 0x1010,
        value: 2,
        text: "Off",
    },
    FujiValueInfo {
        tag: 0x1010,
        value: 3,
        text: "Red-eye reduction",
    },
    FujiValueInfo {
        tag: 0x1020,
        value: 0,
        text: "Off",
    },
    FujiValueInfo {
        tag: 0x1020,
        value: 1,
        text: "On",
    },
    FujiValueInfo {
        tag: 0x1021,
        value: 0,
        text: "Auto",
    },
    FujiValueInfo {
        tag: 0x1021,
        value: 1,
        text: "Manual",
    },
    FujiValueInfo {
        tag: 0x1030,
        value: 0,
        text: "Off",
    },
    FujiValueInfo {
        tag: 0x1030,
        value: 1,
        text: "On",
    },
    FujiValueInfo {
        tag: 0x1031,
        value: 0,
        text: "Auto",
    },
    FujiValueInfo {
        tag: 0x1031,
        value: 1,
        text: "Portrait",
    },
    FujiValueInfo {
        tag: 0x1031,
        value: 2,
        text: "Landscape",
    },
    FujiValueInfo {
        tag: 0x1100,
        value: 0,
        text: "Off",
    },
    FujiValueInfo {
        tag: 0x1100,
        value: 1,
        text: "On",
    },
    FujiValueInfo {
        tag: 0x1210,
        value: 0x00,
        text: "F-Standard",
    },
    FujiValueInfo {
        tag: 0x1210,
        value: 0x10,
        text: "F-Chrome",
    },
    FujiValueInfo {
        tag: 0x1300,
        value: 0,
        text: "No blur",
    },
    FujiValueInfo {
        tag: 0x1300,
        value: 1,
        text: "Blur warning",
    },
    FujiValueInfo {
        tag: 0x1301,
        value: 0,
        text: "Focus good",
    },
    FujiValueInfo {
        tag: 0x1301,
        value: 1,
        text: "Out of focus",
    },
    FujiValueInfo {
        tag: 0x1302,
        value: 0,
        text: "AE good",
    },
    FujiValueInfo {
        tag: 0x1302,
        value: 1,
        text: "Over exposed",
    },
];

fn fuji_lookup_value(tag: c_int, value: u16) -> Option<&'static str> {
    FUJI_VALUES
        .iter()
        .find(|entry| entry.tag == tag && entry.value == value)
        .map(|entry| entry.text)
}

#[repr(C)]
struct MnoteFujiEntry {
    tag: c_int,
    format: ExifFormat,
    components: c_ulong,
    data: *mut c_uchar,
    size: c_uint,
    order: ExifByteOrder,
}

#[repr(C)]
struct ExifMnoteDataFuji {
    parent: ExifMnoteData,
    entries: *mut MnoteFujiEntry,
    count: c_uint,
    order: ExifByteOrder,
    offset: c_uint,
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
unsafe fn fuji_note(note: *mut ExifMnoteData) -> *mut ExifMnoteDataFuji {
    note.cast()
}

unsafe fn log_simple(note: *mut ExifMnoteDataFuji, code: ExifLogCode, format: &[u8]) {
    unsafe {
        exif_log(
            (*note).parent.log,
            code,
            DOMAIN.as_ptr().cast(),
            format.as_ptr().cast(),
        )
    };
}

unsafe fn log_no_memory(note: *mut ExifMnoteDataFuji, size: usize) {
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

unsafe fn clear_impl(note: *mut ExifMnoteDataFuji) {
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

unsafe extern "C" fn exif_mnote_data_fuji_free(note: *mut ExifMnoteData) {
    unsafe { clear_impl(fuji_note(note)) };
}

unsafe extern "C" fn exif_mnote_data_fuji_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || value.is_null() || unsafe { index >= (*note).count } {
        return ptr::null_mut();
    }

    let entry = unsafe { &*(*note).entries.add(index as usize) };
    let tag = entry.tag;
    let components = entry.components as u64;

    if entry.data.is_null() && entry.size != 0 {
        return ptr::null_mut();
    }

    match tag {
        0x0000 => {
            if entry.format != EXIF_FORMAT_UNDEFINED {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry.format, &[EXIF_FORMAT_UNDEFINED]),
                    )
                };
            }
            if components != 4 {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(components, &[4]),
                    )
                };
            }
            if entry.data.is_null() {
                return unsafe { write_str_to_buffer(value, maxlen, "") };
            }
            let copy_len = (entry.size as usize).min(4);
            let bytes = unsafe { std::slice::from_raw_parts(entry.data, copy_len) };
            unsafe { write_slice_to_buffer(value, maxlen, bytes) }
        }
        0x1001 | 0x1002 | 0x1003 | 0x1004 | 0x1010 | 0x1020 | 0x1021 | 0x1030 | 0x1031 | 0x1100
        | 0x1210 | 0x1300 | 0x1301 | 0x1302 | 0x1400 | 0x1401 | 0x1402 => {
            if entry.format != EXIF_FORMAT_SHORT {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry.format, &[EXIF_FORMAT_SHORT]),
                    )
                };
            }
            if components != 1 {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(components, &[1]),
                    )
                };
            }
            let raw = unsafe { exif_get_short(entry.data, entry.order) };
            match fuji_lookup_value(tag, raw) {
                Some(text) => unsafe { write_str_to_buffer(value, maxlen, text) },
                None => unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &format!("Internal error (unknown value {raw})"),
                    )
                },
            }
        }
        0x1023 => {
            if entry.format != EXIF_FORMAT_SHORT {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry.format, &[EXIF_FORMAT_SHORT]),
                    )
                };
            }
            if components != 2 {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(components, &[2]),
                    )
                };
            }
            let first = unsafe { exif_get_short(entry.data, entry.order) };
            let second = unsafe { exif_get_short(entry.data.add(2), entry.order) };
            unsafe { write_str_to_buffer(value, maxlen, &format!("{first}, {second}")) }
        }
        0x1404 | 0x1405 => {
            if entry.format != EXIF_FORMAT_RATIONAL {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry.format, &[EXIF_FORMAT_RATIONAL]),
                    )
                };
            }
            if components != 1 {
                return unsafe {
                    write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(components, &[1]),
                    )
                };
            }
            let rational = unsafe { exif_get_rational(entry.data, entry.order) };
            if rational.denominator == 0 {
                return unsafe { write_str_to_buffer(value, maxlen, "") };
            }
            unsafe {
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{:.2} mm",
                        rational.numerator as f64 / rational.denominator as f64
                    ),
                )
            }
        }
        _ => unsafe {
            match entry.format {
                EXIF_FORMAT_ASCII => {
                    let bytes = if entry.data.is_null() {
                        &[]
                    } else {
                        std::slice::from_raw_parts(entry.data, entry.size as usize)
                    };
                    write_slice_to_buffer(value, maxlen, bytes)
                }
                EXIF_FORMAT_SHORT => write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_short(entry.data, entry.order).to_string(),
                ),
                EXIF_FORMAT_SSHORT => write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_sshort(entry.data, entry.order).to_string(),
                ),
                EXIF_FORMAT_LONG => write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_long(entry.data, entry.order).to_string(),
                ),
                EXIF_FORMAT_SLONG => write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_slong(entry.data, entry.order).to_string(),
                ),
                EXIF_FORMAT_RATIONAL => {
                    let rational = exif_get_rational(entry.data, entry.order);
                    if rational.denominator == 0 {
                        write_str_to_buffer(value, maxlen, "")
                    } else {
                        write_str_to_buffer(
                            value,
                            maxlen,
                            &format!(
                                "{:.4}",
                                rational.numerator as f64 / rational.denominator as f64
                            ),
                        )
                    }
                }
                EXIF_FORMAT_SRATIONAL => {
                    let rational = exif_get_srational(entry.data, entry.order);
                    if rational.denominator == 0 {
                        write_str_to_buffer(value, maxlen, "")
                    } else {
                        write_str_to_buffer(
                            value,
                            maxlen,
                            &format!(
                                "{:.4}",
                                rational.numerator as f64 / rational.denominator as f64
                            ),
                        )
                    }
                }
                _ => write_str_to_buffer(
                    value,
                    maxlen,
                    &format!("{} bytes unknown data", entry.size),
                ),
            }
        },
    }
}

unsafe extern "C" fn exif_mnote_data_fuji_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    buffer_size: *mut c_uint,
) {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || buffer.is_null() || buffer_size.is_null() {
        return;
    }

    let mut out_size = 8usize + 4 + 2 + unsafe { (*note).count as usize } * 12 + 4;
    let mut out =
        unsafe { exif_mem_alloc_impl((*note).parent.mem, out_size as u32) }.cast::<c_uchar>();
    if out.is_null() {
        unsafe { *buffer_size = 0 };
        return;
    }

    unsafe {
        *buffer = out;
        *buffer_size = out_size as c_uint;
        ptr::copy_nonoverlapping(b"FUJIFILM".as_ptr(), out, 8);
        exif_set_long(out.add(8), (*note).order, 12);
        exif_set_short(out.add(12), (*note).order, (*note).count as u16);
    }

    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { &*(*note).entries.add(index) };
        let mut offset = 8 + 4 + 2 + index * 12;
        unsafe {
            exif_set_short(out.add(offset), (*note).order, entry.tag as u16);
            exif_set_short(out.add(offset + 2), (*note).order, entry.format as u16);
            exif_set_long(out.add(offset + 4), (*note).order, entry.components as u32);
        }
        offset += 8;

        let Some(data_size) = (exif_format_get_size_impl(entry.format) as usize)
            .checked_mul(entry.components as usize)
        else {
            continue;
        };
        if data_size > 65_536 {
            continue;
        }

        let data_offset = if data_size > 4 {
            let mut target_size = out_size + data_size;
            if data_size & 1 != 0 {
                target_size += 1;
            }
            let new_out = unsafe {
                exif_mem_realloc_impl((*note).parent.mem, out.cast(), target_size as u32)
            }
            .cast::<c_uchar>();
            if new_out.is_null() {
                return;
            }
            out = new_out;
            unsafe {
                *buffer = out;
                *buffer_size = target_size as c_uint;
            }
            out_size = target_size;
            let mut value_offset = out_size - data_size;
            if data_size & 1 != 0 {
                value_offset -= 1;
                unsafe { *out.add(out_size - 1) = 0 };
            }
            unsafe { exif_set_long(out.add(offset), (*note).order, value_offset as u32) };
            value_offset
        } else {
            offset
        };

        let out_ref = unsafe { *buffer };
        if entry.data.is_null() {
            unsafe { ptr::write_bytes(out_ref.add(data_offset), 0, data_size) };
        } else {
            unsafe { ptr::copy_nonoverlapping(entry.data, out_ref.add(data_offset), data_size) };
        }
    }
}

unsafe extern "C" fn exif_mnote_data_fuji_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    buffer_size: c_uint,
) {
    let note = unsafe { fuji_note(note) };
    let buffer_size = buffer_size as usize;
    if note.is_null() || buffer.is_null() || buffer_size == 0 {
        if !note.is_null() {
            unsafe { log_simple(note, 3, MSG_SHORT) };
        }
        return;
    }

    let mut data_offset = 6 + unsafe { (*note).offset as usize };
    if check_overflow(data_offset, buffer_size, 12) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }

    unsafe { (*note).order = EXIF_BYTE_ORDER_INTEL };
    data_offset +=
        unsafe { exif_get_long(buffer.add(data_offset + 8), EXIF_BYTE_ORDER_INTEL) as usize };
    if check_overflow(data_offset, buffer_size, 2) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }

    let count = unsafe { exif_get_short(buffer.add(data_offset), EXIF_BYTE_ORDER_INTEL) as usize };
    data_offset += 2;
    if count > 150 {
        unsafe {
            exif_log(
                (*note).parent.log,
                3,
                DOMAIN.as_ptr().cast(),
                MSG_TOO_MANY.as_ptr().cast(),
                count as c_int,
            );
        }
        return;
    }

    unsafe { clear_impl(note) };

    let entry_bytes = size_of::<MnoteFujiEntry>().saturating_mul(count);
    let entries = unsafe { exif_mem_alloc_impl((*note).parent.mem, entry_bytes as u32) }
        .cast::<MnoteFujiEntry>();
    if entries.is_null() {
        unsafe { log_no_memory(note, entry_bytes) };
        return;
    }
    unsafe { (*note).entries = entries };

    let mut stored = 0usize;
    let mut offset = data_offset;
    for _ in 0..count {
        if check_overflow(offset, buffer_size, 12) {
            unsafe { log_simple(note, 3, MSG_SHORT) };
            break;
        }

        let entry = unsafe { (*note).entries.add(stored) };
        unsafe {
            (*entry).tag = exif_get_short(buffer.add(offset), (*note).order) as c_int;
            (*entry).format = exif_get_short(buffer.add(offset + 2), (*note).order) as ExifFormat;
            (*entry).components = exif_get_long(buffer.add(offset + 4), (*note).order) as c_ulong;
            (*entry).order = (*note).order;
        }

        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if format_size != 0
            && (buffer_size as u128 / format_size as u128) < unsafe { (*entry).components as u128 }
        {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN.as_ptr().cast(),
                    MSG_OVERFLOW.as_ptr().cast(),
                    format_size as c_uint,
                    (*entry).components,
                );
            }
            offset += 12;
            continue;
        }

        let Some(data_size) = format_size.checked_mul(unsafe { (*entry).components as usize })
        else {
            offset += 12;
            continue;
        };
        unsafe { (*entry).size = data_size as c_uint };

        if data_size != 0 {
            let mut data_offset = offset + 8;
            if data_size > 4 {
                data_offset =
                    unsafe { exif_get_long(buffer.add(data_offset), (*note).order) as usize }
                        + 6
                        + unsafe { (*note).offset as usize };
            }

            if check_overflow(data_offset, buffer_size, data_size) {
                unsafe {
                    exif_log(
                        (*note).parent.log,
                        3,
                        DOMAIN.as_ptr().cast(),
                        MSG_DATA_OVERFLOW.as_ptr().cast(),
                        (data_offset + data_size) as c_uint,
                        buffer_size as c_uint,
                    );
                }
                offset += 12;
                continue;
            }

            let data = unsafe { exif_mem_alloc_impl((*note).parent.mem, data_size as u32) }
                .cast::<c_uchar>();
            if data.is_null() {
                unsafe { log_no_memory(note, data_size) };
                offset += 12;
                continue;
            }

            unsafe {
                ptr::copy_nonoverlapping(buffer.add(data_offset), data, data_size);
                (*entry).data = data;
            }
        }

        stored += 1;
        offset += 12;
    }

    unsafe { (*note).count = stored as c_uint };
}

unsafe extern "C" fn exif_mnote_data_fuji_count(note: *mut ExifMnoteData) -> c_uint {
    let note = unsafe { fuji_note(note) };
    if note.is_null() {
        0
    } else {
        unsafe { (*note).count }
    }
}

unsafe extern "C" fn exif_mnote_data_fuji_get_id(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> c_uint {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        0
    } else {
        unsafe { (*(*note).entries.add(index as usize)).tag as c_uint }
    }
}

unsafe extern "C" fn exif_mnote_data_fuji_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null();
    }
    tag_name_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

unsafe extern "C" fn exif_mnote_data_fuji_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null();
    }
    tag_title_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

unsafe extern "C" fn exif_mnote_data_fuji_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { fuji_note(note) };
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null();
    }
    tag_description_from_table(&TAGS, unsafe { (*(*note).entries.add(index as usize)).tag })
}

unsafe extern "C" fn exif_mnote_data_fuji_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    let note = unsafe { fuji_note(note) };
    if note.is_null() {
        return;
    }

    let original = unsafe { (*note).order };
    unsafe { (*note).order = order };
    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { (*note).entries.add(index) };
        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if unsafe { (*entry).components } != 0
            && unsafe { (*entry).size as usize / (*entry).components as usize } < format_size
        {
            continue;
        }

        unsafe {
            (*entry).order = order;
            exif_array_set_byte_order(
                (*entry).format,
                (*entry).data,
                (*entry).components as c_uint,
                original,
                order,
            );
        }
    }
}

unsafe extern "C" fn exif_mnote_data_fuji_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    let note = unsafe { fuji_note(note) };
    if !note.is_null() {
        unsafe { (*note).offset = offset };
    }
}

pub(crate) unsafe fn identify_impl(_data: *const ExifData, entry: *const ExifEntry) -> c_int {
    if entry.is_null() || unsafe { (*entry).size } < 12 {
        return 0;
    }

    let data = unsafe { std::slice::from_raw_parts((*entry).data, 8) };
    (data == b"FUJIFILM") as c_int
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let note = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifMnoteDataFuji>() as u32) }
        .cast::<ExifMnoteDataFuji>();
    if note.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        crate::mnote::base::exif_mnote_data_construct(ptr::addr_of_mut!((*note).parent), mem)
    };
    unsafe {
        (*note).parent.methods = ExifMnoteDataMethods {
            free: Some(exif_mnote_data_fuji_free),
            save: Some(exif_mnote_data_fuji_save),
            load: Some(exif_mnote_data_fuji_load),
            set_offset: Some(exif_mnote_data_fuji_set_offset),
            set_byte_order: Some(exif_mnote_data_fuji_set_byte_order),
            count: Some(exif_mnote_data_fuji_count),
            get_id: Some(exif_mnote_data_fuji_get_id),
            get_name: Some(exif_mnote_data_fuji_get_name),
            get_title: Some(exif_mnote_data_fuji_get_title),
            get_description: Some(exif_mnote_data_fuji_get_description),
            get_value: Some(exif_mnote_data_fuji_get_value),
        };
        (*note).order = EXIF_BYTE_ORDER_INTEL;
    }

    unsafe { ptr::addr_of_mut!((*note).parent) }
}
