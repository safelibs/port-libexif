use core::ffi::{c_char, c_int, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifEntry, ExifFormat, ExifLog, ExifLogCode, ExifMem, ExifMnoteData,
    ExifMnoteDataMethods, MnotePentaxEntry, MnotePentaxTag, EXIF_BYTE_ORDER_INTEL,
    EXIF_BYTE_ORDER_MOTOROLA, EXIF_FORMAT_SHORT, EXIF_FORMAT_UNDEFINED,
};
use crate::mnote::base::{
    check_overflow, generic_mnote_value, invalid_components_message, invalid_format_message,
    write_slice_to_buffer, write_str_to_buffer,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::primitives::utils::{
    exif_array_set_byte_order, exif_get_long, exif_get_short, exif_set_long, exif_set_short,
};
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_realloc_impl};

const PENTAX_V1: c_int = 1;
const PENTAX_V2: c_int = 2;
const PENTAX_V3: c_int = 3;
const CASIO_V2: c_int = 4;

const MNOTE_PENTAX2_TAG_BASE: c_int = 0x4000;
const MNOTE_CASIO2_TAG_BASE: c_int = MNOTE_PENTAX2_TAG_BASE;

const DOMAIN: &[u8] = b"ExifMnoteDataPentax\0";
const MSG_SHORT: &[u8] = b"Short MakerNote\0";
const MSG_TOO_MANY: &[u8] = b"Too much tags (%d) in Pentax MakerNote\0";
const MSG_OVERFLOW: &[u8] = b"Tag size overflow detected (%u * %lu)\0";
const MSG_PAST_END: &[u8] = b"Tag data past end of buffer (%u > %u)\0";
const MSG_BUFFER_OVERFLOW: &[u8] = b"Buffer overflow\0";
const MSG_NO_MEMORY: &[u8] = b"Could not allocate %u byte(s)\0";

const MNOTE_PENTAX_TAG_MODE: c_int = 0x0001;
const MNOTE_PENTAX_TAG_QUALITY: c_int = 0x0002;
const MNOTE_PENTAX_TAG_FOCUS: c_int = 0x0003;
const MNOTE_PENTAX_TAG_FLASH: c_int = 0x0004;
const MNOTE_PENTAX_TAG_WHITE_BALANCE: c_int = 0x0007;
const MNOTE_PENTAX_TAG_ZOOM: c_int = 0x000a;
const MNOTE_PENTAX_TAG_SHARPNESS: c_int = 0x000b;
const MNOTE_PENTAX_TAG_CONTRAST: c_int = 0x000c;
const MNOTE_PENTAX_TAG_SATURATION: c_int = 0x000d;
const MNOTE_PENTAX_TAG_ISO_SPEED: c_int = 0x0014;
const MNOTE_PENTAX_TAG_COLOR: c_int = 0x0017;
const MNOTE_PENTAX_TAG_PRINTIM: c_int = 0x0e00;
const MNOTE_PENTAX_TAG_TZ_CITY: c_int = 0x1000;
const MNOTE_PENTAX_TAG_TZ_DST: c_int = 0x1001;
const MNOTE_PENTAX2_TAG_MODE: c_int = 0x4001;
const MNOTE_PENTAX2_TAG_PREVIEW_SIZE: c_int = 0x4002;
const MNOTE_PENTAX2_TAG_PREVIEW_LENGTH: c_int = 0x4003;
const MNOTE_PENTAX2_TAG_PREVIEW_START: c_int = 0x4004;
const MNOTE_PENTAX2_TAG_MODEL_ID: c_int = 0x4005;
const MNOTE_PENTAX2_TAG_DATE: c_int = 0x4006;
const MNOTE_PENTAX2_TAG_TIME: c_int = 0x4007;
const MNOTE_PENTAX2_TAG_QUALITY: c_int = 0x4008;
const MNOTE_PENTAX2_TAG_IMAGE_SIZE: c_int = 0x4009;
const MNOTE_PENTAX2_TAG_PICTURE_MODE: c_int = 0x400b;
const MNOTE_PENTAX2_TAG_FLASH_MODE: c_int = 0x400c;
const MNOTE_PENTAX2_TAG_FOCUS_MODE: c_int = 0x400d;
const MNOTE_PENTAX2_TAG_AFPOINT_SELECTED: c_int = 0x400e;
const MNOTE_PENTAX2_TAG_AUTO_AFPOINT: c_int = 0x400f;
const MNOTE_PENTAX2_TAG_FOCUS_POSITION: c_int = 0x4010;
const MNOTE_PENTAX2_TAG_EXPOSURE_TIME: c_int = 0x4012;
const MNOTE_PENTAX2_TAG_FNUMBER: c_int = 0x4013;
const MNOTE_PENTAX2_TAG_ISO: c_int = 0x4014;
const MNOTE_PENTAX2_TAG_EXPOSURE_COMPENSATION: c_int = 0x4016;
const MNOTE_PENTAX2_TAG_METERING_MODE: c_int = 0x4017;
const MNOTE_PENTAX2_TAG_AUTO_BRACKETING: c_int = 0x4018;
const MNOTE_PENTAX2_TAG_WHITE_BALANCE: c_int = 0x4019;
const MNOTE_PENTAX2_TAG_WHITE_BALANCE_MODE: c_int = 0x401a;
const MNOTE_PENTAX2_TAG_BLUE_BALANCE: c_int = 0x401b;
const MNOTE_PENTAX2_TAG_RED_BALANCE: c_int = 0x401c;
const MNOTE_PENTAX2_TAG_FOCAL_LENGTH: c_int = 0x401d;
const MNOTE_PENTAX2_TAG_DIGITAL_ZOOM: c_int = 0x401e;
const MNOTE_PENTAX2_TAG_SATURATION: c_int = 0x401f;
const MNOTE_PENTAX2_TAG_CONTRAST: c_int = 0x4020;
const MNOTE_PENTAX2_TAG_SHARPNESS: c_int = 0x4021;
const MNOTE_PENTAX2_TAG_WORLDTIME_LOCATION: c_int = 0x4022;
const MNOTE_PENTAX2_TAG_HOMETOWN_CITY: c_int = 0x4023;
const MNOTE_PENTAX2_TAG_DESTINATION_CITY: c_int = 0x4024;
const MNOTE_PENTAX2_TAG_HOMETOWN_DST: c_int = 0x4025;
const MNOTE_PENTAX2_TAG_DESTINATION_DST: c_int = 0x4026;
const MNOTE_PENTAX2_TAG_FRAME_NUMBER: c_int = 0x4029;
const MNOTE_PENTAX2_TAG_IMAGE_PROCESSING: c_int = 0x4032;

