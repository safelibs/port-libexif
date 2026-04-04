use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{ExifLong, ExifMem, ExifMemAllocFunc, ExifMemFreeFunc, ExifMemReallocFunc};

unsafe extern "C" {
    fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

#[repr(C)]
pub(crate) struct ExifMemPrivate {
    ref_count: u32,
    alloc_func: ExifMemAllocFunc,
    realloc_func: ExifMemReallocFunc,
    free_func: ExifMemFreeFunc,
}

#[inline]
unsafe fn mem_private(mem: *mut ExifMem) -> *mut ExifMemPrivate {
    mem.cast()
}

unsafe extern "C" fn default_alloc(size: ExifLong) -> *mut c_void {
    unsafe { calloc(size as usize, 1) }
}

unsafe extern "C" fn default_realloc(ptr_: *mut c_void, size: ExifLong) -> *mut c_void {
    unsafe { realloc(ptr_, size as usize) }
}

unsafe extern "C" fn default_free(ptr_: *mut c_void) {
    unsafe { free(ptr_) };
}

pub(crate) unsafe fn exif_mem_new_impl(
    alloc_func: ExifMemAllocFunc,
    realloc_func: ExifMemReallocFunc,
    free_func: ExifMemFreeFunc,
) -> *mut ExifMem {
    if alloc_func.is_none() && realloc_func.is_none() {
        return ptr::null_mut();
    }

    let size = size_of::<ExifMemPrivate>() as ExifLong;
    let mem = if let Some(alloc) = alloc_func {
        unsafe { alloc(size) }
    } else if let Some(realloc_fn) = realloc_func {
        unsafe { realloc_fn(ptr::null_mut(), size) }
    } else {
        ptr::null_mut()
    };

    if mem.is_null() {
        return ptr::null_mut();
    }

    let private = unsafe { &mut *mem.cast::<ExifMemPrivate>() };
    private.ref_count = 1;
    private.alloc_func = alloc_func;
    private.realloc_func = realloc_func;
    private.free_func = free_func;

    mem.cast()
}

pub(crate) unsafe fn exif_mem_new_default_impl() -> *mut ExifMem {
    unsafe {
        exif_mem_new_impl(
            Some(default_alloc),
            Some(default_realloc),
            Some(default_free),
        )
    }
}

pub(crate) unsafe fn exif_mem_ref_impl(mem: *mut ExifMem) {
    if mem.is_null() {
        return;
    }

    let private = unsafe { &mut *mem_private(mem) };
    private.ref_count = private.ref_count.wrapping_add(1);
}

pub(crate) unsafe fn exif_mem_free_impl(mem: *mut ExifMem, ptr_: *mut c_void) {
    if mem.is_null() {
        return;
    }

    if let Some(free_func) = unsafe { (*mem_private(mem)).free_func } {
        unsafe { free_func(ptr_) };
    }
}

pub(crate) unsafe fn exif_mem_unref_impl(mem: *mut ExifMem) {
    if mem.is_null() {
        return;
    }

    let private = unsafe { &mut *mem_private(mem) };
    private.ref_count = private.ref_count.wrapping_sub(1);
    if private.ref_count == 0 {
        unsafe { exif_mem_free_impl(mem, mem.cast()) };
    }
}

pub(crate) unsafe fn exif_mem_alloc_impl(mem: *mut ExifMem, size: ExifLong) -> *mut c_void {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let private = unsafe { &*mem_private(mem) };
    if let Some(alloc_func) = private.alloc_func {
        return unsafe { alloc_func(size) };
    }
    if let Some(realloc_func) = private.realloc_func {
        return unsafe { realloc_func(ptr::null_mut(), size) };
    }

    ptr::null_mut()
}

pub(crate) unsafe fn exif_mem_realloc_impl(
    mem: *mut ExifMem,
    ptr_: *mut c_void,
    size: ExifLong,
) -> *mut c_void {
    if mem.is_null() {
        return ptr::null_mut();
    }

    match unsafe { (*mem_private(mem)).realloc_func } {
        Some(realloc_func) => unsafe { realloc_func(ptr_, size) },
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_new(
    alloc_func: ExifMemAllocFunc,
    realloc_func: ExifMemReallocFunc,
    free_func: ExifMemFreeFunc,
) -> *mut ExifMem {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_mem_new_impl(alloc_func, realloc_func, free_func)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_new_default() -> *mut ExifMem {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_mem_new_default_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_ref(mem: *mut ExifMem) {
    panic_boundary::call_void(|| unsafe { exif_mem_ref_impl(mem) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_unref(mem: *mut ExifMem) {
    panic_boundary::call_void(|| unsafe { exif_mem_unref_impl(mem) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_alloc(mem: *mut ExifMem, size: ExifLong) -> *mut c_void {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_mem_alloc_impl(mem, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_realloc(
    mem: *mut ExifMem,
    ptr_: *mut c_void,
    size: ExifLong,
) -> *mut c_void {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_mem_realloc_impl(mem, ptr_, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mem_free(mem: *mut ExifMem, ptr_: *mut c_void) {
    panic_boundary::call_void(|| unsafe { exif_mem_free_impl(mem, ptr_) });
}
