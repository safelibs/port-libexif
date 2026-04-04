#![allow(dead_code)]

use crate::ffi::types::{
    ExifFormat, ExifTag, EXIF_FORMAT_ASCII, EXIF_FORMAT_BYTE, EXIF_FORMAT_RATIONAL,
    EXIF_FORMAT_SHORT, EXIF_FORMAT_UNDEFINED,
};

#[derive(Clone, Copy)]
pub(crate) struct ExifGpsIfdTagInfo {
    pub tag: u16,
    pub format: ExifFormat,
    pub components: u16,
    pub default_size: u16,
    pub default_value: Option<&'static [u8]>,
}

const GPS_TAGS: [ExifGpsIfdTagInfo; 31] = [
    ExifGpsIfdTagInfo {
        tag: 0x0000,
        format: EXIF_FORMAT_BYTE,
        components: 4,
        default_size: 4,
        default_value: Some(b"\x02\x02\x00\x00"),
    },
    ExifGpsIfdTagInfo { tag: 0x0001, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0002, format: EXIF_FORMAT_RATIONAL, components: 3, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0003, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0004, format: EXIF_FORMAT_RATIONAL, components: 3, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0005, format: EXIF_FORMAT_BYTE, components: 1, default_size: 1, default_value: Some(b"\x00") },
    ExifGpsIfdTagInfo { tag: 0x0006, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0007, format: EXIF_FORMAT_RATIONAL, components: 3, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0008, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0009, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x000a, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x000b, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x000c, format: EXIF_FORMAT_ASCII, components: 2, default_size: 2, default_value: Some(b"K\0") },
    ExifGpsIfdTagInfo { tag: 0x000d, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x000e, format: EXIF_FORMAT_ASCII, components: 2, default_size: 2, default_value: Some(b"T\0") },
    ExifGpsIfdTagInfo { tag: 0x000f, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0010, format: EXIF_FORMAT_ASCII, components: 2, default_size: 2, default_value: Some(b"T\0") },
    ExifGpsIfdTagInfo { tag: 0x0011, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0012, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0013, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0014, format: EXIF_FORMAT_RATIONAL, components: 3, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0015, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0016, format: EXIF_FORMAT_RATIONAL, components: 3, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0017, format: EXIF_FORMAT_ASCII, components: 2, default_size: 2, default_value: Some(b"T\0") },
    ExifGpsIfdTagInfo { tag: 0x0018, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x0019, format: EXIF_FORMAT_ASCII, components: 2, default_size: 2, default_value: Some(b"K\0") },
    ExifGpsIfdTagInfo { tag: 0x001a, format: EXIF_FORMAT_RATIONAL, components: 1, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x001b, format: EXIF_FORMAT_UNDEFINED, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x001c, format: EXIF_FORMAT_UNDEFINED, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x001d, format: EXIF_FORMAT_ASCII, components: 0, default_size: 0, default_value: None },
    ExifGpsIfdTagInfo { tag: 0x001e, format: EXIF_FORMAT_SHORT, components: 1, default_size: 0, default_value: None },
];

pub(crate) fn exif_get_gps_tag_info(tag: ExifTag) -> Option<&'static ExifGpsIfdTagInfo> {
    GPS_TAGS.iter().find(|entry| entry.tag as ExifTag == tag)
}
