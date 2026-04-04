use core::ffi::{c_uchar, c_uint, c_void};
use core::mem::size_of;
use core::ptr;

use std::ffi::CStr;

use crate::ffi::panic_boundary;
use crate::ffi::types::*;
use crate::object::content::{
    exif_content_dump_impl, exif_content_fix_impl, exif_content_foreach_entry_impl,
    exif_content_get_ifd_impl, exif_content_log_impl, exif_content_new_mem_impl,
    exif_content_remove_entry_impl, exif_content_unref_impl,
};
use crate::runtime::log::{exif_log_ref_impl, exif_log_unref_impl};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_new_default_impl, exif_mem_ref_impl,
    exif_mem_unref_impl,
};

#[repr(C)]
pub(crate) struct DataPrivate {
    order: ExifByteOrder,
    md: *mut ExifMnoteData,
    log: *mut ExifLog,
    mem: *mut ExifMem,
    ref_count: u32,
    offset_mnote: c_uint,
    options: ExifDataOption,
    data_type: ExifDataType,
}

#[repr(C)]
struct ByteOrderChangeData {
    old: ExifByteOrder,
    new: ExifByteOrder,
}

#[inline]
unsafe fn data_private(data: *mut ExifData) -> *mut DataPrivate {
    unsafe { (*data).priv_.cast::<DataPrivate>() }
}

#[inline]
unsafe fn data_mem(data: *mut ExifData) -> *mut ExifMem {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*data_private(data)).mem }
    }
}

pub(crate) unsafe fn exif_data_get_mem_impl(data: *mut ExifData) -> *mut ExifMem {
    unsafe { data_mem(data) }
}

pub(crate) unsafe fn exif_data_get_options_impl(data: *mut ExifData) -> ExifDataOption {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        0
    } else {
        unsafe { (*data_private(data)).options }
    }
}

pub(crate) unsafe fn exif_data_set_mnote_offset_impl(data: *mut ExifData, offset: c_uint) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    unsafe {
        (*data_private(data)).offset_mnote = offset;
    }
}

pub(crate) unsafe fn exif_data_reset_impl(data: *mut ExifData) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    for ifd in 0..EXIF_IFD_COUNT as usize {
        let content = unsafe { (*data).ifd[ifd] };
        if content.is_null() {
            continue;
        }

        while unsafe { (*content).count } > 0 {
            let entries = unsafe { (*content).entries };
            if entries.is_null() {
                unsafe {
                    (*content).count = 0;
                }
                break;
            }

            let last = unsafe { *entries.add((*content).count as usize - 1) };
            unsafe { exif_content_remove_entry_impl(content, last) };
        }
    }

    if !unsafe { (*data).data }.is_null() {
        unsafe {
            exif_mem_free_impl(data_mem(data), (*data).data.cast());
            (*data).data = ptr::null_mut();
            (*data).size = 0;
        }
    }

    let private = unsafe { data_private(data) };
    unsafe {
        if !(*private).md.is_null() {
            crate::exif_mnote_data_unref((*private).md);
            (*private).md = ptr::null_mut();
        }
        (*private).offset_mnote = 0;
    }
}

pub(crate) unsafe fn exif_data_get_log_impl(data: *mut ExifData) -> *mut ExifLog {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*data_private(data)).log }
    }
}

pub(crate) unsafe fn exif_data_get_mnote_data_impl(data: *mut ExifData) -> *mut ExifMnoteData {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*data_private(data)).md }
    }
}

pub(crate) unsafe fn exif_data_get_byte_order_impl(data: *mut ExifData) -> ExifByteOrder {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        EXIF_BYTE_ORDER_MOTOROLA
    } else {
        unsafe { (*data_private(data)).order }
    }
}

pub(crate) unsafe fn exif_data_get_data_type_impl(data: *mut ExifData) -> ExifDataType {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        EXIF_DATA_TYPE_UNKNOWN
    } else {
        unsafe { (*data_private(data)).data_type }
    }
}

