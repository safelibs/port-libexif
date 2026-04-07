use core::ffi::{c_char, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;

use std::ffi::CStr;
use std::fs::File;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifData, ExifLoader, ExifLog, ExifLong, ExifMem, EXIF_LOG_CODE_CORRUPT_DATA,
};
use crate::object::data::exif_data_new_mem_impl;
use crate::parser::data_load::exif_data_load_data_impl;
use crate::runtime::log::{exif_log_ref_impl, exif_log_unref_impl};
use crate::runtime::mem::{
    exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_new_default_impl, exif_mem_ref_impl,
    exif_mem_unref_impl,
};

const EXIF_HEADER: [u8; 6] = *b"Exif\0\0";
const JPEG_MARKER_DCT: u8 = 0xc0;
const JPEG_MARKER_DHT: u8 = 0xc4;
const JPEG_MARKER_SOI: u8 = 0xd8;
const JPEG_MARKER_DQT: u8 = 0xdb;
const JPEG_MARKER_APP0: u8 = 0xe0;
const JPEG_MARKER_APP1: u8 = 0xe1;
const JPEG_MARKER_APP2: u8 = 0xe2;
const JPEG_MARKER_APP4: u8 = 0xe4;
const JPEG_MARKER_APP5: u8 = 0xe5;
const JPEG_MARKER_APP11: u8 = 0xeb;
const JPEG_MARKER_APP13: u8 = 0xed;
const JPEG_MARKER_APP14: u8 = 0xee;
const JPEG_MARKER_COM: u8 = 0xfe;

#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum LoaderState {
    Read = 0,
    ReadSizeByte24,
    ReadSizeByte16,
    ReadSizeByte08,
    ReadSizeByte00,
    SkipBytes,
    ExifFound,
}

#[repr(u32)]
#[derive(Clone, Copy, Eq, PartialEq)]
enum LoaderDataFormat {
    Unknown = 0,
    Exif,
    Jpeg,
    FujiRaw,
}

#[repr(C)]
struct Loader {
    state: LoaderState,
    data_format: LoaderDataFormat,
    b: [u8; 12],
    b_len: u8,
    size: c_uint,
    buf: *mut c_uchar,
    bytes_read: c_uint,
    ref_count: u32,
    log: *mut ExifLog,
    mem: *mut ExifMem,
}

#[inline]
fn loader_private(loader: *mut ExifLoader) -> *mut Loader {
    loader.cast::<Loader>()
}

unsafe fn loader_alloc(loader: *mut ExifLoader, size: c_uint) -> *mut c_uchar {
    if loader.is_null() || size == 0 {
        return ptr::null_mut();
    }

    unsafe {
        exif_mem_alloc_impl((*loader_private(loader)).mem, size as ExifLong).cast::<c_uchar>()
    }
}

unsafe fn loader_copy(loader: *mut ExifLoader, buffer: *mut c_uchar, len: c_uint) -> c_uchar {
    if loader.is_null() || (len != 0 && buffer.is_null()) {
        return 0;
    }

    let raw = loader_private(loader);
    let loader = unsafe { &mut *raw };
    if loader.bytes_read >= loader.size {
        return 0;
    }

    if loader.buf.is_null() {
        loader.buf = unsafe { loader_alloc(raw.cast::<ExifLoader>(), loader.size) };
    }
    if loader.buf.is_null() {
        return 0;
    }

    let copy_len = len.min(loader.size.saturating_sub(loader.bytes_read));
    if copy_len != 0 {
        unsafe {
            ptr::copy_nonoverlapping(
                buffer.cast_const(),
                loader.buf.add(loader.bytes_read as usize),
                copy_len as usize,
            );
        }
        loader.bytes_read = loader.bytes_read.saturating_add(copy_len);
    }

    if loader.bytes_read >= loader.size {
        0
    } else {
        1
    }
}

fn matches_known_exif_prefix(window: &[u8], start: usize) -> bool {
    let available = window.len().saturating_sub(start);
    let compare_len = available.min(EXIF_HEADER.len());
    compare_len == 0 || window[start..start + compare_len] == EXIF_HEADER[..compare_len]
}

