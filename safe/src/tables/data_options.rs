use core::ffi::c_char;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifDataOption, EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE,
    EXIF_DATA_OPTION_FOLLOW_SPECIFICATION, EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS,
};
use crate::i18n::{gettext, message, Message};

#[derive(Clone, Copy)]
struct DataOptionEntry {
    option: ExifDataOption,
    name: Message,
    description: Message,
}

const DATA_OPTIONS: [DataOptionEntry; 3] = [
    DataOptionEntry {
        option: EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS,
        name: message(b"Ignore unknown tags\0"),
        description: message(b"Ignore unknown tags when loading EXIF data.\0"),
    },
    DataOptionEntry {
        option: EXIF_DATA_OPTION_FOLLOW_SPECIFICATION,
        name: message(b"Follow specification\0"),
        description: message(
            b"Add, correct and remove entries to get EXIF data that follows the specification.\0",
        ),
    },
    DataOptionEntry {
        option: EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE,
        name: message(b"Do not change maker note\0"),
        description: message(
            b"When loading and resaving Exif data, save the maker note unmodified. Be aware that the maker note can get corrupted.\0",
        ),
    },
];

pub(crate) fn exif_data_option_get_name_impl(option: ExifDataOption) -> *const c_char {
    DATA_OPTIONS
        .iter()
        .find(|entry| entry.option == option)
        .map_or(ptr::null(), |entry| gettext(entry.name))
}

pub(crate) fn exif_data_option_get_description_impl(option: ExifDataOption) -> *const c_char {
    DATA_OPTIONS
        .iter()
        .find(|entry| entry.option == option)
        .map_or(ptr::null(), |entry| gettext(entry.description))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_option_get_name(option: ExifDataOption) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_data_option_get_name_impl(option))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_option_get_description(
    option: ExifDataOption,
) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || exif_data_option_get_description_impl(option))
}
