use core::ffi::{c_char, c_int, c_long, c_uchar, c_uint, c_ulong};
use core::mem::{size_of, MaybeUninit};
use core::ptr;
use core::slice;

use std::fmt::Write;
use std::string::String;

use crate::ffi::panic_boundary;
use crate::ffi::types::*;
use crate::object::content::exif_content_get_ifd_impl;
use crate::object::data::exif_data_get_byte_order_impl;
use crate::primitives::format::exif_format_get_size_impl;
use crate::runtime::cstdio::print_line;
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_new_default_impl, exif_mem_realloc_impl,
    exif_mem_ref_impl, exif_mem_unref_impl,
};
use crate::tables::gps_ifd::exif_get_gps_tag_info;
use crate::tables::tag_table::exif_tag_get_name_in_ifd;

const EXIF_TAG_INTEROPERABILITY_VERSION: ExifTag = 0x0002;
const EXIF_TAG_IMAGE_WIDTH: ExifTag = 0x0100;
const EXIF_TAG_IMAGE_LENGTH: ExifTag = 0x0101;
const EXIF_TAG_BITS_PER_SAMPLE: ExifTag = 0x0102;
const EXIF_TAG_COMPRESSION: ExifTag = 0x0103;
const EXIF_TAG_PHOTOMETRIC_INTERPRETATION: ExifTag = 0x0106;
const EXIF_TAG_IMAGE_DESCRIPTION: ExifTag = 0x010e;
const EXIF_TAG_MAKE: ExifTag = 0x010f;
const EXIF_TAG_MODEL: ExifTag = 0x0110;
const EXIF_TAG_ORIENTATION: ExifTag = 0x0112;
const EXIF_TAG_SAMPLES_PER_PIXEL: ExifTag = 0x0115;
const EXIF_TAG_X_RESOLUTION: ExifTag = 0x011a;
const EXIF_TAG_Y_RESOLUTION: ExifTag = 0x011b;
const EXIF_TAG_PLANAR_CONFIGURATION: ExifTag = 0x011c;
const EXIF_TAG_RESOLUTION_UNIT: ExifTag = 0x0128;
const EXIF_TAG_SOFTWARE: ExifTag = 0x0131;
const EXIF_TAG_DATE_TIME: ExifTag = 0x0132;
const EXIF_TAG_ARTIST: ExifTag = 0x013b;
const EXIF_TAG_WHITE_POINT: ExifTag = 0x013e;
const EXIF_TAG_PRIMARY_CHROMATICITIES: ExifTag = 0x013f;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT: ExifTag = 0x0201;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: ExifTag = 0x0202;
const EXIF_TAG_YCBCR_SUB_SAMPLING: ExifTag = 0x0212;
const EXIF_TAG_YCBCR_POSITIONING: ExifTag = 0x0213;
const EXIF_TAG_REFERENCE_BLACK_WHITE: ExifTag = 0x0214;
const EXIF_TAG_COPYRIGHT: ExifTag = 0x8298;
const EXIF_TAG_EXPOSURE_TIME: ExifTag = 0x829a;
const EXIF_TAG_FNUMBER: ExifTag = 0x829d;
const EXIF_TAG_EXIF_IFD_POINTER: ExifTag = 0x8769;
const EXIF_TAG_EXPOSURE_PROGRAM: ExifTag = 0x8822;
const EXIF_TAG_GPS_INFO_IFD_POINTER: ExifTag = 0x8825;
const EXIF_TAG_ISO_SPEED_RATINGS: ExifTag = 0x8827;
const EXIF_TAG_SENSITIVITY_TYPE: ExifTag = 0x8830;
const EXIF_TAG_EXIF_VERSION: ExifTag = 0x9000;
const EXIF_TAG_DATE_TIME_ORIGINAL: ExifTag = 0x9003;
const EXIF_TAG_DATE_TIME_DIGITIZED: ExifTag = 0x9004;
const EXIF_TAG_COMPONENTS_CONFIGURATION: ExifTag = 0x9101;
const EXIF_TAG_COMPRESSED_BITS_PER_PIXEL: ExifTag = 0x9102;
const EXIF_TAG_SHUTTER_SPEED_VALUE: ExifTag = 0x9201;
const EXIF_TAG_APERTURE_VALUE: ExifTag = 0x9202;
const EXIF_TAG_BRIGHTNESS_VALUE: ExifTag = 0x9203;
const EXIF_TAG_EXPOSURE_BIAS_VALUE: ExifTag = 0x9204;
const EXIF_TAG_MAX_APERTURE_VALUE: ExifTag = 0x9205;
const EXIF_TAG_SUBJECT_DISTANCE: ExifTag = 0x9206;
const EXIF_TAG_METERING_MODE: ExifTag = 0x9207;
const EXIF_TAG_LIGHT_SOURCE: ExifTag = 0x9208;
const EXIF_TAG_FLASH: ExifTag = 0x9209;
const EXIF_TAG_FOCAL_LENGTH: ExifTag = 0x920a;
const EXIF_TAG_SUBJECT_AREA: ExifTag = 0x9214;
const EXIF_TAG_USER_COMMENT: ExifTag = 0x9286;
const EXIF_TAG_SUB_SEC_TIME: ExifTag = 0x9290;
const EXIF_TAG_SUB_SEC_TIME_ORIGINAL: ExifTag = 0x9291;
const EXIF_TAG_SUB_SEC_TIME_DIGITIZED: ExifTag = 0x9292;
const EXIF_TAG_XP_TITLE: ExifTag = 0x9c9b;
const EXIF_TAG_XP_COMMENT: ExifTag = 0x9c9c;
const EXIF_TAG_XP_AUTHOR: ExifTag = 0x9c9d;
const EXIF_TAG_XP_KEYWORDS: ExifTag = 0x9c9e;
const EXIF_TAG_XP_SUBJECT: ExifTag = 0x9c9f;
const EXIF_TAG_FLASH_PIX_VERSION: ExifTag = 0xa000;
const EXIF_TAG_COLOR_SPACE: ExifTag = 0xa001;
const EXIF_TAG_PIXEL_X_DIMENSION: ExifTag = 0xa002;
const EXIF_TAG_PIXEL_Y_DIMENSION: ExifTag = 0xa003;
const EXIF_TAG_INTEROPERABILITY_IFD_POINTER: ExifTag = 0xa005;
const EXIF_TAG_FLASH_ENERGY: ExifTag = 0xa20b;
const EXIF_TAG_FOCAL_PLANE_X_RESOLUTION: ExifTag = 0xa20e;
const EXIF_TAG_FOCAL_PLANE_Y_RESOLUTION: ExifTag = 0xa20f;
const EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT: ExifTag = 0xa210;
const EXIF_TAG_SUBJECT_LOCATION: ExifTag = 0xa214;
const EXIF_TAG_EXPOSURE_INDEX: ExifTag = 0xa215;
const EXIF_TAG_SENSING_METHOD: ExifTag = 0xa217;
const EXIF_TAG_FILE_SOURCE: ExifTag = 0xa300;
const EXIF_TAG_SCENE_TYPE: ExifTag = 0xa301;
const EXIF_TAG_CUSTOM_RENDERED: ExifTag = 0xa401;
const EXIF_TAG_EXPOSURE_MODE: ExifTag = 0xa402;
const EXIF_TAG_WHITE_BALANCE: ExifTag = 0xa403;
const EXIF_TAG_DIGITAL_ZOOM_RATIO: ExifTag = 0xa404;
const EXIF_TAG_FOCAL_LENGTH_IN_35MM_FILM: ExifTag = 0xa405;
const EXIF_TAG_SCENE_CAPTURE_TYPE: ExifTag = 0xa406;
const EXIF_TAG_GAIN_CONTROL: ExifTag = 0xa407;
const EXIF_TAG_CONTRAST: ExifTag = 0xa408;
const EXIF_TAG_SATURATION: ExifTag = 0xa409;
const EXIF_TAG_SHARPNESS: ExifTag = 0xa40a;
const EXIF_TAG_SUBJECT_DISTANCE_RANGE: ExifTag = 0xa40c;

const EXIF_TAG_GPS_VERSION_ID: ExifTag = 0x0000;
const EXIF_TAG_GPS_ALTITUDE_REF: ExifTag = 0x0005;
const EXIF_TAG_GPS_TIME_STAMP: ExifTag = 0x0007;

#[repr(C)]
pub(crate) struct EntryPrivate {
    ref_count: u32,
    mem: *mut ExifMem,
}

#[repr(C)]
struct tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
    tm_gmtoff: c_long,
    tm_zone: *const c_char,
}

unsafe extern "C" {
    fn time(timer: *mut c_long) -> c_long;
    fn localtime_r(timer: *const c_long, result: *mut tm) -> *mut tm;
}

#[derive(Clone, Copy)]
struct IndexedStringTable {
    tag: ExifTag,
    strings: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct IndexedValue {
    index: ExifShort,
    values: &'static [&'static str],
}

#[derive(Clone, Copy)]
struct IndexedValueTable {
    tag: ExifTag,
    entries: &'static [IndexedValue],
}

