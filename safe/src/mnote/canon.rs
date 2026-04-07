use core::ffi::{c_char, c_int, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifDataOption, ExifEntry, ExifFormat, ExifLog, ExifLogCode, ExifMem,
    ExifMnoteData, ExifMnoteDataMethods, MnoteCanonEntry, MnoteCanonTag,
    EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS, EXIF_FORMAT_ASCII, EXIF_FORMAT_LONG, EXIF_FORMAT_SHORT,
};
use crate::mnote::base::{
    check_overflow, generic_mnote_value, invalid_components_message, invalid_format_message,
    write_slice_to_buffer, write_str_to_buffer,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::primitives::utils::{
    exif_array_set_byte_order, exif_get_long, exif_get_short, exif_get_sshort, exif_set_long,
    exif_set_short,
};
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_realloc_impl};

const EXIF_TAG_MAKE: c_int = 0x010f;

const MNOTE_CANON_TAG_SETTINGS_1: c_int = 0x0001;
const MNOTE_CANON_TAG_FOCAL_LENGTH: c_int = 0x0002;
const MNOTE_CANON_TAG_SETTINGS_2: c_int = 0x0004;
const MNOTE_CANON_TAG_PANORAMA: c_int = 0x0005;
const MNOTE_CANON_TAG_IMAGE_TYPE: c_int = 0x0006;
const MNOTE_CANON_TAG_FIRMWARE: c_int = 0x0007;
const MNOTE_CANON_TAG_IMAGE_NUMBER: c_int = 0x0008;
const MNOTE_CANON_TAG_OWNER: c_int = 0x0009;
const MNOTE_CANON_TAG_SERIAL_NUMBER: c_int = 0x000c;
const MNOTE_CANON_TAG_CAMERA_INFO: c_int = 0x000d;
const MNOTE_CANON_TAG_CUSTOM_FUNCS: c_int = 0x000f;
const MNOTE_CANON_TAG_MODEL_ID: c_int = 0x0010;
const MNOTE_CANON_TAG_AF_INFO: c_int = 0x0012;
const MNOTE_CANON_TAG_THUMBNAIL_VALID_AREA: c_int = 0x0013;
const MNOTE_CANON_TAG_COLOR_INFORMATION: c_int = 0x00a0;

const FAILSAFE_SIZE_MAX: u64 = 1_000_000;

const DOMAIN: &[u8] = b"ExifMnoteCanon\0";
const MSG_SHORT: &[u8] = b"Short MakerNote\0";
const MSG_TOO_MANY: &[u8] = b"Too much tags (%d) in Canon MakerNote\0";
const MSG_OVERFLOW: &[u8] = b"Tag size overflow detected (%u * %lu)\0";
const MSG_ZERO_SIZE: &[u8] = b"Invalid zero-length tag size\0";
const MSG_PAST_END: &[u8] = b"Tag data past end of buffer (%u > %u)\0";
const MSG_FAILSAFE: &[u8] = b"Failsafe tag size overflow (%lu > %ld)\0";
const MSG_NO_MEMORY: &[u8] = b"Could not allocate %u byte(s)\0";