const MNOTE_CASIO2_TAG_PREVIEW_START: c_int = 0x6000;
const MNOTE_CASIO2_TAG_WHITE_BALANCE_BIAS: c_int = 0x6011;
const MNOTE_CASIO2_TAG_WHITE_BALANCE: c_int = 0x6012;
const MNOTE_CASIO2_TAG_OBJECT_DISTANCE: c_int = 0x6022;
const MNOTE_CASIO2_TAG_FLASH_DISTANCE: c_int = 0x6034;
const MNOTE_CASIO2_TAG_RECORD_MODE: c_int = 0x7000;
const MNOTE_CASIO2_TAG_SELF_TIMER: c_int = 0x7001;
const MNOTE_CASIO2_TAG_QUALITY: c_int = 0x7002;
const MNOTE_CASIO2_TAG_FOCUS_MODE: c_int = 0x7003;
const MNOTE_CASIO2_TAG_TIME_ZONE: c_int = 0x7006;
const MNOTE_CASIO2_TAG_BESTSHOT_MODE: c_int = 0x7007;
const MNOTE_CASIO2_TAG_CCS_ISO_SENSITIVITY: c_int = 0x7014;
const MNOTE_CASIO2_TAG_COLOR_MODE: c_int = 0x7015;
const MNOTE_CASIO2_TAG_ENHANCEMENT: c_int = 0x7016;
const MNOTE_CASIO2_TAG_FINER: c_int = 0x7017;

#[derive(Clone, Copy)]
struct PentaxSingleValueInfo {
    tag: c_int,
    value: u16,
    text: &'static str,
}

#[derive(Clone, Copy)]
struct PentaxPairValueInfo {
    tag: c_int,
    first: u16,
    second: u16,
    text: &'static str,
}

const PENTAX_SINGLE_VALUES: &[PentaxSingleValueInfo] = &[
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_MODE,
        value: 0,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_MODE,
        value: 1,
        text: "Night scene",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_MODE,
        value: 2,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_MODE,
        value: 4,
        text: "Multi-exposure",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_QUALITY,
        value: 0,
        text: "Good",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_QUALITY,
        value: 1,
        text: "Better",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_QUALITY,
        value: 2,
        text: "Best",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FOCUS,
        value: 2,
        text: "Custom",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FOCUS,
        value: 3,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FLASH,
        value: 1,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FLASH,
        value: 2,
        text: "Flash on",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FLASH,
        value: 4,
        text: "Flash off",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_FLASH,
        value: 6,
        text: "Red-eye reduction",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 0,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 1,
        text: "Daylight",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 2,
        text: "Shade",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 3,
        text: "Tungsten",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 4,
        text: "Fluorescent",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_WHITE_BALANCE,
        value: 5,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SHARPNESS,
        value: 0,
        text: "Normal",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SHARPNESS,
        value: 1,
        text: "Soft",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SHARPNESS,
        value: 2,
        text: "Hard",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_CONTRAST,
        value: 0,
        text: "Normal",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_CONTRAST,
        value: 1,
        text: "Low",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_CONTRAST,
        value: 2,
        text: "High",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SATURATION,
        value: 0,
        text: "Normal",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SATURATION,
        value: 1,
        text: "Low",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_SATURATION,
        value: 2,
        text: "High",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_ISO_SPEED,
        value: 10,
        text: "100",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_ISO_SPEED,
        value: 16,
        text: "200",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_ISO_SPEED,
        value: 100,
        text: "100",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_ISO_SPEED,
        value: 200,
        text: "200",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_COLOR,
        value: 1,
        text: "Full",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_COLOR,
        value: 2,
        text: "Black & white",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX_TAG_COLOR,
        value: 3,
        text: "Sepia",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_MODE,
        value: 0,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_MODE,
        value: 1,
        text: "Night scene",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_MODE,
        value: 2,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_MODE,
        value: 4,
        text: "Multi-exposure",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_QUALITY,
        value: 0,
        text: "Good",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_QUALITY,
        value: 1,
        text: "Better",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_QUALITY,
        value: 2,
        text: "Best",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_QUALITY,
        value: 3,
        text: "TIFF",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_QUALITY,
        value: 4,
        text: "RAW",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 0,
        text: "640x480",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 1,
        text: "Full",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 2,
        text: "1024x768",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 3,
        text: "1280x960",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 4,
        text: "1600x1200",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 5,
        text: "2048x1536",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 8,
        text: "2560x1920 or 2304x1728",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 9,
        text: "3072x2304",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 10,
        text: "3264x2448",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 19,
        text: "320x240",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 20,
        text: "2288x1712",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 21,
        text: "2592x1944",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 22,
        text: "2304x1728 or 2592x1944",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 23,
        text: "3056x2296",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 25,
        text: "2816x2212 or 2816x2112",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 27,
        text: "3648x2736",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        value: 36,
        text: "3008x2008",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 0,
        text: "Program",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 2,
        text: "Program AE",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 3,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 5,
        text: "Portrait",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 6,
        text: "Landscape",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 8,
        text: "Sport",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 9,
        text: "Night scene",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 11,
        text: "Soft",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 12,
        text: "Surf & snow",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 13,
        text: "Sunset or candlelight",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 14,
        text: "Autumn",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 15,
        text: "Macro",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 17,
        text: "Fireworks",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 18,
        text: "Text",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 19,
        text: "Panorama",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 30,
        text: "Self portrait",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 31,
        text: "Illustrations",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 33,
        text: "Digital filter",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 37,
        text: "Museum",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 38,
        text: "Food",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 40,
        text: "Green mode",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 49,
        text: "Light pet",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 50,
        text: "Dark pet",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 51,
        text: "Medium pet",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 53,
        text: "Underwater",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 54,
        text: "Candlelight",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 55,
        text: "Natural skin tone",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 56,
        text: "Synchro sound record",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        value: 58,
        text: "Frame composite",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0000,
        text: "Auto, did not fire",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0001,
        text: "Off",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0003,
        text: "Auto, did not fire, red-eye reduction",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0100,
        text: "Auto, fired",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0102,
        text: "On",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0103,
        text: "Auto, fired, red-eye reduction",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0104,
        text: "On, red-eye reduction",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0105,
        text: "On, wireless",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0108,
        text: "On, soft",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x0109,
        text: "On, slow-sync",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x010a,
        text: "On, slow-sync, red-eye reduction",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FLASH_MODE,
        value: 0x010b,
        text: "On, trailing-curtain sync",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 0,
        text: "Normal",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 1,
        text: "Macro",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 2,
        text: "Infinity",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 3,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 5,
        text: "Pan focus",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 16,
        text: "AF-S",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_FOCUS_MODE,
        value: 17,
        text: "AF-C",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 1,
        text: "Upper-left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 2,
        text: "Top",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 3,
        text: "Upper-right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 4,
        text: "Left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 5,
        text: "Mid-left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 6,
        text: "Center",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 7,
        text: "Mid-right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 8,
        text: "Right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 9,
        text: "Lower-left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 10,
        text: "Bottom",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 11,
        text: "Lower-right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 0xfffe,
        text: "Fixed center",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AFPOINT_SELECTED,
        value: 0xffff,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 0,
        text: "Multiple",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 1,
        text: "Top-left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 2,
        text: "Top-center",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 3,
        text: "Top-right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 4,
        text: "Left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 5,
        text: "Center",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 6,
        text: "Right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 7,
        text: "Bottom-left",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 8,
        text: "Bottom-center",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 9,
        text: "Bottom-right",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_AUTO_AFPOINT,
        value: 0xffff,
        text: "None",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 0,
        text: "Auto",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 1,
        text: "Daylight",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 2,
        text: "Shade",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 3,
        text: "Fluorescent",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 4,
        text: "Tungsten",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 5,
        text: "Manual",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 6,
        text: "Daylight fluorescent",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 7,
        text: "Day white fluorescent",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 8,
        text: "White fluorescent",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 9,
        text: "Flash",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 10,
        text: "Cloudy",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 0xfffe,
        text: "Unknown",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_PENTAX2_TAG_WHITE_BALANCE,
        value: 0xffff,
        text: "User selected",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_CASIO2_TAG_BESTSHOT_MODE,
        value: 0,
        text: "Off",
    },
    PentaxSingleValueInfo {
        tag: MNOTE_CASIO2_TAG_BESTSHOT_MODE,
        value: 1,
        text: "On",
    },
];

