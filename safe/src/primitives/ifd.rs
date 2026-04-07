use core::ffi::c_char;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifIfd, EXIF_IFD_0, EXIF_IFD_1, EXIF_IFD_EXIF, EXIF_IFD_GPS, EXIF_IFD_INTEROPERABILITY,
};
use crate::i18n::{gettext, message, Message};

#[derive(Clone, Copy)]
struct IfdEntry {
    ifd: ExifIfd,
    name: Message,
}

const IFD_TABLE: [IfdEntry; 5] = [
    IfdEntry {
        ifd: EXIF_IFD_0,
        name: message(b"0\0"),
    },
    IfdEntry {
        ifd: EXIF_IFD_1,
        name: message(b"1\0"),
    },
    IfdEntry {
        ifd: EXIF_IFD_EXIF,
        name: message(b"EXIF\0"),
    },
    IfdEntry {
        ifd: EXIF_IFD_GPS,
        name: message(b"GPS\0"),
    },
    IfdEntry {
        ifd: EXIF_IFD_INTEROPERABILITY,
        name: message(b"Interoperability\0"),
    },
];

pub(crate) fn exif_ifd_get_name_impl(ifd: ExifIfd) -> *const c_char {
    IFD_TABLE
        .iter()
        .find(|entry| entry.ifd == ifd)
        .map_or(ptr::null(), |entry| gettext(entry.name))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_ifd_get_name(ifd: ExifIfd) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_ifd_get_name_impl(ifd))
}