#[repr(C)]
struct ExifMnoteDataCanon {
    parent: ExifMnoteData,
    entries: *mut MnoteCanonEntry,
    count: c_uint,
    order: ExifByteOrder,
    offset: c_uint,
    options: ExifDataOption,
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
unsafe fn canon_note(note: *mut ExifMnoteData) -> *mut ExifMnoteDataCanon {
    note.cast()
}

#[derive(Clone, Copy)]
struct CanonSubtagInfo {
    tag: c_int,
    subtag: c_uint,
    name: &'static [u8],
}

#[derive(Clone, Copy)]
struct CanonValueInfo {
    subtag: c_uint,
    value: u16,
    text: &'static str,
}

const CANON_SUBTAGS: &[CanonSubtagInfo] = &[
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 0,
        name: b"Macro Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 1,
        name: b"Self-timer\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 2,
        name: b"Quality\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 3,
        name: b"Flash Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 4,
        name: b"Drive Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 6,
        name: b"Focus Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 8,
        name: b"Record Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 9,
        name: b"Image Size\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 10,
        name: b"Easy Shooting Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 11,
        name: b"Digital Zoom\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 12,
        name: b"Contrast\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 13,
        name: b"Saturation\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 14,
        name: b"Sharpness\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 15,
        name: b"ISO\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 16,
        name: b"Metering Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 17,
        name: b"Focus Range\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 18,
        name: b"AF Point\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 19,
        name: b"Exposure Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 21,
        name: b"Lens Type\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 22,
        name: b"Long Focal Length of Lens\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 23,
        name: b"Short Focal Length of Lens\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 24,
        name: b"Focal Units per mm\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 25,
        name: b"Maximal Aperture\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 26,
        name: b"Minimal Aperture\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 27,
        name: b"Flash Activity\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 28,
        name: b"Flash Details\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 31,
        name: b"Focus Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 32,
        name: b"AE Setting\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 33,
        name: b"Image Stabilization\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 34,
        name: b"Display Aperture\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 35,
        name: b"Zoom Source Width\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 36,
        name: b"Zoom Target Width\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 38,
        name: b"Spot Metering Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 39,
        name: b"Photo Effect\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 40,
        name: b"Manual Flash Output\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_1,
        subtag: 41,
        name: b"Color Tone\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_FOCAL_LENGTH,
        subtag: 0,
        name: b"Focal Type\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_FOCAL_LENGTH,
        subtag: 1,
        name: b"Focal Length\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_FOCAL_LENGTH,
        subtag: 2,
        name: b"Focal Plane X Size\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_FOCAL_LENGTH,
        subtag: 3,
        name: b"Focal Plane Y Size\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 0,
        name: b"Auto ISO\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 1,
        name: b"Shot ISO\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 2,
        name: b"Measured EV\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 3,
        name: b"Target Aperture\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 4,
        name: b"Target Exposure Time\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 5,
        name: b"Exposure Compensation\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 6,
        name: b"White Balance\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 7,
        name: b"Slow Shutter\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 8,
        name: b"Sequence Number\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 9,
        name: b"Optical Zoom Code\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 11,
        name: b"Camera Temperature\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 12,
        name: b"Flash Guide Number\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 13,
        name: b"AF Point\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 14,
        name: b"Flash Exposure Compensation\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 15,
        name: b"AE Bracketing\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 16,
        name: b"AE Bracket Value\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 17,
        name: b"Control Mode\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 18,
        name: b"Focus Distance Upper\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 19,
        name: b"Focus Distance Lower\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 20,
        name: b"F-Number\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 21,
        name: b"Exposure Time\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 22,
        name: b"Measured EV 2\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 23,
        name: b"Bulb Duration\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 25,
        name: b"Camera Type\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 26,
        name: b"Auto Rotate\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 27,
        name: b"ND Filter\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 28,
        name: b"Self-timer\0",
    },
    CanonSubtagInfo {
        tag: MNOTE_CANON_TAG_SETTINGS_2,
        subtag: 32,
        name: b"Manual Flash Output\0",
    },
];

const SETTINGS_1_VALUES: &[CanonValueInfo] = &[
    CanonValueInfo {
        subtag: 0,
        value: 1,
        text: "Macro",
    },
    CanonValueInfo {
        subtag: 0,
        value: 2,
        text: "Normal",
    },
    CanonValueInfo {
        subtag: 2,
        value: 1,
        text: "Economy",
    },
    CanonValueInfo {
        subtag: 2,
        value: 2,
        text: "Normal",
    },
    CanonValueInfo {
        subtag: 2,
        value: 3,
        text: "Fine",
    },
    CanonValueInfo {
        subtag: 2,
        value: 4,
        text: "RAW",
    },
    CanonValueInfo {
        subtag: 2,
        value: 5,
        text: "Superfine",
    },
    CanonValueInfo {
        subtag: 3,
        value: 0,
        text: "Off",
    },
    CanonValueInfo {
        subtag: 4,
        value: 0,
        text: "Single",
    },
    CanonValueInfo {
        subtag: 6,
        value: 4,
        text: "Single",
    },
    CanonValueInfo {
        subtag: 8,
        value: 1,
        text: "JPEG",
    },
    CanonValueInfo {
        subtag: 9,
        value: 2,
        text: "Small",
    },
    CanonValueInfo {
        subtag: 10,
        value: 2,
        text: "Landscape",
    },
    CanonValueInfo {
        subtag: 11,
        value: 0,
        text: "None",
    },
    CanonValueInfo {
        subtag: 12,
        value: 0x0000,
        text: "Normal",
    },
    CanonValueInfo {
        subtag: 13,
        value: 0x0000,
        text: "Normal",
    },
    CanonValueInfo {
        subtag: 14,
        value: 0x0000,
        text: "Normal",
    },
    CanonValueInfo {
        subtag: 15,
        value: 15,
        text: "Auto",
    },
    CanonValueInfo {
        subtag: 16,
        value: 3,
        text: "Evaluative",
    },
    CanonValueInfo {
        subtag: 17,
        value: 1,
        text: "Auto",
    },
    CanonValueInfo {
        subtag: 18,
        value: 0x4001,
        text: "Auto AF point selection",
    },
    CanonValueInfo {
        subtag: 19,
        value: 0,
        text: "Easy shooting",
    },
    CanonValueInfo {
        subtag: 31,
        value: 0,
        text: "Single",
    },
    CanonValueInfo {
        subtag: 32,
        value: 0,
        text: "Normal AE",
    },
    CanonValueInfo {
        subtag: 39,
        value: 0,
        text: "Off",
    },
    CanonValueInfo {
        subtag: 40,
        value: 0,
        text: "Off",
    },
];