const PENTAX_PAIR_VALUES: &[PentaxPairValueInfo] = &[
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 0,
        second: 0,
        text: "2304x1728",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 4,
        second: 0,
        text: "1600x1200",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 5,
        second: 0,
        text: "2048x1536",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 8,
        second: 0,
        text: "2560x1920",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 34,
        second: 0,
        text: "1536x1024",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 36,
        second: 0,
        text: "3008x2008 or 3040x2024",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 37,
        second: 0,
        text: "3008x2000",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 35,
        second: 1,
        text: "2400x1600",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 32,
        second: 2,
        text: "960x480",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 33,
        second: 2,
        text: "1152x768",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_IMAGE_SIZE,
        first: 34,
        second: 2,
        text: "1536x1024",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 0,
        second: 0,
        text: "Auto",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 5,
        second: 0,
        text: "Portrait",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 53,
        second: 0,
        text: "Underwater",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 255,
        second: 0,
        text: "Digital filter?",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 5,
        second: 1,
        text: "Portrait",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 9,
        second: 1,
        text: "Night scene",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 13,
        second: 1,
        text: "Candlelight",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 15,
        second: 1,
        text: "Macro",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 53,
        second: 1,
        text: "Underwater",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 0,
        second: 2,
        text: "Program AE",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 5,
        second: 2,
        text: "Portrait",
    },
    PentaxPairValueInfo {
        tag: MNOTE_PENTAX2_TAG_PICTURE_MODE,
        first: 6,
        second: 2,
        text: "Landscape",
    },
];

fn pentax_lookup_value(tag: c_int, value: u16) -> Option<&'static str> {
    PENTAX_SINGLE_VALUES
        .iter()
        .find(|entry| entry.tag == tag && entry.value == value)
        .map(|entry| entry.text)
}

fn pentax_lookup_pair(tag: c_int, first: u16, second: u16) -> Option<&'static str> {
    PENTAX_PAIR_VALUES
        .iter()
        .find(|entry| entry.tag == tag && entry.first == first && entry.second == second)
        .map(|entry| entry.text)
}

#[repr(C)]
struct ExifMnoteDataPentax {
    parent: ExifMnoteData,
    entries: *mut MnotePentaxEntry,
    count: c_uint,
    order: ExifByteOrder,
    offset: c_uint,
    version: c_int,
}

unsafe extern "C" {
    fn exif_log(
        log: *mut ExifLog,
        code: ExifLogCode,
        domain: *const c_char,
        format: *const c_char,
        ...
    );
}

#[inline]
unsafe fn pentax_note(note: *mut ExifMnoteData) -> *mut ExifMnoteDataPentax {
    note.cast()
}

