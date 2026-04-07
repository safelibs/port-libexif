use std::ffi::CStr;
use std::os::raw::c_char;

use exif::ffi::types::*;

unsafe extern "C" {
    fn exif_content_add_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_content_remove_entry(content: *mut ExifContent, entry: *mut ExifEntry);
    fn exif_data_get_mnote_data(data: *mut ExifData) -> *mut ExifMnoteData;
    safe fn exif_data_new() -> *mut ExifData;
    fn exif_data_new_from_data(source: *const u8, size: u32) -> *mut ExifData;
    fn exif_data_set_byte_order(data: *mut ExifData, order: ExifByteOrder);
    fn exif_data_unref(data: *mut ExifData);
    fn exif_entry_get_value(
        entry: *mut ExifEntry,
        value: *mut c_char,
        maxlen: u32,
    ) -> *const c_char;
    fn exif_entry_initialize(entry: *mut ExifEntry, tag: ExifTag);
    safe fn exif_entry_new() -> *mut ExifEntry;
    fn exif_entry_unref(entry: *mut ExifEntry);
    safe fn exif_mem_new_default() -> *mut ExifMem;
    fn exif_mem_unref(mem: *mut ExifMem);
    fn exif_mnote_data_canon_new(mem: *mut ExifMem, option: ExifDataOption) -> *mut ExifMnoteData;
    fn exif_mnote_data_count(note: *mut ExifMnoteData) -> u32;
    fn exif_mnote_data_load(note: *mut ExifMnoteData, buf: *const u8, size: u32);
    fn exif_mnote_data_set_byte_order(note: *mut ExifMnoteData, order: ExifByteOrder);
    fn exif_mnote_data_set_offset(note: *mut ExifMnoteData, offset: u32);
    fn exif_mnote_data_unref(note: *mut ExifMnoteData);
    fn exif_set_rational(buffer: *mut u8, order: ExifByteOrder, value: ExifRational);
    safe fn mnote_canon_tag_get_description(tag: MnoteCanonTag) -> *const c_char;
    safe fn mnote_canon_tag_get_name(tag: MnoteCanonTag) -> *const c_char;
    safe fn mnote_canon_tag_get_title(tag: MnoteCanonTag) -> *const c_char;
    fn mnote_olympus_entry_get_value(
        entry: *mut MnoteOlympusEntry,
        value: *mut c_char,
        maxlen: u32,
    ) -> *mut c_char;
    safe fn mnote_olympus_tag_get_description(tag: MnoteOlympusTag) -> *const c_char;
    safe fn mnote_olympus_tag_get_name(tag: MnoteOlympusTag) -> *const c_char;
    safe fn mnote_olympus_tag_get_title(tag: MnoteOlympusTag) -> *const c_char;
}

const EXIF_TAG_IMAGE_DESCRIPTION: ExifTag = 0x010e;
const EXIF_TAG_EXIF_IFD_POINTER: ExifTag = 0x8769;
const EXIF_TAG_INTEROPERABILITY_IFD_POINTER: ExifTag = 0xa005;
const EXIF_TAG_FNUMBER: ExifTag = 0x829d;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT: ExifTag = 0x0201;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: ExifTag = 0x0202;

const MNOTE_CANON_TAG_FILE_LENGTH: MnoteCanonTag = 0x000e;
const MNOTE_CANON_TAG_MOVIE_INFO: MnoteCanonTag = 0x0011;
const MNOTE_OLYMPUS_TAG_FOCUSDIST: MnoteOlympusTag = 0x100c;
const MNOTE_OLYMPUS_TAG_MANFOCUS: MnoteOlympusTag = 0x100b;

struct ExifDataHandle(*mut ExifData);

impl ExifDataHandle {
    fn new(bytes: &[u8]) -> Self {
        let ptr = unsafe { exif_data_new_from_data(bytes.as_ptr(), bytes.len() as u32) };
        assert!(!ptr.is_null(), "failed to allocate ExifData");
        Self(ptr)
    }

    fn fresh() -> Self {
        let ptr = exif_data_new();
        assert!(!ptr.is_null(), "failed to allocate ExifData");
        Self(ptr)
    }

    fn as_ptr(&self) -> *mut ExifData {
        self.0
    }
}

impl Drop for ExifDataHandle {
    fn drop(&mut self) {
        unsafe { exif_data_unref(self.0) };
    }
}

struct ExifMnoteHandle(*mut ExifMnoteData);

impl ExifMnoteHandle {
    fn new_canon() -> (Self, *mut ExifMem) {
        let mem = exif_mem_new_default();
        assert!(!mem.is_null(), "failed to allocate ExifMem");
        let note = unsafe { exif_mnote_data_canon_new(mem, 0) };
        assert!(!note.is_null(), "failed to allocate Canon MakerNote");
        (Self(note), mem)
    }

    fn as_ptr(&self) -> *mut ExifMnoteData {
        self.0
    }
}

