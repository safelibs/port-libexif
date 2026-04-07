use core::ffi::{c_char, c_uchar};
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifFormat, EXIF_FORMAT_ASCII, EXIF_FORMAT_BYTE, EXIF_FORMAT_DOUBLE, EXIF_FORMAT_FLOAT,
    EXIF_FORMAT_LONG, EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SBYTE, EXIF_FORMAT_SHORT,
    EXIF_FORMAT_SLONG, EXIF_FORMAT_SRATIONAL, EXIF_FORMAT_SSHORT, EXIF_FORMAT_UNDEFINED,
};
use crate::i18n::{gettext, message, Message};

#[derive(Clone, Copy)]
struct FormatEntry {
    format: ExifFormat,
    name: Message,
    size: c_uchar,
}

const FORMAT_TABLE: [FormatEntry; 12] = [
    FormatEntry {
        format: EXIF_FORMAT_SHORT,
        name: message(b"Short\0"),
        size: 2,
    },
    FormatEntry {
        format: EXIF_FORMAT_RATIONAL,
        name: message(b"Rational\0"),
        size: 8,
    },
    FormatEntry {
        format: EXIF_FORMAT_SRATIONAL,
        name: message(b"SRational\0"),
        size: 8,
    },
    FormatEntry {
        format: EXIF_FORMAT_UNDEFINED,
        name: message(b"Undefined\0"),
        size: 1,
    },
    FormatEntry {
        format: EXIF_FORMAT_ASCII,
        name: message(b"ASCII\0"),
        size: 1,
    },
    FormatEntry {
        format: EXIF_FORMAT_LONG,
        name: message(b"Long\0"),
        size: 4,
    },
    FormatEntry {
        format: EXIF_FORMAT_BYTE,
        name: message(b"Byte\0"),
        size: 1,
    },
    FormatEntry {
        format: EXIF_FORMAT_SBYTE,
        name: message(b"SByte\0"),
        size: 1,
    },
    FormatEntry {
        format: EXIF_FORMAT_SSHORT,
        name: message(b"SShort\0"),
        size: 2,
    },
    FormatEntry {
        format: EXIF_FORMAT_SLONG,
        name: message(b"SLong\0"),
        size: 4,
    },
    FormatEntry {
        format: EXIF_FORMAT_FLOAT,
        name: message(b"Float\0"),
        size: 4,
    },
    FormatEntry {
        format: EXIF_FORMAT_DOUBLE,
        name: message(b"Double\0"),
        size: 8,
    },
];

pub(crate) fn exif_format_get_name_impl(format: ExifFormat) -> *const c_char {
    FORMAT_TABLE
        .iter()
        .find(|entry| entry.format == format)
        .map_or(ptr::null(), |entry| gettext(entry.name))
}

pub(crate) fn exif_format_get_size_impl(format: ExifFormat) -> c_uchar {
    FORMAT_TABLE
        .iter()
        .find(|entry| entry.format == format)
        .map_or(0, |entry| entry.size)
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_format_get_name(format: ExifFormat) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_format_get_name_impl(format))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_format_get_size(format: ExifFormat) -> c_uchar {
    panic_boundary::call_or(0, || exif_format_get_size_impl(format))
}