pub(crate) unsafe fn exif_data_new_mem_impl(mem: *mut ExifMem) -> *mut ExifData {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let data =
        unsafe { exif_mem_alloc_impl(mem, size_of::<ExifData>() as ExifLong) }.cast::<ExifData>();
    if data.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(data.cast::<u8>(), 0, size_of::<ExifData>());
    }

    let private = unsafe { exif_mem_alloc_impl(mem, size_of::<DataPrivate>() as ExifLong) }
        .cast::<DataPrivate>();
    if private.is_null() {
        unsafe { exif_mem_free_impl(mem, data.cast()) };
        return ptr::null_mut();
    }

    unsafe {
        ptr::write_bytes(private.cast::<u8>(), 0, size_of::<DataPrivate>());
        (*private).order = EXIF_BYTE_ORDER_MOTOROLA;
        (*private).md = ptr::null_mut();
        (*private).log = ptr::null_mut();
        (*private).mem = mem;
        (*private).ref_count = 1;
        (*private).offset_mnote = 0;
        (*private).options = 0;
        (*private).data_type = EXIF_DATA_TYPE_COUNT;
        (*data).data = ptr::null_mut();
        (*data).size = 0;
        (*data).priv_ = private.cast();
        exif_mem_ref_impl(mem);
    }

    for ifd in 0..EXIF_IFD_COUNT as usize {
        let content = unsafe { exif_content_new_mem_impl(mem) };
        if content.is_null() {
            unsafe { exif_data_free_impl(data) };
            return ptr::null_mut();
        }
        unsafe {
            (*content).parent = data;
            (*data).ifd[ifd] = content;
        }
    }

    unsafe {
        exif_data_set_option_impl(data, EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS);
        exif_data_set_option_impl(data, EXIF_DATA_OPTION_FOLLOW_SPECIFICATION);
        exif_data_set_data_type_impl(data, EXIF_DATA_TYPE_COUNT);
    }

    data
}

pub(crate) unsafe fn exif_data_new_impl() -> *mut ExifData {
    let mem = unsafe { exif_mem_new_default_impl() };
    let data = unsafe { exif_data_new_mem_impl(mem) };
    unsafe { exif_mem_unref_impl(mem) };
    data
}

pub(crate) unsafe fn exif_data_new_from_data_impl(
    source: *const c_uchar,
    size: c_uint,
) -> *mut ExifData {
    let data = unsafe { exif_data_new_impl() };
    if !data.is_null() {
        unsafe { crate::parser::data_load::exif_data_load_data_impl(data, source, size) };
    }
    data
}

pub(crate) unsafe fn exif_data_ref_impl(data: *mut ExifData) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *data_private(data) };
    private.ref_count = private.ref_count.wrapping_add(1);
}

pub(crate) unsafe fn exif_data_unref_impl(data: *mut ExifData) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *data_private(data) };
    private.ref_count = private.ref_count.wrapping_sub(1);
    if private.ref_count == 0 {
        unsafe { exif_data_free_impl(data) };
    }
}

pub(crate) unsafe fn exif_data_free_impl(data: *mut ExifData) {
    if data.is_null() {
        return;
    }

    let mem = unsafe { data_mem(data) };

    for ifd in 0..EXIF_IFD_COUNT as usize {
        let content = unsafe { (*data).ifd[ifd] };
        if !content.is_null() {
            unsafe {
                exif_content_unref_impl(content);
                (*data).ifd[ifd] = ptr::null_mut();
            }
        }
    }

    if !unsafe { (*data).data }.is_null() {
        unsafe {
            exif_mem_free_impl(mem, (*data).data.cast());
            (*data).data = ptr::null_mut();
            (*data).size = 0;
        }
    }

    if !unsafe { (*data).priv_ }.is_null() {
        let private = unsafe { data_private(data) };
        unsafe {
            exif_log_unref_impl((*private).log);
            if !(*private).md.is_null() {
                crate::exif_mnote_data_unref((*private).md);
                (*private).md = ptr::null_mut();
            }
            exif_mem_free_impl(mem, private.cast());
            exif_mem_free_impl(mem, data.cast());
            exif_mem_unref_impl(mem);
        }
    }
}