impl Drop for ExifMnoteHandle {
    fn drop(&mut self) {
        unsafe { exif_mnote_data_unref(self.0) };
    }
}

fn make_payload(tiff_len: usize) -> Vec<u8> {
    let mut payload = vec![0u8; 6 + tiff_len];
    payload[..6].copy_from_slice(b"Exif\0\0");

    let tiff = &mut payload[6..];
    tiff[..2].copy_from_slice(b"II");
    write_u16(tiff, 2, 0x002a);
    write_u32(tiff, 4, 8);
    payload
}

fn write_u16(buf: &mut [u8], offset: usize, value: u16) {
    buf[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn cyclic_ifd_payload() -> Vec<u8> {
    let mut payload = make_payload(26);
    let tiff = &mut payload[6..];

    write_u16(tiff, 8, 1);
    write_u16(tiff, 10, EXIF_TAG_EXIF_IFD_POINTER as u16);
    write_u16(tiff, 12, EXIF_FORMAT_LONG as u16);
    write_u32(tiff, 14, 1);
    write_u32(tiff, 18, 8);
    write_u32(tiff, 22, 0);

    payload
}

fn interoperability_budget_payload() -> Vec<u8> {
    let mut payload = make_payload(46);
    let tiff = &mut payload[6..];

    write_u16(tiff, 8, 1);
    write_u16(tiff, 10, EXIF_TAG_EXIF_IFD_POINTER as u16);
    write_u16(tiff, 12, EXIF_FORMAT_LONG as u16);
    write_u32(tiff, 14, 1);
    write_u32(tiff, 18, 26);
    write_u32(tiff, 22, 0);

    write_u16(tiff, 26, 1);
    write_u16(tiff, 28, EXIF_TAG_INTEROPERABILITY_IFD_POINTER as u16);
    write_u16(tiff, 30, EXIF_FORMAT_LONG as u16);
    write_u32(tiff, 32, 1);
    write_u32(tiff, 36, 44);
    write_u32(tiff, 40, 0);

    write_u16(tiff, 44, u16::MAX);
    payload
}

fn thumbnail_overflow_payload() -> Vec<u8> {
    let mut payload = make_payload(44);
    let tiff = &mut payload[6..];

    write_u16(tiff, 8, 0);
    write_u32(tiff, 10, 14);

    write_u16(tiff, 14, 2);
    write_u16(tiff, 16, EXIF_TAG_JPEG_INTERCHANGE_FORMAT as u16);
    write_u16(tiff, 18, EXIF_FORMAT_LONG as u16);
    write_u32(tiff, 20, 1);
    write_u32(tiff, 24, 0xffff_fff0);

    write_u16(tiff, 28, EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH as u16);
    write_u16(tiff, 30, EXIF_FORMAT_LONG as u16);
    write_u32(tiff, 32, 1);
    write_u32(tiff, 36, 64);
    write_u32(tiff, 40, 0);

    payload
}

fn entry_payload_overflow() -> Vec<u8> {
    let mut payload = make_payload(26);
    let tiff = &mut payload[6..];

    write_u16(tiff, 8, 1);
    write_u16(tiff, 10, EXIF_TAG_IMAGE_DESCRIPTION as u16);
    write_u16(tiff, 12, EXIF_FORMAT_ASCII as u16);
    write_u32(tiff, 14, 32);
    write_u32(tiff, 18, 0xffff_fff0);
    write_u32(tiff, 22, 0);

    payload
}

fn rational_bytes(numerator: u32, denominator: u32) -> [u8; 8] {
    let mut bytes = [0u8; 8];
    bytes[..4].copy_from_slice(&numerator.to_le_bytes());
    bytes[4..].copy_from_slice(&denominator.to_le_bytes());
    bytes
}

fn ifd_count(data: *mut ExifData, ifd: ExifIfd) -> u32 {
    unsafe {
        let content = (*data).ifd[ifd as usize];
        if content.is_null() {
            0
        } else {
            (*content).count
        }
    }
}

fn c_buf_to_string(buffer: &[c_char]) -> String {
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn c_ptr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

#[test]
fn cve_2007_6351_cyclic_ifd_is_rejected() {
    let payload = cyclic_ifd_payload();
    let data = ExifDataHandle::new(&payload);

    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_0), 0);
    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_EXIF), 0);
    assert!(unsafe { exif_data_get_mnote_data(data.as_ptr()) }.is_null());
}

#[test]
fn cve_2012_2837_olympus_zero_denominator_formats_without_dividing() {
    let mut bytes = rational_bytes(42, 0);
    let mut entry = MnoteOlympusEntry {
        tag: MNOTE_OLYMPUS_TAG_FOCUSDIST,
        format: EXIF_FORMAT_RATIONAL,
        components: 1,
        data: bytes.as_mut_ptr(),
        size: bytes.len() as u32,
        order: EXIF_BYTE_ORDER_INTEL,
    };
    let mut buffer = [0 as c_char; 32];

    let returned = unsafe {
        mnote_olympus_entry_get_value(&mut entry, buffer.as_mut_ptr(), buffer.len() as u32)
    };

    assert_eq!(returned, buffer.as_mut_ptr());
    assert_eq!(c_buf_to_string(&buffer), "Unknown");
}