const FOCAL_LENGTH_VALUES: &[CanonValueInfo] = &[
    CanonValueInfo {
        subtag: 0,
        value: 1,
        text: "Fixed",
    },
    CanonValueInfo {
        subtag: 0,
        value: 2,
        text: "Zoom",
    },
];

const SETTINGS_2_VALUES: &[CanonValueInfo] = &[
    CanonValueInfo {
        subtag: 6,
        value: 0,
        text: "Auto",
    },
    CanonValueInfo {
        subtag: 7,
        value: 0,
        text: "Off",
    },
    CanonValueInfo {
        subtag: 15,
        value: 0,
        text: "Off",
    },
    CanonValueInfo {
        subtag: 25,
        value: 250,
        text: "Compact",
    },
    CanonValueInfo {
        subtag: 26,
        value: 1,
        text: "Rotate 90 CW",
    },
    CanonValueInfo {
        subtag: 27,
        value: 0,
        text: "Off",
    },
    CanonValueInfo {
        subtag: 32,
        value: 0,
        text: "Off",
    },
];

fn canon_lookup_subtag_name(tag: c_int, subtag: c_uint) -> Option<&'static [u8]> {
    CANON_SUBTAGS
        .iter()
        .find(|entry| entry.tag == tag && entry.subtag == subtag)
        .map(|entry| entry.name)
}

fn canon_has_subtags(tag: c_int) -> bool {
    CANON_SUBTAGS.iter().any(|entry| entry.tag == tag)
}

fn canon_lookup_value(
    table: &[CanonValueInfo],
    subtag: c_uint,
    value: u16,
) -> Option<&'static str> {
    table
        .iter()
        .find(|entry| entry.subtag == subtag && entry.value == value)
        .map(|entry| entry.text)
}

fn canon_lookup_bitfield(table: &[CanonValueInfo], subtag: c_uint, value: u16) -> String {
    let mut labels = Vec::new();
    for entry in table.iter().filter(|entry| entry.subtag == subtag) {
        if ((value >> entry.value) & 1) != 0 {
            labels.push(entry.text);
        }
    }
    labels.join(", ")
}

fn apex_value_to_aperture(value: f64) -> f64 {
    2f64.powf(value / 2.0)
}

fn apex_value_to_shutter_speed(value: f64) -> f64 {
    1.0 / 2f64.powf(value)
}

fn apex_value_to_iso_speed(value: f64) -> f64 {
    3.125 * 2f64.powf(value)
}

fn canon_tag_name_impl(tag: MnoteCanonTag) -> Option<&'static [u8]> {
    match tag {
        0x0001 => Some(b"Settings1\0"),
        0x0002 => Some(b"FocalLength\0"),
        0x0004 => Some(b"Settings2\0"),
        0x0005 => Some(b"Panorama\0"),
        0x0006 => Some(b"ImageType\0"),
        0x0007 => Some(b"FirmwareVersion\0"),
        0x0008 => Some(b"ImageNumber\0"),
        0x0009 => Some(b"OwnerName\0"),
        0x000c => Some(b"SerialNumber\0"),
        MNOTE_CANON_TAG_CAMERA_INFO => Some(b"CameraInfo\0"),
        0x000f => Some(b"CustomFunctions\0"),
        MNOTE_CANON_TAG_MODEL_ID => Some(b"ModelID\0"),
        MNOTE_CANON_TAG_AF_INFO => Some(b"AFInfo\0"),
        MNOTE_CANON_TAG_THUMBNAIL_VALID_AREA => Some(b"ThumbnailValidArea\0"),
        0x00a0 => Some(b"ColorInformation\0"),
        _ => None,
    }
}

