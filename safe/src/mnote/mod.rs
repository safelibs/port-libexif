pub mod apple;
pub mod base;
pub mod canon;
pub mod fuji;
pub mod olympus;
pub mod pentax;

use core::ffi::c_uint;

use crate::ffi::types::{ExifData, ExifEntry, ExifMnoteData, ExifTag};
use crate::object::content::exif_content_get_entry_impl;
use crate::object::data::{
    exif_data_get_byte_order_impl, exif_data_get_log_impl, exif_data_get_mem_impl,
    exif_data_get_mnote_offset_impl, exif_data_get_options_impl, exif_data_set_mnote_data_impl,
};

const EXIF_TAG_MAKER_NOTE: ExifTag = 0x927c;

pub(crate) unsafe fn find_entry_impl(data: *mut ExifData, tag: ExifTag) -> *mut ExifEntry {
    if data.is_null() {
        return core::ptr::null_mut();
    }

    let exif_content = unsafe { (*data).ifd[crate::ffi::types::EXIF_IFD_EXIF as usize] };
    let exif_entry = unsafe { exif_content_get_entry_impl(exif_content, tag) };
    if !exif_entry.is_null() {
        return exif_entry;
    }

    for ifd in 0..crate::ffi::types::EXIF_IFD_COUNT as usize {
        if ifd == crate::ffi::types::EXIF_IFD_EXIF as usize {
            continue;
        }
        let content = unsafe { (*data).ifd[ifd] };
        let entry = unsafe { exif_content_get_entry_impl(content, tag) };
        if !entry.is_null() {
            return entry;
        }
    }

    core::ptr::null_mut()
}

unsafe fn construct_note_impl(data: *mut ExifData, entry: *mut ExifEntry) -> *mut ExifMnoteData {
    if data.is_null() || entry.is_null() {
        return core::ptr::null_mut();
    }

    let options = unsafe { exif_data_get_options_impl(data) };
    let mem = unsafe { exif_data_get_mem_impl(data) };

    if unsafe { olympus::identify_impl(data.cast_const(), entry.cast_const()) } != 0 {
        return unsafe { olympus::new_impl(mem) };
    }
    if unsafe { canon::identify_impl(data.cast_const(), entry.cast_const()) } != 0 {
        return unsafe { canon::new_impl(mem, options) };
    }
    if unsafe { fuji::identify_impl(data.cast_const(), entry.cast_const()) } != 0 {
        return unsafe { fuji::new_impl(mem) };
    }
    if unsafe { pentax::identify_impl(data.cast_const(), entry.cast_const()) } != 0 {
        return unsafe { pentax::new_impl(mem) };
    }
    if unsafe { apple::identify_impl(data.cast_const(), entry.cast_const()) } != 0 {
        return unsafe { apple::new_impl(mem) };
    }

    core::ptr::null_mut()
}

pub(crate) unsafe fn interpret_maker_note_impl(
    data: *mut ExifData,
    buffer: *const u8,
    size: usize,
) {
    if data.is_null() || buffer.is_null() || size == 0 {
        return;
    }

    let entry = unsafe { find_entry_impl(data, EXIF_TAG_MAKER_NOTE) };
    if entry.is_null() {
        return;
    }

    let note = unsafe { construct_note_impl(data, entry) };
    unsafe { exif_data_set_mnote_data_impl(data, note) };
    if note.is_null() {
        return;
    }

    unsafe {
        crate::mnote::base::exif_mnote_data_log(note, exif_data_get_log_impl(data));
        crate::mnote::base::exif_mnote_data_set_byte_order(
            note,
            exif_data_get_byte_order_impl(data),
        );
        crate::mnote::base::exif_mnote_data_set_offset(note, exif_data_get_mnote_offset_impl(data));
        crate::mnote::base::exif_mnote_data_load(note, buffer, size as c_uint);
    }
}

pub(crate) unsafe fn prepare_maker_note_for_save_impl(
    data: *mut ExifData,
    entry: *mut ExifEntry,
    offset: c_uint,
) {
    if data.is_null()
        || entry.is_null()
        || unsafe { (*entry).tag } != EXIF_TAG_MAKER_NOTE
        || (unsafe { exif_data_get_options_impl(data) }
            & crate::ffi::types::EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE)
            != 0
    {
        return;
    }

    let note = unsafe { crate::object::data::exif_data_get_mnote_data_impl(data) };
    if note.is_null() {
        return;
    }

    unsafe {
        crate::runtime::mem::exif_mem_free_impl(exif_data_get_mem_impl(data), (*entry).data.cast());
        (*entry).data = core::ptr::null_mut();
        (*entry).size = 0;
        crate::mnote::base::exif_mnote_data_set_offset(note, offset);
        crate::mnote::base::exif_mnote_data_save(note, &mut (*entry).data, &mut (*entry).size);
        (*entry).components = (*entry).size.into();
        if crate::primitives::format::exif_format_get_size_impl((*entry).format) != 1 {
            (*entry).format = crate::ffi::types::EXIF_FORMAT_UNDEFINED;
        }
    }
}