const LIST_TABLE: &[IndexedStringTable] = &[
    IndexedStringTable {
        tag: EXIF_TAG_PLANAR_CONFIGURATION,
        strings: &["Chunky format", "Planar format"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_SENSING_METHOD,
        strings: &[
            "",
            "Not defined",
            "One-chip color area sensor",
            "Two-chip color area sensor",
            "Three-chip color area sensor",
            "Color sequential area sensor",
            "",
            "Trilinear sensor",
            "Color sequential linear sensor",
        ],
    },
    IndexedStringTable {
        tag: EXIF_TAG_ORIENTATION,
        strings: &[
            "",
            "Top-left",
            "Top-right",
            "Bottom-right",
            "Bottom-left",
            "Left-top",
            "Right-top",
            "Right-bottom",
            "Left-bottom",
        ],
    },
    IndexedStringTable {
        tag: EXIF_TAG_YCBCR_POSITIONING,
        strings: &["", "Centered", "Co-sited"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_PHOTOMETRIC_INTERPRETATION,
        strings: &[
            "Reversed mono",
            "Normal mono",
            "RGB",
            "Palette",
            "",
            "CMYK",
            "YCbCr",
            "",
            "CieLAB",
        ],
    },
    IndexedStringTable {
        tag: EXIF_TAG_CUSTOM_RENDERED,
        strings: &["Normal process", "Custom process"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_EXPOSURE_MODE,
        strings: &["Auto exposure", "Manual exposure", "Auto bracket"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_WHITE_BALANCE,
        strings: &["Auto white balance", "Manual white balance"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_SCENE_CAPTURE_TYPE,
        strings: &["Standard", "Landscape", "Portrait", "Night scene"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_GAIN_CONTROL,
        strings: &[
            "Normal",
            "Low gain up",
            "High gain up",
            "Low gain down",
            "High gain down",
        ],
    },
    IndexedStringTable {
        tag: EXIF_TAG_SATURATION,
        strings: &["Normal", "Low saturation", "High saturation"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_CONTRAST,
        strings: &["Normal", "Soft", "Hard"],
    },
    IndexedStringTable {
        tag: EXIF_TAG_SHARPNESS,
        strings: &["Normal", "Soft", "Hard"],
    },
];

const LIST2_TABLE: &[IndexedValueTable] = &[
    IndexedValueTable {
        tag: EXIF_TAG_METERING_MODE,
        entries: &[
            IndexedValue { index: 0, values: &["Unknown"] },
            IndexedValue { index: 1, values: &["Average", "Avg"] },
            IndexedValue {
                index: 2,
                values: &["Center-weighted average", "Center-weight"],
            },
            IndexedValue { index: 3, values: &["Spot"] },
            IndexedValue { index: 4, values: &["Multi spot"] },
            IndexedValue { index: 5, values: &["Pattern"] },
            IndexedValue { index: 6, values: &["Partial"] },
            IndexedValue { index: 255, values: &["Other"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_COMPRESSION,
        entries: &[
            IndexedValue { index: 1, values: &["Uncompressed"] },
            IndexedValue { index: 5, values: &["LZW compression"] },
            IndexedValue { index: 6, values: &["JPEG compression"] },
            IndexedValue { index: 7, values: &["JPEG compression"] },
            IndexedValue {
                index: 8,
                values: &["Deflate/ZIP compression"],
            },
            IndexedValue { index: 32773, values: &["PackBits compression"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_LIGHT_SOURCE,
        entries: &[
            IndexedValue { index: 0, values: &["Unknown"] },
            IndexedValue { index: 1, values: &["Daylight"] },
            IndexedValue { index: 2, values: &["Fluorescent"] },
            IndexedValue {
                index: 3,
                values: &["Tungsten incandescent light", "Tungsten"],
            },
            IndexedValue { index: 4, values: &["Flash"] },
            IndexedValue { index: 9, values: &["Fine weather"] },
            IndexedValue { index: 10, values: &["Cloudy weather", "Cloudy"] },
            IndexedValue { index: 11, values: &["Shade"] },
            IndexedValue { index: 12, values: &["Daylight fluorescent"] },
            IndexedValue { index: 13, values: &["Day white fluorescent"] },
            IndexedValue { index: 14, values: &["Cool white fluorescent"] },
            IndexedValue { index: 15, values: &["White fluorescent"] },
            IndexedValue { index: 17, values: &["Standard light A"] },
            IndexedValue { index: 18, values: &["Standard light B"] },
            IndexedValue { index: 19, values: &["Standard light C"] },
            IndexedValue { index: 20, values: &["D55"] },
            IndexedValue { index: 21, values: &["D65"] },
            IndexedValue { index: 22, values: &["D75"] },
            IndexedValue { index: 24, values: &["ISO studio tungsten"] },
            IndexedValue { index: 255, values: &["Other"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT,
        entries: &[
            IndexedValue { index: 2, values: &["Inch", "in"] },
            IndexedValue { index: 3, values: &["Centimeter", "cm"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_RESOLUTION_UNIT,
        entries: &[
            IndexedValue { index: 2, values: &["Inch", "in"] },
            IndexedValue { index: 3, values: &["Centimeter", "cm"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_EXPOSURE_PROGRAM,
        entries: &[
            IndexedValue { index: 0, values: &["Not defined"] },
            IndexedValue { index: 1, values: &["Manual"] },
            IndexedValue { index: 2, values: &["Normal program", "Normal"] },
            IndexedValue { index: 3, values: &["Aperture priority", "Aperture"] },
            IndexedValue { index: 4, values: &["Shutter priority", "Shutter"] },
            IndexedValue {
                index: 5,
                values: &[
                    "Creative program (biased toward depth of field)",
                    "Creative",
                ],
            },
            IndexedValue {
                index: 6,
                values: &[
                    "Creative program (biased toward fast shutter speed)",
                    "Action",
                ],
            },
            IndexedValue {
                index: 7,
                values: &[
                    "Portrait mode (for closeup photos with the background out of focus)",
                    "Portrait",
                ],
            },
            IndexedValue {
                index: 8,
                values: &[
                    "Landscape mode (for landscape photos with the background in focus)",
                    "Landscape",
                ],
            },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_SENSITIVITY_TYPE,
        entries: &[
            IndexedValue { index: 0, values: &["Unknown"] },
            IndexedValue {
                index: 1,
                values: &["Standard output sensitivity (SOS)"],
            },
            IndexedValue {
                index: 2,
                values: &["Recommended exposure index (REI)"],
            },
            IndexedValue { index: 3, values: &["ISO speed"] },
            IndexedValue {
                index: 4,
                values: &["Standard output sensitivity (SOS) and recommended exposure index (REI)"],
            },
            IndexedValue {
                index: 5,
                values: &["Standard output sensitivity (SOS) and ISO speed"],
            },
            IndexedValue {
                index: 6,
                values: &["Recommended exposure index (REI) and ISO speed"],
            },
            IndexedValue {
                index: 7,
                values: &["Standard output sensitivity (SOS) and recommended exposure index (REI) and ISO speed"],
            },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_FLASH,
        entries: &[
            IndexedValue {
                index: 0x0000,
                values: &["Flash did not fire", "No flash"],
            },
            IndexedValue {
                index: 0x0001,
                values: &["Flash fired", "Flash", "Yes"],
            },
            IndexedValue {
                index: 0x0005,
                values: &["Strobe return light not detected", "Without strobe"],
            },
            IndexedValue {
                index: 0x0007,
                values: &["Strobe return light detected", "With strobe"],
            },
            IndexedValue {
                index: 0x0008,
                values: &["Flash did not fire"],
            },
            IndexedValue {
                index: 0x0009,
                values: &["Flash fired, compulsory flash mode"],
            },
            IndexedValue {
                index: 0x000d,
                values: &["Flash fired, compulsory flash mode, return light not detected"],
            },
            IndexedValue {
                index: 0x000f,
                values: &["Flash fired, compulsory flash mode, return light detected"],
            },
            IndexedValue {
                index: 0x0010,
                values: &["Flash did not fire, compulsory flash mode"],
            },
            IndexedValue {
                index: 0x0018,
                values: &["Flash did not fire, auto mode"],
            },
            IndexedValue {
                index: 0x0019,
                values: &["Flash fired, auto mode"],
            },
            IndexedValue {
                index: 0x001d,
                values: &["Flash fired, auto mode, return light not detected"],
            },
            IndexedValue {
                index: 0x001f,
                values: &["Flash fired, auto mode, return light detected"],
            },
            IndexedValue {
                index: 0x0020,
                values: &["No flash function"],
            },
            IndexedValue {
                index: 0x0041,
                values: &["Flash fired, red-eye reduction mode"],
            },
            IndexedValue {
                index: 0x0045,
                values: &["Flash fired, red-eye reduction mode, return light not detected"],
            },
            IndexedValue {
                index: 0x0047,
                values: &["Flash fired, red-eye reduction mode, return light detected"],
            },
            IndexedValue {
                index: 0x0049,
                values: &["Flash fired, compulsory flash mode, red-eye reduction mode"],
            },
            IndexedValue {
                index: 0x004d,
                values: &["Flash fired, compulsory flash mode, red-eye reduction mode, return light not detected"],
            },
            IndexedValue {
                index: 0x004f,
                values: &["Flash fired, compulsory flash mode, red-eye reduction mode, return light detected"],
            },
            IndexedValue {
                index: 0x0058,
                values: &["Flash did not fire, auto mode, red-eye reduction mode"],
            },
            IndexedValue {
                index: 0x0059,
                values: &["Flash fired, auto mode, red-eye reduction mode"],
            },
            IndexedValue {
                index: 0x005d,
                values: &["Flash fired, auto mode, return light not detected, red-eye reduction mode"],
            },
            IndexedValue {
                index: 0x005f,
                values: &["Flash fired, auto mode, return light detected, red-eye reduction mode"],
            },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_SUBJECT_DISTANCE_RANGE,
        entries: &[
            IndexedValue { index: 0, values: &["Unknown", "?"] },
            IndexedValue { index: 1, values: &["Macro"] },
            IndexedValue { index: 2, values: &["Close view", "Close"] },
            IndexedValue { index: 3, values: &["Distant view", "Distant"] },
        ],
    },
    IndexedValueTable {
        tag: EXIF_TAG_COLOR_SPACE,
        entries: &[
            IndexedValue { index: 1, values: &["sRGB"] },
            IndexedValue { index: 2, values: &["Adobe RGB"] },
            IndexedValue { index: 0xffff, values: &["Uncalibrated"] },
        ],
    },
];

#[inline]
unsafe fn entry_private(entry: *mut ExifEntry) -> *mut EntryPrivate {
    unsafe { (*entry).priv_.cast::<EntryPrivate>() }
}

#[inline]
unsafe fn entry_mem(entry: *mut ExifEntry) -> *mut ExifMem {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*entry_private(entry)).mem }
    }
}

fn clear_entry(entry: *mut ExifEntry) {
    unsafe {
        (*entry).components = 0;
        (*entry).size = 0;
    }
}

unsafe fn entry_alloc(entry: *mut ExifEntry, size: c_uint) -> *mut c_uchar {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() || size == 0 {
        return ptr::null_mut();
    }

    let data = unsafe { exif_mem_alloc_impl(entry_mem(entry), size) }.cast::<c_uchar>();
    if !data.is_null() {
        unsafe { ptr::write_bytes(data, 0, size as usize) };
    }
    data
}

unsafe fn entry_realloc(entry: *mut ExifEntry, data: *mut c_uchar, size: c_uint) -> *mut c_uchar {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        return ptr::null_mut();
    }
    if size == 0 {
        unsafe { exif_mem_free_impl(entry_mem(entry), data.cast()) };
        return ptr::null_mut();
    }
    unsafe { exif_mem_realloc_impl(entry_mem(entry), data.cast(), size) }.cast::<c_uchar>()
}

pub(crate) unsafe fn exif_entry_new_mem_impl(mem: *mut ExifMem) -> *mut ExifEntry {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let entry =
        unsafe { exif_mem_alloc_impl(mem, size_of::<ExifEntry>() as ExifLong) }.cast::<ExifEntry>();
    if entry.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(entry.cast::<u8>(), 0, size_of::<ExifEntry>());
    }

    let private = unsafe { exif_mem_alloc_impl(mem, size_of::<EntryPrivate>() as ExifLong) }
        .cast::<EntryPrivate>();
    if private.is_null() {
        unsafe { exif_mem_free_impl(mem, entry.cast()) };
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(private.cast::<u8>(), 0, size_of::<EntryPrivate>());
        (*private).ref_count = 1;
        (*private).mem = mem;
        (*entry).priv_ = private.cast();
        exif_mem_ref_impl(mem);
    }

    entry
}

pub(crate) unsafe fn exif_entry_new_impl() -> *mut ExifEntry {
    let mem = unsafe { exif_mem_new_default_impl() };
    let entry = unsafe { exif_entry_new_mem_impl(mem) };
    unsafe { exif_mem_unref_impl(mem) };
    entry
}

pub(crate) unsafe fn exif_entry_ref_impl(entry: *mut ExifEntry) {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *entry_private(entry) };
    private.ref_count = private.ref_count.wrapping_add(1);
}

pub(crate) unsafe fn exif_entry_unref_impl(entry: *mut ExifEntry) {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *entry_private(entry) };
    private.ref_count = private.ref_count.wrapping_sub(1);
    if private.ref_count == 0 {
        unsafe { exif_entry_free_impl(entry) };
    }
}

pub(crate) unsafe fn exif_entry_free_impl(entry: *mut ExifEntry) {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        return;
    }

    let mem = unsafe { entry_mem(entry) };
    if !unsafe { (*entry).data }.is_null() {
        unsafe { exif_mem_free_impl(mem, (*entry).data.cast()) };
    }
    unsafe {
        exif_mem_free_impl(mem, (*entry).priv_.cast());
        exif_mem_free_impl(mem, entry.cast());
        exif_mem_unref_impl(mem);
    }
}

fn get_short_convert(
    buffer: *const c_uchar,
    format: ExifFormat,
    order: ExifByteOrder,
) -> ExifShort {
    match format {
        EXIF_FORMAT_LONG => unsafe {
            crate::primitives::utils::exif_get_long(buffer, order) as ExifShort
        },
        EXIF_FORMAT_SLONG => unsafe {
            crate::primitives::utils::exif_get_slong(buffer, order) as ExifShort
        },
        EXIF_FORMAT_SHORT => unsafe {
            crate::primitives::utils::exif_get_short(buffer, order) as ExifShort
        },
        EXIF_FORMAT_SSHORT => unsafe {
            crate::primitives::utils::exif_get_sshort(buffer, order) as ExifShort
        },
        EXIF_FORMAT_BYTE | EXIF_FORMAT_SBYTE => {
            if buffer.is_null() {
                0
            } else {
                unsafe { *buffer as ExifShort }
            }
        }
        _ => 0,
    }
}

pub(crate) unsafe fn exif_entry_fix_impl(entry: *mut ExifEntry) {
    if entry.is_null() || unsafe { (*entry).priv_ }.is_null() {
        return;
    }

    match unsafe { (*entry).tag } {
        EXIF_TAG_YCBCR_SUB_SAMPLING
        | EXIF_TAG_SUBJECT_AREA
        | EXIF_TAG_COLOR_SPACE
        | EXIF_TAG_PLANAR_CONFIGURATION
        | EXIF_TAG_SENSING_METHOD
        | EXIF_TAG_ORIENTATION
        | EXIF_TAG_YCBCR_POSITIONING
        | EXIF_TAG_PHOTOMETRIC_INTERPRETATION
        | EXIF_TAG_CUSTOM_RENDERED
        | EXIF_TAG_EXPOSURE_MODE
        | EXIF_TAG_WHITE_BALANCE
        | EXIF_TAG_SCENE_CAPTURE_TYPE
        | EXIF_TAG_GAIN_CONTROL
        | EXIF_TAG_SATURATION
        | EXIF_TAG_CONTRAST
        | EXIF_TAG_SHARPNESS
        | EXIF_TAG_ISO_SPEED_RATINGS => match unsafe { (*entry).format } {
            EXIF_FORMAT_LONG | EXIF_FORMAT_SLONG | EXIF_FORMAT_BYTE | EXIF_FORMAT_SBYTE
            | EXIF_FORMAT_SSHORT => {
                if unsafe { (*entry).parent }.is_null()
                    || unsafe { (*(*entry).parent).parent }.is_null()
                {
                    return;
                }
                let order = unsafe { exif_data_get_byte_order_impl((*(*entry).parent).parent) };
                let new_size = unsafe { (*entry).components as usize }
                    .saturating_mul(exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize);
                let new_data = unsafe { entry_alloc(entry, new_size as c_uint) };
                if new_data.is_null() {
                    return;
                }
                for index in 0..unsafe { (*entry).components as usize } {
                    let source = unsafe {
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl((*entry).format) as usize)
                    };
                    unsafe {
                        crate::primitives::utils::exif_set_short(
                            new_data
                                .add(index * exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize),
                            order,
                            get_short_convert(source.cast_const(), (*entry).format, order),
                        );
                    }
                }
                unsafe {
                    exif_mem_free_impl(entry_mem(entry), (*entry).data.cast());
                    (*entry).data = new_data;
                    (*entry).size = new_size as c_uint;
                    (*entry).format = EXIF_FORMAT_SHORT;
                }
            }
            _ => {}
        },
        EXIF_TAG_FNUMBER
        | EXIF_TAG_APERTURE_VALUE
        | EXIF_TAG_EXPOSURE_TIME
        | EXIF_TAG_FOCAL_LENGTH => {
            if unsafe { (*entry).format } == EXIF_FORMAT_SRATIONAL
                && !unsafe { (*entry).parent }.is_null()
                && !unsafe { (*(*entry).parent).parent }.is_null()
            {
                let order = unsafe { exif_data_get_byte_order_impl((*(*entry).parent).parent) };
                for index in 0..unsafe { (*entry).components as usize } {
                    let sr = unsafe {
                        crate::primitives::utils::exif_get_srational(
                            (*entry)
                                .data
                                .add(
                                    index
                                        * exif_format_get_size_impl(EXIF_FORMAT_SRATIONAL) as usize,
                                )
                                .cast_const(),
                            order,
                        )
                    };
                    let r = ExifRational {
                        numerator: sr.numerator as ExifLong,
                        denominator: sr.denominator as ExifLong,
                    };
                    unsafe {
                        crate::primitives::utils::exif_set_rational(
                            (*entry).data.add(
                                index * exif_format_get_size_impl(EXIF_FORMAT_RATIONAL) as usize,
                            ),
                            order,
                            r,
                        );
                    }
                }
                unsafe {
                    (*entry).format = EXIF_FORMAT_RATIONAL;
                }
            }
        }
        EXIF_TAG_EXPOSURE_BIAS_VALUE | EXIF_TAG_BRIGHTNESS_VALUE | EXIF_TAG_SHUTTER_SPEED_VALUE => {
            if unsafe { (*entry).format } == EXIF_FORMAT_RATIONAL
                && !unsafe { (*entry).parent }.is_null()
                && !unsafe { (*(*entry).parent).parent }.is_null()
            {
                let order = unsafe { exif_data_get_byte_order_impl((*(*entry).parent).parent) };
                for index in 0..unsafe { (*entry).components as usize } {
                    let r = unsafe {
                        crate::primitives::utils::exif_get_rational(
                            (*entry)
                                .data
                                .add(
                                    index
                                        * exif_format_get_size_impl(EXIF_FORMAT_RATIONAL) as usize,
                                )
                                .cast_const(),
                            order,
                        )
                    };
                    let sr = ExifSRational {
                        numerator: r.numerator as ExifSLong,
                        denominator: r.denominator as ExifSLong,
                    };
                    unsafe {
                        crate::primitives::utils::exif_set_srational(
                            (*entry).data.add(
                                index * exif_format_get_size_impl(EXIF_FORMAT_SRATIONAL) as usize,
                            ),
                            order,
                            sr,
                        );
                    }
                }
                unsafe {
                    (*entry).format = EXIF_FORMAT_SRATIONAL;
                }
            }
        }
        EXIF_TAG_USER_COMMENT => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED {
                unsafe {
                    (*entry).format = EXIF_FORMAT_UNDEFINED;
                }
            }

            if unsafe { (*entry).size } >= 8
                && !unsafe { (*entry).data }.is_null()
                && unsafe { *(*entry).data } == 0
            {
                unsafe {
                    ptr::copy_nonoverlapping(b"\0\0\0\0\0\0\0\0".as_ptr(), (*entry).data, 8);
                }
            }

            if unsafe { (*entry).size } < 8 {
                let old_size = unsafe { (*entry).size };
                let old_data = unsafe { (*entry).data };
                let new_data = unsafe { entry_realloc(entry, old_data, 8 + old_size) };
                unsafe { (*entry).data = new_data };
                if unsafe { (*entry).data }.is_null() {
                    clear_entry(entry);
                    return;
                }
                unsafe {
                    ptr::copy((*entry).data, (*entry).data.add(8), old_size as usize);
                    ptr::copy_nonoverlapping(b"ASCII\0\0\0".as_ptr(), (*entry).data, 8);
                    (*entry).size += 8;
                    (*entry).components += 8;
                }
                return;
            }

            if !unsafe { (*entry).data }.is_null() {
                let bytes = unsafe { slice::from_raw_parts((*entry).data, (*entry).size as usize) };
                let mut index = 0usize;
                while index < bytes.len() && bytes[index] == 0 {
                    index += 1;
                }
                if index == 0 {
                    while index < bytes.len() && bytes[index] == b' ' {
                        index += 1;
                    }
                }
                if index >= 8 && index < bytes.len() {
                    unsafe {
                        ptr::copy_nonoverlapping(b"ASCII\0\0\0".as_ptr(), (*entry).data, 8);
                    }
                    return;
                }

                let prefix = &bytes[..8];
                if prefix != b"ASCII\0\0\0"
                    && prefix != b"UNICODE\0"
                    && prefix != b"JIS\0\0\0\0\0"
                    && prefix != b"\0\0\0\0\0\0\0\0"
                {
                    let old_size = unsafe { (*entry).size };
                    let old_data = unsafe { (*entry).data };
                    let new_data = unsafe { entry_realloc(entry, old_data, 8 + old_size) };
                    unsafe { (*entry).data = new_data };
                    if unsafe { (*entry).data }.is_null() {
                        clear_entry(entry);
                        return;
                    }
                    unsafe {
                        ptr::copy((*entry).data, (*entry).data.add(8), old_size as usize);
                        ptr::copy_nonoverlapping(b"ASCII\0\0\0".as_ptr(), (*entry).data, 8);
                        (*entry).size += 8;
                        (*entry).components += 8;
                    }
                }
            }
        }
        _ => {}
    }
}

fn date_time_now_string() -> String {
    let mut timer = 0 as c_long;
    let now = unsafe { time(&mut timer) };
    let mut storage = MaybeUninit::<tm>::zeroed();
    let tm = unsafe { localtime_r(&now, storage.as_mut_ptr()) };
    if tm.is_null() {
        return "0000:00:00 00:00:00".to_owned();
    }
    let tm = unsafe { storage.assume_init() };
    format!(
        "{:04}:{:02}:{:02} {:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

fn allocate_initialized(entry: *mut ExifEntry, format: ExifFormat, components: c_ulong) -> bool {
    unsafe {
        (*entry).format = format;
        (*entry).components = components;
        (*entry).size = (exif_format_get_size_impl(format) as usize)
            .saturating_mul(components as usize) as c_uint;
        (*entry).data = entry_alloc(entry, (*entry).size);
        !(*entry).data.is_null()
    }
}

unsafe fn initialize_gps_entry(entry: *mut ExifEntry, tag: ExifTag) {
    let Some(info) = exif_get_gps_tag_info(tag) else {
        unsafe {
            (*entry).components = 0;
            (*entry).format = EXIF_FORMAT_UNDEFINED;
            (*entry).size = 0;
            (*entry).data = ptr::null_mut();
        }
        return;
    };

    unsafe {
        (*entry).format = info.format;
        (*entry).components = info.components as c_ulong;
        if info.components == 0 {
            (*entry).size = 0;
            (*entry).data = ptr::null_mut();
            return;
        }
        let has_default = info.default_size != 0 && info.default_value.is_some();
        let alloc_size = if has_default {
            info.default_size as c_uint
        } else {
            (exif_format_get_size_impl(info.format) as c_uint) * info.components as c_uint
        };
        (*entry).size = alloc_size;
        (*entry).data = entry_alloc(entry, alloc_size);
        if (*entry).data.is_null() {
            clear_entry(entry);
            return;
        }
        if let Some(default_value) = info.default_value {
            ptr::copy_nonoverlapping(
                default_value.as_ptr(),
                (*entry).data,
                info.default_size as usize,
            );
        }
    }
}

pub(crate) unsafe fn exif_entry_initialize_impl(entry: *mut ExifEntry, tag: ExifTag) {
    if entry.is_null()
        || unsafe { (*entry).parent }.is_null()
        || !unsafe { (*entry).data }.is_null()
        || unsafe { (*(*entry).parent).parent }.is_null()
    {
        return;
    }

    let order = unsafe { exif_data_get_byte_order_impl((*(*entry).parent).parent) };
    unsafe {
        (*entry).tag = tag;
    }

    if unsafe { exif_content_get_ifd_impl((*entry).parent) } == EXIF_IFD_GPS {
        unsafe { initialize_gps_entry(entry, tag) };
        return;
    }

    match tag {
        EXIF_TAG_PIXEL_X_DIMENSION
        | EXIF_TAG_PIXEL_Y_DIMENSION
        | EXIF_TAG_EXIF_IFD_POINTER
        | EXIF_TAG_GPS_INFO_IFD_POINTER
        | EXIF_TAG_INTEROPERABILITY_IFD_POINTER
        | EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH
        | EXIF_TAG_JPEG_INTERCHANGE_FORMAT => {
            if !allocate_initialized(entry, EXIF_FORMAT_LONG, 1) {
                clear_entry(entry);
            }
        }
        EXIF_TAG_SUBJECT_LOCATION
        | EXIF_TAG_SENSING_METHOD
        | EXIF_TAG_PHOTOMETRIC_INTERPRETATION
        | EXIF_TAG_COMPRESSION
        | EXIF_TAG_EXPOSURE_MODE
        | EXIF_TAG_WHITE_BALANCE
        | EXIF_TAG_FOCAL_LENGTH_IN_35MM_FILM
        | EXIF_TAG_GAIN_CONTROL
        | EXIF_TAG_SUBJECT_DISTANCE_RANGE
        | EXIF_TAG_FLASH
        | EXIF_TAG_ISO_SPEED_RATINGS
        | EXIF_TAG_SENSITIVITY_TYPE
        | EXIF_TAG_IMAGE_WIDTH
        | EXIF_TAG_IMAGE_LENGTH
        | EXIF_TAG_EXPOSURE_PROGRAM
        | EXIF_TAG_LIGHT_SOURCE
        | EXIF_TAG_METERING_MODE
        | EXIF_TAG_CUSTOM_RENDERED
        | EXIF_TAG_SCENE_CAPTURE_TYPE
        | EXIF_TAG_CONTRAST
        | EXIF_TAG_SATURATION
        | EXIF_TAG_SHARPNESS => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 1) {
                clear_entry(entry);
                return;
            }
            let default = match tag {
                EXIF_TAG_ORIENTATION
                | EXIF_TAG_PLANAR_CONFIGURATION
                | EXIF_TAG_YCBCR_POSITIONING => 1,
                EXIF_TAG_RESOLUTION_UNIT | EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT => 2,
                EXIF_TAG_SAMPLES_PER_PIXEL => 3,
                EXIF_TAG_COLOR_SPACE => 0xffff,
                _ => 0,
            };
            unsafe {
                crate::primitives::utils::exif_set_short((*entry).data, order, default);
            }
        }
        EXIF_TAG_ORIENTATION | EXIF_TAG_PLANAR_CONFIGURATION | EXIF_TAG_YCBCR_POSITIONING => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { crate::primitives::utils::exif_set_short((*entry).data, order, 1) };
        }
        EXIF_TAG_RESOLUTION_UNIT | EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { crate::primitives::utils::exif_set_short((*entry).data, order, 2) };
        }
        EXIF_TAG_SAMPLES_PER_PIXEL => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { crate::primitives::utils::exif_set_short((*entry).data, order, 3) };
        }
        EXIF_TAG_COLOR_SPACE => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { crate::primitives::utils::exif_set_short((*entry).data, order, 0xffff) };
        }
        EXIF_TAG_BITS_PER_SAMPLE => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 3) {
                clear_entry(entry);
                return;
            }
            for offset in 0..3usize {
                unsafe {
                    crate::primitives::utils::exif_set_short(
                        (*entry)
                            .data
                            .add(offset * exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize),
                        order,
                        8,
                    );
                }
            }
        }
        EXIF_TAG_YCBCR_SUB_SAMPLING => {
            if !allocate_initialized(entry, EXIF_FORMAT_SHORT, 2) {
                clear_entry(entry);
                return;
            }
            unsafe {
                crate::primitives::utils::exif_set_short((*entry).data, order, 2);
                crate::primitives::utils::exif_set_short(
                    (*entry)
                        .data
                        .add(exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize),
                    order,
                    1,
                );
            }
        }
        EXIF_TAG_EXPOSURE_BIAS_VALUE | EXIF_TAG_BRIGHTNESS_VALUE | EXIF_TAG_SHUTTER_SPEED_VALUE => {
            if !allocate_initialized(entry, EXIF_FORMAT_SRATIONAL, 1) {
                clear_entry(entry);
            }
        }
        EXIF_TAG_EXPOSURE_TIME
        | EXIF_TAG_FOCAL_PLANE_X_RESOLUTION
        | EXIF_TAG_FOCAL_PLANE_Y_RESOLUTION
        | EXIF_TAG_EXPOSURE_INDEX
        | EXIF_TAG_FLASH_ENERGY
        | EXIF_TAG_FNUMBER
        | EXIF_TAG_FOCAL_LENGTH
        | EXIF_TAG_SUBJECT_DISTANCE
        | EXIF_TAG_MAX_APERTURE_VALUE
        | EXIF_TAG_APERTURE_VALUE
        | EXIF_TAG_COMPRESSED_BITS_PER_PIXEL
        | EXIF_TAG_PRIMARY_CHROMATICITIES
        | EXIF_TAG_DIGITAL_ZOOM_RATIO => {
            if !allocate_initialized(entry, EXIF_FORMAT_RATIONAL, 1) {
                clear_entry(entry);
            }
        }
        EXIF_TAG_X_RESOLUTION | EXIF_TAG_Y_RESOLUTION => {
            if !allocate_initialized(entry, EXIF_FORMAT_RATIONAL, 1) {
                clear_entry(entry);
                return;
            }
            unsafe {
                crate::primitives::utils::exif_set_rational(
                    (*entry).data,
                    order,
                    ExifRational {
                        numerator: 72,
                        denominator: 1,
                    },
                );
            }
        }
        EXIF_TAG_WHITE_POINT => {
            if !allocate_initialized(entry, EXIF_FORMAT_RATIONAL, 2) {
                clear_entry(entry);
            }
        }
        EXIF_TAG_REFERENCE_BLACK_WHITE => {
            if !allocate_initialized(entry, EXIF_FORMAT_RATIONAL, 6) {
                clear_entry(entry);
                return;
            }
            let values = [0, 255, 0, 255, 0, 255];
            for (index, value) in values.into_iter().enumerate() {
                unsafe {
                    crate::primitives::utils::exif_set_rational(
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl(EXIF_FORMAT_RATIONAL) as usize),
                        order,
                        ExifRational {
                            numerator: value,
                            denominator: 1,
                        },
                    );
                }
            }
        }
        EXIF_TAG_DATE_TIME | EXIF_TAG_DATE_TIME_ORIGINAL | EXIF_TAG_DATE_TIME_DIGITIZED => {
            unsafe {
                (*entry).components = 20;
                (*entry).format = EXIF_FORMAT_ASCII;
                (*entry).size = 20;
                (*entry).data = entry_alloc(entry, 20);
            }
            if unsafe { (*entry).data }.is_null() {
                clear_entry(entry);
                return;
            }
            let value = date_time_now_string();
            unsafe {
                ptr::copy_nonoverlapping(
                    value.as_bytes().as_ptr(),
                    (*entry).data,
                    value.len().min(19),
                );
            }
        }
        EXIF_TAG_SUB_SEC_TIME
        | EXIF_TAG_SUB_SEC_TIME_ORIGINAL
        | EXIF_TAG_SUB_SEC_TIME_DIGITIZED => unsafe {
            (*entry).components = 0;
            (*entry).format = EXIF_FORMAT_ASCII;
            (*entry).size = 0;
            (*entry).data = ptr::null_mut();
        },
        EXIF_TAG_IMAGE_DESCRIPTION
        | EXIF_TAG_MAKE
        | EXIF_TAG_MODEL
        | EXIF_TAG_SOFTWARE
        | EXIF_TAG_ARTIST => {
            let none = b"[None]\0";
            unsafe {
                (*entry).components = none.len() as c_ulong;
                (*entry).format = EXIF_FORMAT_ASCII;
                (*entry).size = none.len() as c_uint;
                (*entry).data = entry_alloc(entry, none.len() as c_uint);
            }
            if unsafe { (*entry).data }.is_null() {
                clear_entry(entry);
                return;
            }
            unsafe { ptr::copy_nonoverlapping(none.as_ptr(), (*entry).data, none.len()) };
        }
        EXIF_TAG_COPYRIGHT => {
            let none = b"[None]\0";
            let size = none.len() * 2;
            unsafe {
                (*entry).components = size as c_ulong;
                (*entry).format = EXIF_FORMAT_ASCII;
                (*entry).size = size as c_uint;
                (*entry).data = entry_alloc(entry, size as c_uint);
            }
            if unsafe { (*entry).data }.is_null() {
                clear_entry(entry);
                return;
            }
            unsafe {
                ptr::copy_nonoverlapping(none.as_ptr(), (*entry).data, none.len());
                ptr::copy_nonoverlapping(none.as_ptr(), (*entry).data.add(none.len()), none.len());
            }
        }
        EXIF_TAG_SCENE_TYPE => {
            if !allocate_initialized(entry, EXIF_FORMAT_UNDEFINED, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { *(*entry).data = 0x01 };
        }
        EXIF_TAG_FILE_SOURCE => {
            if !allocate_initialized(entry, EXIF_FORMAT_UNDEFINED, 1) {
                clear_entry(entry);
                return;
            }
            unsafe { *(*entry).data = 0x03 };
        }
        EXIF_TAG_FLASH_PIX_VERSION => {
            if !allocate_initialized(entry, EXIF_FORMAT_UNDEFINED, 4) {
                clear_entry(entry);
                return;
            }
            unsafe { ptr::copy_nonoverlapping(b"0100".as_ptr(), (*entry).data, 4) };
        }
        EXIF_TAG_EXIF_VERSION => {
            if !allocate_initialized(entry, EXIF_FORMAT_UNDEFINED, 4) {
                clear_entry(entry);
                return;
            }
            unsafe { ptr::copy_nonoverlapping(b"0210".as_ptr(), (*entry).data, 4) };
        }
        EXIF_TAG_COMPONENTS_CONFIGURATION => {
            if !allocate_initialized(entry, EXIF_FORMAT_UNDEFINED, 4) {
                clear_entry(entry);
                return;
            }
            unsafe {
                *(*entry).data.add(0) = 1;
                *(*entry).data.add(1) = 2;
                *(*entry).data.add(2) = 3;
                *(*entry).data.add(3) = 0;
            }
        }
        _ => unsafe {
            (*entry).components = 0;
            (*entry).format = EXIF_FORMAT_UNDEFINED;
            (*entry).size = 0;
            (*entry).data = ptr::null_mut();
        },
    }
}

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn bytes_until_nul(bytes: &[u8]) -> &[u8] {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..end]
}

fn data_bytes(entry: *mut ExifEntry) -> &'static [u8] {
    unsafe {
        if entry.is_null() || (*entry).data.is_null() || (*entry).size == 0 {
            &[]
        } else {
            slice::from_raw_parts((*entry).data, (*entry).size as usize)
        }
    }
}

fn match_repeated_char(data: &[u8], ch: u8) -> bool {
    for &byte in data {
        if byte == 0 {
            return true;
        }
        if byte != ch {
            return false;
        }
    }
    true
}

fn choose_best_fit(values: &[&str], maxlen: c_uint) -> Option<String> {
    if let Some(selected) = choose_best_fit_static(values, maxlen) {
        return Some(selected.to_owned());
    }

    let mut selected: Option<&str> = None;
    for value in values {
        let length = value.len();
        if (maxlen as usize) > length && selected.map_or(true, |current| current.len() < length) {
            selected = Some(value);
        }
    }
    selected.map(ToOwned::to_owned)
}

fn choose_best_fit_static<'a>(values: &'a [&'a str], maxlen: c_uint) -> Option<&'a str> {
    let mut selected: Option<&str> = None;
    for value in values {
        let length = value.len();
        if (maxlen as usize) > length && selected.map_or(true, |current| current.len() < length) {
            selected = Some(value);
        }
    }
    selected
}

fn format_c_float_with_min_width(value: f64, decimals: usize) -> String {
    format!(
        "{:>width$.precision$}",
        value,
        width = 2,
        precision = decimals
    )
}

fn generic_format_value(entry: *mut ExifEntry, order: ExifByteOrder) -> String {
    let bytes = data_bytes(entry);
    if bytes.is_empty() {
        return String::new();
    }

    let components = unsafe { (*entry).components as usize };
    match unsafe { (*entry).format } {
        EXIF_FORMAT_UNDEFINED => format!("{} bytes undefined data", unsafe { (*entry).size }),
        EXIF_FORMAT_BYTE | EXIF_FORMAT_SBYTE => {
            let mut out = String::with_capacity(components.saturating_mul(6));
            for (index, byte) in bytes.iter().take(components).enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                let _ = write!(out, "0x{byte:02x}");
            }
            out
        }
        EXIF_FORMAT_SHORT => {
            let mut out = String::with_capacity(components.saturating_mul(8));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_short(
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize)
                            .cast_const(),
                        order,
                    )
                };
                let _ = write!(out, "{value}");
            }
            out
        }
        EXIF_FORMAT_SSHORT => {
            let mut out = String::with_capacity(components.saturating_mul(8));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_sshort(
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl(EXIF_FORMAT_SSHORT) as usize)
                            .cast_const(),
                        order,
                    )
                };
                let _ = write!(out, "{value}");
            }
            out
        }
        EXIF_FORMAT_LONG => {
            let mut out = String::with_capacity(components.saturating_mul(12));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_long(
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl(EXIF_FORMAT_LONG) as usize)
                            .cast_const(),
                        order,
                    )
                };
                let _ = write!(out, "{value}");
            }
            out
        }
        EXIF_FORMAT_SLONG => {
            let mut out = String::with_capacity(components.saturating_mul(12));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_slong(
                        (*entry)
                            .data
                            .add(index * exif_format_get_size_impl(EXIF_FORMAT_SLONG) as usize)
                            .cast_const(),
                        order,
                    )
                };
                let _ = write!(out, "{value}");
            }
            out
        }
        EXIF_FORMAT_ASCII => bytes_to_string(bytes_until_nul(bytes)),
        EXIF_FORMAT_RATIONAL => {
            let mut out = String::with_capacity(components.saturating_mul(18));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_rational(
                        (*entry).data.add(8 * index).cast_const(),
                        order,
                    )
                };
                if value.denominator != 0 {
                    let decimals = ((value.denominator as f64).log10() - 0.08 + 1.0)
                        .max(0.0)
                        .floor() as usize;
                    out.push_str(&format_c_float_with_min_width(
                        value.numerator as f64 / value.denominator as f64,
                        decimals,
                    ));
                } else {
                    let _ = write!(out, "{}/{}", value.numerator, value.denominator);
                }
            }
            out
        }
        EXIF_FORMAT_SRATIONAL => {
            let mut out = String::with_capacity(components.saturating_mul(18));
            for index in 0..components {
                if index > 0 {
                    out.push_str(", ");
                }
                let value = unsafe {
                    crate::primitives::utils::exif_get_srational(
                        (*entry).data.add(8 * index).cast_const(),
                        order,
                    )
                };
                if value.denominator != 0 {
                    let decimals = ((value.denominator.unsigned_abs() as f64).log10() - 0.08 + 1.0)
                        .max(0.0)
                        .floor() as usize;
                    out.push_str(&format_c_float_with_min_width(
                        value.numerator as f64 / value.denominator as f64,
                        decimals,
                    ));
                } else {
                    let _ = write!(out, "{}/{}", value.numerator, value.denominator);
                }
            }
            out
        }
        _ => format!("{} bytes unsupported data type", unsafe { (*entry).size }),
    }
}