fn canon_tag_title_impl(tag: MnoteCanonTag) -> Option<&'static [u8]> {
    match tag {
        0x0001 => Some(b"Settings (First Part)\0"),
        0x0002 => Some(b"Focal Length\0"),
        0x0004 => Some(b"Settings (Second Part)\0"),
        0x0005 => Some(b"Panorama\0"),
        0x0006 => Some(b"Image Type\0"),
        0x0007 => Some(b"Firmware Version\0"),
        0x0008 => Some(b"Image Number\0"),
        0x0009 => Some(b"Owner Name\0"),
        0x000c => Some(b"Serial Number\0"),
        MNOTE_CANON_TAG_CAMERA_INFO => Some(b"Camera Info\0"),
        0x000f => Some(b"Custom Functions\0"),
        MNOTE_CANON_TAG_MODEL_ID => Some(b"Model ID\0"),
        MNOTE_CANON_TAG_AF_INFO => Some(b"AF Info\0"),
        MNOTE_CANON_TAG_THUMBNAIL_VALID_AREA => Some(b"Thumbnail Valid Area\0"),
        0x00a0 => Some(b"Color Information\0"),
        _ => None,
    }
}

unsafe fn log_simple(note: *mut ExifMnoteDataCanon, code: ExifLogCode, format: &[u8]) {
    unsafe {
        exif_log(
            (*note).parent.log,
            code,
            DOMAIN.as_ptr().cast(),
            format.as_ptr().cast(),
        )
    };
}

unsafe fn log_no_memory(note: *mut ExifMnoteDataCanon, size: usize) {
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

unsafe fn clear_impl(note: *mut ExifMnoteDataCanon) {
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

unsafe extern "C" fn exif_mnote_data_canon_free(note: *mut ExifMnoteData) {
    unsafe { clear_impl(canon_note(note)) };
}

fn mnote_canon_entry_count_values_impl(entry: *const MnoteCanonEntry) -> c_uint {
    if entry.is_null() {
        return 0;
    }
    let entry = unsafe { &*entry };
    match entry.tag {
        MNOTE_CANON_TAG_FOCAL_LENGTH | MNOTE_CANON_TAG_PANORAMA => entry.components as c_uint,
        MNOTE_CANON_TAG_SETTINGS_1
        | MNOTE_CANON_TAG_SETTINGS_2
        | MNOTE_CANON_TAG_CUSTOM_FUNCS
        | MNOTE_CANON_TAG_COLOR_INFORMATION => {
            if entry.format != EXIF_FORMAT_SHORT || entry.data.is_null() || entry.size < 2 {
                return 0;
            }
            let values = unsafe { exif_get_short(entry.data, entry.order) as usize };
            ((entry.size as usize).saturating_sub(2).min(values) / 2) as c_uint
        }
        _ => 1,
    }
}

unsafe fn get_tags_impl(
    note: *mut ExifMnoteDataCanon,
    index: c_uint,
    entry_index: *mut c_uint,
    subtag_index: *mut c_uint,
) {
    if note.is_null() || entry_index.is_null() {
        return;
    }
    let mut from = 0u32;
    for current in 0..unsafe { (*note).count } {
        let to = from
            + mnote_canon_entry_count_values_impl(unsafe { (*note).entries.add(current as usize) });
        if to > index {
            unsafe {
                *entry_index = current;
                if !subtag_index.is_null() {
                    *subtag_index = index - from;
                }
            }
            break;
        }
        from = to;
    }
}

unsafe extern "C" fn exif_mnote_data_canon_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return ptr::null_mut();
    }
    let mut entry_index = 0;
    let mut subtag_index = 0;
    unsafe { get_tags_impl(note, index, &mut entry_index, &mut subtag_index) };
    if entry_index >= unsafe { (*note).count } {
        return ptr::null_mut();
    }
    unsafe {
        mnote_canon_entry_get_value(
            (*note).entries.add(entry_index as usize),
            subtag_index,
            value,
            maxlen,
        )
    }
}

