use core::ffi::{c_char, c_int, c_uchar, c_uint};
use core::mem::size_of;
use core::ptr;
use std::fmt::Write;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifEntry, ExifFormat, ExifLog, ExifLogCode, ExifMem, ExifMnoteData,
    ExifMnoteDataMethods, MnoteOlympusEntry, MnoteOlympusTag, EXIF_BYTE_ORDER_INTEL,
    EXIF_BYTE_ORDER_MOTOROLA, EXIF_FORMAT_ASCII, EXIF_FORMAT_BYTE, EXIF_FORMAT_LONG,
    EXIF_FORMAT_RATIONAL, EXIF_FORMAT_SHORT, EXIF_FORMAT_SLONG, EXIF_FORMAT_SRATIONAL,
    EXIF_FORMAT_SSHORT, EXIF_FORMAT_UNDEFINED,
};
use crate::i18n::{empty_message, message};
use crate::mnote::base::{
    check_overflow, invalid_components_message, invalid_format_message, tag_description_from_table,
    tag_name_from_table, tag_title_from_table, write_slice_to_buffer, write_str_to_buffer,
    zero_buffer, TagInfo,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::primitives::utils::{
    exif_array_set_byte_order, exif_get_long, exif_get_rational, exif_get_short, exif_get_slong,
    exif_get_srational, exif_get_sshort, exif_set_long, exif_set_short,
};
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl, exif_mem_realloc_impl};

const UNRECOGNIZED: c_int = 0;
const NIKON_V1: c_int = 1;
const NIKON_V2: c_int = 2;
const OLYMPUS_V1: c_int = 3;
const OLYMPUS_V2: c_int = 4;
const SANYO_V1: c_int = 5;
const EPSON_V1: c_int = 6;
const NIKON_V0: c_int = 7;

const MNOTE_NIKON1_TAG_BASE: c_int = 0x8000;

const MNOTE_NIKON_TAG_FIRMWARE: c_int = 0x0001;
const MNOTE_NIKON_TAG_ISO: c_int = 0x0002;
const MNOTE_NIKON_TAG_COLORMODE1: c_int = 0x0003;
const MNOTE_NIKON_TAG_QUALITY: c_int = 0x0004;
const MNOTE_NIKON_TAG_WHITEBALANCE: c_int = 0x0005;
const MNOTE_NIKON_TAG_SHARPENING: c_int = 0x0006;
const MNOTE_NIKON_TAG_FOCUSMODE: c_int = 0x0007;
const MNOTE_NIKON_TAG_FLASHSETTING: c_int = 0x0008;
const MNOTE_NIKON_TAG_FLASHMODE: c_int = 0x0009;
const MNOTE_NIKON_TAG_WHITEBALANCEFINE: c_int = 0x000b;
const MNOTE_NIKON_TAG_ISOSELECTION: c_int = 0x000f;
const MNOTE_NIKON_TAG_PREVIEWIMAGE: c_int = 0x0011;
const MNOTE_NIKON_TAG_FACEDETECT: c_int = 0x0021;
const MNOTE_NIKON_TAG_IMAGEADJUSTMENT: c_int = 0x0080;
const MNOTE_NIKON_TAG_MANUALFOCUSDISTANCE: c_int = 0x0085;
const MNOTE_NIKON_TAG_DIGITALZOOM: c_int = 0x0086;
const MNOTE_NIKON_TAG_AFFOCUSPOSITION: c_int = 0x0088;
const MNOTE_NIKON_TAG_SHOTINFO: c_int = 0x0091;
const MNOTE_NIKON_TAG_SATURATION: c_int = 0x0094;
const MNOTE_NIKON_TAG_NOISEREDUCTION: c_int = 0x0095;
const MNOTE_NIKON_TAG_RETOUCHHISTORY: c_int = 0x009e;
const MNOTE_NIKON_TAG_IMAGEBOUNDARY: c_int = 0x0016;

const MNOTE_OLYMPUS_TAG_THUMBNAILIMAGE: c_int = 0x0100;
const MNOTE_OLYMPUS_TAG_MODE: c_int = 0x0200;
const MNOTE_OLYMPUS_TAG_QUALITY: c_int = 0x0201;
const MNOTE_OLYMPUS_TAG_MACRO: c_int = 0x0202;
const MNOTE_OLYMPUS_TAG_BWMODE: c_int = 0x0203;
const MNOTE_OLYMPUS_TAG_DIGIZOOM: c_int = 0x0204;
const MNOTE_OLYMPUS_TAG_FOCALPLANEDIAGONAL: c_int = 0x0205;
const MNOTE_OLYMPUS_TAG_LENSDISTORTION: c_int = 0x0206;
const MNOTE_OLYMPUS_TAG_VERSION: c_int = 0x0207;
const MNOTE_OLYMPUS_TAG_INFO: c_int = 0x0208;
const MNOTE_OLYMPUS_TAG_ID: c_int = 0x0209;
const MNOTE_OLYMPUS_TAG_DATADUMP: c_int = 0x0f00;
const MNOTE_SANYO_TAG_SEQUENTIALSHOT: c_int = 0x020e;
const MNOTE_SANYO_TAG_WIDERANGE: c_int = 0x020f;
const MNOTE_SANYO_TAG_COLORADJUSTMENTMODE: c_int = 0x0210;
const MNOTE_SANYO_TAG_FOCUSMODE: c_int = 0x0212;
const MNOTE_SANYO_TAG_QUICKSHOT: c_int = 0x0213;
const MNOTE_SANYO_TAG_SELFTIMER: c_int = 0x0214;
const MNOTE_SANYO_TAG_VOICEMEMO: c_int = 0x0216;
const MNOTE_SANYO_TAG_RECORDSHUTTERRELEASE: c_int = 0x0217;
const MNOTE_SANYO_TAG_FLICKERREDUCE: c_int = 0x0218;
const MNOTE_SANYO_TAG_OPTICALZOOM: c_int = 0x0219;
const MNOTE_SANYO_TAG_CCDSENSITIVITY: c_int = 0x021a;
const MNOTE_SANYO_TAG_DIGITALZOOM: c_int = 0x021b;
const MNOTE_SANYO_TAG_LIGHTSOURCESPECIAL: c_int = 0x021d;
const MNOTE_SANYO_TAG_RESAVED: c_int = 0x021e;
const MNOTE_SANYO_TAG_SCENESELECT: c_int = 0x021f;
const MNOTE_SANYO_TAG_MANUALFOCUSDISTANCE: c_int = 0x0223;
const MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL: c_int = 0x0224;
const MNOTE_OLYMPUS_TAG_FLASHMODE: c_int = 0x1004;
const MNOTE_OLYMPUS_TAG_FLASHDEVICE: c_int = 0x1005;
const MNOTE_OLYMPUS_TAG_SENSORTEMPERATURE: c_int = 0x1007;
const MNOTE_OLYMPUS_TAG_LENSTEMPERATURE: c_int = 0x1008;
const MNOTE_OLYMPUS_TAG_FOCUSRANGE: c_int = 0x100a;
const MNOTE_OLYMPUS_TAG_MANFOCUS: c_int = 0x100b;
const MNOTE_OLYMPUS_TAG_FOCUSDIST: c_int = 0x100c;
const MNOTE_OLYMPUS_TAG_SHARPNESS: c_int = 0x100f;
const MNOTE_OLYMPUS_TAG_WBALANCE: c_int = 0x1015;
const MNOTE_OLYMPUS_TAG_REDBALANCE: c_int = 0x1017;
const MNOTE_OLYMPUS_TAG_BLUEBALANCE: c_int = 0x1018;
const MNOTE_OLYMPUS_TAG_BLACKLEVEL: c_int = 0x1012;
const MNOTE_OLYMPUS_TAG_COLORMATRIX: c_int = 0x1011;
const MNOTE_OLYMPUS_TAG_EXTERNALFLASHBOUNCE: c_int = 0x1026;
const MNOTE_OLYMPUS_TAG_CONTRAST: c_int = 0x1029;
const MNOTE_OLYMPUS_TAG_COLORCONTROL: c_int = 0x102b;
const MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID: c_int = 0x1035;
const MNOTE_OLYMPUS_TAG_CCDSCANMODE: c_int = 0x1039;
const MNOTE_OLYMPUS_TAG_NOISEREDUCTION: c_int = 0x103a;
const MNOTE_OLYMPUS_TAG_UNKNOWN_4: c_int = 0x0f04;

#[derive(Clone, Copy)]
struct OlympusEnumValue {
    tag: c_int,
    value: u16,
    text: &'static str,
}