fn indexed_string_value_static(tag: ExifTag, value: ExifShort) -> Option<&'static str> {
    let table = LIST_TABLE.iter().find(|table| table.tag == tag)?;
    let string = table.strings.get(value as usize)?;
    if string.is_empty() {
        None
    } else {
        Some(*string)
    }
}

fn indexed_string_value(tag: ExifTag, value: ExifShort) -> Option<String> {
    if let Some(string) = indexed_string_value_static(tag, value) {
        Some(string.to_owned())
    } else if LIST_TABLE.iter().any(|table| table.tag == tag) {
        Some(format!("Unknown value {value}"))
    } else {
        None
    }
}

fn indexed_value_value_static(
    tag: ExifTag,
    value: ExifShort,
    maxlen: c_uint,
) -> Option<&'static str> {
    let table = LIST2_TABLE.iter().find(|table| table.tag == tag)?;
    let entry = table.entries.iter().find(|entry| entry.index == value)?;
    choose_best_fit_static(entry.values, maxlen)
}

fn indexed_value_value(tag: ExifTag, value: ExifShort, maxlen: c_uint) -> Option<String> {
    if let Some(string) = indexed_value_value_static(tag, value, maxlen) {
        Some(string.to_owned())
    } else {
        let table = LIST2_TABLE.iter().find(|table| table.tag == tag)?;
        let entry = table.entries.iter().find(|entry| entry.index == value)?;
        choose_best_fit(entry.values, maxlen).or_else(|| Some(value.to_string()))
    }
}

