use core::ffi::{c_uint, c_void};
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::*;
use crate::object::data::exif_data_get_data_type_impl;
use crate::object::entry::{
    exif_entry_fix_impl, exif_entry_initialize_impl, exif_entry_new_impl, exif_entry_ref_impl,
    exif_entry_unref_impl,
};
use crate::runtime::cstdio::print_line;
use crate::runtime::log::{exif_log_ref_impl, exif_log_unref_impl};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_new_default_impl, exif_mem_realloc_impl,
    exif_mem_ref_impl, exif_mem_unref_impl,
};
use crate::tables::tag_table::{exif_tag_get_support_level_in_ifd, TAG_TABLE};

#[repr(C)]
pub(crate) struct ContentPrivate {
    ref_count: u32,
    mem: *mut ExifMem,
    log: *mut ExifLog,
    entry_capacity: c_uint,
}

#[inline]
unsafe fn content_private(content: *mut ExifContent) -> *mut ContentPrivate {
    unsafe { (*content).priv_.cast::<ContentPrivate>() }
}

#[inline]
unsafe fn content_mem(content: *mut ExifContent) -> *mut ExifMem {
    if content.is_null() || unsafe { (*content).priv_ }.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*content_private(content)).mem }
    }
}

pub(crate) unsafe fn exif_content_new_mem_impl(mem: *mut ExifMem) -> *mut ExifContent {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let content = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifContent>() as ExifLong) }
        .cast::<ExifContent>();
    if content.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(content.cast::<u8>(), 0, size_of::<ExifContent>());
    }

    let private = unsafe { exif_mem_alloc_impl(mem, size_of::<ContentPrivate>() as ExifLong) }
        .cast::<ContentPrivate>();
    if private.is_null() {
        unsafe { exif_mem_free_impl(mem, content.cast()) };
        return ptr::null_mut();
    }
    unsafe {
        ptr::write_bytes(private.cast::<u8>(), 0, size_of::<ContentPrivate>());
        (*private).ref_count = 1;
        (*private).mem = mem;
        (*private).log = ptr::null_mut();
        (*private).entry_capacity = 0;
        (*content).entries = ptr::null_mut();
        (*content).count = 0;
        (*content).parent = ptr::null_mut();
        (*content).priv_ = private.cast();
        exif_mem_ref_impl(mem);
    }

    content
}

pub(crate) unsafe fn exif_content_new_impl() -> *mut ExifContent {
    let mem = unsafe { exif_mem_new_default_impl() };
    let content = unsafe { exif_content_new_mem_impl(mem) };
    unsafe { exif_mem_unref_impl(mem) };
    content
}

pub(crate) unsafe fn exif_content_ref_impl(content: *mut ExifContent) {
    if content.is_null() || unsafe { (*content).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *content_private(content) };
    private.ref_count = private.ref_count.wrapping_add(1);
}

pub(crate) unsafe fn exif_content_unref_impl(content: *mut ExifContent) {
    if content.is_null() || unsafe { (*content).priv_ }.is_null() {
        return;
    }

    let private = unsafe { &mut *content_private(content) };
    private.ref_count = private.ref_count.wrapping_sub(1);
    if private.ref_count == 0 {
        unsafe { exif_content_free_impl(content) };
    }
}

pub(crate) unsafe fn exif_content_free_impl(content: *mut ExifContent) {
    if content.is_null() {
        return;
    }

    let mem = unsafe { content_mem(content) };

    if !unsafe { (*content).entries }.is_null() {
        for index in 0..unsafe { (*content).count as usize } {
            let entry = unsafe { *(*content).entries.add(index) };
            unsafe { exif_entry_unref_impl(entry) };
        }
        unsafe { exif_mem_free_impl(mem, (*content).entries.cast()) };
    }

    if !unsafe { (*content).priv_ }.is_null() {
        let private = unsafe { content_private(content) };
        unsafe { exif_log_unref_impl((*private).log) };
        unsafe { exif_mem_free_impl(mem, private.cast()) };
    }

    unsafe { exif_mem_free_impl(mem, content.cast()) };
    unsafe { exif_mem_unref_impl(mem) };
}

pub(crate) unsafe fn exif_content_add_entry_impl(content: *mut ExifContent, entry: *mut ExifEntry) {
    if content.is_null()
        || unsafe { (*content).priv_ }.is_null()
        || entry.is_null()
        || !unsafe { (*entry).parent }.is_null()
    {
        return;
    }

    if !unsafe { exif_content_get_entry_impl(content, (*entry).tag) }.is_null() {
        return;
    }

    let count = unsafe { (*content).count as usize };
    let Some(new_len) = count.checked_add(1) else {
        return;
    };

    let current_capacity = unsafe { (*content_private(content)).entry_capacity as usize };
    let target_capacity = if new_len <= current_capacity {
        current_capacity
    } else {
        current_capacity.max(4).saturating_mul(2).max(new_len)
    };
    if unsafe { !exif_content_reserve_entries_impl(content, target_capacity) } {
        return;
    }

    let entries = unsafe { (*content).entries };
    if entries.is_null() {
        return;
    }

    unsafe {
        (*entry).parent = content;
        *entries.add(count) = entry;
        (*content).count = new_len as c_uint;
        exif_entry_ref_impl(entry);
    }
}