const OLYMPUS_ENUM_VALUES: &[OlympusEnumValue] = &[
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 1,
        text: "Normal, SQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 2,
        text: "Normal, HQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 3,
        text: "Normal, SHQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 4,
        text: "Normal, RAW",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 5,
        text: "Normal, SQ1",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 6,
        text: "Normal, SQ2",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 7,
        text: "Normal, super high",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 17,
        text: "Normal, standard",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x101,
        text: "Fine, SQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x102,
        text: "Fine, HQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x103,
        text: "Fine, SHQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x104,
        text: "Fine, RAW",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x105,
        text: "Fine, SQ1",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x106,
        text: "Fine, SQ2",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x107,
        text: "Fine, super high",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x201,
        text: "Super fine, SQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x202,
        text: "Super fine, HQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x203,
        text: "Super fine, SHQ",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x204,
        text: "Super fine, RAW",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x205,
        text: "Super fine, SQ1",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x206,
        text: "Super fine, SQ2",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x207,
        text: "Super fine, super high",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_QUALITY,
        value: 0x211,
        text: "Super fine, high",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_MACRO,
        value: 0,
        text: "No",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_MACRO,
        value: 1,
        text: "Yes",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_MACRO,
        value: 2,
        text: "Super macro",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_BWMODE,
        value: 0,
        text: "No",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_BWMODE,
        value: 1,
        text: "Yes",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHMODE,
        value: 0,
        text: "Auto",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHMODE,
        value: 1,
        text: "Red-eye reduction",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHMODE,
        value: 2,
        text: "Fill",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHMODE,
        value: 3,
        text: "Off",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHDEVICE,
        value: 0,
        text: "None",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHDEVICE,
        value: 1,
        text: "Internal",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHDEVICE,
        value: 4,
        text: "External",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FLASHDEVICE,
        value: 5,
        text: "Internal + external",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FOCUSRANGE,
        value: 0,
        text: "Normal",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_FOCUSRANGE,
        value: 1,
        text: "Macro",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_MANFOCUS,
        value: 0,
        text: "Auto",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_MANFOCUS,
        value: 1,
        text: "Manual",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_SHARPNESS,
        value: 0,
        text: "Normal",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_SHARPNESS,
        value: 1,
        text: "Hard",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_SHARPNESS,
        value: 2,
        text: "Soft",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_EXTERNALFLASHBOUNCE,
        value: 0,
        text: "No",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_EXTERNALFLASHBOUNCE,
        value: 1,
        text: "Yes",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_CONTRAST,
        value: 0,
        text: "Hard",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_CONTRAST,
        value: 1,
        text: "Normal",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_CONTRAST,
        value: 2,
        text: "Soft",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID,
        value: 0,
        text: "No",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID,
        value: 1,
        text: "Yes",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_CCDSCANMODE,
        value: 0,
        text: "Interlaced",
    },
    OlympusEnumValue {
        tag: MNOTE_OLYMPUS_TAG_CCDSCANMODE,
        value: 1,
        text: "Progressive",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENTIALSHOT,
        value: 0,
        text: "None",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENTIALSHOT,
        value: 1,
        text: "Standard",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENTIALSHOT,
        value: 2,
        text: "Best",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENTIALSHOT,
        value: 3,
        text: "Adjust exposure",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_FOCUSMODE,
        value: 1,
        text: "Spot focus",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_FOCUSMODE,
        value: 2,
        text: "Normal focus",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_RECORDSHUTTERRELEASE,
        value: 0,
        text: "Record while down",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_RECORDSHUTTERRELEASE,
        value: 1,
        text: "Press start, press stop",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_RESAVED,
        value: 0,
        text: "No",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_RESAVED,
        value: 1,
        text: "Yes",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_CCDSENSITIVITY,
        value: 0,
        text: "Auto",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_CCDSENSITIVITY,
        value: 1,
        text: "ISO 50",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_CCDSENSITIVITY,
        value: 3,
        text: "ISO 100",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_CCDSENSITIVITY,
        value: 4,
        text: "ISO 200",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_CCDSENSITIVITY,
        value: 5,
        text: "ISO 400",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 0,
        text: "Off",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 1,
        text: "Sport",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 2,
        text: "TV",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 3,
        text: "Night",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 4,
        text: "User 1",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 5,
        text: "User 2",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SCENESELECT,
        value: 6,
        text: "Lamp",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL,
        value: 0,
        text: "5 frames/sec",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL,
        value: 1,
        text: "10 frames/sec",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL,
        value: 2,
        text: "15 frames/sec",
    },
    OlympusEnumValue {
        tag: MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL,
        value: 3,
        text: "20 frames/sec",
    },
];

fn olympus_lookup_enum(tag: c_int, value: u16) -> Option<&'static str> {
    OLYMPUS_ENUM_VALUES
        .iter()
        .find(|entry| entry.tag == tag && entry.value == value)
        .map(|entry| entry.text)
}

const DOMAIN: &[u8] = b"ExifMnoteDataOlympus\0";
const MSG_SHORT: &[u8] = b"Short MakerNote\0";
const MSG_TOO_MANY: &[u8] = b"Too much tags (%d) in Olympus MakerNote\0";
const MSG_OVERFLOW: &[u8] = b"Tag size overflow detected (%u * %lu)\0";
const MSG_PAST_END: &[u8] = b"Tag data past end of buffer (%u > %u)\0";
const MSG_UNKNOWN_VARIANT: &[u8] = b"Unknown Olympus variant %i.\0";
const MSG_UNKNOWN_ORDER: &[u8] = b"Unknown byte order '%c%c'\0";
const MSG_NO_MEMORY: &[u8] = b"Could not allocate %u byte(s)\0";

