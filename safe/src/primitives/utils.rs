use core::ffi::{c_uchar, c_uint};

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifFormat, ExifLong, ExifRational, ExifSLong, ExifSRational, ExifSShort,
    ExifShort, EXIF_BYTE_ORDER_INTEL, EXIF_BYTE_ORDER_MOTOROLA, EXIF_FORMAT_LONG,
    EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SHORT, EXIF_FORMAT_SLONG, EXIF_FORMAT_SRATIONAL,
    EXIF_FORMAT_SSHORT,
};
use crate::primitives::format::exif_format_get_size_impl;

#[inline]
fn get_sshort_impl(buffer: *const c_uchar, order: ExifByteOrder) -> ExifSShort {
    if buffer.is_null() {
        return 0;
    }

    unsafe {
        match order {
            EXIF_BYTE_ORDER_MOTOROLA => (((*buffer as u16) << 8) | *buffer.add(1) as u16) as ExifSShort,
            EXIF_BYTE_ORDER_INTEL => (((*buffer.add(1) as u16) << 8) | *buffer as u16) as ExifSShort,
            _ => 0,
        }
    }
}

#[inline]
fn get_slong_impl(buffer: *const c_uchar, order: ExifByteOrder) -> ExifSLong {
    if buffer.is_null() {
        return 0;
    }

    unsafe {
        match order {
            EXIF_BYTE_ORDER_MOTOROLA => {
                (((*buffer as u32) << 24)
                    | ((*buffer.add(1) as u32) << 16)
                    | ((*buffer.add(2) as u32) << 8)
                    | *buffer.add(3) as u32) as ExifSLong
            }
            EXIF_BYTE_ORDER_INTEL => {
                (((*buffer.add(3) as u32) << 24)
                    | ((*buffer.add(2) as u32) << 16)
                    | ((*buffer.add(1) as u32) << 8)
                    | *buffer as u32) as ExifSLong
            }
            _ => 0,
        }
    }
}

#[inline]
fn set_sshort_impl(buffer: *mut c_uchar, order: ExifByteOrder, value: ExifSShort) {
    if buffer.is_null() {
        return;
    }

    unsafe {
        match order {
            EXIF_BYTE_ORDER_MOTOROLA => {
                *buffer = (value >> 8) as u8;
                *buffer.add(1) = value as u8;
            }
            EXIF_BYTE_ORDER_INTEL => {
                *buffer = value as u8;
                *buffer.add(1) = (value >> 8) as u8;
            }
            _ => {}
        }
    }
}