pub(crate) unsafe fn exif_loader_write_impl(
    loader: *mut ExifLoader,
    mut buffer: *mut c_uchar,
    mut len: c_uint,
) -> c_uchar {
    if loader.is_null() || (len != 0 && buffer.is_null()) {
        return 0;
    }

    loop {
        {
            let loader_ref = unsafe { &mut *loader_private(loader) };
            match loader_ref.state {
                LoaderState::ExifFound => {
                    return unsafe { loader_copy(loader, buffer, len) };
                }
                LoaderState::SkipBytes => {
                    if loader_ref.size > len {
                        loader_ref.size -= len;
                        return 1;
                    }

                    len -= loader_ref.size;
                    if loader_ref.size != 0 {
                        buffer = unsafe { buffer.add(loader_ref.size as usize) };
                    }
                    loader_ref.size = 0;
                    loader_ref.b_len = 0;
                    loader_ref.state = match loader_ref.data_format {
                        LoaderDataFormat::FujiRaw => LoaderState::ReadSizeByte24,
                        _ => LoaderState::Read,
                    };
                }
                _ => {}
            }
        }

        if len == 0 {
            return 1;
        }

        {
            let loader_ref = unsafe { &mut *loader_private(loader) };
            let to_copy = (loader_ref.b.len() - loader_ref.b_len as usize).min(len as usize);
            if to_copy != 0 {
                unsafe {
                    ptr::copy_nonoverlapping(
                        buffer.cast_const(),
                        loader_ref.b.as_mut_ptr().add(loader_ref.b_len as usize),
                        to_copy,
                    );
                }
                loader_ref.b_len += to_copy as u8;
                if loader_ref.b_len < loader_ref.b.len() as u8 {
                    return 1;
                }
                buffer = unsafe { buffer.add(to_copy) };
                len -= to_copy as c_uint;
            }
        }

        {
            let loader_ref = unsafe { &mut *loader_private(loader) };
            if loader_ref.data_format == LoaderDataFormat::Unknown {
                if loader_ref.b[..8] == *b"FUJIFILM" {
                    loader_ref.data_format = LoaderDataFormat::FujiRaw;
                    loader_ref.size = 84;
                    loader_ref.state = LoaderState::SkipBytes;
                } else if loader_ref.b[2..8] == EXIF_HEADER {
                    loader_ref.data_format = LoaderDataFormat::Exif;
                    loader_ref.size = 0;
                    loader_ref.state = LoaderState::ReadSizeByte08;
                }
            }
        }

        let mut index = 0usize;
        while index < 12 {
            let mut advance = true;
            let byte = unsafe { (*loader_private(loader)).b[index] };
            let loader_ref = unsafe { &mut *loader_private(loader) };

            match loader_ref.state {
                LoaderState::ExifFound => {
                    if unsafe {
                        loader_copy(
                            loader,
                            loader_ref.b.as_mut_ptr().add(index),
                            (loader_ref.b.len() - index) as c_uint,
                        )
                    } == 0
                    {
                        return 0;
                    }
                    return unsafe { loader_copy(loader, buffer, len) };
                }
                LoaderState::SkipBytes => match loader_ref.size {
                    0 => {
                        loader_ref.state = LoaderState::Read;
                        advance = false;
                    }
                    1 => {
                        loader_ref.size = 0;
                        loader_ref.state = LoaderState::Read;
                    }
                    _ => loader_ref.size -= 1,
                },
                LoaderState::ReadSizeByte24 => {
                    loader_ref.size |= (byte as c_uint) << 24;
                    loader_ref.state = LoaderState::ReadSizeByte16;
                }
                LoaderState::ReadSizeByte16 => {
                    loader_ref.size |= (byte as c_uint) << 16;
                    loader_ref.state = LoaderState::ReadSizeByte08;
                }
                LoaderState::ReadSizeByte08 => {
                    loader_ref.size |= (byte as c_uint) << 8;
                    loader_ref.state = LoaderState::ReadSizeByte00;
                }
                LoaderState::ReadSizeByte00 => {
                    loader_ref.size |= byte as c_uint;
                    match loader_ref.data_format {
                        LoaderDataFormat::Jpeg => {
                            loader_ref.state = LoaderState::SkipBytes;
                            if loader_ref.size >= 2 {
                                loader_ref.size -= 2;
                            } else {
                                loader_ref.size = 0;
                            }
                        }
                        LoaderDataFormat::FujiRaw => {
                            loader_ref.data_format = LoaderDataFormat::Exif;
                            loader_ref.state = LoaderState::SkipBytes;
                            if loader_ref.size >= 86 {
                                loader_ref.size -= 86;
                            } else {
                                loader_ref.size = 0;
                            }
                        }
                        LoaderDataFormat::Exif => {
                            loader_ref.state = LoaderState::ExifFound;
                        }
                        LoaderDataFormat::Unknown => {}
                    }
                }
                LoaderState::Read => match byte {
                    JPEG_MARKER_APP1 => {
                        if matches_known_exif_prefix(&loader_ref.b, index + 3) {
                            loader_ref.data_format = LoaderDataFormat::Exif;
                        } else {
                            loader_ref.data_format = LoaderDataFormat::Jpeg;
                        }
                        loader_ref.size = 0;
                        loader_ref.state = LoaderState::ReadSizeByte08;
                    }
                    JPEG_MARKER_DCT | JPEG_MARKER_DHT | JPEG_MARKER_DQT | JPEG_MARKER_APP0
                    | JPEG_MARKER_APP2 | JPEG_MARKER_APP4 | JPEG_MARKER_APP5
                    | JPEG_MARKER_APP11 | JPEG_MARKER_APP13 | JPEG_MARKER_APP14
                    | JPEG_MARKER_COM => {
                        loader_ref.data_format = LoaderDataFormat::Jpeg;
                        loader_ref.size = 0;
                        loader_ref.state = LoaderState::ReadSizeByte08;
                    }
                    0xff | JPEG_MARKER_SOI => {}
                    _ => {
                        let _ = EXIF_LOG_CODE_CORRUPT_DATA;
                        unsafe { exif_loader_reset_impl(loader) };
                        return 0;
                    }
                },
            }

            if advance {
                index += 1;
            }
        }

        unsafe {
            (*loader_private(loader)).b_len = 0;
        }
    }
}

