use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

use exif::ffi::types::*;

unsafe extern "C" {
    safe fn exif_data_new() -> *mut ExifData;
    fn exif_data_unref(data: *mut ExifData);
    fn exif_data_fix(data: *mut ExifData);
    fn exif_data_get_data_type(data: *mut ExifData) -> ExifDataType;
    fn exif_data_set_byte_order(data: *mut ExifData, order: ExifByteOrder);
    fn exif_data_save_data(data: *mut ExifData, buffer: *mut *mut u8, size: *mut u32);
    fn exif_content_add_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_content_remove_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_content_get_entry(content: *mut ExifContent, tag: ExifTag) -> *mut ExifEntry;
    fn exif_content_get_ifd(content: *mut ExifContent) -> ExifIfd;
    safe fn exif_entry_new() -> *mut ExifEntry;
    fn exif_entry_new_mem(mem: *mut ExifMem) -> *mut ExifEntry;
    fn exif_entry_unref(entry: *mut ExifEntry);
    fn exif_entry_initialize(entry: *mut ExifEntry, tag: ExifTag);
    fn exif_entry_get_value(
        entry: *mut ExifEntry,
        value: *mut c_char,
        maxlen: u32,
    ) -> *const c_char;
    safe fn exif_mem_new_default() -> *mut ExifMem;
    fn exif_mem_unref(mem: *mut ExifMem);
    fn exif_mem_alloc(mem: *mut ExifMem, size: ExifLong) -> *mut c_void;
    fn exif_mem_free(mem: *mut ExifMem, ptr_: *mut c_void);
    fn exif_loader_new() -> *mut ExifLoader;
    fn exif_loader_unref(loader: *mut ExifLoader);
    fn exif_loader_write(loader: *mut ExifLoader, buffer: *mut u8, size: u32) -> u8;
    fn exif_loader_get_buf(loader: *mut ExifLoader, buffer: *mut *const u8, size: *mut u32);
    fn exif_loader_get_data(loader: *mut ExifLoader) -> *mut ExifData;
    fn exif_loader_reset(loader: *mut ExifLoader);
    fn free(ptr_: *mut c_void);
}

const EXIF_TAG_IMAGE_WIDTH: ExifTag = 0x0100;
const EXIF_TAG_IMAGE_DESCRIPTION: ExifTag = 0x010e;
const EXIF_TAG_X_RESOLUTION: ExifTag = 0x011a;

#[test]
fn duplicate_tag_rejection_keeps_count_and_duplicate_unattached() {
    unsafe {
        let data = exif_data_new();
        assert!(!data.is_null());
        let content = (*data).ifd[EXIF_IFD_0 as usize];
        assert!(!content.is_null());

        let first = exif_entry_new();
        let duplicate = exif_entry_new();
        assert!(!first.is_null());
        assert!(!duplicate.is_null());

        (*first).tag = EXIF_TAG_IMAGE_WIDTH;
        (*duplicate).tag = EXIF_TAG_IMAGE_WIDTH;

        exif_content_add_entry(content, first);
        assert_eq!((*content).count, 1);
        assert_eq!((*first).parent, content);

        exif_content_add_entry(content, duplicate);
        assert_eq!((*content).count, 1);
        assert_eq!((*duplicate).parent, ptr::null_mut());
        assert_eq!(exif_content_get_entry(content, EXIF_TAG_IMAGE_WIDTH), first);

        (*content).count = 0;
        assert!(exif_content_get_entry(content, EXIF_TAG_IMAGE_WIDTH).is_null());
        (*content).count = 1;

        exif_entry_unref(first);
        exif_entry_unref(duplicate);
        exif_data_unref(data);
    }
}

#[test]
fn entry_value_reads_current_public_data_fields() {
    unsafe {
        let data = exif_data_new();
        let mem = exif_mem_new_default();
        assert!(!data.is_null());
        assert!(!mem.is_null());

        let content = (*data).ifd[EXIF_IFD_0 as usize];
        let entry = exif_entry_new_mem(mem);
        assert!(!entry.is_null());

        exif_content_add_entry(content, entry);
        exif_entry_initialize(entry, EXIF_TAG_IMAGE_DESCRIPTION);

        let old_data = (*entry).data;
        exif_mem_free(mem, old_data.cast());

        let replacement = b"Changed\0";
        let new_data = exif_mem_alloc(mem, replacement.len() as ExifLong).cast::<u8>();
        assert!(!new_data.is_null());
        ptr::copy_nonoverlapping(replacement.as_ptr(), new_data, replacement.len());
        (*entry).data = new_data;
        (*entry).size = replacement.len() as u32;
        (*entry).components = replacement.len() as u64;

        let mut buffer = [0 as c_char; 32];
        let returned = exif_entry_get_value(entry, buffer.as_mut_ptr(), buffer.len() as u32);
        assert_eq!(returned, buffer.as_ptr());
        assert_eq!(c_str(buffer.as_ptr()), "Changed");

        exif_content_remove_entry(content, entry);
        exif_entry_unref(entry);
        exif_mem_unref(mem);
        exif_data_unref(data);
    }
}