#[inline]
fn set_slong_impl(buffer: *mut c_uchar, order: ExifByteOrder, value: ExifSLong) {
    if buffer.is_null() {
        return;
    }

    unsafe {
        match order {
            EXIF_BYTE_ORDER_MOTOROLA => {
                *buffer = (value >> 24) as u8;
                *buffer.add(1) = (value >> 16) as u8;
                *buffer.add(2) = (value >> 8) as u8;
                *buffer.add(3) = value as u8;
            }
            EXIF_BYTE_ORDER_INTEL => {
                *buffer.add(3) = (value >> 24) as u8;
                *buffer.add(2) = (value >> 16) as u8;
                *buffer.add(1) = (value >> 8) as u8;
                *buffer = value as u8;
            }
            _ => {}
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_sshort(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifSShort {
    panic_boundary::call_or(0, || get_sshort_impl(buffer, order))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_short(buffer: *const c_uchar, order: ExifByteOrder) -> ExifShort {
    panic_boundary::call_or(0, || (get_sshort_impl(buffer, order) as u16) & 0xffff)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_sshort(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSShort,
) {
    panic_boundary::call_void(|| set_sshort_impl(buffer, order, value));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_short(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifShort,
) {
    panic_boundary::call_void(|| set_sshort_impl(buffer, order, value as ExifSShort));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_slong(buffer: *const c_uchar, order: ExifByteOrder) -> ExifSLong {
    panic_boundary::call_or(0, || get_slong_impl(buffer, order))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_long(buffer: *const c_uchar, order: ExifByteOrder) -> ExifLong {
    panic_boundary::call_or(0, || (get_slong_impl(buffer, order) as u32) & 0xffff_ffff)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_slong(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSLong,
) {
    panic_boundary::call_void(|| set_slong_impl(buffer, order, value));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_long(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifLong,
) {
    panic_boundary::call_void(|| set_slong_impl(buffer, order, value as ExifSLong));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_srational(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifSRational {
    panic_boundary::call_or(ExifSRational::default(), || ExifSRational {
        numerator: if buffer.is_null() {
            0
        } else {
            get_slong_impl(buffer, order)
        },
        denominator: if buffer.is_null() {
            0
        } else {
            get_slong_impl(unsafe { buffer.add(4) }, order)
        },
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_get_rational(
    buffer: *const c_uchar,
    order: ExifByteOrder,
) -> ExifRational {
    panic_boundary::call_or(ExifRational::default(), || ExifRational {
        numerator: if buffer.is_null() {
            0
        } else {
            (get_slong_impl(buffer, order) as u32) & 0xffff_ffff
        },
        denominator: if buffer.is_null() {
            0
        } else {
            (get_slong_impl(unsafe { buffer.add(4) }, order) as u32) & 0xffff_ffff
        },
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_srational(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifSRational,
) {
    panic_boundary::call_void(|| {
        if buffer.is_null() {
            return;
        }
        set_slong_impl(buffer, order, value.numerator);
        set_slong_impl(unsafe { buffer.add(4) }, order, value.denominator);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_set_rational(
    buffer: *mut c_uchar,
    order: ExifByteOrder,
    value: ExifRational,
) {
    panic_boundary::call_void(|| {
        if buffer.is_null() {
            return;
        }
        set_slong_impl(buffer, order, value.numerator as ExifSLong);
        set_slong_impl(unsafe { buffer.add(4) }, order, value.denominator as ExifSLong);
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
        let field_size = exif_format_get_size_impl(format) as usize;
        if buffer.is_null() || count == 0 || field_size == 0 {
            return;
        }

        for index in 0..count as usize {
            let slot = unsafe { buffer.add(index * field_size) };
            match format {
                EXIF_FORMAT_SHORT => {
                    let value = get_sshort_impl(slot.cast_const(), original_order) as ExifShort;
                    set_sshort_impl(slot, new_order, value as ExifSShort);
                }
                EXIF_FORMAT_SSHORT => {
                    let value = get_sshort_impl(slot.cast_const(), original_order);
                    set_sshort_impl(slot, new_order, value);
                }
                EXIF_FORMAT_LONG => {
                    let value = get_slong_impl(slot.cast_const(), original_order) as ExifLong;
                    set_slong_impl(slot, new_order, value as ExifSLong);
                }
                EXIF_FORMAT_RATIONAL => {
                    let value = ExifRational {
                        numerator: get_slong_impl(slot.cast_const(), original_order) as ExifLong,
                        denominator: get_slong_impl(unsafe { slot.add(4) }.cast_const(), original_order)
                            as ExifLong,
                    };
                    set_slong_impl(slot, new_order, value.numerator as ExifSLong);
                    set_slong_impl(unsafe { slot.add(4) }, new_order, value.denominator as ExifSLong);
                }
                EXIF_FORMAT_SLONG => {
                    let value = get_slong_impl(slot.cast_const(), original_order);
                    set_slong_impl(slot, new_order, value);
                }
                EXIF_FORMAT_SRATIONAL => {
                    let value = ExifSRational {
                        numerator: get_slong_impl(slot.cast_const(), original_order),
                        denominator: get_slong_impl(unsafe { slot.add(4) }.cast_const(), original_order),
                    };
                    set_slong_impl(slot, new_order, value.numerator);
                    set_slong_impl(unsafe { slot.add(4) }, new_order, value.denominator);
                }
                _ => {}
            }
        }
    });
}