pub(crate) unsafe fn exif_loader_new_mem_impl(mem: *mut ExifMem) -> *mut ExifLoader {
    if mem.is_null() {
        return ptr::null_mut();
    }

    let loader =
        unsafe { exif_mem_alloc_impl(mem, size_of::<Loader>() as ExifLong) }.cast::<Loader>();
    if loader.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        ptr::write_bytes(loader.cast::<u8>(), 0, size_of::<Loader>());
        (*loader).state = LoaderState::Read;
        (*loader).data_format = LoaderDataFormat::Unknown;
        (*loader).ref_count = 1;
        (*loader).mem = mem;
        exif_mem_ref_impl(mem);
    }

    loader.cast::<ExifLoader>()
}

pub(crate) unsafe fn exif_loader_new_impl() -> *mut ExifLoader {
    let mem = unsafe { exif_mem_new_default_impl() };
    let loader = unsafe { exif_loader_new_mem_impl(mem) };
    unsafe { exif_mem_unref_impl(mem) };
    loader
}

pub(crate) unsafe fn exif_loader_ref_impl(loader: *mut ExifLoader) {
    if loader.is_null() {
        return;
    }

    unsafe {
        (*loader_private(loader)).ref_count = (*loader_private(loader)).ref_count.wrapping_add(1);
    }
}

unsafe fn exif_loader_free_impl(loader: *mut ExifLoader) {
    if loader.is_null() {
        return;
    }

    let mem = unsafe { (*loader_private(loader)).mem };
    unsafe {
        exif_loader_reset_impl(loader);
        exif_log_unref_impl((*loader_private(loader)).log);
        exif_mem_free_impl(mem, loader.cast());
        exif_mem_unref_impl(mem);
    }
}

pub(crate) unsafe fn exif_loader_unref_impl(loader: *mut ExifLoader) {
    if loader.is_null() {
        return;
    }

    let loader_ref = unsafe { &mut *loader_private(loader) };
    loader_ref.ref_count = loader_ref.ref_count.wrapping_sub(1);
    if loader_ref.ref_count == 0 {
        unsafe { exif_loader_free_impl(loader) };
    }
}

pub(crate) unsafe fn exif_loader_reset_impl(loader: *mut ExifLoader) {
    if loader.is_null() {
        return;
    }

    let loader_ref = unsafe { &mut *loader_private(loader) };
    unsafe {
        exif_mem_free_impl(loader_ref.mem, loader_ref.buf.cast());
    }
    loader_ref.buf = ptr::null_mut();
    loader_ref.size = 0;
    loader_ref.bytes_read = 0;
    loader_ref.state = LoaderState::Read;
    loader_ref.data_format = LoaderDataFormat::Unknown;
    loader_ref.b_len = 0;
    loader_ref.b = [0; 12];
}