#[repr(C)]
struct ExifMnoteDataOlympus {
    parent: ExifMnoteData,
    entries: *mut MnoteOlympusEntry,
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
fn olympus_note(note: *mut ExifMnoteData) -> *mut ExifMnoteDataOlympus {
    note.cast()
}

const OLYMPUS_TAGS: &[TagInfo] = &[
    TagInfo {
        tag: 0x0001,
        name: Some(message(b"Firmware\0")),
        title: Some(message(b"Firmware Version\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0002,
        name: Some(message(b"ISO\0")),
        title: Some(message(b"ISO Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0003,
        name: Some(message(b"ColorMode1\0")),
        title: Some(message(b"Color Mode (?)\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0004,
        name: Some(message(b"Quality\0")),
        title: Some(message(b"Quality\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0005,
        name: Some(message(b"WhiteBalance\0")),
        title: Some(message(b"White Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0006,
        name: Some(message(b"Sharpening\0")),
        title: Some(message(b"Image Sharpening\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0007,
        name: Some(message(b"FocusMode\0")),
        title: Some(message(b"Focus Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0008,
        name: Some(message(b"FlashSetting\0")),
        title: Some(message(b"Flash Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0009,
        name: Some(message(b"FlashMode\0")),
        title: Some(message(b"Flash Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000b,
        name: Some(message(b"WhiteBalanceFine\0")),
        title: Some(message(b"White Balance Fine Adjustment\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000c,
        name: Some(message(b"WhiteBalanceRB\0")),
        title: Some(message(b"White Balance RB\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000d,
        name: Some(message(b"ProgramShift\0")),
        title: Some(message(b"Program Shift\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x000f,
        name: Some(message(b"ISOSelection\0")),
        title: Some(message(b"ISO Selection\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0011,
        name: Some(message(b"PreviewImage\0")),
        title: Some(message(b"Preview Image IFD\0")),
        description: Some(message(
            b"Offset of the preview image directory (IFD) inside the file.\0",
        )),
    },
    TagInfo {
        tag: 0x000e,
        name: Some(message(b"ExposureDiff\0")),
        title: Some(message(b"Exposurediff ?\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0012,
        name: Some(message(b"FlashExpCompensation\0")),
        title: Some(message(b"Flash Exposure Compensation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0013,
        name: Some(message(b"ISO\0")),
        title: Some(message(b"ISO Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0016,
        name: Some(message(b"ImageBoundary\0")),
        title: Some(message(b"Image Boundary\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0017,
        name: Some(message(b"ExternalFlashExpCompensation\0")),
        title: Some(message(b"External Flash Exposure Compensation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0018,
        name: Some(message(b"FlashExposureBracketVal\0")),
        title: Some(message(b"Flash Exposure Bracket Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0019,
        name: Some(message(b"ExposureBracketVal\0")),
        title: Some(message(b"Exposure Bracket Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0080,
        name: Some(message(b"ImageAdjustment\0")),
        title: Some(message(b"Image Adjustment\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0081,
        name: Some(message(b"ToneCompensation\0")),
        title: Some(message(b"Tone Compensation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0082,
        name: Some(message(b"Adapter\0")),
        title: Some(message(b"Adapter\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0083,
        name: Some(message(b"LensType\0")),
        title: Some(message(b"Lens Type\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0084,
        name: Some(message(b"Lens\0")),
        title: Some(message(b"Lens\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0085,
        name: Some(message(b"ManualFocusDistance\0")),
        title: Some(message(b"Manual Focus Distance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0086,
        name: Some(message(b"DigitalZoom\0")),
        title: Some(message(b"Digital Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0087,
        name: Some(message(b"FlashUsed\0")),
        title: Some(message(b"Flash Used\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0088,
        name: Some(message(b"AFFocusPosition\0")),
        title: Some(message(b"AF Focus Position\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0089,
        name: Some(message(b"Bracketing\0")),
        title: Some(message(b"Bracketing\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x008a,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x008b,
        name: Some(message(b"LensFStops\0")),
        title: Some(message(b"Lens F Stops\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x008c,
        name: Some(message(b"Curve,\0")),
        title: Some(message(b"Contrast Curve\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x008d,
        name: Some(message(b"ColorMode,\0")),
        title: Some(message(b"Color Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0090,
        name: Some(message(b"LightType,\0")),
        title: Some(message(b"Light Type\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0091,
        name: Some(message(b"ShotInfo\0")),
        title: Some(message(b"Shot Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0092,
        name: Some(message(b"Hue\0")),
        title: Some(message(b"Hue Adjustment\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0094,
        name: Some(message(b"Saturation\0")),
        title: Some(message(b"Saturation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0095,
        name: Some(message(b"NoiseReduction,\0")),
        title: Some(message(b"Noise Reduction\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0097,
        name: Some(message(b"ColorBalance\0")),
        title: Some(message(b"Color Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0098,
        name: Some(message(b"LensData\0")),
        title: Some(message(b"Lens Data\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x009a,
        name: Some(message(b"SensorPixelSize\0")),
        title: Some(message(b"Sensor Pixel Size\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x009b,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x009e,
        name: Some(message(b"RetouchHistory\0")),
        title: Some(message(b"Retouch History\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x001d,
        name: Some(message(b"SerialNumber\0")),
        title: Some(message(b"Serial Number\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00a2,
        name: Some(message(b"ImageDataSize\0")),
        title: Some(message(b"Image Data Size\0")),
        description: Some(message(b"Size of compressed image data in bytes.\0")),
    },
    TagInfo {
        tag: 0x00a3,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00a7,
        name: Some(message(b"TotalPictures,\0")),
        title: Some(message(b"Total Number of Pictures Taken\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00a8,
        name: Some(message(b"FlashInfo\0")),
        title: Some(message(b"Flash Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00a9,
        name: Some(message(b"Optimization,\0")),
        title: Some(message(b"Optimize Image\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0094,
        name: Some(message(b"Saturation\0")),
        title: Some(message(b"Saturation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00ab,
        name: Some(message(b"VariProgram\0")),
        title: Some(message(b"Vari Program\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0e01,
        name: Some(message(b"CaptureEditorData\0")),
        title: Some(message(b"Capture Editor Data\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0e09,
        name: Some(message(b"CaptureEditorVer\0")),
        title: Some(message(b"Capture Editor Version\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0e0e,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0e10,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x001b,
        name: Some(message(b"CropHiSpeed\0")),
        title: Some(message(b"Crop HiSpeed\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x001c,
        name: Some(message(b"ExposureTuning\0")),
        title: Some(message(b"Exposure Tuning\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x001e,
        name: Some(message(b"ColorSpace\0")),
        title: Some(message(b"Color Space\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x001f,
        name: Some(message(b"VRInfo\0")),
        title: Some(message(b"VR Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0020,
        name: Some(message(b"ImageAuthentication\0")),
        title: Some(message(b"Image Authentication\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0021,
        name: Some(message(b"FaceDetect\0")),
        title: Some(message(b"Face Detect\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0022,
        name: Some(message(b"ActiveDLighting\0")),
        title: Some(message(b"Active DLighting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0023,
        name: Some(message(b"PictureControlData\0")),
        title: Some(message(b"Picture Control Data\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0024,
        name: Some(message(b"WorldTime\0")),
        title: Some(message(b"World Time\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0025,
        name: Some(message(b"ISOInfo\0")),
        title: Some(message(b"ISO Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x002a,
        name: Some(message(b"VignetteControl\0")),
        title: Some(message(b"Vignette Control\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x002b,
        name: Some(message(b"DistortInfo\0")),
        title: Some(message(b"Distort Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0034,
        name: Some(message(b"ShutterMode\0")),
        title: Some(message(b"Shutter Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0035,
        name: Some(message(b"HDRInfo\0")),
        title: Some(message(b"HDR Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0037,
        name: Some(message(b"MechanicalShutterCount\0")),
        title: Some(message(b"Mechanical Shutter Count\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0039,
        name: Some(message(b"LocationInfo\0")),
        title: Some(message(b"MNOTE_NIKON_TAG_LOCATIONINFO\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x003d,
        name: Some(message(b"BlackLevel\0")),
        title: Some(message(b"Black Level\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x003e,
        name: Some(message(b"ImageSizeRaw\0")),
        title: Some(message(b"Image Size Raw\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0045,
        name: Some(message(b"CropArea\0")),
        title: Some(message(b"Crop Area\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x004e,
        name: Some(message(b"NikonSettings\0")),
        title: Some(message(b"Nikon Settings\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x004f,
        name: Some(message(b"ColorTemperatureAuto\0")),
        title: Some(message(b"Color Temperature Auto\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00a0,
        name: Some(message(b"SerialNumber2\0")),
        title: Some(message(b"Serial Number 2\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00aa,
        name: Some(message(b"Saturation2\0")),
        title: Some(message(b"Saturation 2\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b0,
        name: Some(message(b"MultiExposure\0")),
        title: Some(message(b"Multi Exposure\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b1,
        name: Some(message(b"HighISONr\0")),
        title: Some(message(b"High ISO Noise Reduction\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b3,
        name: Some(message(b"ToningEffect\0")),
        title: Some(message(b"Toning Effect\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b6,
        name: Some(message(b"PowerupTime\0")),
        title: Some(message(b"Powerup Time\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b7,
        name: Some(message(b"AFInfo2\0")),
        title: Some(message(b"AF Info 2\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00b8,
        name: Some(message(b"FileInfo\0")),
        title: Some(message(b"File Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x00bb,
        name: Some(message(b"RetouchInfo\0")),
        title: Some(message(b"Retouch Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0011,
        name: Some(message(b"PreviewImage\0")),
        title: Some(message(b"Preview Image\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8002,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8003,
        name: Some(message(b"Quality\0")),
        title: Some(message(b"Quality\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8004,
        name: Some(message(b"ColorMode,\0")),
        title: Some(message(b"Color Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8005,
        name: Some(message(b"ImageAdjustment\0")),
        title: Some(message(b"Image Adjustment\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8006,
        name: Some(message(b"CCDSensitivity\0")),
        title: Some(message(b"CCD Sensitivity\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8007,
        name: Some(message(b"WhiteBalance\0")),
        title: Some(message(b"White Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8008,
        name: Some(message(b"Focus\0")),
        title: Some(message(b"Focus\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x8009,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x800a,
        name: Some(message(b"DigitalZoom\0")),
        title: Some(message(b"Digital Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x800b,
        name: Some(message(b"Converter\0")),
        title: Some(message(b"Converter\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0100,
        name: Some(message(b"ThumbnailImage\0")),
        title: Some(message(b"Thumbnail Image\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0200,
        name: Some(message(b"Mode\0")),
        title: Some(message(b"Speed/Sequence/Panorama Direction\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0201,
        name: Some(message(b"Quality\0")),
        title: Some(message(b"Quality\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0202,
        name: Some(message(b"Macro\0")),
        title: Some(message(b"Macro\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0203,
        name: Some(message(b"BWMode\0")),
        title: Some(message(b"Black & White Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0204,
        name: Some(message(b"DigiZoom\0")),
        title: Some(message(b"Digital Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0205,
        name: Some(message(b"FocalPlaneDiagonal\0")),
        title: Some(message(b"Focal Plane Diagonal\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0206,
        name: Some(message(b"LensDistortionParams\0")),
        title: Some(message(b"Lens Distortion Parameters\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0207,
        name: Some(message(b"FirmwareVersion\0")),
        title: Some(message(b"Firmware Version\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0208,
        name: Some(message(b"Info\0")),
        title: Some(message(b"Info\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0209,
        name: Some(message(b"CameraID\0")),
        title: Some(message(b"Camera ID\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0300,
        name: Some(message(b"PreCaptureFrames\0")),
        title: Some(message(b"Precapture Frames\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0301,
        name: Some(message(b"WhiteBoard\0")),
        title: Some(message(b"White Board\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0302,
        name: Some(message(b"OneTouchWB\0")),
        title: Some(message(b"One Touch White Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0303,
        name: Some(message(b"WhiteBalanceBracket\0")),
        title: Some(message(b"White Balance Bracket\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0304,
        name: Some(message(b"WhiteBalanceBias\0")),
        title: Some(message(b"White Balance Bias\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0f00,
        name: Some(message(b"DataDump\0")),
        title: Some(message(b"Data Dump\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0f04,
        name: None,
        title: None,
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1000,
        name: Some(message(b"ShutterSpeed\0")),
        title: Some(message(b"Shutter Speed\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1001,
        name: Some(message(b"ISOValue\0")),
        title: Some(message(b"ISO Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1002,
        name: Some(message(b"ApertureValue\0")),
        title: Some(message(b"Aperture Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1003,
        name: Some(message(b"BrightnessValue\0")),
        title: Some(message(b"Brightness Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1004,
        name: Some(message(b"FlashMode\0")),
        title: Some(message(b"Flash Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1005,
        name: Some(message(b"FlashDevice\0")),
        title: Some(message(b"Flash Device\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1006,
        name: Some(message(b"ExposureCompensation\0")),
        title: Some(message(b"Exposure Compensation\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1007,
        name: Some(message(b"SensorTemperature\0")),
        title: Some(message(b"Sensor Temperature\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1008,
        name: Some(message(b"LensTemperature\0")),
        title: Some(message(b"Lens Temperature\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1009,
        name: Some(message(b"LightCondition\0")),
        title: Some(message(b"Light Condition\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x100a,
        name: Some(message(b"FocusRange\0")),
        title: Some(message(b"Focus Range\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x100b,
        name: Some(message(b"FocusMode\0")),
        title: Some(message(b"Focus Mode\0")),
        description: Some(message(b"Automatic or manual focusing mode\0")),
    },
    TagInfo {
        tag: 0x100c,
        name: Some(message(b"ManualFocusDistance\0")),
        title: Some(message(b"Manual Focus Distance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x100d,
        name: Some(message(b"ZoomStepCount\0")),
        title: Some(message(b"Zoom Step Count\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x100e,
        name: Some(message(b"FocusStepCount\0")),
        title: Some(message(b"Focus Step Count\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x100f,
        name: Some(message(b"Sharpness\0")),
        title: Some(message(b"Sharpness Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1010,
        name: Some(message(b"FlashChargeLevel\0")),
        title: Some(message(b"Flash Charge Level\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1011,
        name: Some(message(b"ColorMatrix\0")),
        title: Some(message(b"Color Matrix\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1012,
        name: Some(message(b"BlackLevel\0")),
        title: Some(message(b"Black Level\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1015,
        name: Some(message(b"WhiteBalance\0")),
        title: Some(message(b"White Balance Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1017,
        name: Some(message(b"RedBalance\0")),
        title: Some(message(b"Red Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1018,
        name: Some(message(b"BlueBalance\0")),
        title: Some(message(b"Blue Balance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1019,
        name: Some(message(b"ColorMatrixNumber\0")),
        title: Some(message(b"Color Matrix Number\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x101a,
        name: Some(message(b"SerialNumber\0")),
        title: Some(message(b"Serial Number\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1023,
        name: Some(message(b"FlashExposureComp\0")),
        title: Some(message(b"Flash Exposure Comp\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1024,
        name: Some(message(b"InternalFlashTable\0")),
        title: Some(message(b"Internal Flash Table\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1025,
        name: Some(message(b"ExternalFlashGValue\0")),
        title: Some(message(b"External Flash G Value\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1026,
        name: Some(message(b"ExternalFlashBounce\0")),
        title: Some(message(b"External Flash Bounce\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1027,
        name: Some(message(b"ExternalFlashZoom\0")),
        title: Some(message(b"External Flash Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1028,
        name: Some(message(b"ExternalFlashMode\0")),
        title: Some(message(b"External Flash Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1029,
        name: Some(message(b"Contrast\0")),
        title: Some(message(b"Contrast Setting\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x102a,
        name: Some(message(b"SharpnessFactor\0")),
        title: Some(message(b"Sharpness Factor\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x102b,
        name: Some(message(b"ColorControl\0")),
        title: Some(message(b"Color Control\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x102e,
        name: Some(message(b"OlympusImageWidth\0")),
        title: Some(message(b"Olympus Image Width\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x102f,
        name: Some(message(b"OlympusImageHeight\0")),
        title: Some(message(b"Olympus Image Height\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1030,
        name: Some(message(b"SceneDetect\0")),
        title: Some(message(b"Scene Detect\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1034,
        name: Some(message(b"CompressionRatio\0")),
        title: Some(message(b"Compression Ratio\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1035,
        name: Some(message(b"PreviewImageValid\0")),
        title: Some(message(b"Preview Image Valid\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1038,
        name: Some(message(b"AFResult\0")),
        title: Some(message(b"AF Result\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x1039,
        name: Some(message(b"CCDScanMode\0")),
        title: Some(message(b"CCD Scan Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x103a,
        name: Some(message(b"NoiseReduction\0")),
        title: Some(message(b"Noise Reduction\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x103b,
        name: Some(message(b"InfinityLensStep\0")),
        title: Some(message(b"Infinity Lens Step\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x103c,
        name: Some(message(b"NearLensStep\0")),
        title: Some(message(b"Near Lens Step\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x103d,
        name: Some(message(b"LightValueCenter\0")),
        title: Some(message(b"Light Value Center\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x103e,
        name: Some(message(b"LightValuePeriphery\0")),
        title: Some(message(b"Light Value Periphery\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x020e,
        name: Some(message(b"SequentialShot\0")),
        title: Some(message(b"Sequential Shot\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x020f,
        name: Some(message(b"WideRange\0")),
        title: Some(message(b"Wide Range\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0210,
        name: Some(message(b"ColorAdjustmentMode\0")),
        title: Some(message(b"Color Adjustment Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0212,
        name: Some(message(b"FocusMode\0")),
        title: Some(message(b"Focus Mode\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0213,
        name: Some(message(b"QuickShot\0")),
        title: Some(message(b"Quick Shot\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0214,
        name: Some(message(b"SelfTimer\0")),
        title: Some(message(b"Self-timer\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0216,
        name: Some(message(b"VoiceMemo\0")),
        title: Some(message(b"Voice Memo\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0217,
        name: Some(message(b"RecordShutterRelease\0")),
        title: Some(message(b"Record Shutter Release\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0218,
        name: Some(message(b"FlickerReduce\0")),
        title: Some(message(b"Flicker Reduce\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0219,
        name: Some(message(b"OpticalZoom\0")),
        title: Some(message(b"Optical Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x021b,
        name: Some(message(b"DigitalZoom\0")),
        title: Some(message(b"Digital Zoom\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x021d,
        name: Some(message(b"LightSourceSpecial\0")),
        title: Some(message(b"Light Source Special\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x021e,
        name: Some(message(b"Resaved\0")),
        title: Some(message(b"Resaved\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x021a,
        name: Some(message(b"CCDSensitivity\0")),
        title: Some(message(b"CCD Sensitivity\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x021f,
        name: Some(message(b"SceneSelect\0")),
        title: Some(message(b"Scene Select\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0223,
        name: Some(message(b"ManualFocusDistance\0")),
        title: Some(message(b"Manual Focus Distance\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x0224,
        name: Some(message(b"SequenceShotInterval\0")),
        title: Some(message(b"Sequence Shot Interval\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x020b,
        name: Some(message(b"EpsonImageWidth\0")),
        title: Some(message(b"Epson Image Width\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x020c,
        name: Some(message(b"EpsonImageHeight\0")),
        title: Some(message(b"Epson Image Height\0")),
        description: Some(empty_message()),
    },
    TagInfo {
        tag: 0x020d,
        name: Some(message(b"EpsonSoftware\0")),
        title: Some(message(b"Epson Software Version\0")),
        description: Some(empty_message()),
    },
];

fn olympus_tag_name_impl(tag: MnoteOlympusTag) -> *const c_char {
    tag_name_from_table(OLYMPUS_TAGS, tag)
}

fn olympus_tag_title_impl(tag: MnoteOlympusTag) -> *const c_char {
    tag_title_from_table(OLYMPUS_TAGS, tag)
}

fn olympus_tag_description_impl(tag: MnoteOlympusTag) -> *const c_char {
    tag_description_from_table(OLYMPUS_TAGS, tag)
}

unsafe fn log_simple(note: *mut ExifMnoteDataOlympus, code: ExifLogCode, format: &[u8]) {
    unsafe {
        exif_log(
            (*note).parent.log,
            code,
            DOMAIN.as_ptr().cast(),
            format.as_ptr().cast(),
        )
    };
}

unsafe fn log_no_memory(note: *mut ExifMnoteDataOlympus, size: usize) {
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

unsafe fn clear_impl(note: *mut ExifMnoteDataOlympus) {
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

unsafe extern "C" fn exif_mnote_data_olympus_free(note: *mut ExifMnoteData) {
    unsafe { clear_impl(olympus_note(note)) };
}

unsafe extern "C" fn exif_mnote_data_olympus_get_value(
    note: *mut ExifMnoteData,
    index: c_uint,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    let note = olympus_note(note);
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null_mut();
    }
    unsafe { mnote_olympus_entry_get_value((*note).entries.add(index as usize), value, maxlen) }
}

unsafe extern "C" fn exif_mnote_data_olympus_save(
    note: *mut ExifMnoteData,
    buffer: *mut *mut c_uchar,
    buffer_size: *mut c_uint,
) {
    let note = olympus_note(note);
    if note.is_null() || buffer.is_null() || buffer_size.is_null() {
        return;
    }

    let mut base = 0usize;
    let mut o2 = 6usize + 2;
    let mut data_offset = 0usize;
    let mut out_size = 6usize + 2 + 2 + unsafe { (*note).count as usize } * 12;
    let mut out = match unsafe { (*note).version } {
        OLYMPUS_V1 | SANYO_V1 | EPSON_V1 => unsafe {
            let out = exif_mem_alloc_impl((*note).parent.mem, out_size as u32).cast::<c_uchar>();
            if out.is_null() {
                log_no_memory(note, out_size);
                return;
            }
            let header = if (*note).version == SANYO_V1 {
                b"SANYO\0"
            } else if (*note).version == EPSON_V1 {
                b"EPSON\0"
            } else {
                b"OLYMP\0"
            };
            ptr::copy_nonoverlapping(header.as_ptr(), out, 6);
            exif_set_short(out.add(6), (*note).order, 1);
            data_offset = (*note).offset as usize;
            out
        },
        OLYMPUS_V2 => unsafe {
            out_size += 2 + 4;
            let out = exif_mem_alloc_impl((*note).parent.mem, out_size as u32).cast::<c_uchar>();
            if out.is_null() {
                log_no_memory(note, out_size);
                return;
            }
            ptr::copy_nonoverlapping(b"OLYMPUS\0".as_ptr(), out, 8);
            exif_set_short(
                out.add(8),
                (*note).order,
                if (*note).order == EXIF_BYTE_ORDER_INTEL {
                    u16::from(b'I') << 8 | u16::from(b'I')
                } else {
                    u16::from(b'M') << 8 | u16::from(b'M')
                },
            );
            exif_set_short(out.add(10), (*note).order, 3);
            o2 += 4;
            out
        },
        NIKON_V1 => unsafe {
            base = MNOTE_NIKON1_TAG_BASE as usize;
            data_offset += (*note).offset as usize + 10;
            out_size = out_size.saturating_sub(10);
            let out =
                exif_mem_alloc_impl((*note).parent.mem, out_size as u32 + 14).cast::<c_uchar>();
            if out.is_null() {
                log_no_memory(note, out_size + 14);
                return;
            }
            ptr::copy_nonoverlapping(b"Nikon\0".as_ptr(), out, 6);
            *out.add(6) = (*note).version as u8;
            data_offset = data_offset.wrapping_sub(10);
            exif_set_long(
                out.add(o2 + 2 + (*note).count as usize * 12),
                (*note).order,
                0,
            );
            out_size += 10 + 4;
            out
        },
        NIKON_V2 | NIKON_V0 => unsafe {
            out_size += 10 + 4;
            let out = exif_mem_alloc_impl((*note).parent.mem, out_size as u32).cast::<c_uchar>();
            if out.is_null() {
                log_no_memory(note, out_size);
                return;
            }
            ptr::copy_nonoverlapping(b"Nikon\0".as_ptr(), out, 6);
            *out.add(6) = (*note).version as u8;
            exif_set_short(
                out.add(10),
                (*note).order,
                if (*note).order == EXIF_BYTE_ORDER_INTEL {
                    u16::from(b'I') << 8 | u16::from(b'I')
                } else {
                    u16::from(b'M') << 8 | u16::from(b'M')
                },
            );
            exif_set_short(out.add(12), (*note).order, 0x2A);
            exif_set_long(out.add(14), (*note).order, 8);
            o2 += 10;
            data_offset = data_offset.wrapping_sub(10);
            exif_set_long(
                out.add(o2 + 2 + (*note).count as usize * 12),
                (*note).order,
                0,
            );
            out
        },
        _ => return,
    };

    unsafe {
        *buffer = out;
        *buffer_size = out_size as c_uint;
        exif_set_short(out.add(o2), (*note).order, (*note).count as u16);
    }
    o2 += 2;

    for index in 0..unsafe { (*note).count as usize } {
        let entry = unsafe { &*(*note).entries.add(index) };
        let mut offset = o2 + index * 12;
        unsafe {
            exif_set_short(
                out.add(offset),
                (*note).order,
                (entry.tag as usize - base) as u16,
            );
            exif_set_short(out.add(offset + 2), (*note).order, entry.format as u16);
            exif_set_long(out.add(offset + 4), (*note).order, entry.components as u32);
        }
        offset += 8;

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
                    out.add(offset),
                    (*note).order,
                    data_offset.wrapping_add(value_offset) as u32,
                )
            };
            value_offset
        } else {
            offset
        };

        let out_ref = unsafe { *buffer };
        if entry.data.is_null() {
            unsafe { ptr::write_bytes(out_ref.add(data_at), 0, data_size) };
        } else {
            unsafe { ptr::copy_nonoverlapping(entry.data, out_ref.add(data_at), data_size) };
        }
    }
}

unsafe extern "C" fn exif_mnote_data_olympus_load(
    note: *mut ExifMnoteData,
    buffer: *const c_uchar,
    buffer_size: c_uint,
) {
    let note = olympus_note(note);
    let buffer_size = buffer_size as usize;
    if note.is_null() || buffer.is_null() || buffer_size == 0 {
        if !note.is_null() {
            unsafe { log_simple(note, 3, MSG_SHORT) };
        }
        return;
    }

    let mut o2 = 6 + unsafe { (*note).offset as usize };
    let mut data_offset = 6usize;
    let mut base = 0usize;
    if check_overflow(o2, buffer_size, 10) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }

    unsafe { (*note).version = identify_variant(buffer.add(o2), (buffer_size - o2) as c_uint) };
    match unsafe { (*note).version } {
        OLYMPUS_V1 | SANYO_V1 | EPSON_V1 => {
            let bytes = unsafe { std::slice::from_raw_parts(buffer.add(o2), 8) };
            unsafe {
                if bytes[6] == 1 {
                    (*note).order = EXIF_BYTE_ORDER_INTEL;
                } else if bytes[7] == 1 {
                    (*note).order = EXIF_BYTE_ORDER_MOTOROLA;
                }
            }
            o2 += 8;
            let c = unsafe { exif_get_short(buffer.add(o2), (*note).order) };
            if (c & 0x00ff) == 0 && c > 0x0500 {
                unsafe {
                    (*note).order = if (*note).order == EXIF_BYTE_ORDER_INTEL {
                        EXIF_BYTE_ORDER_MOTOROLA
                    } else {
                        EXIF_BYTE_ORDER_INTEL
                    };
                }
            }
        }
        OLYMPUS_V2 => {
            data_offset = o2;
            o2 += 8;
            if check_overflow(o2, buffer_size, 4) {
                return;
            }
            let order_marker = unsafe { std::slice::from_raw_parts(buffer.add(o2), 2) };
            unsafe {
                if order_marker == b"II" {
                    (*note).order = EXIF_BYTE_ORDER_INTEL;
                } else if order_marker == b"MM" {
                    (*note).order = EXIF_BYTE_ORDER_MOTOROLA;
                }
            }
            o2 += 4;
        }
        NIKON_V1 => {
            o2 += 8;
            base = MNOTE_NIKON1_TAG_BASE as usize;
            let c = unsafe { exif_get_short(buffer.add(o2), (*note).order) };
            if (c & 0x00ff) == 0 && c > 0x0500 {
                unsafe {
                    (*note).order = if (*note).order == EXIF_BYTE_ORDER_INTEL {
                        EXIF_BYTE_ORDER_MOTOROLA
                    } else {
                        EXIF_BYTE_ORDER_INTEL
                    };
                }
            }
        }
        NIKON_V2 => {
            o2 += 6;
            if check_overflow(o2, buffer_size, 12) {
                return;
            }
            o2 += 4;
            data_offset = o2;
            let order_marker = unsafe { std::slice::from_raw_parts(buffer.add(o2), 2) };
            unsafe {
                if order_marker == b"II" {
                    (*note).order = EXIF_BYTE_ORDER_INTEL;
                } else if order_marker == b"MM" {
                    (*note).order = EXIF_BYTE_ORDER_MOTOROLA;
                } else {
                    exif_log(
                        (*note).parent.log,
                        1,
                        DOMAIN.as_ptr().cast(),
                        MSG_UNKNOWN_ORDER.as_ptr().cast(),
                        *buffer.add(o2) as c_int,
                        *buffer.add(o2 + 1) as c_int,
                    );
                    return;
                }
            }
            o2 += 4;
            o2 = data_offset + unsafe { exif_get_long(buffer.add(o2), (*note).order) as usize };
        }
        NIKON_V0 => unsafe {
            (*note).order = EXIF_BYTE_ORDER_MOTOROLA;
        },
        _ => unsafe {
            exif_log(
                (*note).parent.log,
                1,
                DOMAIN.as_ptr().cast(),
                MSG_UNKNOWN_VARIANT.as_ptr().cast(),
                (*note).version,
            );
            return;
        },
    }

    if check_overflow(o2, buffer_size, 2) {
        unsafe { log_simple(note, 3, MSG_SHORT) };
        return;
    }
    let count = unsafe { exif_get_short(buffer.add(o2), (*note).order) as usize };
    o2 += 2;
    if count > 300 {
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
    let entry_bytes = size_of::<MnoteOlympusEntry>().saturating_mul(count);
    let entries = unsafe { exif_mem_alloc_impl((*note).parent.mem, entry_bytes as u32) }
        .cast::<MnoteOlympusEntry>();
    if entries.is_null() {
        unsafe { log_no_memory(note, entry_bytes) };
        return;
    }
    unsafe { (*note).entries = entries };

    let mut stored = 0usize;
    let mut offset = o2;
    for _ in 0..count {
        if check_overflow(offset, buffer_size, 12) {
            unsafe { log_simple(note, 3, MSG_SHORT) };
            break;
        }
        let entry = unsafe { (*note).entries.add(stored) };
        unsafe {
            (*entry).tag = exif_get_short(buffer.add(offset), (*note).order) as MnoteOlympusTag
                + base as MnoteOlympusTag;
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
            offset += 12;
            continue;
        }
        let Some(data_size) = format_size.checked_mul(unsafe { (*entry).components as usize })
        else {
            offset += 12;
            continue;
        };
        unsafe { (*entry).size = data_size as c_uint };
        if data_size != 0 {
            let mut data_at = offset + 8;
            if data_size > 4 {
                data_at = unsafe { exif_get_long(buffer.add(data_at), (*note).order) as usize }
                    + data_offset;
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

unsafe extern "C" fn exif_mnote_data_olympus_count(note: *mut ExifMnoteData) -> c_uint {
    let note = olympus_note(note);
    if note.is_null() {
        0
    } else {
        unsafe { (*note).count }
    }
}

unsafe extern "C" fn exif_mnote_data_olympus_get_id(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> c_uint {
    let note = olympus_note(note);
    if note.is_null() || unsafe { (*note).count <= index } {
        0
    } else {
        unsafe { (*(*note).entries.add(index as usize)).tag as c_uint }
    }
}

unsafe extern "C" fn exif_mnote_data_olympus_get_name(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = olympus_note(note);
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(index as usize)).tag };
    olympus_tag_name_impl(tag)
}

unsafe extern "C" fn exif_mnote_data_olympus_get_title(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = olympus_note(note);
    if note.is_null() || unsafe { index >= (*note).count } {
        return ptr::null();
    }
    let tag = unsafe { (*(*note).entries.add(index as usize)).tag };
    olympus_tag_title_impl(tag)
}

unsafe extern "C" fn exif_mnote_data_olympus_get_description(
    note: *mut ExifMnoteData,
    index: c_uint,
) -> *const c_char {
    let note = olympus_note(note);
    if note.is_null() || unsafe { index >= (*note).count } {
        ptr::null()
    } else {
        let tag = unsafe { (*(*note).entries.add(index as usize)).tag };
        olympus_tag_description_impl(tag)
    }
}

unsafe extern "C" fn exif_mnote_data_olympus_set_byte_order(
    note: *mut ExifMnoteData,
    order: ExifByteOrder,
) {
    let note = olympus_note(note);
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

unsafe extern "C" fn exif_mnote_data_olympus_set_offset(note: *mut ExifMnoteData, offset: c_uint) {
    let note = olympus_note(note);
    if !note.is_null() {
        unsafe { (*note).offset = offset };
    }
}

unsafe fn identify_variant(buffer: *const c_uchar, size: c_uint) -> c_int {
    if buffer.is_null() {
        return UNRECOGNIZED;
    }
    let size = size as usize;
    if size >= 8 {
        let bytes = unsafe { std::slice::from_raw_parts(buffer, 8) };
        if bytes == b"OLYMPUS\0" {
            return OLYMPUS_V2;
        }
        if &bytes[..6] == b"OLYMP\0" {
            return OLYMPUS_V1;
        }
        if &bytes[..6] == b"SANYO\0" {
            return SANYO_V1;
        }
        if &bytes[..6] == b"EPSON\0" {
            return EPSON_V1;
        }
        if &bytes[..6] == b"Nikon\0" {
            return match bytes[6] {
                1 => NIKON_V1,
                2 => NIKON_V2,
                _ => UNRECOGNIZED,
            };
        }
    }
    if size >= 2 {
        let bytes = unsafe { std::slice::from_raw_parts(buffer, 2) };
        if bytes == [0x00, 0x1b] {
            return NIKON_V0;
        }
    }
    UNRECOGNIZED
}

pub(crate) unsafe fn identify_impl(data: *const ExifData, entry: *const ExifEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    let mut variant = unsafe { identify_variant((*entry).data, (*entry).size) };
    if variant == NIKON_V0 {
        variant = UNRECOGNIZED;
        if !data.is_null() {
            let make_entry =
                unsafe { crate::mnote::find_entry_impl(data as *mut ExifData, 0x010f) };
            if !make_entry.is_null() {
                let mut value = [0 as c_char; 5];
                let ptr = unsafe {
                    crate::object::entry::exif_entry_get_value_impl(
                        make_entry,
                        value.as_mut_ptr(),
                        value.len() as c_uint,
                    )
                };
                if !ptr.is_null() {
                    let make = unsafe { std::ffi::CStr::from_ptr(ptr) }.to_string_lossy();
                    if make.starts_with("Nikon") || make.starts_with("NIKON") {
                        variant = NIKON_V0;
                    }
                }
            }
        }
    }
    variant
}

pub(crate) unsafe fn new_impl(mem: *mut ExifMem) -> *mut ExifMnoteData {
    if mem.is_null() {
        return ptr::null_mut();
    }
    let note = unsafe { exif_mem_alloc_impl(mem, size_of::<ExifMnoteDataOlympus>() as u32) }
        .cast::<ExifMnoteDataOlympus>();
    if note.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        crate::mnote::base::exif_mnote_data_construct(ptr::addr_of_mut!((*note).parent), mem)
    };
    unsafe {
        (*note).parent.methods = ExifMnoteDataMethods {
            free: Some(exif_mnote_data_olympus_free),
            save: Some(exif_mnote_data_olympus_save),
            load: Some(exif_mnote_data_olympus_load),
            set_offset: Some(exif_mnote_data_olympus_set_offset),
            set_byte_order: Some(exif_mnote_data_olympus_set_byte_order),
            count: Some(exif_mnote_data_olympus_count),
            get_id: Some(exif_mnote_data_olympus_get_id),
            get_name: Some(exif_mnote_data_olympus_get_name),
            get_title: Some(exif_mnote_data_olympus_get_title),
            get_description: Some(exif_mnote_data_olympus_get_description),
            get_value: Some(exif_mnote_data_olympus_get_value),
        };
    }
    unsafe { ptr::addr_of_mut!((*note).parent) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_mnote_data_olympus_new(mem: *mut ExifMem) -> *mut ExifMnoteData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { new_impl(mem) })
}

fn olympus_unknown_data_string(size: usize, data: &[u8]) -> String {
    let mut rendered = format!("{size} bytes unknown data: ");
    for byte in data {
        let _ = write!(&mut rendered, "{byte:02x}");
    }
    rendered
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mnote_olympus_entry_get_value(
    entry: *mut MnoteOlympusEntry,
    value: *mut c_char,
    maxlen: c_uint,
) -> *mut c_char {
    panic_boundary::call_or(ptr::null_mut(), || unsafe {
        if entry.is_null() || value.is_null() {
            return ptr::null_mut();
        }
        let entry_ref = &*entry;
        if !zero_buffer(value, maxlen) {
            return ptr::null_mut();
        }
        if entry_ref.data.is_null() {
            if entry_ref.components > 0 || entry_ref.size == 0 {
                return value;
            }
            if entry_ref.size > 0 {
                return ptr::null_mut();
            }
        }

        match entry_ref.tag {
            MNOTE_NIKON_TAG_FIRMWARE => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_UNDEFINED]),
                    );
                }
                if entry_ref.components != 4 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[4]),
                    );
                }
                let raw = exif_get_long(entry_ref.data, EXIF_BYTE_ORDER_INTEL);
                if (raw & 0xF0F0_F0F0) == 0x3030_3030 {
                    return write_slice_to_buffer(
                        value,
                        maxlen,
                        std::slice::from_raw_parts(
                            entry_ref.data,
                            (entry_ref.size as usize).min(4),
                        ),
                    );
                }
                return write_str_to_buffer(value, maxlen, &format!("{raw:04x}"));
            }
            MNOTE_NIKON_TAG_ISO => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 2 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[2]),
                    );
                }
                let iso = exif_get_short(entry_ref.data.add(2), entry_ref.order);
                return write_str_to_buffer(value, maxlen, &format!("ISO {iso}"));
            }
            MNOTE_NIKON_TAG_QUALITY
            | MNOTE_NIKON_TAG_COLORMODE1
            | MNOTE_NIKON_TAG_WHITEBALANCE
            | MNOTE_NIKON_TAG_SHARPENING
            | MNOTE_NIKON_TAG_FOCUSMODE
            | MNOTE_NIKON_TAG_FLASHSETTING
            | MNOTE_NIKON_TAG_ISOSELECTION
            | MNOTE_NIKON_TAG_FLASHMODE
            | MNOTE_NIKON_TAG_IMAGEADJUSTMENT => {
                if entry_ref.format != EXIF_FORMAT_ASCII {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_ASCII]),
                    );
                }
                return write_slice_to_buffer(
                    value,
                    maxlen,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                );
            }
            MNOTE_NIKON_TAG_WHITEBALANCEFINE
            | MNOTE_NIKON_TAG_SATURATION
            | MNOTE_OLYMPUS_TAG_SENSORTEMPERATURE
            | MNOTE_OLYMPUS_TAG_LENSTEMPERATURE => {
                if entry_ref.format != EXIF_FORMAT_SSHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SSHORT]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                return write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_sshort(entry_ref.data, entry_ref.order).to_string(),
                );
            }
            MNOTE_NIKON_TAG_MANUALFOCUSDISTANCE => {
                if entry_ref.format != EXIF_FORMAT_RATIONAL {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_RATIONAL]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let distance = exif_get_rational(entry_ref.data, entry_ref.order);
                if distance.numerator == 0 || distance.denominator == 0 {
                    return write_str_to_buffer(value, maxlen, "No manual focus selection");
                }
                return write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{:.2} meters",
                        distance.numerator as f64 / distance.denominator as f64
                    ),
                );
            }
            MNOTE_NIKON_TAG_AFFOCUSPOSITION => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_UNDEFINED]),
                    );
                }
                if entry_ref.components != 4 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[4]),
                    );
                }
                let text = match *entry_ref.data.add(1) {
                    0 => "AF position: center",
                    1 => "AF position: top",
                    2 => "AF position: bottom",
                    3 => "AF position: left",
                    4 => "AF position: right",
                    5 => "AF position: upper-left",
                    6 => "AF position: upper-right",
                    7 => "AF position: lower-left",
                    8 => "AF position: lower-right",
                    9 => "AF position: far left",
                    10 => "AF position: far right",
                    _ => "Unknown AF position",
                };
                return write_str_to_buffer(value, maxlen, text);
            }
            MNOTE_OLYMPUS_TAG_FLASHDEVICE => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 2 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[2]),
                    );
                }
                let raw = exif_get_short(entry_ref.data, entry_ref.order);
                return match olympus_lookup_enum(entry_ref.tag, raw) {
                    Some(text) => write_str_to_buffer(value, maxlen, text),
                    None => write_str_to_buffer(value, maxlen, &format!("Unknown value {raw}")),
                };
            }
            MNOTE_OLYMPUS_TAG_DIGIZOOM => {
                if entry_ref.format == EXIF_FORMAT_RATIONAL {
                    if entry_ref.components != 1 {
                        return write_str_to_buffer(
                            value,
                            maxlen,
                            &invalid_components_message(entry_ref.components as u64, &[1]),
                        );
                    }
                    let ratio = exif_get_rational(entry_ref.data, entry_ref.order);
                    if ratio.numerator == 0 || ratio.denominator == 0 {
                        return write_str_to_buffer(value, maxlen, "None");
                    }
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &format!("{:.2}", ratio.numerator as f64 / ratio.denominator as f64),
                    );
                }
                let raw = match entry_ref.format {
                    EXIF_FORMAT_BYTE | EXIF_FORMAT_UNDEFINED => *entry_ref.data as u16,
                    EXIF_FORMAT_SHORT => exif_get_short(entry_ref.data, entry_ref.order),
                    _ => {
                        return write_str_to_buffer(
                            value,
                            maxlen,
                            &invalid_format_message(
                                entry_ref.format,
                                &[EXIF_FORMAT_BYTE, EXIF_FORMAT_SHORT],
                            ),
                        )
                    }
                };
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                return match olympus_lookup_enum(entry_ref.tag, raw) {
                    Some(text) => write_str_to_buffer(value, maxlen, text),
                    None => write_str_to_buffer(value, maxlen, &format!("Unknown value {raw}")),
                };
            }
            MNOTE_OLYMPUS_TAG_QUALITY
            | MNOTE_OLYMPUS_TAG_MACRO
            | MNOTE_OLYMPUS_TAG_BWMODE
            | MNOTE_OLYMPUS_TAG_FLASHMODE
            | MNOTE_OLYMPUS_TAG_FOCUSRANGE
            | MNOTE_OLYMPUS_TAG_MANFOCUS
            | MNOTE_OLYMPUS_TAG_SHARPNESS
            | MNOTE_OLYMPUS_TAG_EXTERNALFLASHBOUNCE
            | MNOTE_OLYMPUS_TAG_CONTRAST
            | MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID
            | MNOTE_OLYMPUS_TAG_CCDSCANMODE
            | MNOTE_SANYO_TAG_SEQUENTIALSHOT
            | MNOTE_SANYO_TAG_FOCUSMODE
            | MNOTE_SANYO_TAG_RECORDSHUTTERRELEASE
            | MNOTE_SANYO_TAG_RESAVED
            | MNOTE_SANYO_TAG_CCDSENSITIVITY
            | MNOTE_SANYO_TAG_SCENESELECT
            | MNOTE_SANYO_TAG_SEQUENCESHOTINTERVAL => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let raw = match entry_ref.format {
                    EXIF_FORMAT_BYTE | EXIF_FORMAT_UNDEFINED => *entry_ref.data as u16,
                    EXIF_FORMAT_SHORT => exif_get_short(entry_ref.data, entry_ref.order),
                    EXIF_FORMAT_LONG if entry_ref.tag == MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID => {
                        exif_get_long(entry_ref.data, entry_ref.order) as u16
                    }
                    _ => {
                        let expected = if entry_ref.tag == MNOTE_OLYMPUS_TAG_PREVIEWIMAGEVALID {
                            &[EXIF_FORMAT_LONG][..]
                        } else {
                            &[EXIF_FORMAT_SHORT][..]
                        };
                        return write_str_to_buffer(
                            value,
                            maxlen,
                            &invalid_format_message(entry_ref.format, expected),
                        );
                    }
                };
                return match olympus_lookup_enum(entry_ref.tag, raw) {
                    Some(text) => write_str_to_buffer(value, maxlen, text),
                    None => write_str_to_buffer(value, maxlen, &format!("Unknown value {raw}")),
                };
            }
            MNOTE_OLYMPUS_TAG_NOISEREDUCTION
            | MNOTE_SANYO_TAG_WIDERANGE
            | MNOTE_SANYO_TAG_COLORADJUSTMENTMODE
            | MNOTE_SANYO_TAG_QUICKSHOT
            | MNOTE_SANYO_TAG_VOICEMEMO
            | MNOTE_SANYO_TAG_FLICKERREDUCE
            | MNOTE_SANYO_TAG_OPTICALZOOM
            | MNOTE_SANYO_TAG_DIGITALZOOM
            | MNOTE_SANYO_TAG_LIGHTSOURCESPECIAL => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let raw = exif_get_short(entry_ref.data, entry_ref.order);
                return match raw {
                    0 => write_str_to_buffer(value, maxlen, "Off"),
                    1 => write_str_to_buffer(value, maxlen, "On"),
                    _ => write_str_to_buffer(value, maxlen, &format!("Unknown {raw}")),
                };
            }
            MNOTE_SANYO_TAG_SELFTIMER => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let raw = exif_get_short(entry_ref.data, entry_ref.order);
                return match raw {
                    0 => write_str_to_buffer(value, maxlen, "Off"),
                    1 => write_str_to_buffer(value, maxlen, "On"),
                    2 => write_str_to_buffer(value, maxlen, "2 sec."),
                    _ => write_str_to_buffer(value, maxlen, &format!("Unknown {raw}")),
                };
            }
            MNOTE_OLYMPUS_TAG_MODE => {
                if entry_ref.format != EXIF_FORMAT_LONG {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_LONG]),
                    );
                }
                if entry_ref.components != 3 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[3]),
                    );
                }
                let first = exif_get_long(entry_ref.data, entry_ref.order);
                let second = exif_get_long(entry_ref.data.add(4), entry_ref.order);
                let third = exif_get_long(entry_ref.data.add(8), entry_ref.order);
                let head = match first {
                    0 => "Normal".to_owned(),
                    1 => "Unknown".to_owned(),
                    2 => "Fast".to_owned(),
                    3 => "Panorama".to_owned(),
                    _ => first.to_string(),
                };
                let tail = match third {
                    1 => "Left to right".to_owned(),
                    2 => "Right to left".to_owned(),
                    3 => "Bottom to top".to_owned(),
                    4 => "Top to bottom".to_owned(),
                    _ => third.to_string(),
                };
                return write_str_to_buffer(value, maxlen, &format!("{head}/{second}/{tail}"));
            }
            MNOTE_OLYMPUS_TAG_LENSDISTORTION => {
                if entry_ref.format == EXIF_FORMAT_SHORT {
                    if entry_ref.components != 1 {
                        return write_str_to_buffer(
                            value,
                            maxlen,
                            &invalid_components_message(entry_ref.components as u64, &[1]),
                        );
                    }
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &exif_get_short(entry_ref.data, entry_ref.order).to_string(),
                    );
                }
                if entry_ref.format != EXIF_FORMAT_SSHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(
                            entry_ref.format,
                            &[EXIF_FORMAT_SHORT, EXIF_FORMAT_SSHORT],
                        ),
                    );
                }
                if entry_ref.components != 6 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[6]),
                    );
                }
                let mut rendered = String::new();
                for index in 0..entry_ref.components as usize {
                    let value_sshort =
                        exif_get_sshort(entry_ref.data.add(2 * index), entry_ref.order);
                    let _ = write!(&mut rendered, "{value_sshort} ");
                }
                return write_str_to_buffer(value, maxlen, &rendered);
            }
            MNOTE_OLYMPUS_TAG_VERSION => {
                if entry_ref.format != EXIF_FORMAT_ASCII {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_ASCII]),
                    );
                }
                if entry_ref.components != 5
                    && entry_ref.components != 6
                    && entry_ref.components != 8
                {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &format!(
                            "Invalid number of components ({}, expected 5, 6 or 8).",
                            entry_ref.components
                        ),
                    );
                }
                return write_slice_to_buffer(
                    value,
                    maxlen,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                );
            }
            MNOTE_OLYMPUS_TAG_INFO => {
                if entry_ref.format != EXIF_FORMAT_ASCII {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_ASCII]),
                    );
                }
                if entry_ref.components != 52 && entry_ref.components != 60 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[52, 60]),
                    );
                }
                return write_slice_to_buffer(
                    value,
                    maxlen,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                );
            }
            MNOTE_OLYMPUS_TAG_ID => {
                if entry_ref.format != EXIF_FORMAT_UNDEFINED {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_UNDEFINED]),
                    );
                }
                if entry_ref.components != 32 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[32]),
                    );
                }
                return write_slice_to_buffer(
                    value,
                    maxlen,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                );
            }
            MNOTE_OLYMPUS_TAG_UNKNOWN_4 => {
                if entry_ref.format != EXIF_FORMAT_LONG {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_LONG]),
                    );
                }
                if entry_ref.components != 30 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[30]),
                    );
                }
                let mut rendered = String::new();
                for index in 0..entry_ref.components as usize {
                    let item = exif_get_long(entry_ref.data.add(4 * index), entry_ref.order);
                    let _ = write!(&mut rendered, "{item} ");
                }
                return write_str_to_buffer(value, maxlen, &rendered);
            }
            MNOTE_OLYMPUS_TAG_FOCUSDIST => {
                if entry_ref.format != EXIF_FORMAT_RATIONAL {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_RATIONAL]),
                    );
                }
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let distance = exif_get_rational(entry_ref.data, entry_ref.order);
                if distance.numerator == 0 || distance.denominator == 0 {
                    return write_str_to_buffer(value, maxlen, "Unknown");
                }
                return write_str_to_buffer(
                    value,
                    maxlen,
                    &format!("{} mm", distance.numerator / distance.denominator),
                );
            }
            MNOTE_OLYMPUS_TAG_WBALANCE => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 2 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[2]),
                    );
                }
                let mode = exif_get_short(entry_ref.data, entry_ref.order);
                return match mode {
                    1 => write_str_to_buffer(value, maxlen, "Automatic"),
                    2 => {
                        let preset = exif_get_short(entry_ref.data.add(2), entry_ref.order);
                        let color_temp = match preset {
                            2 => Some(3000),
                            3 => Some(3700),
                            4 => Some(4000),
                            5 => Some(4500),
                            6 => Some(5500),
                            7 => Some(6500),
                            9 => Some(7500),
                            _ => None,
                        };
                        match color_temp {
                            Some(kelvin) => {
                                write_str_to_buffer(value, maxlen, &format!("Manual: {kelvin}K"))
                            }
                            None => write_str_to_buffer(value, maxlen, "Manual: unknown"),
                        }
                    }
                    3 => write_str_to_buffer(value, maxlen, "One-touch"),
                    _ => write_str_to_buffer(value, maxlen, "Unknown"),
                };
            }
            MNOTE_OLYMPUS_TAG_REDBALANCE | MNOTE_OLYMPUS_TAG_BLUEBALANCE => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.components != 2 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[2]),
                    );
                }
                let first = exif_get_short(entry_ref.data, entry_ref.order);
                let second = exif_get_short(entry_ref.data.add(2), entry_ref.order);
                return write_str_to_buffer(value, maxlen, &format!("{first} {second}"));
            }
            MNOTE_OLYMPUS_TAG_BLACKLEVEL
            | MNOTE_NIKON_TAG_IMAGEBOUNDARY
            | MNOTE_OLYMPUS_TAG_COLORMATRIX => {
                if entry_ref.format != EXIF_FORMAT_SHORT {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_format_message(entry_ref.format, &[EXIF_FORMAT_SHORT]),
                    );
                }
                if entry_ref.tag == MNOTE_OLYMPUS_TAG_COLORMATRIX && entry_ref.components != 9 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[9]),
                    );
                }
                if (entry_ref.tag == MNOTE_OLYMPUS_TAG_BLACKLEVEL
                    || entry_ref.tag == MNOTE_NIKON_TAG_IMAGEBOUNDARY)
                    && entry_ref.components != 4
                {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[4]),
                    );
                }
                let mut rendered = String::new();
                for index in 0..entry_ref.components as usize {
                    let item = exif_get_short(entry_ref.data.add(2 * index), entry_ref.order);
                    let _ = write!(&mut rendered, "{item} ");
                }
                return write_str_to_buffer(value, maxlen, &rendered);
            }
            _ => {}
        }

        match entry_ref.format {
            EXIF_FORMAT_ASCII => write_slice_to_buffer(
                value,
                maxlen,
                std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
            ),
            EXIF_FORMAT_SHORT => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_short(entry_ref.data, entry_ref.order).to_string(),
                )
            }
            EXIF_FORMAT_SSHORT => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_sshort(entry_ref.data, entry_ref.order).to_string(),
                )
            }
            EXIF_FORMAT_LONG => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_long(entry_ref.data, entry_ref.order).to_string(),
                )
            }
            EXIF_FORMAT_SLONG => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &exif_get_slong(entry_ref.data, entry_ref.order).to_string(),
                )
            }
            EXIF_FORMAT_RATIONAL => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let rational = exif_get_rational(entry_ref.data, entry_ref.order);
                if rational.denominator == 0 {
                    return write_str_to_buffer(value, maxlen, "Infinite");
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{:.3}",
                        rational.numerator as f64 / rational.denominator as f64
                    ),
                )
            }
            EXIF_FORMAT_SRATIONAL => {
                if entry_ref.components != 1 {
                    return write_str_to_buffer(
                        value,
                        maxlen,
                        &invalid_components_message(entry_ref.components as u64, &[1]),
                    );
                }
                let rational = exif_get_srational(entry_ref.data, entry_ref.order);
                if rational.denominator == 0 {
                    return write_str_to_buffer(value, maxlen, "Infinite");
                }
                write_str_to_buffer(
                    value,
                    maxlen,
                    &format!(
                        "{:.3}",
                        rational.numerator as f64 / rational.denominator as f64
                    ),
                )
            }
            EXIF_FORMAT_UNDEFINED | _ => write_str_to_buffer(
                value,
                maxlen,
                &olympus_unknown_data_string(
                    entry_ref.size as usize,
                    std::slice::from_raw_parts(entry_ref.data, entry_ref.size as usize),
                ),
            ),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn mnote_olympus_tag_get_description(tag: MnoteOlympusTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || olympus_tag_description_impl(tag))
}

#[unsafe(no_mangle)]
pub extern "C" fn mnote_olympus_tag_get_name(tag: MnoteOlympusTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || olympus_tag_name_impl(tag))
}

#[unsafe(no_mangle)]
pub extern "C" fn mnote_olympus_tag_get_title(tag: MnoteOlympusTag) -> *const c_char {
    panic_boundary::call_or(ptr::null(), || olympus_tag_title_impl(tag))
}