#[test]
fn data_fix_populates_mandatory_entries_and_null_defaults_match_phase_requirements() {
    unsafe {
        assert_eq!(exif_content_get_ifd(ptr::null_mut()), EXIF_IFD_COUNT);
        assert_eq!(
            exif_data_get_data_type(ptr::null_mut()),
            EXIF_DATA_TYPE_UNKNOWN
        );

        let mut buffer = [b'*' as c_char; 4];
        let returned =
            exif_entry_get_value(ptr::null_mut(), buffer.as_mut_ptr(), buffer.len() as u32);
        assert_eq!(returned, buffer.as_ptr());
        assert_eq!(buffer[0], b'*' as c_char);

        let data = exif_data_new();
        assert!(!data.is_null());
        exif_data_fix(data);

        let ifd0 = (*data).ifd[EXIF_IFD_0 as usize];
        assert_eq!(exif_content_get_ifd(ifd0), EXIF_IFD_0);
        assert!(!exif_content_get_entry(ifd0, EXIF_TAG_X_RESOLUTION).is_null());

        exif_data_unref(data);
    }
}

#[test]
fn loader_roundtrip_retains_saved_bytes_and_reset_clears_them() {
    unsafe {
        let data = exif_data_new();
        assert!(!data.is_null());
        exif_data_set_byte_order(data, EXIF_BYTE_ORDER_INTEL);

        let ifd0 = (*data).ifd[EXIF_IFD_0 as usize];
        assert!(!ifd0.is_null());

        let entry = exif_entry_new();
        assert!(!entry.is_null());
        exif_content_add_entry(ifd0, entry);
        exif_entry_initialize(entry, EXIF_TAG_IMAGE_DESCRIPTION);
        exif_entry_unref(entry);

        let mut raw_data = ptr::null_mut();
        let mut raw_size = 0;
        exif_data_save_data(data, &mut raw_data, &mut raw_size);
        assert!(!raw_data.is_null());
        assert!(raw_size > 0);

        let wrapped = wrap_loader_data(slice::from_raw_parts(raw_data, raw_size as usize));
        let loader = exif_loader_new();
        assert!(!loader.is_null());
        assert_eq!(exif_loader_write(loader, wrapped.as_ptr().cast_mut(), 1), 1);
        assert_eq!(
            exif_loader_write(
                loader,
                wrapped.as_ptr().cast_mut().add(1),
                (wrapped.len() - 1) as u32,
            ),
            0
        );

        let mut loader_buf = ptr::null();
        let mut loader_size = 0;
        exif_loader_get_buf(loader, &mut loader_buf, &mut loader_size);
        assert_eq!(loader_size, raw_size);
        assert!(!loader_buf.is_null());
        assert_eq!(
            slice::from_raw_parts(loader_buf, loader_size as usize),
            slice::from_raw_parts(raw_data, raw_size as usize)
        );

        let from_loader = exif_loader_get_data(loader);
        assert!(!from_loader.is_null());
        let from_loader_ifd0 = (*from_loader).ifd[EXIF_IFD_0 as usize];
        assert!(!from_loader_ifd0.is_null());
        assert!(!exif_content_get_entry(from_loader_ifd0, EXIF_TAG_IMAGE_DESCRIPTION).is_null());
        exif_data_unref(from_loader);

        exif_loader_reset(loader);
        exif_loader_get_buf(loader, &mut loader_buf, &mut loader_size);
        assert!(loader_buf.is_null());
        assert_eq!(loader_size, 0);
        assert!(exif_loader_get_data(loader).is_null());

        exif_loader_unref(loader);
        free(raw_data.cast());
        exif_data_unref(data);
    }
}

fn c_str(ptr_: *const c_char) -> String {
    unsafe { CStr::from_ptr(ptr_) }
        .to_string_lossy()
        .into_owned()
}

fn wrap_loader_data(raw_data: &[u8]) -> Vec<u8> {
    let mut wrapped = vec![0u8; raw_data.len() + 2];
    wrapped[..2].copy_from_slice(&(raw_data.len() as u16).to_be_bytes());
    wrapped[2..].copy_from_slice(raw_data);
    wrapped
}