pub(crate) unsafe fn exif_data_foreach_content_impl(
    data: *mut ExifData,
    func: ExifDataForeachContentFunc,
    user_data: *mut c_void,
) {
    let Some(callback) = func else {
        return;
    };
    if data.is_null() {
        return;
    }

    for ifd in 0..EXIF_IFD_COUNT as usize {
        unsafe { callback((*data).ifd[ifd], user_data) };
    }
}

unsafe extern "C" fn entry_set_byte_order_callback(entry: *mut ExifEntry, data: *mut c_void) {
    if entry.is_null() || data.is_null() {
        return;
    }

    let change = unsafe { &*(data.cast::<ByteOrderChangeData>()) };
    unsafe {
        crate::primitives::utils::exif_array_set_byte_order(
            (*entry).format,
            (*entry).data,
            (*entry).components as c_uint,
            change.old,
            change.new,
        );
    }
}

unsafe extern "C" fn content_set_byte_order_callback(content: *mut ExifContent, data: *mut c_void) {
    unsafe {
        exif_content_foreach_entry_impl(content, Some(entry_set_byte_order_callback), data);
    }
}

pub(crate) unsafe fn exif_data_set_byte_order_impl(data: *mut ExifData, order: ExifByteOrder) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    let private = unsafe { data_private(data) };
    if unsafe { (*private).order } == order {
        return;
    }

    let mut change = ByteOrderChangeData {
        old: unsafe { (*private).order },
        new: order,
    };
    unsafe {
        exif_data_foreach_content_impl(
            data,
            Some(content_set_byte_order_callback),
            (&mut change as *mut ByteOrderChangeData).cast(),
        );
        (*private).order = order;
        if !(*private).md.is_null() {
            crate::exif_mnote_data_set_byte_order((*private).md, order);
        }
    }
}

pub(crate) unsafe fn exif_data_log_impl(data: *mut ExifData, log: *mut ExifLog) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *data_private(data) };
    unsafe { exif_log_unref_impl(private.log) };
    private.log = log;
    unsafe { exif_log_ref_impl(log) };

    for ifd in 0..EXIF_IFD_COUNT as usize {
        unsafe { exif_content_log_impl((*data).ifd[ifd], log) };
    }
}

pub(crate) unsafe fn exif_data_set_option_impl(data: *mut ExifData, option: ExifDataOption) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    unsafe {
        (*data_private(data)).options |= option;
    }
}

pub(crate) unsafe fn exif_data_unset_option_impl(data: *mut ExifData, option: ExifDataOption) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    unsafe {
        (*data_private(data)).options &= !option;
    }
}

pub(crate) unsafe fn exif_data_set_data_type_impl(data: *mut ExifData, data_type: ExifDataType) {
    if data.is_null() || unsafe { (*data).priv_ }.is_null() {
        return;
    }

    unsafe {
        (*data_private(data)).data_type = data_type;
    }
}

unsafe extern "C" fn fix_content_callback(content: *mut ExifContent, _data: *mut c_void) {
    if content.is_null() {
        return;
    }

    match unsafe { exif_content_get_ifd_impl(content) } {
        EXIF_IFD_1 => {
            if !unsafe { (*content).parent }.is_null() && !unsafe { (*(*content).parent).data }.is_null() {
                unsafe { exif_content_fix_impl(content) };
            } else {
                while unsafe { (*content).count } > 0 {
                    let entries = unsafe { (*content).entries };
                    if entries.is_null() {
                        break;
                    }
                    let last_index = unsafe { (*content).count as usize - 1 };
                    let last = unsafe { *entries.add(last_index) };
                    let count = unsafe { (*content).count };
                    unsafe { exif_content_remove_entry_impl(content, last) };
                    if count == unsafe { (*content).count } {
                        unsafe {
                            (*content).count = (*content).count.saturating_sub(1);
                        }
                    }
                }
            }
        }
        _ => unsafe { exif_content_fix_impl(content) },
    }
}