fn pentax_tag_name_impl(tag: MnotePentaxTag) -> Option<&'static [u8]> {
    match tag {
        MNOTE_PENTAX_TAG_MODE | MNOTE_PENTAX2_TAG_MODE => Some(b"Mode\0"),
        MNOTE_PENTAX_TAG_QUALITY | MNOTE_PENTAX2_TAG_QUALITY => Some(b"Quality\0"),
        MNOTE_PENTAX_TAG_FOCUS => Some(b"Focus\0"),
        MNOTE_PENTAX_TAG_FLASH => Some(b"Flash\0"),
        MNOTE_PENTAX_TAG_WHITE_BALANCE | MNOTE_PENTAX2_TAG_WHITE_BALANCE => Some(b"WhiteBalance\0"),
        MNOTE_PENTAX_TAG_ZOOM => Some(b"Zoom\0"),
        MNOTE_PENTAX_TAG_SHARPNESS | MNOTE_PENTAX2_TAG_SHARPNESS => Some(b"Sharpness\0"),
        MNOTE_PENTAX_TAG_CONTRAST | MNOTE_PENTAX2_TAG_CONTRAST => Some(b"Contrast\0"),
        MNOTE_PENTAX_TAG_SATURATION | MNOTE_PENTAX2_TAG_SATURATION => Some(b"Saturation\0"),
        MNOTE_PENTAX_TAG_ISO_SPEED => Some(b"ISOSpeed\0"),
        MNOTE_PENTAX_TAG_COLOR => Some(b"Color\0"),
        MNOTE_PENTAX_TAG_PRINTIM => Some(b"PrintIM\0"),
        MNOTE_PENTAX_TAG_TZ_CITY | MNOTE_CASIO2_TAG_TIME_ZONE => Some(b"TimeZone\0"),
        MNOTE_PENTAX_TAG_TZ_DST => Some(b"DaylightSavings\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_SIZE => Some(b"PentaxPreviewSize\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_LENGTH => Some(b"PentaxPreviewLength\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_START => Some(b"PentaxPreviewStart\0"),
        MNOTE_PENTAX2_TAG_MODEL_ID => Some(b"ModelID\0"),
        MNOTE_PENTAX2_TAG_DATE => Some(b"Date\0"),
        MNOTE_PENTAX2_TAG_TIME => Some(b"Time\0"),
        MNOTE_PENTAX2_TAG_IMAGE_SIZE => Some(b"ImageSize\0"),
        MNOTE_PENTAX2_TAG_PICTURE_MODE => Some(b"PictureMode\0"),
        MNOTE_PENTAX2_TAG_FLASH_MODE => Some(b"FlashMode\0"),
        MNOTE_PENTAX2_TAG_FOCUS_MODE => Some(b"FocusMode\0"),
        MNOTE_PENTAX2_TAG_AFPOINT_SELECTED => Some(b"AFPointSelected\0"),
        MNOTE_PENTAX2_TAG_AUTO_AFPOINT => Some(b"AutoAFPoint\0"),
        MNOTE_PENTAX2_TAG_FOCUS_POSITION => Some(b"FocusPosition\0"),
        MNOTE_PENTAX2_TAG_EXPOSURE_TIME => Some(b"ExposureTime\0"),
        MNOTE_PENTAX2_TAG_FNUMBER => Some(b"FNumber\0"),
        MNOTE_PENTAX2_TAG_ISO => Some(b"ISO\0"),
        MNOTE_PENTAX2_TAG_EXPOSURE_COMPENSATION => Some(b"ExposureCompensation\0"),
        MNOTE_PENTAX2_TAG_METERING_MODE => Some(b"MeteringMode\0"),
        MNOTE_PENTAX2_TAG_AUTO_BRACKETING => Some(b"AutoBracketing\0"),
        MNOTE_PENTAX2_TAG_WHITE_BALANCE_MODE => Some(b"WhiteBalanceMode\0"),
        MNOTE_PENTAX2_TAG_BLUE_BALANCE => Some(b"BlueBalance\0"),
        MNOTE_PENTAX2_TAG_RED_BALANCE => Some(b"RedBalance\0"),
        MNOTE_PENTAX2_TAG_FOCAL_LENGTH => Some(b"FocalLength\0"),
        MNOTE_PENTAX2_TAG_DIGITAL_ZOOM => Some(b"DigitalZoom\0"),
        MNOTE_PENTAX2_TAG_WORLDTIME_LOCATION => Some(b"WorldTimeLocation\0"),
        MNOTE_PENTAX2_TAG_HOMETOWN_CITY => Some(b"HometownCity\0"),
        MNOTE_PENTAX2_TAG_DESTINATION_CITY => Some(b"DestinationCity\0"),
        MNOTE_PENTAX2_TAG_HOMETOWN_DST => Some(b"HometownDST,\0"),
        MNOTE_PENTAX2_TAG_DESTINATION_DST => Some(b"DestinationDST\0"),
        MNOTE_PENTAX2_TAG_FRAME_NUMBER => Some(b"FrameNumber\0"),
        MNOTE_PENTAX2_TAG_IMAGE_PROCESSING => Some(b"ImageProcessing\0"),
        MNOTE_CASIO2_TAG_PREVIEW_START => Some(b"CasioPreviewStart\0"),
        MNOTE_CASIO2_TAG_WHITE_BALANCE_BIAS => Some(b"WhiteBalanceBias\0"),
        MNOTE_CASIO2_TAG_WHITE_BALANCE => Some(b"WhiteBalance\0"),
        MNOTE_CASIO2_TAG_OBJECT_DISTANCE => Some(b"ObjectDistance\0"),
        MNOTE_CASIO2_TAG_FLASH_DISTANCE => Some(b"FlashDistance\0"),
        MNOTE_CASIO2_TAG_RECORD_MODE => Some(b"RecordMode\0"),
        MNOTE_CASIO2_TAG_SELF_TIMER => Some(b"SelfTimer\0"),
        MNOTE_CASIO2_TAG_QUALITY => Some(b"CasioQuality\0"),
        MNOTE_CASIO2_TAG_FOCUS_MODE => Some(b"CasioFocusMode\0"),
        MNOTE_CASIO2_TAG_BESTSHOT_MODE => Some(b"BestshotMode\0"),
        MNOTE_CASIO2_TAG_CCS_ISO_SENSITIVITY => Some(b"CCSISOSensitivity\0"),
        MNOTE_CASIO2_TAG_COLOR_MODE => Some(b"ColorMode\0"),
        MNOTE_CASIO2_TAG_ENHANCEMENT => Some(b"Enhancement\0"),
        MNOTE_CASIO2_TAG_FINER => Some(b"Finer\0"),
        _ => None,
    }
}

fn pentax_tag_title_impl(tag: MnotePentaxTag) -> Option<&'static [u8]> {
    match tag {
        MNOTE_PENTAX_TAG_MODE | MNOTE_PENTAX2_TAG_MODE => Some(b"Capture Mode\0"),
        MNOTE_PENTAX_TAG_QUALITY | MNOTE_PENTAX2_TAG_QUALITY | MNOTE_CASIO2_TAG_QUALITY => {
            Some(b"Quality Level\0")
        }
        MNOTE_PENTAX_TAG_FOCUS | MNOTE_PENTAX2_TAG_FOCUS_MODE | MNOTE_CASIO2_TAG_FOCUS_MODE => {
            Some(b"Focus Mode\0")
        }
        MNOTE_PENTAX_TAG_FLASH | MNOTE_PENTAX2_TAG_FLASH_MODE => Some(b"Flash Mode\0"),
        MNOTE_PENTAX_TAG_WHITE_BALANCE
        | MNOTE_PENTAX2_TAG_WHITE_BALANCE
        | MNOTE_CASIO2_TAG_WHITE_BALANCE => Some(b"White Balance\0"),
        MNOTE_PENTAX_TAG_ZOOM => Some(b"Zoom\0"),
        MNOTE_PENTAX_TAG_SHARPNESS | MNOTE_PENTAX2_TAG_SHARPNESS => Some(b"Sharpness\0"),
        MNOTE_PENTAX_TAG_CONTRAST | MNOTE_PENTAX2_TAG_CONTRAST => Some(b"Contrast\0"),
        MNOTE_PENTAX_TAG_SATURATION | MNOTE_PENTAX2_TAG_SATURATION => Some(b"Saturation\0"),
        MNOTE_PENTAX_TAG_ISO_SPEED => Some(b"ISO Speed\0"),
        MNOTE_PENTAX_TAG_COLOR => Some(b"Colors\0"),
        MNOTE_PENTAX_TAG_PRINTIM => Some(b"PrintIM Settings\0"),
        MNOTE_PENTAX_TAG_TZ_CITY | MNOTE_CASIO2_TAG_TIME_ZONE => Some(b"Time Zone\0"),
        MNOTE_PENTAX_TAG_TZ_DST => Some(b"Daylight Savings\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_SIZE => Some(b"Preview Size\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_LENGTH => Some(b"Preview Length\0"),
        MNOTE_PENTAX2_TAG_PREVIEW_START | MNOTE_CASIO2_TAG_PREVIEW_START => {
            Some(b"Preview Start\0")
        }
        MNOTE_PENTAX2_TAG_MODEL_ID => Some(b"Model Identification\0"),
        MNOTE_PENTAX2_TAG_DATE => Some(b"Date\0"),
        MNOTE_PENTAX2_TAG_TIME => Some(b"Time\0"),
        MNOTE_PENTAX2_TAG_IMAGE_SIZE => Some(b"Image Size\0"),
        MNOTE_PENTAX2_TAG_PICTURE_MODE => Some(b"Picture Mode\0"),
        MNOTE_PENTAX2_TAG_AFPOINT_SELECTED => Some(b"AF Point Selected\0"),
        MNOTE_PENTAX2_TAG_AUTO_AFPOINT => Some(b"Auto AF Point\0"),
        MNOTE_PENTAX2_TAG_FOCUS_POSITION => Some(b"Focus Position\0"),
        MNOTE_PENTAX2_TAG_EXPOSURE_TIME => Some(b"Exposure Time\0"),
        MNOTE_PENTAX2_TAG_FNUMBER => Some(b"F-Number\0"),
        MNOTE_PENTAX2_TAG_ISO | MNOTE_CASIO2_TAG_CCS_ISO_SENSITIVITY => Some(b"ISO Number\0"),
        MNOTE_PENTAX2_TAG_EXPOSURE_COMPENSATION => Some(b"Exposure Compensation\0"),
        MNOTE_PENTAX2_TAG_METERING_MODE => Some(b"Metering Mode\0"),
        MNOTE_PENTAX2_TAG_AUTO_BRACKETING => Some(b"Auto Bracketing\0"),
        MNOTE_PENTAX2_TAG_WHITE_BALANCE_MODE => Some(b"White Balance Mode\0"),
        MNOTE_PENTAX2_TAG_BLUE_BALANCE => Some(b"Blue Balance\0"),
        MNOTE_PENTAX2_TAG_RED_BALANCE => Some(b"Red Balance\0"),
        MNOTE_PENTAX2_TAG_FOCAL_LENGTH => Some(b"Focal Length\0"),
        MNOTE_PENTAX2_TAG_DIGITAL_ZOOM => Some(b"Digital Zoom\0"),
        MNOTE_PENTAX2_TAG_WORLDTIME_LOCATION => Some(b"World Time Location\0"),
        MNOTE_PENTAX2_TAG_HOMETOWN_CITY => Some(b"Hometown City\0"),
        MNOTE_PENTAX2_TAG_DESTINATION_CITY => Some(b"Destination City\0"),
        MNOTE_PENTAX2_TAG_HOMETOWN_DST => Some(b"Hometown DST\0"),
        MNOTE_PENTAX2_TAG_DESTINATION_DST => Some(b"Destination DST\0"),
        MNOTE_PENTAX2_TAG_FRAME_NUMBER => Some(b"Frame Number\0"),
        MNOTE_PENTAX2_TAG_IMAGE_PROCESSING => Some(b"Image Processing\0"),
        MNOTE_CASIO2_TAG_WHITE_BALANCE_BIAS => Some(b"White Balance Bias\0"),
        MNOTE_CASIO2_TAG_OBJECT_DISTANCE => Some(b"Object Distance\0"),
        MNOTE_CASIO2_TAG_FLASH_DISTANCE => Some(b"Flash Distance\0"),
        MNOTE_CASIO2_TAG_RECORD_MODE => Some(b"Record Mode\0"),
        MNOTE_CASIO2_TAG_SELF_TIMER => Some(b"Self Timer\0"),
        MNOTE_CASIO2_TAG_BESTSHOT_MODE => Some(b"Bestshot Mode\0"),
        MNOTE_CASIO2_TAG_COLOR_MODE => Some(b"Color Mode\0"),
        MNOTE_CASIO2_TAG_ENHANCEMENT => Some(b"Enhancement\0"),
        MNOTE_CASIO2_TAG_FINER => Some(b"Finer\0"),
        _ => None,
    }
}

unsafe fn log_simple(note: *mut ExifMnoteDataPentax, code: ExifLogCode, format: &[u8]) {
    unsafe {
        exif_log(
            (*note).parent.log,
            code,
            DOMAIN.as_ptr().cast(),
            format.as_ptr().cast(),
        )
    };
}

unsafe fn log_no_memory(note: *mut ExifMnoteDataPentax, size: usize) {
    unsafe {
        exif_log(
            (*note).parent.log,
            2,
            DOMAIN.as_ptr().cast(),
            MSG_NO_MEMORY.as_ptr().cast(),
            size as c_uint,
        );
    }
}

unsafe fn clear_impl(note: *mut ExifMnoteDataPentax) {
    if note.is_null() {
        return;
    }

    if !unsafe { (*note).entries }.is_null() {
        for index in 0..unsafe { (*note).count as usize } {
            let entry = unsafe { (*note).entries.add(index) };
            if !unsafe { (*entry).data }.is_null() {
                unsafe {
                    exif_mem_free_impl((*note).parent.mem, (*entry).data.cast());
                    (*entry).data = ptr::null_mut();
                }
            }
        }
        unsafe {
            exif_mem_free_impl((*note).parent.mem, (*note).entries.cast());
            (*note).entries = ptr::null_mut();
            (*note).count = 0;
        }
    }
}

unsafe extern "C" fn exif_mnote_data_pentax_free(note: *mut ExifMnoteData) {
    unsafe { clear_impl(pentax_note(note)) };
}

unsafe extern "C" fn exif_mnote_data_pentax_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    buffer_size: *mut c_uint,
) {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || buffer.is_null() || buffer_size.is_null() {
        return;
    }

    let mut base = 0usize;
    let mut count_offset = 6usize;
    let data_offset = unsafe { (*note).offset as usize };
    let mut out_size = count_offset + 2 + unsafe { (*note).count as usize } * 12 + 4;
    if unsafe { (*note).version } == PENTAX_V1 {
        out_size -= 6;
        count_offset -= 6;
    }
    let mut out =
        unsafe { exif_mem_alloc_impl((*note).parent.mem, out_size as u32) }.cast::<c_uchar>();
    if out.is_null() {
        unsafe { log_no_memory(note, out_size) };
        return;
    }
    unsafe {
        *buffer = out;
        *buffer_size = out_size as c_uint;
    }

    match unsafe { (*note).version } {
        PENTAX_V3 => unsafe {
            base = MNOTE_PENTAX2_TAG_BASE as usize;
            ptr::copy_nonoverlapping(b"AOC\0".as_ptr(), out, 4);
            exif_set_short(
                out.add(4),
                (*note).order,
                if (*note).order == EXIF_BYTE_ORDER_INTEL {
                    (b'I' as u16) << 8 | b'I' as u16
                } else {
                    (b'M' as u16) << 8 | b'M' as u16
                },
            );
        },
        PENTAX_V2 => unsafe {
            base = MNOTE_PENTAX2_TAG_BASE as usize;
            ptr::copy_nonoverlapping(b"AOC\0\0\0".as_ptr(), out, 6);
        },
        CASIO_V2 => unsafe {
            base = MNOTE_CASIO2_TAG_BASE as usize;
            ptr::copy_nonoverlapping(b"QVC\0\0\0".as_ptr(), out, 6);
        },
        _ => {}
    }

    unsafe { exif_set_short(out.add(count_offset), (*note).order, (*note).count as u16) };
    let entries_offset = count_offset + 2;

    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { &*(*note).entries.add(index) };
        let mut item_offset = entries_offset + index * 12;
        unsafe {
            exif_set_short(
                out.add(item_offset),
                (*note).order,
                (entry.tag as usize - base) as u16,
            );
            exif_set_short(out.add(item_offset + 2), (*note).order, entry.format as u16);
            exif_set_long(
                out.add(item_offset + 4),
                (*note).order,
                entry.components as u32,
            );
        }
        item_offset += 8;

        let Some(data_size) = (exif_format_get_size_impl(entry.format) as usize)
            .checked_mul(entry.components as usize)
        else {
            continue;
        };
        if data_size > 65_536 {
            continue;
        }

        let data_at = if data_size > 4 {
            let target_size = out_size + data_size;
            let new_out = unsafe {
                exif_mem_realloc_impl((*note).parent.mem, out.cast(), target_size as u32)
            }
            .cast::<c_uchar>();
            if new_out.is_null() {
                unsafe { log_no_memory(note, target_size) };
                return;
            }
            out = new_out;
            unsafe {
                *buffer = out;
                *buffer_size = target_size as c_uint;
            }
            out_size = target_size;
            let value_offset = out_size - data_size;
            unsafe {
                exif_set_long(
                    out.add(item_offset),
                    (*note).order,
                    (data_offset + value_offset) as u32,
                )
            };
            value_offset
        } else {
            item_offset
        };

        let out_ref = unsafe { *buffer };
        if entry.data.is_null() {
            unsafe { ptr::write_bytes(out_ref.add(data_at), 0, data_size) };
        } else {
            unsafe { ptr::copy_nonoverlapping(entry.data, out_ref.add(data_at), data_size) };
        }
    }

    let next_ifd_offset = entries_offset + unsafe { (*note).count as usize } * 12;
    if out_size < next_ifd_offset + 4 {
        unsafe { log_simple(note, 3, MSG_BUFFER_OVERFLOW) };
        return;
    }
    unsafe { exif_set_long((*buffer).add(next_ifd_offset), (*note).order, 0) };
}

