use core::ffi::c_int;

use crate::ffi::types::{ExifData, ExifEntry, ExifMem, ExifMnoteData};

unsafe extern "C" {
    fn safe_helper_exif_mnote_data_fuji_identify(
        data: *const ExifData,
        entry: *const ExifEntry,
    ) -> c_int;
    fn safe_helper_exif_mnote_data_fuji_new(mem: *mut ExifMem) -> *mut ExifMnoteData;
}

pub(crate) unsafe fn identify_impl(data: *const ExifData, entry: *const ExifEntry) -> c_int {
    unsafe { safe_helper_exif_mnote_data_fuji_identify(data, entry) }
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    unsafe { safe_helper_exif_mnote_data_fuji_new(mem) }
}