fn convert_utf16_to_utf8(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        let value = u16::from_le_bytes([bytes[index], bytes[index + 1]]);
        if value == 0 {
            break;
        }
        if let Some(ch) = char::from_u32(value as u32) {
            out.push(ch);
        }
        index += 2;
    }
    out
}

fn user_comment_value(entry: *mut ExifEntry) -> String {
    let bytes = data_bytes(entry);
    if (unsafe { (*entry).format } != EXIF_FORMAT_ASCII)
        || unsafe { (*entry).size } <= 8
        || (!bytes.starts_with(b"ASCII\0\0\0")
            && !bytes.starts_with(b"UNICODE\0")
            && !bytes.starts_with(b"JIS\0\0\0\0\0")
            && !bytes.starts_with(b"\0\0\0\0\0\0\0\0"))
    {
        if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED {
            return String::new();
        }
    }

    if bytes.starts_with(b"ASCII\0\0\0") {
        return bytes_to_string(&bytes[8..]);
    }
    if bytes.starts_with(b"UNICODE\0") {
        return "Unsupported UNICODE string".to_owned();
    }
    if bytes.starts_with(b"JIS\0\0\0\0\0") {
        return "Unsupported JIS string".to_owned();
    }

    let mut index = 0usize;
    while index < bytes.len() && (bytes[index] == 0 || bytes[index] == b' ') {
        index += 1;
    }
    if index == bytes.len() {
        return String::new();
    }

    let mut out = String::with_capacity(bytes.len() - index);
    for byte in &bytes[index..] {
        out.push(if byte.is_ascii_graphic() || *byte == b' ' {
            *byte as char
        } else {
            '.'
        });
    }
    out
}