unsafe extern "C" fn exif_mnote_data_canon_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    let note = unsafe { canon_note(note) };
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

unsafe extern "C" fn exif_mnote_data_canon_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    let note = unsafe { canon_note(note) };
    if !note.is_null() {
        unsafe { (*note).offset = offset };
    }
}

unsafe extern "C" fn exif_mnote_data_canon_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    buffer_size: *mut c_uint,
) {
    let note = unsafe { canon_note(note) };
    if note.is_null() || buffer.is_null() || buffer_size.is_null() {
        return;
    }

    let mut out_size = 2usize + unsafe { (*note).count as usize } * 12 + 4;
    let mut out =
        unsafe { exif_mem_alloc_impl((*note).parent.mem, out_size as u32) }.cast::<c_uchar>();
    if out.is_null() {
        unsafe { log_no_memory(note, out_size) };
        return;
    }
    unsafe {
        *buffer = out;
        *buffer_size = out_size as c_uint;
        exif_set_short(out, (*note).order, (*note).count as u16);
    }

    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { &*(*note).entries.add(index) };
        let mut offset = 2 + index * 12;
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
                unsafe { log_no_memory(note, target_size) };
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
            unsafe {
                exif_set_long(
                    out.add(offset),
                    (*note).order,
                    (*note).offset + value_offset as u32,
                )
            };
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
        if data_size < 4 {
            unsafe { ptr::write_bytes(out_ref.add(data_offset + data_size), 0, 4 - data_size) };
        }
    }
}

unsafe extern "C" fn exif_mnote_data_canon_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    buffer_size: c_uint,
) {
    let note = unsafe { canon_note(note) };
    let buffer_size = buffer_size as usize;
    if note.is_null() || buffer.is_null() || buffer_size == 0 {
        if !note.is_null() {
            unsafe { log_simple(note, 3, MSG_SHORT) };
        }
        return;
    }

    let data_offset = 6 + unsafe { (*note).offset as usize };
    if check_overflow(data_offset, buffer_size, 2) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }

    let count = unsafe { exif_get_short(buffer.add(data_offset), (*note).order) as usize };
    let mut offset = data_offset + 2;
    if count > 250 {
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
    let entry_bytes = size_of::<MnoteCanonEntry>().saturating_mul(count);
    let entries = unsafe { exif_mem_alloc_impl((*note).parent.mem, entry_bytes as u32) }
        .cast::<MnoteCanonEntry>();
    if entries.is_null() {
        unsafe { log_no_memory(note, entry_bytes) };
        return;
    }
    unsafe { (*note).entries = entries };

    let mut stored = 0usize;
    let mut failsafe_size = 0u64;
    for _ in 0..count {
        if check_overflow(offset, buffer_size, 12) {
            unsafe { log_simple(note, 3, MSG_SHORT) };
            break;
        }

        let entry = unsafe { (*note).entries.add(stored) };
        unsafe {
            (*entry).tag = exif_get_short(buffer.add(offset), (*note).order) as MnoteCanonTag;
            (*entry).format = exif_get_short(buffer.add(offset + 2), (*note).order) as ExifFormat;
            (*entry).components = exif_get_long(buffer.add(offset + 4), (*note).order).into();
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

        if data_size == 0 {
            unsafe { log_simple(note, 3, MSG_ZERO_SIZE) };
            offset += 12;
            continue;
        }

        let mut data_offset = offset + 8;
        if data_size > 4 {
            data_offset =
                unsafe { exif_get_long(buffer.add(data_offset), (*note).order) as usize } + 6;
        }
        if check_overflow(data_offset, buffer_size, data_size) {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    1,
                    DOMAIN.as_ptr().cast(),
                    MSG_PAST_END.as_ptr().cast(),
                    (data_offset + data_size) as c_uint,
                    buffer_size as c_uint,
                );
            }
            offset += 12;
            continue;
        }

        let data =
            unsafe { exif_mem_alloc_impl((*note).parent.mem, data_size as u32) }.cast::<c_uchar>();
        if data.is_null() {
            unsafe { log_no_memory(note, data_size) };
            offset += 12;
            continue;
        }
        unsafe {
            ptr::copy_nonoverlapping(buffer.add(data_offset), data, data_size);
            (*entry).data = data;
        }

        failsafe_size += mnote_canon_entry_count_values_impl(entry) as u64;
        if failsafe_size > FAILSAFE_SIZE_MAX {
            unsafe {
                exif_mem_free_impl((*note).parent.mem, (*entry).data.cast());
                (*entry).data = ptr::null_mut();
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN.as_ptr().cast(),
                    MSG_FAILSAFE.as_ptr().cast(),
                    failsafe_size,
                    FAILSAFE_SIZE_MAX as i64,
                );
            }
            break;
        }

        stored += 1;
        offset += 12;
    }

    unsafe { (*note).count = stored as c_uint };
}

