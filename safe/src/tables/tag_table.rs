use core::ffi::c_char;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::*;
use crate::i18n::{empty_message, gettext, Message};

const DATA_TYPE_COUNT: usize = EXIF_DATA_TYPE_COUNT as usize;
const IFD_COUNT: usize = EXIF_IFD_COUNT as usize;

#[derive(Clone, Copy)]
pub(crate) struct TagEntry {
    pub tag: ExifTag,
    pub name: Option<Message>,
    pub title: Option<Message>,
    pub description: Option<Message>,
    pub support_levels: [[ExifSupportLevel; DATA_TYPE_COUNT]; IFD_COUNT],
}

include!(concat!(env!("OUT_DIR"), "/tag_table_data.rs"));

const SEARCH_IFDS: [ExifIfd; IFD_COUNT] = [
    EXIF_IFD_EXIF,
    EXIF_IFD_0,
    EXIF_IFD_1,
    EXIF_IFD_INTEROPERABILITY,
    EXIF_IFD_GPS,
];

fn first_index(tag: ExifTag) -> Option<usize> {
    let search = &TAG_TABLE[..TAG_TABLE.len().saturating_sub(1)];
    let mut index = search.binary_search_by_key(&tag, |entry| entry.tag).ok()?;
    while index > 0 && search[index - 1].tag == tag {
        index -= 1;
    }
    Some(index)
}

fn is_recorded(entry: &TagEntry, ifd: usize) -> bool {
    entry.support_levels[ifd]
        .iter()
        .any(|&level| level != EXIF_SUPPORT_LEVEL_NOT_RECORDED)
}

fn find_entry_in_ifd(tag: ExifTag, ifd: ExifIfd) -> Option<&'static TagEntry> {
    if ifd < 0 || ifd >= EXIF_IFD_COUNT {
        return None;
    }

    let mut index = first_index(tag)?;
    while let Some(entry) = TAG_TABLE.get(index) {
        if entry.name.is_none() {
            return None;
        }
        if entry.tag != tag {
            return None;
        }
        if is_recorded(entry, ifd as usize) {
            return Some(entry);
        }
        index += 1;
    }
    None
}

fn option_ptr(value: Option<Message>) -> *const c_char {
    value.map_or(ptr::null(), gettext)
}

fn name_ptr_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    find_entry_in_ifd(tag, ifd).map_or(ptr::null(), |entry| option_ptr(entry.name))
}

fn title_ptr_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    find_entry_in_ifd(tag, ifd).map_or(ptr::null(), |entry| option_ptr(entry.title))
}

fn description_ptr(entry: &TagEntry) -> *const c_char {
    match entry.description {
        Some(description) if description.is_empty() => gettext(empty_message()),
        Some(description) => gettext(description),
        None => ptr::null(),
    }
}

fn description_ptr_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    find_entry_in_ifd(tag, ifd).map_or(ptr::null(), description_ptr)
}

fn get_stuff(tag: ExifTag, getter: impl Fn(ExifTag, ExifIfd) -> *const c_char) -> *const c_char {
    for ifd in SEARCH_IFDS {
        let result = getter(tag, ifd);
        if !result.is_null() {
            return result;
        }
    }
    ptr::null()
}

fn support_level_in_ifd(tag: ExifTag, ifd: ExifIfd, data_type: ExifDataType) -> ExifSupportLevel {
    let mut index = match first_index(tag) {
        Some(index) => index,
        None => return EXIF_SUPPORT_LEVEL_NOT_RECORDED,
    };

    while let Some(entry) = TAG_TABLE.get(index) {
        if entry.name.is_none() || entry.tag != tag {
            break;
        }
        let support = entry.support_levels[ifd as usize][data_type as usize];
        if support != EXIF_SUPPORT_LEVEL_NOT_RECORDED {
            return support;
        }
        index += 1;
    }

    EXIF_SUPPORT_LEVEL_NOT_RECORDED
}

fn support_level_any_type(tag: ExifTag, ifd: ExifIfd) -> ExifSupportLevel {
    let mut index = match first_index(tag) {
        Some(index) => index,
        None => return EXIF_SUPPORT_LEVEL_UNKNOWN,
    };

    while let Some(entry) = TAG_TABLE.get(index) {
        if entry.name.is_none() || entry.tag != tag {
            break;
        }
        let support = entry.support_levels[ifd as usize][0];
        if support != EXIF_SUPPORT_LEVEL_NOT_RECORDED
            && entry.support_levels[ifd as usize]
                .iter()
                .all(|&level| level == support)
        {
            return support;
        }
        index += 1;
    }

    EXIF_SUPPORT_LEVEL_UNKNOWN
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_table_count() -> u32 {
    panic_boundary::call_or(0, || TAG_TABLE.len() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_table_get_tag(index: u32) -> ExifTag {
    panic_boundary::call_or(0, || {
        TAG_TABLE.get(index as usize).map_or(0, |entry| entry.tag)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_table_get_name(index: u32) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        TAG_TABLE
            .get(index as usize)
            .map_or(ptr::null(), |entry| option_ptr(entry.name))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_name_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || name_ptr_in_ifd(tag, ifd))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_title_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || title_ptr_in_ifd(tag, ifd))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_description_in_ifd(tag: ExifTag, ifd: ExifIfd) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || description_ptr_in_ifd(tag, ifd))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_name(tag: ExifTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || get_stuff(tag, name_ptr_in_ifd))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_title(tag: ExifTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || get_stuff(tag, title_ptr_in_ifd))
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_description(tag: ExifTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || get_stuff(tag, description_ptr_in_ifd))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_tag_from_name(name: *const c_char) -> ExifTag {
    panic_boundary::call_or(0, || {
        if name.is_null() {
            return 0;
        }

        let query = unsafe { core::ffi::CStr::from_ptr(name) }.to_bytes();
        for entry in TAG_TABLE.iter() {
            let Some(message) = entry.name else {
                break;
            };
            let bytes = message.bytes();
            if query == &bytes[..bytes.len() - 1] {
                return entry.tag;
            }
        }

        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn exif_tag_get_support_level_in_ifd(
    tag: ExifTag,
    ifd: ExifIfd,
    data_type: ExifDataType,
) -> ExifSupportLevel {
    panic_boundary::call_or(EXIF_SUPPORT_LEVEL_UNKNOWN, || {
        if ifd < 0 || ifd >= EXIF_IFD_COUNT {
            return EXIF_SUPPORT_LEVEL_UNKNOWN;
        }

        if data_type < 0 || data_type >= EXIF_DATA_TYPE_COUNT {
            support_level_any_type(tag, ifd)
        } else {
            support_level_in_ifd(tag, ifd, data_type)
        }
    })
}
