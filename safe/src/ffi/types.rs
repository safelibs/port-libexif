#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int, c_schar, c_uchar, c_uint, c_ulong, c_void};

pub type ExifByteOrder = c_int;
pub const EXIF_BYTE_ORDER_MOTOROLA: ExifByteOrder = 0;
pub const EXIF_BYTE_ORDER_INTEL: ExifByteOrder = 1;

pub type ExifDataType = c_int;
pub const EXIF_DATA_TYPE_UNCOMPRESSED_CHUNKY: ExifDataType = 0;
pub const EXIF_DATA_TYPE_UNCOMPRESSED_PLANAR: ExifDataType = 1;
pub const EXIF_DATA_TYPE_UNCOMPRESSED_YCC: ExifDataType = 2;
pub const EXIF_DATA_TYPE_COMPRESSED: ExifDataType = 3;
pub const EXIF_DATA_TYPE_COUNT: ExifDataType = 4;
pub const EXIF_DATA_TYPE_UNKNOWN: ExifDataType = EXIF_DATA_TYPE_COUNT;

pub type ExifFormat = c_int;
pub const EXIF_FORMAT_BYTE: ExifFormat = 1;
pub const EXIF_FORMAT_ASCII: ExifFormat = 2;
pub const EXIF_FORMAT_SHORT: ExifFormat = 3;
pub const EXIF_FORMAT_LONG: ExifFormat = 4;
pub const EXIF_FORMAT_RATIONAL: ExifFormat = 5;
pub const EXIF_FORMAT_SBYTE: ExifFormat = 6;
pub const EXIF_FORMAT_UNDEFINED: ExifFormat = 7;
pub const EXIF_FORMAT_SSHORT: ExifFormat = 8;
pub const EXIF_FORMAT_SLONG: ExifFormat = 9;
pub const EXIF_FORMAT_SRATIONAL: ExifFormat = 10;
pub const EXIF_FORMAT_FLOAT: ExifFormat = 11;
pub const EXIF_FORMAT_DOUBLE: ExifFormat = 12;

pub type ExifIfd = c_int;
pub const EXIF_IFD_0: ExifIfd = 0;
pub const EXIF_IFD_1: ExifIfd = 1;
pub const EXIF_IFD_EXIF: ExifIfd = 2;
pub const EXIF_IFD_GPS: ExifIfd = 3;
pub const EXIF_IFD_INTEROPERABILITY: ExifIfd = 4;
pub const EXIF_IFD_COUNT: ExifIfd = 5;

pub type ExifTag = c_int;

pub type ExifSupportLevel = c_int;
pub const EXIF_SUPPORT_LEVEL_UNKNOWN: ExifSupportLevel = 0;
pub const EXIF_SUPPORT_LEVEL_NOT_RECORDED: ExifSupportLevel = 1;
pub const EXIF_SUPPORT_LEVEL_MANDATORY: ExifSupportLevel = 2;
pub const EXIF_SUPPORT_LEVEL_OPTIONAL: ExifSupportLevel = 3;

pub type ExifDataOption = c_int;
pub const EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS: ExifDataOption = 1 << 0;
pub const EXIF_DATA_OPTION_FOLLOW_SPECIFICATION: ExifDataOption = 1 << 1;
pub const EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE: ExifDataOption = 1 << 2;

pub type ExifLogCode = c_int;
pub const EXIF_LOG_CODE_NONE: ExifLogCode = 0;
pub const EXIF_LOG_CODE_DEBUG: ExifLogCode = 1;
pub const EXIF_LOG_CODE_NO_MEMORY: ExifLogCode = 2;
pub const EXIF_LOG_CODE_CORRUPT_DATA: ExifLogCode = 3;