fn exif_version_value_static(bytes: &[u8]) -> Option<&'static str> {
    match bytes.get(..4) {
        Some(b"0110") => Some("Exif Version 1.1"),
        Some(b"0120") => Some("Exif Version 1.2"),
        Some(b"0200") => Some("Exif Version 2.0"),
        Some(b"0210") => Some("Exif Version 2.1"),
        Some(b"0220") => Some("Exif Version 2.2"),
        Some(b"0221") => Some("Exif Version 2.21"),
        Some(b"0230") => Some("Exif Version 2.3"),
        Some(b"0231") => Some("Exif Version 2.31"),
        Some(b"0232") => Some("Exif Version 2.32"),
        _ => None,
    }
}

fn exif_version_value(bytes: &[u8]) -> String {
    exif_version_value_static(bytes)
        .unwrap_or("Unknown Exif Version")
        .to_owned()
}

fn flash_pix_version_value_static(bytes: &[u8]) -> Option<&'static str> {
    match bytes.get(..4) {
        Some(b"0100") => Some("FlashPix Version 1.0"),
        Some(b"0101") => Some("FlashPix Version 1.01"),
        _ => None,
    }
}

fn flash_pix_version_value(bytes: &[u8]) -> String {
    flash_pix_version_value_static(bytes)
        .unwrap_or("Unknown FlashPix Version")
        .to_owned()
}