pub(crate) unsafe fn exif_content_reserve_entries_impl(
    content: *mut ExifContent,
    capacity: usize,
) -> bool {
    if content.is_null() || unsafe { (*content).priv_ }.is_null() {
        return false;
    }

    let private = unsafe { &mut *content_private(content) };
    let current_capacity = private.entry_capacity as usize;
    if capacity <= current_capacity {
        return true;
    }

    let Some(bytes) = capacity.checked_mul(size_of::<*mut ExifEntry>()) else {
        return false;
    };
    let entries = unsafe {
        exif_mem_realloc_impl(
            content_mem(content),
            (*content).entries.cast(),
            bytes as ExifLong,
        )
    }
    .cast::<*mut ExifEntry>();
    if entries.is_null() {
        return false;
    }

    unsafe {
        (*content).entries = entries;
    }
    private.entry_capacity = capacity as c_uint;
    true
}

pub(crate) unsafe fn exif_content_remove_entry_impl(
    content: *mut ExifContent,
    entry: *mut ExifEntry,
) {
    if content.is_null()
        || unsafe { (*content).priv_ }.is_null()
        || entry.is_null()
        || unsafe { (*entry).parent } != content
    {
        return;
    }

    let count = unsafe { (*content).count as usize };
    let entries = unsafe { (*content).entries };
    if count == 0 || entries.is_null() {
        return;
    }

    let mut index = 0usize;
    while index < count {
        if unsafe { *entries.add(index) } == entry {
            break;
        }
        index += 1;
    }
    if index == count {
        return;
    }

    if count > 1 {
        let new_count = count - 1;
        unsafe {
            if index != new_count {
                ptr::copy(
                    entries.add(index + 1),
                    entries.add(index),
                    new_count - index,
                );
            }
            *entries.add(new_count) = ptr::null_mut();
            (*content).count = new_count as c_uint;
        }
    } else {
        unsafe {
            exif_mem_free_impl(content_mem(content), entries.cast());
            (*content).entries = ptr::null_mut();
            (*content).count = 0;
            (*content_private(content)).entry_capacity = 0;
        }
    }

    unsafe {
        (*entry).parent = ptr::null_mut();
        exif_entry_unref_impl(entry);
    }
}

pub(crate) unsafe fn exif_content_get_entry_impl(
    content: *mut ExifContent,
    tag: ExifTag,
) -> *mut ExifEntry {
    if content.is_null() {
        return ptr::null_mut();
    }

    let count = unsafe { (*content).count as usize };
    let entries = unsafe { (*content).entries };
    if count == 0 || entries.is_null() {
        return ptr::null_mut();
    }

    for index in 0..count {
        let entry = unsafe { *entries.add(index) };
        if !entry.is_null() && unsafe { (*entry).tag == tag } {
            return entry;
        }
    }

    ptr::null_mut()
}

pub(crate) unsafe fn exif_content_foreach_entry_impl(
    content: *mut ExifContent,
    func: ExifContentForeachEntryFunc,
    user_data: *mut c_void,
) {
    let Some(callback) = func else {
        return;
    };
    if content.is_null() {
        return;
    }

    let count = unsafe { (*content).count as usize };
    let entries = unsafe { (*content).entries };
    if count > 0 && entries.is_null() {
        return;
    }

    for index in 0..count {
        unsafe { callback(*entries.add(index), user_data) };
    }
}

pub(crate) unsafe fn exif_content_log_impl(content: *mut ExifContent, log: *mut ExifLog) {
    if content.is_null()
        || unsafe { (*content).priv_ }.is_null()
        || log.is_null()
        || unsafe { (*content_private(content)).log } == log
    {
        return;
    }

    let private = unsafe { &mut *content_private(content) };
    unsafe { exif_log_unref_impl(private.log) };
    private.log = log;
    unsafe { exif_log_ref_impl(log) };
}

pub(crate) unsafe fn exif_content_get_ifd_impl(content: *mut ExifContent) -> ExifIfd {
    if content.is_null() || unsafe { (*content).parent }.is_null() {
        return EXIF_IFD_COUNT;
    }

    let data = unsafe { (*content).parent };
    unsafe {
        if (*data).ifd[EXIF_IFD_EXIF as usize] == content {
            EXIF_IFD_EXIF
        } else if (*data).ifd[EXIF_IFD_0 as usize] == content {
            EXIF_IFD_0
        } else if (*data).ifd[EXIF_IFD_1 as usize] == content {
            EXIF_IFD_1
        } else if (*data).ifd[EXIF_IFD_GPS as usize] == content {
            EXIF_IFD_GPS
        } else if (*data).ifd[EXIF_IFD_INTEROPERABILITY as usize] == content {
            EXIF_IFD_INTEROPERABILITY
        } else {
            EXIF_IFD_COUNT
        }
    }
}

