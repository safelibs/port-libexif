use core::ffi::c_char;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{ExifByteOrder, EXIF_BYTE_ORDER_INTEL, EXIF_BYTE_ORDER_MOTOROLA};
use crate::i18n::{gettext, message};

const MOTOROLA_NAME: crate::i18n::Message = message(b"Motorola\0");
const INTEL_NAME: crate::i18n::Message = message(b"Intel\0");

pub(crate) fn exif_byte_order_get_name_impl(order: ExifByteOrder) -> *const c_char {
    match order {
        EXIF_BYTE_ORDER_MOTOROLA => gettext(MOTOROLA_NAME),
        EXIF_BYTE_ORDER_INTEL => gettext(INTEL_NAME),
        _ => ptr::null(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_byte_order_get_name(order: ExifByteOrder) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_byte_order_get_name_impl(order))
}