fn copyright_value(entry: *mut ExifEntry) -> String {
    let bytes = data_bytes(entry);
    let first_end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    let first = &bytes[..first_end];
    let mut out = String::with_capacity(bytes.len().saturating_add(32));
    if !first.is_empty() && !match_repeated_char(first, b' ') {
        out.push_str(&bytes_to_string(first));
    } else {
        out.push_str("[None]");
    }
    out.push_str(" (Photographer) - ");

    let second = if first_end < bytes.len() {
        let second_bytes = &bytes[first_end + 1..];
        let second_end = second_bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(second_bytes.len());
        &second_bytes[..second_end]
    } else {
        &[]
    };

    if !second.is_empty() && !match_repeated_char(second, b' ') {
        out.push_str(&bytes_to_string(second));
    } else {
        out.push_str("[None]");
    }
    out.push_str(" (Editor)");
    out
}

fn entry_value_string(entry: *mut ExifEntry, order: ExifByteOrder, maxlen: c_uint) -> String {
    let bytes = data_bytes(entry);
    match unsafe { (*entry).tag } {
        EXIF_TAG_USER_COMMENT => user_comment_value(entry),
        EXIF_TAG_EXIF_VERSION => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED
                || unsafe { (*entry).components } != 4
            {
                String::new()
            } else {
                exif_version_value(bytes)
            }
        }
        EXIF_TAG_FLASH_PIX_VERSION => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED
                || unsafe { (*entry).components } != 4
            {
                String::new()
            } else {
                flash_pix_version_value(bytes)
            }
        }
        EXIF_TAG_COPYRIGHT => {
            if unsafe { (*entry).format } != EXIF_FORMAT_ASCII {
                String::new()
            } else {
                copyright_value(entry)
            }
        }
        EXIF_TAG_FNUMBER => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                generic_format_value(entry, order)
            } else {
                format!("f/{:.1}", value.numerator as f64 / value.denominator as f64)
            }
        }
        EXIF_TAG_APERTURE_VALUE | EXIF_TAG_MAX_APERTURE_VALUE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 || value.numerator == 0x8000_0000 {
                generic_format_value(entry, order)
            } else {
                let ev = value.numerator as f64 / value.denominator as f64;
                format!("{ev:.2} EV (f/{:.1})", 2f64.powf(ev / 2.0))
            }
        }
        EXIF_TAG_FOCAL_LENGTH => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                return generic_format_value(entry, order);
            }
            let focal = value.numerator as f64 / value.denominator as f64;
            let mut extra = String::new();
            let ifd0 = unsafe { (*(*entry).parent).parent };
            if !ifd0.is_null() {
                let make = unsafe {
                    crate::object::content::exif_content_get_entry_impl(
                        (*ifd0).ifd[EXIF_IFD_0 as usize],
                        EXIF_TAG_MAKE,
                    )
                };
                if !make.is_null() {
                    let make_bytes = data_bytes(make);
                    if make_bytes.starts_with(b"Minolta") {
                        let model = unsafe {
                            crate::object::content::exif_content_get_entry_impl(
                                (*ifd0).ifd[EXIF_IFD_0 as usize],
                                EXIF_TAG_MODEL,
                            )
                        };
                        if !model.is_null() {
                            let model_bytes = data_bytes(model);
                            let factor = if model_bytes.starts_with(b"DiMAGE 7") {
                                3.9
                            } else if model_bytes.starts_with(b"DiMAGE 5") {
                                4.9
                            } else {
                                0.0
                            };
                            if factor != 0.0 {
                                extra = format!(" (35 equivalent: {:.0} mm)", factor * focal);
                            }
                        }
                    }
                }
            }
            format!("{focal:.1} mm{extra}")
        }
        EXIF_TAG_SUBJECT_DISTANCE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                generic_format_value(entry, order)
            } else {
                format!("{:.1} m", value.numerator as f64 / value.denominator as f64)
            }
        }
        EXIF_TAG_EXPOSURE_TIME => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                return generic_format_value(entry, order);
            }
            let seconds = value.numerator as f64 / value.denominator as f64;
            if seconds < 1.0 && seconds != 0.0 {
                format!("1/{:.0} sec.", 1.0 / seconds)
            } else {
                format!("{seconds:.0} sec.")
            }
        }
        EXIF_TAG_SHUTTER_SPEED_VALUE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SRATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_srational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                return generic_format_value(entry, order);
            }
            let ev = value.numerator as f64 / value.denominator as f64;
            let seconds = 1.0 / 2f64.powf(ev);
            if seconds < 1.0 && seconds != 0.0 {
                format!("{ev:.2} EV (1/{:.0} sec.)", 1.0 / seconds)
            } else {
                format!("{ev:.2} EV ({seconds:.0} sec.)")
            }
        }
        EXIF_TAG_BRIGHTNESS_VALUE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SRATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_srational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                return generic_format_value(entry, order);
            }
            let ev = value.numerator as f64 / value.denominator as f64;
            let cdm2 = 1.0 / (core::f64::consts::PI * 0.3048 * 0.3048) * 2f64.powf(ev);
            format!("{ev:.2} EV ({cdm2:.2} cd/m^2)")
        }
        EXIF_TAG_FILE_SOURCE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            match bytes.first().copied().unwrap_or_default() {
                3 => "DSC".to_owned(),
                value => format!("Internal error (unknown value {value})"),
            }
        }
        EXIF_TAG_COMPONENTS_CONFIGURATION => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED
                || unsafe { (*entry).components } != 4
            {
                return String::new();
            }
            let mut out = String::with_capacity(16);
            for (index, byte) in bytes.iter().take(4).enumerate() {
                if index > 0 {
                    out.push(' ');
                }
                out.push_str(match byte {
                    0 => "-",
                    1 => "Y",
                    2 => "Cb",
                    3 => "Cr",
                    4 => "R",
                    5 => "G",
                    6 => "B",
                    _ => "Reserved",
                });
            }
            out
        }
        EXIF_TAG_EXPOSURE_BIAS_VALUE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SRATIONAL
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_srational((*entry).data.cast_const(), order)
            };
            if value.denominator == 0 {
                generic_format_value(entry, order)
            } else {
                format!(
                    "{:.2} EV",
                    value.numerator as f64 / value.denominator as f64
                )
            }
        }
        EXIF_TAG_SCENE_TYPE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_UNDEFINED
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            match bytes.first().copied().unwrap_or_default() {
                1 => "Directly photographed".to_owned(),
                value => format!("Internal error (unknown value {value})"),
            }
        }
        EXIF_TAG_YCBCR_SUB_SAMPLING => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SHORT
                || unsafe { (*entry).components } != 2
            {
                return String::new();
            }
            let first = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            let second = unsafe {
                crate::primitives::utils::exif_get_short(
                    (*entry)
                        .data
                        .add(exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize)
                        .cast_const(),
                    order,
                )
            };
            match (first, second) {
                (2, 1) => "YCbCr4:2:2".to_owned(),
                (2, 2) => "YCbCr4:2:0".to_owned(),
                _ => format!("{first}, {second}"),
            }
        }
        EXIF_TAG_SUBJECT_AREA => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SHORT {
                return String::new();
            }
            match unsafe { (*entry).components } {
                2 => {
                    let x = unsafe {
                        crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
                    };
                    let y = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(2).cast_const(),
                            order,
                        )
                    };
                    format!("(x,y) = ({x},{y})")
                }
                3 => {
                    let x = unsafe {
                        crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
                    };
                    let y = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(2).cast_const(),
                            order,
                        )
                    };
                    let distance = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(4).cast_const(),
                            order,
                        )
                    };
                    format!("Within distance {distance} of (x,y) = ({x},{y})")
                }
                4 => {
                    let x = unsafe {
                        crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
                    };
                    let y = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(2).cast_const(),
                            order,
                        )
                    };
                    let width = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(4).cast_const(),
                            order,
                        )
                    };
                    let height = unsafe {
                        crate::primitives::utils::exif_get_short(
                            (*entry).data.add(6).cast_const(),
                            order,
                        )
                    };
                    format!("Within rectangle (width {width}, height {height}) around (x,y) = ({x},{y})")
                }
                components => {
                    format!("Unexpected number of components ({components}, expected 2, 3, or 4).")
                }
            }
        }
        EXIF_TAG_GPS_VERSION_ID => {
            if unsafe { (*entry).format } != EXIF_FORMAT_BYTE || unsafe { (*entry).components } != 4
            {
                return String::new();
            }
            let mut out = String::with_capacity(15);
            for (index, byte) in bytes
                .iter()
                .take(unsafe { (*entry).components as usize })
                .enumerate()
            {
                if index > 0 {
                    out.push('.');
                }
                let _ = write!(out, "{byte}");
            }
            out
        }
        EXIF_TAG_INTEROPERABILITY_VERSION => {
            if unsafe { (*entry).format } == EXIF_FORMAT_UNDEFINED {
                bytes_to_string(bytes_until_nul(bytes))
            } else {
                generic_format_value(entry, order)
            }
        }
        EXIF_TAG_GPS_ALTITUDE_REF => {
            if unsafe { (*entry).format } != EXIF_FORMAT_BYTE || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            match bytes.first().copied().unwrap_or_default() {
                0 => "Sea level".to_owned(),
                1 => "Sea level reference".to_owned(),
                value => format!("Internal error (unknown value {value})"),
            }
        }
        EXIF_TAG_GPS_TIME_STAMP => {
            if unsafe { (*entry).format } != EXIF_FORMAT_RATIONAL
                || unsafe { (*entry).components } != 3
            {
                return String::new();
            }
            let h = unsafe {
                crate::primitives::utils::exif_get_rational((*entry).data.cast_const(), order)
            };
            let m = unsafe {
                crate::primitives::utils::exif_get_rational(
                    (*entry).data.add(8).cast_const(),
                    order,
                )
            };
            let s = unsafe {
                crate::primitives::utils::exif_get_rational(
                    (*entry).data.add(16).cast_const(),
                    order,
                )
            };
            if h.denominator == 0 || m.denominator == 0 || s.denominator == 0 {
                return generic_format_value(entry, order);
            }
            format!(
                "{:02}:{:02}:{:05.2}",
                h.numerator / h.denominator,
                m.numerator / m.denominator,
                s.numerator as f64 / s.denominator as f64
            )
        }
        EXIF_TAG_METERING_MODE
        | EXIF_TAG_COMPRESSION
        | EXIF_TAG_LIGHT_SOURCE
        | EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT
        | EXIF_TAG_RESOLUTION_UNIT
        | EXIF_TAG_EXPOSURE_PROGRAM
        | EXIF_TAG_SENSITIVITY_TYPE
        | EXIF_TAG_FLASH
        | EXIF_TAG_SUBJECT_DISTANCE_RANGE
        | EXIF_TAG_COLOR_SPACE => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SHORT
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            indexed_value_value(unsafe { (*entry).tag }, value, maxlen)
                .unwrap_or_else(|| format!("Internal error (unknown value {value})"))
        }
        EXIF_TAG_PLANAR_CONFIGURATION
        | EXIF_TAG_SENSING_METHOD
        | EXIF_TAG_ORIENTATION
        | EXIF_TAG_YCBCR_POSITIONING
        | EXIF_TAG_PHOTOMETRIC_INTERPRETATION
        | EXIF_TAG_CUSTOM_RENDERED
        | EXIF_TAG_EXPOSURE_MODE
        | EXIF_TAG_WHITE_BALANCE
        | EXIF_TAG_SCENE_CAPTURE_TYPE
        | EXIF_TAG_GAIN_CONTROL
        | EXIF_TAG_SATURATION
        | EXIF_TAG_CONTRAST
        | EXIF_TAG_SHARPNESS => {
            if unsafe { (*entry).format } != EXIF_FORMAT_SHORT
                || unsafe { (*entry).components } != 1
            {
                return String::new();
            }
            let value = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            indexed_string_value(unsafe { (*entry).tag }, value)
                .unwrap_or_else(|| value.to_string())
        }
        EXIF_TAG_XP_TITLE | EXIF_TAG_XP_COMMENT | EXIF_TAG_XP_AUTHOR | EXIF_TAG_XP_KEYWORDS
        | EXIF_TAG_XP_SUBJECT => convert_utf16_to_utf8(bytes),
        _ => generic_format_value(entry, order),
    }
}