pub type ExifByte = c_uchar;
pub type ExifSByte = c_schar;
pub type ExifAscii = *mut c_char;
pub type ExifShort = u16;
pub type ExifSShort = i16;
pub type ExifLong = u32;
pub type ExifSLong = i32;
pub type ExifUndefined = c_char;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExifRational {
    pub numerator: ExifLong,
    pub denominator: ExifLong,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ExifSRational {
    pub numerator: ExifSLong,
    pub denominator: ExifSLong,
}

#[repr(C)]
pub struct ExifDataPrivate {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifContentPrivate {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifEntryPrivate {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifMnoteDataPriv {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifData {
    pub ifd: [*mut ExifContent; EXIF_IFD_COUNT as usize],
    pub data: *mut c_uchar,
    pub size: c_uint,
    pub priv_: *mut ExifDataPrivate,
}

#[repr(C)]
pub struct ExifContent {
    pub entries: *mut *mut ExifEntry,
    pub count: c_uint,
    pub parent: *mut ExifData,
    pub priv_: *mut ExifContentPrivate,
}

#[repr(C)]
pub struct ExifEntry {
    pub tag: ExifTag,
    pub format: ExifFormat,
    pub components: c_ulong,
    pub data: *mut c_uchar,
    pub size: c_uint,
    pub parent: *mut ExifContent,
    pub priv_: *mut ExifEntryPrivate,
}

#[repr(C)]
pub struct ExifLoader {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifLog {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ExifMem {
    _private: [u8; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct ExifMnoteDataMethods {
    pub free: Option<unsafe extern "C" fn(*mut ExifMnoteData)>,
    pub save: Option<unsafe extern "C" fn(*mut ExifMnoteData, *mut *mut c_uchar, *mut c_uint)>,
    pub load: Option<unsafe extern "C" fn(*mut ExifMnoteData, *const c_uchar, c_uint)>,
    pub set_offset: Option<unsafe extern "C" fn(*mut ExifMnoteData, c_uint)>,
    pub set_byte_order: Option<unsafe extern "C" fn(*mut ExifMnoteData, ExifByteOrder)>,
    pub count: Option<unsafe extern "C" fn(*mut ExifMnoteData) -> c_uint>,
    pub get_id: Option<unsafe extern "C" fn(*mut ExifMnoteData, c_uint) -> c_uint>,
    pub get_name: Option<unsafe extern "C" fn(*mut ExifMnoteData, c_uint) -> *const c_char>,
    pub get_title: Option<unsafe extern "C" fn(*mut ExifMnoteData, c_uint) -> *const c_char>,
    pub get_description: Option<unsafe extern "C" fn(*mut ExifMnoteData, c_uint) -> *const c_char>,
    pub get_value: Option<
        unsafe extern "C" fn(*mut ExifMnoteData, c_uint, *mut c_char, c_uint) -> *mut c_char,
    >,
}

#[repr(C)]
pub struct ExifMnoteData {
    pub priv_: *mut ExifMnoteDataPriv,
    pub methods: ExifMnoteDataMethods,
    pub log: *mut ExifLog,
    pub mem: *mut ExifMem,
}

pub type ExifMemAllocFunc = Option<unsafe extern "C" fn(ExifLong) -> *mut c_void>;
pub type ExifMemReallocFunc = Option<unsafe extern "C" fn(*mut c_void, ExifLong) -> *mut c_void>;
pub type ExifMemFreeFunc = Option<unsafe extern "C" fn(*mut c_void)>;

pub type ExifContentForeachEntryFunc = Option<unsafe extern "C" fn(*mut ExifEntry, *mut c_void)>;
pub type ExifDataForeachContentFunc = Option<unsafe extern "C" fn(*mut ExifContent, *mut c_void)>;

/*
 * Stable Rust cannot model C variadic exports or va_list parameters precisely.
 * The logging edge is handled by the C shim, and Rust treats the callback slot
 * as an opaque pointer-shaped ABI carrier.
 */
pub type ExifLogFunc = Option<
    unsafe extern "C" fn(
        *mut ExifLog,
        ExifLogCode,
        *const c_char,
        *const c_char,
        *mut c_void,
        *mut c_void,
    ),
>;

pub type MnoteCanonTag = c_int;
pub type MnoteOlympusTag = c_int;
pub type MnotePentaxTag = c_int;

#[repr(C)]
pub struct MnoteCanonEntry {
    pub tag: MnoteCanonTag,
    pub format: ExifFormat,
    pub components: c_ulong,
    pub data: *mut c_uchar,
    pub size: c_uint,
    pub order: ExifByteOrder,
}

#[repr(C)]
pub struct MnoteOlympusEntry {
    pub tag: MnoteOlympusTag,
    pub format: ExifFormat,
    pub components: c_ulong,
    pub data: *mut c_uchar,
    pub size: c_uint,
    pub order: ExifByteOrder,
}

#[repr(C)]
pub struct MnotePentaxEntry {
    pub tag: MnotePentaxTag,
    pub format: ExifFormat,
    pub components: c_ulong,
    pub data: *mut c_uchar,
    pub size: c_uint,
    pub order: ExifByteOrder,
}
