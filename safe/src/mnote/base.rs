use core::ffi::{c_char, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{ExifByteOrder, ExifLog, ExifMem, ExifMnoteData, ExifMnoteDataPriv};
use crate::runtime::log::{exif_log_ref_impl, exif_log_unref_impl};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_ref_impl, exif_mem_unref_impl,
};

#[repr(C)]
struct MnoteDataPrivate {
    ref_count: u32,
}

#[inline]
unsafe fn mnote_private(note: *mut ExifMnoteData) -> *mut MnoteDataPrivate {
    unsafe { (*note).priv_.cast::<MnoteDataPrivate>() }
}

pub(crate) unsafe fn exif_mnote_data_free_impl(note: *mut ExifMnoteData) {
    if note.is_null() {
        return;
    }

    let mem = unsafe { (*note).mem };
    if !unsafe { (*note).priv_ }.is_null() {
        unsafe {
            if let Some(free_fn) = (*note).methods.free {
                free_fn(note);
            }
            exif_mem_free_impl(mem, (*note).priv_.cast());
            (*note).priv_ = ptr::null_mut::<ExifMnoteDataPriv>();
        }
    }

    unsafe {
        exif_log_unref_impl((*note).log);
        exif_mem_free_impl(mem, note.cast());
        exif_mem_unref_impl(mem);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_construct(note: *mut ExifMnoteData, mem: *mut ExifMem) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || mem.is_null() || !(*note).priv_.is_null() {
            return;
        }

        let private = exif_mem_alloc_impl(mem, size_of::<MnoteDataPrivate>() as u32)
            .cast::<MnoteDataPrivate>();
        if private.is_null() {
            return;
        }

        (*private).ref_count = 1;
        (*note).priv_ = private.cast();
        (*note).mem = mem;
        exif_mem_ref_impl(mem);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_ref(note: *mut ExifMnoteData) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || (*note).priv_.is_null() {
            return;
        }

        let private = &mut *mnote_private(note);
        private.ref_count = private.ref_count.wrapping_add(1);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_unref(note: *mut ExifMnoteData) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() || (*note).priv_.is_null() {
            return;
        }

        let private = &mut *mnote_private(note);
        if private.ref_count > 0 {
            private.ref_count -= 1;
        }
        if private.ref_count == 0 {
            exif_mnote_data_free_impl(note);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    size: c_uint,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(load_fn) = (*note).methods.load {
            load_fn(note, buffer, size);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(save_fn) = (*note).methods.save {
            save_fn(note, buffer, size);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_log(note: *mut ExifMnoteData, log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }

        exif_log_unref_impl((*note).log);
        (*note).log = log;
        exif_log_ref_impl(log);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(setter) = (*note).methods.set_byte_order {
            setter(note, order);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    panic_boundary::call_void(|| unsafe {
        if note.is_null() {
            return;
        }
        if let Some(setter) = (*note).methods.set_offset {
            setter(note, offset);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_count(note: *mut ExifMnoteData) -> c_uint {
    panic_boundary::call_or(0, || unsafe {
        if note.is_null() {
            return 0;
        }
        (*note).methods.count.map_or(0, |count_fn| count_fn(note))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_id(note: *mut ExifMnoteData, index: c_uint) -> c_uint {
    panic_boundary::call_or(0, || unsafe {
        if note.is_null() {
            return 0;
        }
        (*note)
            .methods
            .get_id
            .map_or(0, |get_id_fn| get_id_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_name
            .map_or(ptr::null(), |get_name_fn| get_name_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_title
            .map_or(ptr::null(), |get_title_fn| get_title_fn(note, index))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || unsafe {
        if note.is_null() {
            return ptr::null();
        }
        (*note)
            .methods
            .get_description
            .map_or(ptr::null(), |get_description_fn| {
                get_description_fn(note, index)
            })
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        if note.is_null() {
            return ptr::null_mut();
        }
        (*note)
            .methods
            .get_value
            .map_or(ptr::null_mut(), |get_value_fn| {
                get_value_fn(note, index, value, maxlen)
            })
    })
}