fn entry_value_static(
    entry: *mut ExifEntry,
    order: ExifByteOrder,
    maxlen: c_uint,
) -> Option<&'static str> {
    let bytes = data_bytes(entry);
    match unsafe { (*entry).tag } {
        EXIF_TAG_EXIF_VERSION
            if unsafe { (*entry).format } == EXIF_FORMAT_UNDEFINED
                && unsafe { (*entry).components } == 4 =>
        {
            Some(exif_version_value_static(bytes).unwrap_or("Unknown Exif Version"))
        }
        EXIF_TAG_FLASH_PIX_VERSION
            if unsafe { (*entry).format } == EXIF_FORMAT_UNDEFINED
                && unsafe { (*entry).components } == 4 =>
        {
            Some(flash_pix_version_value_static(bytes).unwrap_or("Unknown FlashPix Version"))
        }
        EXIF_TAG_FILE_SOURCE
            if unsafe { (*entry).format } == EXIF_FORMAT_UNDEFINED
                && unsafe { (*entry).components } == 1 =>
        {
            match bytes.first().copied().unwrap_or_default() {
                3 => Some("DSC"),
                _ => None,
            }
        }
        EXIF_TAG_SCENE_TYPE
            if unsafe { (*entry).format } == EXIF_FORMAT_UNDEFINED
                && unsafe { (*entry).components } == 1 =>
        {
            match bytes.first().copied().unwrap_or_default() {
                1 => Some("Directly photographed"),
                _ => None,
            }
        }
        EXIF_TAG_GPS_ALTITUDE_REF
            if unsafe { (*entry).format } == EXIF_FORMAT_BYTE
                && unsafe { (*entry).components } == 1 =>
        {
            match bytes.first().copied().unwrap_or_default() {
                0 => Some("Sea level"),
                1 => Some("Sea level reference"),
                _ => None,
            }
        }
        EXIF_TAG_YCBCR_SUB_SAMPLING
            if unsafe { (*entry).format } == EXIF_FORMAT_SHORT
                && unsafe { (*entry).components } == 2 =>
        {
            let first = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            let second = unsafe {
                crate::primitives::utils::exif_get_short(
                    (*entry)
                        .data
                        .add(exif_format_get_size_impl(EXIF_FORMAT_SHORT) as usize)
                        .cast_const(),
                    order,
                )
            };
            match (first, second) {
                (2, 1) => Some("YCbCr4:2:2"),
                (2, 2) => Some("YCbCr4:2:0"),
                _ => None,
            }
        }
        EXIF_TAG_METERING_MODE
        | EXIF_TAG_COMPRESSION
        | EXIF_TAG_LIGHT_SOURCE
        | EXIF_TAG_FOCAL_PLANE_RESOLUTION_UNIT
        | EXIF_TAG_RESOLUTION_UNIT
        | EXIF_TAG_EXPOSURE_PROGRAM
        | EXIF_TAG_SENSITIVITY_TYPE
        | EXIF_TAG_FLASH
        | EXIF_TAG_SUBJECT_DISTANCE_RANGE
        | EXIF_TAG_COLOR_SPACE
            if unsafe { (*entry).format } == EXIF_FORMAT_SHORT
                && unsafe { (*entry).components } == 1 =>
        {
            let value = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            indexed_value_value_static(unsafe { (*entry).tag }, value, maxlen)
        }
        EXIF_TAG_PLANAR_CONFIGURATION
        | EXIF_TAG_SENSING_METHOD
        | EXIF_TAG_ORIENTATION
        | EXIF_TAG_YCBCR_POSITIONING
        | EXIF_TAG_PHOTOMETRIC_INTERPRETATION
        | EXIF_TAG_CUSTOM_RENDERED
        | EXIF_TAG_EXPOSURE_MODE
        | EXIF_TAG_WHITE_BALANCE
        | EXIF_TAG_SCENE_CAPTURE_TYPE
        | EXIF_TAG_GAIN_CONTROL
        | EXIF_TAG_SATURATION
        | EXIF_TAG_CONTRAST
        | EXIF_TAG_SHARPNESS
            if unsafe { (*entry).format } == EXIF_FORMAT_SHORT
                && unsafe { (*entry).components } == 1 =>
        {
            let value = unsafe {
                crate::primitives::utils::exif_get_short((*entry).data.cast_const(), order)
            };
            indexed_string_value_static(unsafe { (*entry).tag }, value)
        }
        _ => None,
    }
}