unsafe extern "C" fn exif_mnote_data_canon_count(note: *mut ExifMnoteData) -> c_uint {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return 0;
    }
    let mut total = 0;
    for index in 0..unsafe { (*note).count as usize } {
        total += mnote_canon_entry_count_values_impl(unsafe { (*note).entries.add(index) });
    }
    total
}

unsafe extern "C" fn exif_mnote_data_canon_get_id(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> c_uint {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return 0;
    }
    let mut entry_index = 0;
    unsafe { get_tags_impl(note, index, &mut entry_index, ptr::null_mut()) };
    if entry_index >= unsafe { (*note).count } {
        0
    } else {
        unsafe { (*(*note).entries.add(entry_index as usize)).tag as c_uint }
    }
}

unsafe extern "C" fn exif_mnote_data_canon_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return ptr::null();
    }
    let mut entry_index = 0;
    let mut subtag_index = 0;
    unsafe { get_tags_impl(note, index, &mut entry_index, &mut subtag_index) };
    if entry_index >= unsafe { (*note).count } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(entry_index as usize)).tag as c_int };
    if let Some(name) = canon_lookup_subtag_name(tag, subtag_index) {
        return name.as_ptr().cast();
    }
    if canon_has_subtags(tag)
        && unsafe { (*note).options & EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS } != 0
    {
        return ptr::null();
    }
    canon_tag_name_impl(tag).map_or(ptr::null(), |name| name.as_ptr().cast())
}

unsafe extern "C" fn exif_mnote_data_canon_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return ptr::null();
    }
    let mut entry_index = 0;
    let mut subtag_index = 0;
    unsafe { get_tags_impl(note, index, &mut entry_index, &mut subtag_index) };
    if entry_index >= unsafe { (*note).count } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(entry_index as usize)).tag as c_int };
    if let Some(title) = canon_lookup_subtag_name(tag, subtag_index) {
        return title.as_ptr().cast();
    }
    if canon_has_subtags(tag)
        && unsafe { (*note).options & EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS } != 0
    {
        return ptr::null();
    }
    canon_tag_title_impl(tag).map_or(ptr::null(), |title| title.as_ptr().cast())
}

unsafe extern "C" fn exif_mnote_data_canon_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { canon_note(note) };
    if note.is_null() {
        return ptr::null();
    }
    let mut entry_index = 0;
    unsafe { get_tags_impl(note, index, &mut entry_index, ptr::null_mut()) };
    if entry_index >= unsafe { (*note).count } {
        return ptr::null();
    }
    b"\0".as_ptr().cast()
}

