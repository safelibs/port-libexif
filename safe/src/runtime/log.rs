use core::ffi::{c_char, c_void};
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifLog, ExifLogCode, ExifLogFunc, ExifMem, EXIF_LOG_CODE_CORRUPT_DATA, EXIF_LOG_CODE_DEBUG,
    EXIF_LOG_CODE_NO_MEMORY,
};
use crate::i18n::{gettext, message, Message};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_new_default_impl, exif_mem_ref_impl,
    exif_mem_unref_impl,
};

#[repr(C)]
pub(crate) struct ExifLogPrivate {
    pub ref_count: u32,
    pub func: ExifLogFunc,
    pub data: *mut c_void,
    pub mem: *mut ExifMem,
}

#[derive(Clone, Copy)]
struct LogCodeEntry {
    code: ExifLogCode,
    title: Message,
    message: Message,
}

const LOG_CODES: [LogCodeEntry; 3] = [
    LogCodeEntry {
        code: EXIF_LOG_CODE_DEBUG,
        title: message(b"Debugging information\0"),
        message: message(b"Debugging information is available.\0"),
    },
    LogCodeEntry {
        code: EXIF_LOG_CODE_NO_MEMORY,
        title: message(b"Not enough memory\0"),
        message: message(b"The system cannot provide enough memory.\0"),
    },
    LogCodeEntry {
        code: EXIF_LOG_CODE_CORRUPT_DATA,
        title: message(b"Corrupt data\0"),
        message: message(b"The data provided does not follow the specification.\0"),
    },
];

#[inline]
fn log_private(log: *mut ExifLog) -> *mut ExifLogPrivate {
    log.cast()
}

pub(crate) unsafe fn exif_log_new_mem_impl(mem: *mut ExifMem) -> *mut ExifLog {
    let log = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifLogPrivate>() as u32) }
        .cast::<ExifLogPrivate>();
    if log.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        (*log).ref_count = 1;
        (*log).func = None;
        (*log).data = ptr::null_mut();
        (*log).mem = mem;
        exif_mem_ref_impl(mem);
    }

    log.cast()
}

pub(crate) unsafe fn exif_log_new_impl() -> *mut ExifLog {
    let mem = unsafe { exif_mem_new_default_impl() };
    let log = unsafe { exif_log_new_mem_impl(mem) };
    unsafe { exif_mem_unref_impl(mem) };
    log
}

pub(crate) unsafe fn exif_log_ref_impl(log: *mut ExifLog) {
    if log.is_null() {
        return;
    }

    let private = unsafe { &mut *log_private(log) };
    private.ref_count = private.ref_count.wrapping_add(1);
}

pub(crate) unsafe fn exif_log_free_impl(log: *mut ExifLog) {
    if log.is_null() {
        return;
    }

    let mem = unsafe { (*log_private(log)).mem };
    unsafe { exif_mem_free_impl(mem, log.cast()) };
    unsafe { exif_mem_unref_impl(mem) };
}

pub(crate) unsafe fn exif_log_unref_impl(log: *mut ExifLog) {
    if log.is_null() {
        return;
    }

    let private = unsafe { &mut *log_private(log) };
    if private.ref_count > 0 {
        private.ref_count -= 1;
    }
    if private.ref_count == 0 {
        unsafe { exif_log_free_impl(log) };
    }
}

pub(crate) unsafe fn exif_log_set_func_impl(
    log: *mut ExifLog,
    func: ExifLogFunc,
    data: *mut c_void,
) {
    if log.is_null() {
        return;
    }

    unsafe {
        (*log_private(log)).func = func;
        (*log_private(log)).data = data;
    }
}

fn find_code(code: ExifLogCode) -> Option<LogCodeEntry> {
    LOG_CODES.iter().copied().find(|entry| entry.code == code)
}

pub(crate) fn exif_log_code_get_title_impl(code: ExifLogCode) -> *const c_char {
    find_code(code).map_or(ptr::null(), |entry| gettext(entry.title))
}

pub(crate) fn exif_log_code_get_message_impl(code: ExifLogCode) -> *const c_char {
    find_code(code).map_or(ptr::null(), |entry| gettext(entry.message))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_log_new_mem(mem: *mut ExifMem) -> *mut ExifLog {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_log_new_mem_impl(mem) })
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_log_new() -> *mut ExifLog {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_log_new_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_log_ref(log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_log_ref_impl(log) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_log_unref(log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_log_unref_impl(log) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_log_free(log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_log_free_impl(log) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_log_set_func(
    log: *mut ExifLog,
    func: ExifLogFunc,
    data: *mut c_void,
) {
    panic_boundary::call_void(|| unsafe { exif_log_set_func_impl(log, func, data) });
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_log_code_get_title(code: ExifLogCode) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_log_code_get_title_impl(code))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_log_code_get_message(code: ExifLogCode) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_log_code_get_message_impl(code))
}