unsafe extern "C" fn fix_entry_callback(entry: *mut ExifEntry, _data: *mut c_void) {
    unsafe { exif_entry_fix_impl(entry) };
}

unsafe extern "C" fn remove_not_recorded_callback(entry: *mut ExifEntry, _data: *mut c_void) {
    if entry.is_null() || unsafe { (*entry).parent }.is_null() {
        return;
    }

    let content = unsafe { (*entry).parent };
    let ifd = unsafe { exif_content_get_ifd_impl(content) };
    let data_type = unsafe { exif_data_get_data_type_impl((*content).parent) };
    let support = unsafe { exif_tag_get_support_level_in_ifd((*entry).tag, ifd, data_type) };
    if support == EXIF_SUPPORT_LEVEL_NOT_RECORDED {
        unsafe { exif_content_remove_entry_impl(content, entry) };
    }
}

pub(crate) unsafe fn exif_content_fix_impl(content: *mut ExifContent) {
    if content.is_null() {
        return;
    }

    let ifd = unsafe { exif_content_get_ifd_impl(content) };
    let data_type = unsafe { exif_data_get_data_type_impl((*content).parent) };

    unsafe {
        exif_content_foreach_entry_impl(content, Some(fix_entry_callback), ptr::null_mut());
    }

    loop {
        let count = unsafe { (*content).count };
        unsafe {
            exif_content_foreach_entry_impl(
                content,
                Some(remove_not_recorded_callback),
                ptr::null_mut(),
            );
        }
        if count == unsafe { (*content).count } {
            break;
        }
    }

    for tag_entry in TAG_TABLE.iter() {
        if tag_entry.name.is_none() {
            break;
        }
        let support = unsafe { exif_tag_get_support_level_in_ifd(tag_entry.tag, ifd, data_type) };
        if support != EXIF_SUPPORT_LEVEL_MANDATORY {
            continue;
        }
        if !unsafe { exif_content_get_entry_impl(content, tag_entry.tag) }.is_null() {
            continue;
        }

        let entry = unsafe { exif_entry_new_impl() };
        if entry.is_null() {
            continue;
        }
        unsafe {
            exif_content_add_entry_impl(content, entry);
            exif_entry_initialize_impl(entry, tag_entry.tag);
            exif_entry_unref_impl(entry);
        }
    }
}

pub(crate) unsafe fn exif_content_dump_impl(content: *mut ExifContent, indent: c_uint) {
    if content.is_null() {
        return;
    }

    let depth = (indent as usize).saturating_mul(2);
    let prefix = " ".repeat(depth.min(1023));
    print_line(&format!(
        "{prefix}Dumping exif content ({} entries)...",
        unsafe { (*content).count }
    ));

    let count = unsafe { (*content).count as usize };
    let entries = unsafe { (*content).entries };
    if count > 0 && entries.is_null() {
        return;
    }
    for index in 0..count {
        unsafe {
            crate::object::entry::exif_entry_dump_impl(*entries.add(index), indent + 1);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_new() -> *mut ExifContent {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_content_new_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_new_mem(mem: *mut ExifMem) -> *mut ExifContent {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_content_new_mem_impl(mem)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_ref(content: *mut ExifContent) {
    panic_boundary::call_void(|| unsafe { exif_content_ref_impl(content) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_unref(content: *mut ExifContent) {
    panic_boundary::call_void(|| unsafe { exif_content_unref_impl(content) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_free(content: *mut ExifContent) {
    panic_boundary::call_void(|| unsafe { exif_content_free_impl(content) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_add_entry(content: *mut ExifContent, entry: *mut ExifEntry) {
    panic_boundary::call_void(|| unsafe { exif_content_add_entry_impl(content, entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_remove_entry(
    content: *mut ExifContent,
    entry: *mut ExifEntry,
) {
    panic_boundary::call_void(|| unsafe { exif_content_remove_entry_impl(content, entry) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_get_entry(
    content: *mut ExifContent,
    tag: ExifTag,
) -> *mut ExifEntry {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_content_get_entry_impl(content, tag)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_fix(content: *mut ExifContent) {
    panic_boundary::call_void(|| unsafe { exif_content_fix_impl(content) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_foreach_entry(
    content: *mut ExifContent,
    func: ExifContentForeachEntryFunc,
    user_data: *mut c_void,
) {
    panic_boundary::call_void(|| unsafe {
        exif_content_foreach_entry_impl(content, func, user_data)
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_get_ifd(content: *mut ExifContent) -> ExifIfd {
    panic_boundary::call_or(EXIF_IFD_COUNT, || unsafe {
        exif_content_get_ifd_impl(content)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_dump(content: *mut ExifContent, indent: c_uint) {
    panic_boundary::call_void(|| unsafe { exif_content_dump_impl(content, indent) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_content_log(content: *mut ExifContent, log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_content_log_impl(content, log) });
}