pub(crate) unsafe fn exif_data_fix_impl(data: *mut ExifData) {
    unsafe { exif_data_foreach_content_impl(data, Some(fix_content_callback), ptr::null_mut()) };
}

pub(crate) unsafe fn exif_data_dump_impl(data: *mut ExifData) {
    if data.is_null() {
        return;
    }

    for ifd in 0..EXIF_IFD_COUNT as usize {
        let content = unsafe { (*data).ifd[ifd] };
        if !content.is_null() && unsafe { (*content).count } > 0 {
            let name_ptr = unsafe { crate::primitives::ifd::exif_ifd_get_name(ifd as ExifIfd) };
            let name = if name_ptr.is_null() {
                ""
            } else {
                unsafe { CStr::from_ptr(name_ptr) }
                    .to_str()
                    .unwrap_or("")
            };
            println!(
                "Dumping IFD '{}'...",
                name
            );
            unsafe { exif_content_dump_impl(content, 0) };
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_new() -> *mut ExifData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_new_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_new_mem(mem: *mut ExifMem) -> *mut ExifData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_new_mem_impl(mem) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_new_from_data(
    source: *const c_uchar,
    size: c_uint,
) -> *mut ExifData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_new_from_data_impl(source, size) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_ref(data: *mut ExifData) {
    panic_boundary::call_void(|| unsafe { exif_data_ref_impl(data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_unref(data: *mut ExifData) {
    panic_boundary::call_void(|| unsafe { exif_data_unref_impl(data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_free(data: *mut ExifData) {
    panic_boundary::call_void(|| unsafe { exif_data_free_impl(data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_get_byte_order(data: *mut ExifData) -> ExifByteOrder {
    panic_boundary::call_or(EXIF_BYTE_ORDER_MOTOROLA, || unsafe {
        exif_data_get_byte_order_impl(data)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_set_byte_order(data: *mut ExifData, order: ExifByteOrder) {
    panic_boundary::call_void(|| unsafe { exif_data_set_byte_order_impl(data, order) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_get_mnote_data(data: *mut ExifData) -> *mut ExifMnoteData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_get_mnote_data_impl(data) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_fix(data: *mut ExifData) {
    panic_boundary::call_void(|| unsafe { exif_data_fix_impl(data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_foreach_content(
    data: *mut ExifData,
    func: ExifDataForeachContentFunc,
    user_data: *mut c_void,
) {
    panic_boundary::call_void(|| unsafe { exif_data_foreach_content_impl(data, func, user_data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_set_option(data: *mut ExifData, option: ExifDataOption) {
    panic_boundary::call_void(|| unsafe { exif_data_set_option_impl(data, option) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_unset_option(data: *mut ExifData, option: ExifDataOption) {
    panic_boundary::call_void(|| unsafe { exif_data_unset_option_impl(data, option) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_set_data_type(data: *mut ExifData, data_type: ExifDataType) {
    panic_boundary::call_void(|| unsafe { exif_data_set_data_type_impl(data, data_type) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_get_data_type(data: *mut ExifData) -> ExifDataType {
    panic_boundary::call_or(EXIF_DATA_TYPE_UNKNOWN, || unsafe {
        exif_data_get_data_type_impl(data)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_dump(data: *mut ExifData) {
    panic_boundary::call_void(|| unsafe { exif_data_dump_impl(data) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_log(data: *mut ExifData, log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_data_log_impl(data, log) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_get_log(data: *mut ExifData) -> *mut ExifLog {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_get_log_impl(data) })
}