unsafe extern "C" fn exif_mnote_data_pentax_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    buffer_size: c_uint,
) {
    let note = unsafe { pentax_note(note) };
    let buffer_size = buffer_size as usize;
    if note.is_null() || buffer.is_null() || buffer_size == 0 {
        if !note.is_null() {
            unsafe { log_simple(note, 3, MSG_SHORT) };
        }
        return;
    }

    let mut data_offset = 6 + unsafe { (*note).offset as usize };
    if check_overflow(data_offset, buffer_size, 8) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }

    let buf = unsafe { std::slice::from_raw_parts(buffer.add(data_offset), 6) };
    let mut base = 0usize;
    if &buf[..4] == b"AOC\0" {
        if buf[4] == b'I' && buf[5] == b'I' {
            unsafe {
                (*note).version = PENTAX_V3;
                (*note).order = EXIF_BYTE_ORDER_INTEL;
            }
        } else if buf[4] == b'M' && buf[5] == b'M' {
            unsafe {
                (*note).version = PENTAX_V3;
                (*note).order = EXIF_BYTE_ORDER_MOTOROLA;
            }
        } else {
            unsafe { (*note).version = PENTAX_V2 };
        }
        data_offset += 6;
        base = MNOTE_PENTAX2_TAG_BASE as usize;
    } else if &buf[..4] == b"QVC\0" {
        unsafe { (*note).version = CASIO_V2 };
        data_offset += 6;
        base = MNOTE_CASIO2_TAG_BASE as usize;
    } else {
        unsafe { (*note).version = PENTAX_V1 };
    }

    let count = unsafe { exif_get_short(buffer.add(data_offset), (*note).order) as usize };
    data_offset += 2;
    if count > 200 {
        unsafe {
            exif_log(
                (*note).parent.log,
                3,
                DOMAIN.as_ptr().cast(),
                MSG_TOO_MANY.as_ptr().cast(),
                count as c_int,
            );
        }
        return;
    }

    unsafe { clear_impl(note) };
    let entry_bytes = size_of::<MnotePentaxEntry>().saturating_mul(count);
    let entries = unsafe { exif_mem_alloc_impl((*note).parent.mem, entry_bytes as u32) }
        .cast::<MnotePentaxEntry>();
    if entries.is_null() {
        unsafe { log_no_memory(note, entry_bytes) };
        return;
    }
    unsafe { (*note).entries = entries };

    let mut stored = 0usize;
    let mut offset = data_offset;
    for _ in 0..count {
        if check_overflow(offset, buffer_size, 12) {
            unsafe { log_simple(note, 3, MSG_SHORT) };
            break;
        }

        let entry = unsafe { (*note).entries.add(stored) };
        unsafe {
            (*entry).tag = exif_get_short(buffer.add(offset), (*note).order) as MnotePentaxTag
                + base as MnotePentaxTag;
            (*entry).format = exif_get_short(buffer.add(offset + 2), (*note).order) as ExifFormat;
            (*entry).components = exif_get_long(buffer.add(offset + 4), (*note).order).into();
            (*entry).order = (*note).order;
        }

        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if format_size != 0
            && (buffer_size as u128 / format_size as u128) < unsafe { (*entry).components as u128 }
        {
            unsafe {
                exif_log(
                    (*note).parent.log,
                    3,
                    DOMAIN.as_ptr().cast(),
                    MSG_OVERFLOW.as_ptr().cast(),
                    format_size as c_uint,
                    (*entry).components,
                );
            }
            break;
        }

        let Some(data_size) = format_size.checked_mul(unsafe { (*entry).components as usize })
        else {
            break;
        };
        unsafe { (*entry).size = data_size as c_uint };
        if data_size != 0 {
            let mut data_at = offset + 8;
            if data_size > 4 {
                data_at = unsafe { exif_get_long(buffer.add(data_at), (*note).order) as usize } + 6;
            }
            if check_overflow(data_at, buffer_size, data_size) {
                unsafe {
                    exif_log(
                        (*note).parent.log,
                        1,
                        DOMAIN.as_ptr().cast(),
                        MSG_PAST_END.as_ptr().cast(),
                        (data_at + data_size) as c_uint,
                        buffer_size as c_uint,
                    );
                }
                offset += 12;
                continue;
            }
            let data = unsafe { exif_mem_alloc_impl((*note).parent.mem, data_size as u32) }
                .cast::<c_uchar>();
            if data.is_null() {
                unsafe { log_no_memory(note, data_size) };
                offset += 12;
                continue;
            }
            unsafe {
                ptr::copy_nonoverlapping(buffer.add(data_at), data, data_size);
                (*entry).data = data;
            }
        }
        stored += 1;
        offset += 12;
    }
    unsafe { (*note).count = stored as c_uint };
}