#[test]
fn cve_2018_20030_interoperability_budget_is_bounded() {
    let payload = interoperability_budget_payload();
    let data = ExifDataHandle::new(&payload);

    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_EXIF), 0);
    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_INTEROPERABILITY), 0);
}

#[test]
fn cve_2020_0181_thumbnail_offset_overflow_is_rejected() {
    let payload = thumbnail_overflow_payload();
    let data = ExifDataHandle::new(&payload);

    assert!(unsafe { (*data.as_ptr()).data }.is_null());
    assert_eq!(unsafe { (*data.as_ptr()).size }, 0);
    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_1), 0);
}

#[test]
fn cve_2020_0198_entry_payload_overflow_is_rejected() {
    let payload = entry_payload_overflow();
    let data = ExifDataHandle::new(&payload);

    assert_eq!(ifd_count(data.as_ptr(), EXIF_IFD_0), 0);
}

#[test]
fn cve_2020_12767_entry_value_zero_denominator_falls_back_to_fraction() {
    let data = ExifDataHandle::fresh();
    let content = unsafe { (*data.as_ptr()).ifd[EXIF_IFD_EXIF as usize] };
    let entry = exif_entry_new();
    assert!(!content.is_null());
    assert!(!entry.is_null());

    unsafe {
        exif_data_set_byte_order(data.as_ptr(), EXIF_BYTE_ORDER_INTEL);
        exif_content_add_entry(content, entry);
        exif_entry_initialize(entry, EXIF_TAG_FNUMBER);
        exif_set_rational(
            (*entry).data,
            EXIF_BYTE_ORDER_INTEL,
            ExifRational {
                numerator: 1,
                denominator: 0,
            },
        );
    }

    let mut buffer = [0 as c_char; 32];
    let returned = unsafe { exif_entry_get_value(entry, buffer.as_mut_ptr(), buffer.len() as u32) };
    assert_eq!(returned, buffer.as_ptr());
    assert_eq!(c_buf_to_string(&buffer), "1/0");

    unsafe {
        exif_content_remove_entry(content, entry);
        exif_entry_unref(entry);
    }
}

#[test]
fn cve_2020_13114_canon_makernote_tag_limit_prevents_expansion_abuse() {
    let (note, mem) = ExifMnoteHandle::new_canon();
    let mut buffer = vec![0u8; 8];
    write_u16(&mut buffer, 6, 251);

    unsafe {
        exif_mnote_data_set_byte_order(note.as_ptr(), EXIF_BYTE_ORDER_INTEL);
        exif_mnote_data_set_offset(note.as_ptr(), 0);
        exif_mnote_data_load(note.as_ptr(), buffer.as_ptr(), buffer.len() as u32);
        assert_eq!(exif_mnote_data_count(note.as_ptr()), 0);
        exif_mem_unref(mem);
    }
}

#[test]
fn maker_note_canon_tag_metadata_matches_upstream_visible_tags() {
    assert_eq!(
        c_ptr_to_string(mnote_canon_tag_get_name(MNOTE_CANON_TAG_FILE_LENGTH)).as_deref(),
        Some("FileLength")
    );
    assert_eq!(
        c_ptr_to_string(mnote_canon_tag_get_title(MNOTE_CANON_TAG_FILE_LENGTH)).as_deref(),
        Some("File Length")
    );
    assert_eq!(
        c_ptr_to_string(mnote_canon_tag_get_description(MNOTE_CANON_TAG_FILE_LENGTH)).as_deref(),
        Some("")
    );
    assert_eq!(
        c_ptr_to_string(mnote_canon_tag_get_name(MNOTE_CANON_TAG_MOVIE_INFO)).as_deref(),
        Some("MovieInfo")
    );
    assert_eq!(
        c_ptr_to_string(mnote_canon_tag_get_title(MNOTE_CANON_TAG_MOVIE_INFO)).as_deref(),
        Some("Movie Info")
    );
}

#[test]
fn maker_note_olympus_tag_metadata_matches_upstream_visible_tags() {
    assert_eq!(
        c_ptr_to_string(mnote_olympus_tag_get_name(MNOTE_OLYMPUS_TAG_MANFOCUS)).as_deref(),
        Some("FocusMode")
    );
    assert_eq!(
        c_ptr_to_string(mnote_olympus_tag_get_title(MNOTE_OLYMPUS_TAG_MANFOCUS)).as_deref(),
        Some("Focus Mode")
    );
    assert_eq!(
        c_ptr_to_string(mnote_olympus_tag_get_description(
            MNOTE_OLYMPUS_TAG_MANFOCUS
        ))
        .as_deref(),
        Some("Automatic or manual focusing mode")
    );
}