pub(crate) unsafe fn exif_loader_write_file_impl(loader: *mut ExifLoader, path: *const c_char) {
    if loader.is_null() || path.is_null() {
        return;
    }

    #[cfg(unix)]
    let Ok(mut file) = File::open(Path::new(std::ffi::OsStr::from_bytes(unsafe {
        CStr::from_ptr(path).to_bytes()
    }))) else {
        return;
    };
    #[cfg(not(unix))]
    let Ok(mut file) = File::open(
        unsafe { CStr::from_ptr(path) }
            .to_string_lossy()
            .into_owned(),
    ) else {
        return;
    };

    let mut chunk = [0u8; 8192];
    loop {
        let Ok(read) = file.read(&mut chunk) else {
            break;
        };
        if read == 0 {
            break;
        }
        if unsafe { exif_loader_write_impl(loader, chunk.as_mut_ptr(), read as c_uint) } == 0 {
            break;
        }
    }
}

pub(crate) unsafe fn exif_loader_get_data_impl(loader: *mut ExifLoader) -> *mut ExifData {
    if loader.is_null() {
        return ptr::null_mut();
    }

    let loader_ref = unsafe { &mut *loader_private(loader) };
    if loader_ref.data_format == LoaderDataFormat::Unknown || loader_ref.bytes_read == 0 {
        return ptr::null_mut();
    }

    let data = unsafe { exif_data_new_mem_impl(loader_ref.mem) };
    if data.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        crate::object::data::exif_data_log_impl(data, loader_ref.log);
        exif_data_load_data_impl(data, loader_ref.buf.cast_const(), loader_ref.bytes_read);
    }

    data
}

pub(crate) unsafe fn exif_loader_get_buf_impl(
    loader: *mut ExifLoader,
    buffer: *mut *const c_uchar,
    size: *mut c_uint,
) {
    let mut out_buffer = ptr::null();
    let mut out_size = 0;

    if !loader.is_null() {
        let loader_ref = unsafe { &*loader_private(loader) };
        if loader_ref.data_format != LoaderDataFormat::Unknown {
            out_buffer = loader_ref.buf.cast_const();
            out_size = loader_ref.bytes_read;
        }
    }

    unsafe {
        if !buffer.is_null() {
            *buffer = out_buffer;
        }
        if !size.is_null() {
            *size = out_size;
        }
    }
}

pub(crate) unsafe fn exif_loader_log_impl(loader: *mut ExifLoader, log: *mut ExifLog) {
    if loader.is_null() {
        return;
    }

    let loader_ref = unsafe { &mut *loader_private(loader) };
    unsafe {
        exif_log_unref_impl(loader_ref.log);
        loader_ref.log = log;
        exif_log_ref_impl(log);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_loader_new() -> *mut ExifLoader {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_loader_new_impl() })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_new_mem(mem: *mut ExifMem) -> *mut ExifLoader {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_loader_new_mem_impl(mem) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_ref(loader: *mut ExifLoader) {
    panic_boundary::call_void(|| unsafe { exif_loader_ref_impl(loader) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_unref(loader: *mut ExifLoader) {
    panic_boundary::call_void(|| unsafe { exif_loader_unref_impl(loader) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_write_file(loader: *mut ExifLoader, path: *const c_char) {
    panic_boundary::call_void(|| unsafe { exif_loader_write_file_impl(loader, path) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_write(
    loader: *mut ExifLoader,
    buffer: *mut c_uchar,
    size: c_uint,
) -> c_uchar {
    panic_boundary::call_or(0, || unsafe {
        exif_loader_write_impl(loader, buffer, size)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_reset(loader: *mut ExifLoader) {
    panic_boundary::call_void(|| unsafe { exif_loader_reset_impl(loader) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_get_data(loader: *mut ExifLoader) -> *mut ExifData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        exif_loader_get_data_impl(loader)
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_get_buf(
    loader: *mut ExifLoader,
    buffer: *mut *const c_uchar,
    size: *mut c_uint,
) {
    panic_boundary::call_void(|| unsafe { exif_loader_get_buf_impl(loader, buffer, size) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_loader_log(loader: *mut ExifLoader, log: *mut ExifLog) {
    panic_boundary::call_void(|| unsafe { exif_loader_log_impl(loader, log) });
}