unsafe extern "C" fn exif_mnote_data_pentax_count(note: *mut ExifMnoteData) -> c_uint {
    let note = unsafe { pentax_note(note) };
    if note.is_null() {
        0
    } else {
        unsafe { (*note).count }
    }
}

unsafe extern "C" fn exif_mnote_data_pentax_get_id(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> c_uint {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        0
    } else {
        unsafe { (*(*note).entries.add(index as usize)).tag as c_uint }
    }
}

unsafe extern "C" fn exif_mnote_data_pentax_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(index as usize)).tag };
    pentax_tag_name_impl(tag).map_or(ptr::null(), |name| name.as_ptr().cast())
}

unsafe extern "C" fn exif_mnote_data_pentax_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(index as usize)).tag };
    pentax_tag_title_impl(tag).map_or(ptr::null(), |title| title.as_ptr().cast())
}

unsafe extern "C" fn exif_mnote_data_pentax_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        ptr::null()
    } else {
        b"\0".as_ptr().cast()
    }
}

unsafe extern "C" fn exif_mnote_data_pentax_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    let note = unsafe { pentax_note(note) };
    if !note.is_null() {
        unsafe { (*note).offset = offset };
    }
}

unsafe extern "C" fn exif_mnote_data_pentax_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    let note = unsafe { pentax_note(note) };
    if note.is_null() {
        return;
    }
    let original = unsafe { (*note).order };
    unsafe { (*note).order = order };
    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { (*note).entries.add(index) };
        let format_size = exif_format_get_size_impl(unsafe { (*entry).format }) as usize;
        if unsafe { (*entry).components } != 0
            && unsafe { (*entry).size as usize / (*entry).components as usize } < format_size
        {
            continue;
        }
        unsafe {
            (*entry).order = order;
            exif_array_set_byte_order(
                (*entry).format,
                (*entry).data,
                (*entry).components as c_uint,
                original,
                order,
            );
        }
    }
}