pub(crate) unsafe fn identify_impl(data: *const ExifData, _entry: *const ExifEntry) -> c_int {
    if data.is_null() {
        return 0;
    }
    let make_entry = unsafe { crate::mnote::find_entry_impl(data as *mut ExifData, EXIF_TAG_MAKE) };
    if make_entry.is_null() {
        return 0;
    }

    let mut buffer = [0 as c_char; 8];
    let value = unsafe {
        crate::object::entry::exif_entry_get_value_impl(
            make_entry,
            buffer.as_mut_ptr(),
            buffer.len() as c_uint,
        )
    };
    if value.is_null() {
        return 0;
    }

    let value = unsafe { std::ffi::CStr::from_ptr(value) }.to_string_lossy();
    (value == "Canon") as c_int
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem, option: ExifDataOption) -> *mut ExifMnoteData {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let note = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifMnoteDataCanon>() as u32) }
        .cast::<ExifMnoteDataCanon>();
    if note.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        crate::mnote::base::exif_mnote_data_construct(ptr::addr_of_mut!((*note).parent), mem)
    };
    unsafe {
        (*note).parent.methods = ExifMnoteDataMethods {
            free: Some(exif_mnote_data_canon_free),
            save: Some(exif_mnote_data_canon_save),
            load: Some(exif_mnote_data_canon_load),
            set_offset: Some(exif_mnote_data_canon_set_offset),
            set_byte_order: Some(exif_mnote_data_canon_set_byte_order),
            count: Some(exif_mnote_data_canon_count),
            get_id: Some(exif_mnote_data_canon_get_id),
            get_name: Some(exif_mnote_data_canon_get_name),
            get_title: Some(exif_mnote_data_canon_get_title),
            get_description: Some(exif_mnote_data_canon_get_description),
            get_value: Some(exif_mnote_data_canon_get_value),
        };
        (*note).options = option;
    }

    unsafe { ptr::addr_of_mut!((*note).parent) }
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
        if entry.is_null() || value.is_null() {
            return ptr::null_mut();
        }
        let entry_ref = &*entry;
        match entry_ref.tag {
            MNOTE_CANON_TAG_SETTINGS_1
            | MNOTE_CANON_TAG_SETTINGS_2
            | MNOTE_CANON_TAG_CUSTOM_FUNCS => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.data.is_null() || entry_ref.size < 2 {
                    return ptr::null_mut();
                }

                let count = exif_get_short(entry_ref.data, entry_ref.order) as usize / 2;
                if subtag as usize >= count
                    || (entry_ref.size as usize) < 2 + subtag as usize * 2 + 2
                {
                    return ptr::null_mut();
                }
                if entry_ref.components != count as u64 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components, &[count as u64]),
                    );
                }

                let item_u16 =
                    exif_get_short(entry_ref.data.add(2 + subtag as usize * 2), entry_ref.order);
                let item_i16 =
                    exif_get_sshort(entry_ref.data.add(2 + subtag as usize * 2), entry_ref.order);

                let rendered = match entry_ref.tag {
                    MNOTE_CANON_TAG_SETTINGS_1 => match subtag {
                        1 => {
                            if item_u16 == 0 {
                                "Off".to_owned()
                            } else {
                                format!("{} (ms)", item_u16 as u32 * 100)
                            }
                        }
                        15 => {
                            if (item_u16 & 0xc000) == 0x4000 && item_u16 != 0x7fff {
                                format!("{}", item_u16 & !0x4000)
                            } else {
                                canon_lookup_value(SETTINGS_1_VALUES, subtag, item_u16)
                                    .map(str::to_owned)
                                    .unwrap_or_else(|| format!("0x{item_u16:04x}"))
                            }
                        }
                        22 | 23 | 24 | 35 | 36 => item_u16.to_string(),
                        25 | 26 => format!("{:.2}", apex_value_to_aperture(item_u16 as f64 / 32.0)),
                        28 => canon_lookup_bitfield(SETTINGS_1_VALUES, subtag, item_u16),
                        34 => format!("{:.2}", item_u16 as f64 / 10.0),
                        _ => canon_lookup_value(SETTINGS_1_VALUES, subtag, item_u16)
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("0x{item_u16:04x}")),
                    },
                    MNOTE_CANON_TAG_SETTINGS_2 => match subtag {
                        0 => format!("{:.3}", 2f64.powf(item_i16 as f64 / 32.0)),
                        1 => format!("{:.0}", apex_value_to_iso_speed(item_i16 as f64 / 32.0)),
                        2 | 5 | 14 | 16 => format!("{:.2} EV", item_i16 as f64 / 32.0),
                        3 | 20 => format!("{:.2}", apex_value_to_aperture(item_i16 as f64 / 32.0)),
                        4 | 21 => {
                            let shutter = apex_value_to_shutter_speed(item_i16 as f64 / 32.0);
                            if shutter < 1.0 {
                                format!("1/{:.0}", 1.0 / shutter)
                            } else {
                                format!("{shutter:.0}")
                            }
                        }
                        8 => item_i16.to_string(),
                        12 => format!("{:.2}", item_i16 as f64 / 32.0),
                        18 | 19 => format!("{item_i16} mm"),
                        28 => {
                            if item_i16 <= 0 {
                                "Off".to_owned()
                            } else {
                                format!("{} (ms)", item_i16 as i32 * 100)
                            }
                        }
                        _ => canon_lookup_value(SETTINGS_2_VALUES, subtag, item_u16)
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("0x{item_u16:04x}")),
                    },
                    _ => item_u16.to_string(),
                };
                write_str_to_buffer(value, maxlen, &rendered)
            }
            MNOTE_CANON_TAG_COLOR_INFORMATION => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.data.is_null() || entry_ref.size < 2 {
                    return ptr::null_mut();
                }
                let count = exif_get_short(entry_ref.data, entry_ref.order) as usize / 2;
                if subtag as usize >= count
                    || (entry_ref.size as usize) < 2 + subtag as usize * 2 + 2
                {
                    return ptr::null_mut();
                }
                if entry_ref.components != count as u64 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components, &[count as u64]),
                    );
                }
                let item =
                    exif_get_short(entry_ref.data.add(2 + subtag as usize * 2), entry_ref.order);
                write_str_to_buffer(value, maxlen, &format!("0x{item:04x}"))
            }
            MNOTE_CANON_TAG_FOCAL_LENGTH => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.data.is_null() || (entry_ref.size as usize) < subtag as usize * 2 + 2 {
                    return ptr::null_mut();
                }
                let item = exif_get_short(entry_ref.data.add(subtag as usize * 2), entry_ref.order);
                let rendered = match subtag {
                    1 => item.to_string(),
                    2 | 3 => format!("{:.2} mm", item as f64 * 25.4 / 1000.0),
                    _ => canon_lookup_value(FOCAL_LENGTH_VALUES, subtag, item)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("0x{item:04x}")),
                };
                write_str_to_buffer(value, maxlen, &rendered)
            }
            MNOTE_CANON_TAG_PANORAMA => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.data.is_null() || (entry_ref.size as usize) < subtag as usize * 2 + 2 {
                    return ptr::null_mut();
                }
                let item = exif_get_short(entry_ref.data.add(subtag as usize * 2), entry_ref.order);
                write_str_to_buffer(value, maxlen, &format!("0x{item:04x}"))
            }
            MNOTE_CANON_TAG_OWNER | MNOTE_CANON_TAG_IMAGE_TYPE | MNOTE_CANON_TAG_FIRMWARE => {
                if entry_ref.format != EXIF_FORMAT_ASCII || entry_ref.data.is_null() {
                    return ptr::null_mut();
                }
                if entry_ref.tag == MNOTE_CANON_TAG_OWNER && entry_ref.components != 32 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components, &[32]),
                    );
                }
                write_slice_to_buffer(
                    value,
                    maxlen,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                )
            }
            MNOTE_CANON_TAG_IMAGE_NUMBER => {
                if entry_ref.format != EXIF_FORMAT_LONG || entry_ref.size < 4 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_LONG]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components, &[1]),
                    );
                }
                let raw = exif_get_long(entry_ref.data, entry_ref.order) as u64;
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!("{:03}-{:04}", raw / 10_000, raw % 10_000),
                )
            }
            MNOTE_CANON_TAG_SERIAL_NUMBER => {
                if entry_ref.format != EXIF_FORMAT_LONG || entry_ref.size < 4 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_LONG]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components, &[1]),
                    );
                }
                let raw = exif_get_long(entry_ref.data, entry_ref.order);
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!("{:04X}-{:05}", raw >> 16, raw & 0xffff),
                )
            }
            _ => generic_mnote_value(
                entry_ref.format,
                entry_ref.components as u64,
                entry_ref.data,
                entry_ref.size as usize,
                entry_ref.order,
                value,
                maxlen,
            ),
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_description(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || match tag {
        MNOTE_CANON_TAG_SETTINGS_1
        | MNOTE_CANON_TAG_FOCAL_LENGTH
        | MNOTE_CANON_TAG_SETTINGS_2
        | MNOTE_CANON_TAG_PANORAMA
        | MNOTE_CANON_TAG_IMAGE_TYPE
        | MNOTE_CANON_TAG_FIRMWARE
        | MNOTE_CANON_TAG_IMAGE_NUMBER
        | MNOTE_CANON_TAG_OWNER
        | MNOTE_CANON_TAG_SERIAL_NUMBER
        | MNOTE_CANON_TAG_CUSTOM_FUNCS
        | MNOTE_CANON_TAG_COLOR_INFORMATION => b"\0".as_ptr().cast(),
        _ => ptr::null(),
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_name(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        canon_tag_name_impl(tag).map_or(ptr::null(), |name| name.as_ptr().cast())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_canon_tag_get_title(tag: MnoteCanonTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        canon_tag_title_impl(tag).map_or(ptr::null(), |title| title.as_ptr().cast())
    })
}