unsafe fn copy_to_buffer(value: &str, buffer: *mut c_char, maxlen: c_uint) {
    if buffer.is_null() || maxlen == 0 {
        return;
    }
    let limit = maxlen as usize - 1;
    let bytes = value.as_bytes();
    let count = bytes.len().min(limit);
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), buffer, count);
        *buffer.add(count) = 0;
    }
}

pub(crate) unsafe fn exif_entry_get_value_impl(
    entry: *mut ExifEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *const c_char {
    if entry.is_null()
        || unsafe { (*entry).parent }.is_null()
        || unsafe { (*(*entry).parent).parent }.is_null()
        || maxlen == 0
        || value.is_null()
    {
        return value.cast_const();
    }

    unsafe {
        ptr::write_bytes(value.cast::<u8>(), 0, maxlen as usize);
    }

    let expected_size = (unsafe { (*entry).components } as u128)
        * (exif_format_get_size_impl(unsafe { (*entry).format }) as u128);
    if expected_size != unsafe { (*entry).size as u128 } {
        let message = format!(
            "Invalid size of entry ({}, expected {} x {}).",
            unsafe { (*entry).size },
            unsafe { (*entry).components },
            exif_format_get_size_impl(unsafe { (*entry).format })
        );
        unsafe { copy_to_buffer(&message, value, maxlen) };
        return value.cast_const();
    }

    let order = unsafe { exif_data_get_byte_order_impl((*(*entry).parent).parent) };
    if let Some(static_value) = entry_value_static(entry, order, maxlen) {
        unsafe { copy_to_buffer(static_value, value, maxlen) };
        return value.cast_const();
    }
    let formatted = entry_value_string(entry, order, maxlen);
    unsafe { copy_to_buffer(&formatted, value, maxlen) };
    value.cast_const()
}

pub(crate) unsafe fn exif_entry_dump_impl(entry: *mut ExifEntry, indent: c_uint) {
    if entry.is_null() {
        return;
    }

    let prefix = " ".repeat((indent as usize).saturating_mul(2).min(1023));
    let mut value = [0 as c_char; 1024];
    unsafe {
        exif_entry_get_value_impl(entry, value.as_mut_ptr(), value.len() as c_uint);
    }
    let rendered = unsafe {
        std::ffi::CStr::from_ptr(value.as_ptr())
            .to_string_lossy()
            .into_owned()
    };
    let ifd = unsafe { exif_content_get_ifd_impl((*entry).parent) };
    let tag_name_ptr = exif_tag_get_name_in_ifd((*entry).tag, ifd);
    let tag_name = if tag_name_ptr.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(tag_name_ptr) }
            .to_str()
            .unwrap_or("")
    };
    let format_name_ptr =
        crate::primitives::format::exif_format_get_name_impl(unsafe { (*entry).format });
    let format_name = if format_name_ptr.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(format_name_ptr) }
            .to_str()
            .unwrap_or("")
    };

    print_line(&format!(
        "{prefix}Tag: 0x{:x} ('{}')",
        unsafe { (*entry).tag },
        tag_name
    ));
    print_line(&format!(
        "{prefix}  Format: {} ('{}')",
        unsafe { (*entry).format },
        format_name
    ));
    print_line(&format!("{prefix}  Components: {}", unsafe {
        (*entry).components
    }));
    print_line(&format!("{prefix}  Size: {}", unsafe { (*entry).size }));
    print_line(&format!("{prefix}  Value: {rendered}"));
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_entry_new() -> *mut ExifEntry {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_entry_new_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_new_mem(mem: *mut ExifMem) -> *mut ExifEntry {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_entry_new_mem_impl(mem) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_ref(entry: *mut ExifEntry) {
    panic_boundary::call_void(|| unsafe { exif_entry_ref_impl(entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_unref(entry: *mut ExifEntry) {
    panic_boundary::call_void(|| unsafe { exif_entry_unref_impl(entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_free(entry: *mut ExifEntry) {
    panic_boundary::call_void(|| unsafe { exif_entry_free_impl(entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_initialize(entry: *mut ExifEntry, tag: ExifTag) {
    panic_boundary::call_void(|| unsafe { exif_entry_initialize_impl(entry, tag) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_fix(entry: *mut ExifEntry) {
    panic_boundary::call_void(|| unsafe { exif_entry_fix_impl(entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_get_value(
    entry: *mut ExifEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *const c_char {
    panic_boundary::call_or(value.cast_const(), || unsafe {
        exif_entry_get_value_impl(entry, value, maxlen)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_entry_dump(entry: *mut ExifEntry, indent: c_uint) {
    panic_boundary::call_void(|| unsafe { exif_entry_dump_impl(entry, indent) });
}