pub(crate) unsafe fn identify_impl(_data: *const ExifData, entry: *const ExifEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    let size = unsafe { (*entry).size as usize };
    if size >= 8 {
        let prefix = unsafe { std::slice::from_raw_parts((*entry).data, 6) };
        if &prefix[..4] == b"AOC\0" {
            if (prefix[4] == b'I' && prefix[5] == b'I') || (prefix[4] == b'M' && prefix[5] == b'M')
            {
                return PENTAX_V3;
            }
            return PENTAX_V2;
        }
        if &prefix[..4] == b"QVC\0" {
            return CASIO_V2;
        }
    }
    if size >= 2 {
        let bytes = unsafe { std::slice::from_raw_parts((*entry).data, 2) };
        if bytes == [0x00, 0x1b] {
            return PENTAX_V1;
        }
    }
    0
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    if mem.is_null() {
        return ptr::null_mut();
    }
    let note = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifMnoteDataPentax>() as u32) }
        .cast::<ExifMnoteDataPentax>();
    if note.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        crate::mnote::base::exif_mnote_data_construct(ptr::addr_of_mut!((*note).parent), mem)
    };
    unsafe {
        (*note).parent.methods = ExifMnoteDataMethods {
            free: Some(exif_mnote_data_pentax_free),
            save: Some(exif_mnote_data_pentax_save),
            load: Some(exif_mnote_data_pentax_load),
            set_offset: Some(exif_mnote_data_pentax_set_offset),
            set_byte_order: Some(exif_mnote_data_pentax_set_byte_order),
            count: Some(exif_mnote_data_pentax_count),
            get_id: Some(exif_mnote_data_pentax_get_id),
            get_name: Some(exif_mnote_data_pentax_get_name),
            get_title: Some(exif_mnote_data_pentax_get_title),
            get_description: Some(exif_mnote_data_pentax_get_description),
            get_value: Some(exif_mnote_data_pentax_get_value),
        };
    }
    unsafe { ptr::addr_of_mut!((*note).parent) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_pentax_new(mem: *mut ExifMem) -> *mut ExifMnoteData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { new_impl(mem) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_entry_get_value(
    entry: *mut MnotePentaxEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        if entry.is_null() || value.is_null() {
            return ptr::null_mut();
        }
        let entry_ref = &*entry;
        match entry_ref.tag {
            MNOTE_PENTAX_TAG_MODE
            | MNOTE_PENTAX_TAG_QUALITY
            | MNOTE_PENTAX_TAG_FOCUS
            | MNOTE_PENTAX_TAG_FLASH
            | MNOTE_PENTAX_TAG_WHITE_BALANCE
            | MNOTE_PENTAX_TAG_SHARPNESS
            | MNOTE_PENTAX_TAG_CONTRAST
            | MNOTE_PENTAX_TAG_SATURATION
            | MNOTE_PENTAX_TAG_ISO_SPEED
            | MNOTE_PENTAX_TAG_COLOR
            | MNOTE_PENTAX2_TAG_MODE
            | MNOTE_PENTAX2_TAG_QUALITY
            | MNOTE_PENTAX2_TAG_FLASH_MODE
            | MNOTE_PENTAX2_TAG_FOCUS_MODE
            | MNOTE_PENTAX2_TAG_AFPOINT_SELECTED
            | MNOTE_PENTAX2_TAG_AUTO_AFPOINT
            | MNOTE_PENTAX2_TAG_WHITE_BALANCE
            | MNOTE_PENTAX2_TAG_PICTURE_MODE
            | MNOTE_PENTAX2_TAG_IMAGE_SIZE
            | MNOTE_CASIO2_TAG_BESTSHOT_MODE => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                let components = entry_ref.components as u64;
                if components != 1 && components != 2 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(components, &[1, 2]),
                    );
                }

                let first = exif_get_short(entry_ref.data, entry_ref.order);
                if components == 1 {
                    return match pentax_lookup_value(entry_ref.tag, first) {
                        Some(text) => write_str_to_buffer(value, maxlen, text),
                        None => write_str_to_buffer(
                            value,
                            maxlen,
                            &format!("Internal error (unknown value {first})"),
                        ),
                    };
                }

                let second = exif_get_short(entry_ref.data.add(2), entry_ref.order);
                match pentax_lookup_pair(entry_ref.tag, first, second) {
                    Some(text) => write_str_to_buffer(value, maxlen, text),
                    None => write_str_to_buffer(
                        value,
                        maxlen,
                        &format!("Internal error (unknown value {first} {second})"),
                    ),
                }
            }
            MNOTE_PENTAX_TAG_TZ_CITY | MNOTE_PENTAX_TAG_TZ_DST => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED
                    || entry_ref.components != 4
                    || entry_ref.data.is_null()
                {
                    return ptr::null_mut();
                }
                let bytes = std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize);
                let end = bytes
                    .iter()
                    .position(|byte| *byte == 0)
                    .unwrap_or(bytes.len());
                write_slice_to_buffer(value, maxlen, &bytes[..end])
            }
            MNOTE_PENTAX2_TAG_DATE => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED
                    || entry_ref.components != 4
                    || entry_ref.data.is_null()
                {
                    return ptr::null_mut();
                }
                let year = exif_get_short(entry_ref.data, EXIF_BYTE_ORDER_MOTOROLA);
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{year}:{:02}:{:02}",
                        unsafe { *entry_ref.data.add(2) },
                        unsafe { *entry_ref.data.add(3) }
                    ),
                )
            }
            MNOTE_PENTAX2_TAG_TIME => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED
                    || !(entry_ref.components == 3 || entry_ref.components == 4)
                    || entry_ref.data.is_null()
                {
                    return ptr::null_mut();
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{:02}:{:02}:{:02}",
                        unsafe { *entry_ref.data },
                        unsafe { *entry_ref.data.add(1) },
                        unsafe { *entry_ref.data.add(2) }
                    ),
                )
            }
            _ => generic_mnote_value(
                entry_ref.format,
                entry_ref.components as u64,
                entry_ref.data,
                entry_ref.size as usize,
                entry_ref.order,
                value,
                maxlen,
            ),
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_description(tag: MnotePentaxTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || match tag {
        0x2022 => b"Distance of photographed object in millimeters.\0"
            .as_ptr()
            .cast(),
        0x4025 => b"Home Daylight Savings Time\0".as_ptr().cast(),
        0x4026 => b"Destination Daylight Savings Time\0".as_ptr().cast(),
        _ if pentax_tag_name_impl(tag).is_some() => b"\0".as_ptr().cast(),
        _ => ptr::null(),
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_name(tag: MnotePentaxTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        pentax_tag_name_impl(tag).map_or(ptr::null(), |name| name.as_ptr().cast())
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_pentax_tag_get_title(tag: MnotePentaxTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || {
        pentax_tag_title_impl(tag).map_or(ptr::null(), |title| title.as_ptr().cast())
    })
}

unsafe extern "C" fn exif_mnote_data_pentax_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    let note = unsafe { pentax_note(note) };
    if note.is_null() || unsafe { (*note).count <= index } {
        return ptr::null_mut();
    }
    unsafe { mnote_pentax_entry_get_value((*note).entries.add(index as usize), value, maxlen) }
}
