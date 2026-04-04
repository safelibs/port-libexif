use core::ffi::{c_char, c_uchar, c_uint};
use core::ptr;
use core::slice;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifData, ExifEntry, ExifIfd, ExifLong, ExifTag, EXIF_BYTE_ORDER_INTEL,
    EXIF_BYTE_ORDER_MOTOROLA, EXIF_IFD_0, EXIF_IFD_1, EXIF_IFD_COUNT, EXIF_IFD_EXIF,
    EXIF_IFD_GPS, EXIF_IFD_INTEROPERABILITY,
};
use crate::object::content::exif_content_add_entry_impl;
use crate::object::data::{
    exif_data_fix_impl, exif_data_get_mem_impl, exif_data_get_options_impl,
    exif_data_reset_impl, exif_data_set_byte_order_impl, exif_data_set_mnote_offset_impl,
};
use crate::object::entry::{exif_entry_new_mem_impl, exif_entry_unref_impl};
use crate::parser::limits::{
    checked_add, checked_mul, ifd_index, ParseBudget, ParseError, MAX_APP1_LENGTH,
    MAX_RECURSION_DEPTH,
};
use crate::parser::loader::{
    exif_loader_get_data_impl, exif_loader_new_impl, exif_loader_unref_impl,
    exif_loader_write_file_impl,
};
use crate::primitives::format::exif_format_get_size_impl;
use crate::runtime::mem::{exif_mem_alloc_impl, exif_mem_free_impl};
use crate::tables::tag_table::exif_tag_get_name_in_ifd;

const EXIF_HEADER: [u8; 6] = *b"Exif\0\0";
const JPEG_MARKER_SOI: u8 = 0xd8;
const JPEG_MARKER_APP1: u8 = 0xe1;

const EXIF_TAG_EXIF_IFD_POINTER: ExifTag = 0x8769;
const EXIF_TAG_GPS_INFO_IFD_POINTER: ExifTag = 0x8825;
const EXIF_TAG_INTEROPERABILITY_IFD_POINTER: ExifTag = 0xa005;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT: ExifTag = 0x0201;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: ExifTag = 0x0202;
const EXIF_TAG_MAKER_NOTE: ExifTag = 0x927c;

struct LoadContext {
    data: *mut ExifData,
    order: ExifByteOrder,
    budget: ParseBudget,
    loaded_ifds: [bool; EXIF_IFD_COUNT as usize],
}

impl LoadContext {
    fn new(data: *mut ExifData, order: ExifByteOrder, input_size: usize) -> Self {
        Self {
            data,
            order,
            budget: ParseBudget::new(input_size),
            loaded_ifds: [false; EXIF_IFD_COUNT as usize],
        }
    }
}

fn read_short(bytes: &[u8], offset: usize, order: ExifByteOrder) -> Result<u16, ParseError> {
    let end = checked_add(offset, 2, "EXIF short read overflow")?;
    let field = bytes
        .get(offset..end)
        .ok_or(ParseError::Corrupt("EXIF short read past end of buffer"))?;
    Ok(match order {
        EXIF_BYTE_ORDER_INTEL => u16::from_le_bytes([field[0], field[1]]),
        EXIF_BYTE_ORDER_MOTOROLA => u16::from_be_bytes([field[0], field[1]]),
        _ => return Err(ParseError::Corrupt("Unknown EXIF byte order")),
    })
}

fn read_long(bytes: &[u8], offset: usize, order: ExifByteOrder) -> Result<u32, ParseError> {
    let end = checked_add(offset, 4, "EXIF long read overflow")?;
    let field = bytes
        .get(offset..end)
        .ok_or(ParseError::Corrupt("EXIF long read past end of buffer"))?;
    Ok(match order {
        EXIF_BYTE_ORDER_INTEL => u32::from_le_bytes([field[0], field[1], field[2], field[3]]),
        EXIF_BYTE_ORDER_MOTOROLA => u32::from_be_bytes([field[0], field[1], field[2], field[3]]),
        _ => return Err(ParseError::Corrupt("Unknown EXIF byte order")),
    })
}

fn slice_range<'a>(
    bytes: &'a [u8],
    offset: usize,
    len: usize,
    context: &'static str,
) -> Result<&'a [u8], ParseError> {
    let end = checked_add(offset, len, context)?;
    bytes.get(offset..end).ok_or(ParseError::Corrupt(context))
}

fn resolve_exif_payload<'a>(source: &'a [u8]) -> Result<&'a [u8], ParseError> {
    if source.len() < EXIF_HEADER.len() {
        return Err(ParseError::Corrupt(
            "Size of data too small to allow for EXIF data",
        ));
    }

    if source.starts_with(&EXIF_HEADER) {
        return Ok(source);
    }

    let mut offset = 0usize;
    let mut remaining = source.len();
    while remaining >= 3 {
        while remaining != 0 && source[offset] == 0xff {
            offset += 1;
            remaining -= 1;
        }

        if remaining != 0 && source[offset] == JPEG_MARKER_SOI {
            offset += 1;
            remaining -= 1;
            continue;
        }

        if remaining != 0 && source[offset] == JPEG_MARKER_APP1 {
            if remaining >= 9 && source[offset + 3..offset + 9] == EXIF_HEADER {
                break;
            }
        }

        if remaining >= 3 && (0xe0..=0xef).contains(&source[offset]) {
            offset += 1;
            remaining -= 1;
            let segment_len = ((source[offset] as usize) << 8) | source[offset + 1] as usize;
            if segment_len > remaining {
                return Err(ParseError::Corrupt(
                    "APP marker length extends past end of buffer",
                ));
            }
            offset = checked_add(offset, segment_len, "APP marker offset overflow")?;
            remaining -= segment_len;
            continue;
        }

        return Err(ParseError::Corrupt("EXIF marker not found"));
    }

    if remaining < 3 {
        return Err(ParseError::Corrupt(
            "Size of data too small to allow for EXIF data",
        ));
    }

    offset += 1;
    remaining -= 1;
    let payload_len = ((source[offset] as usize) << 8) | source[offset + 1] as usize;
    if payload_len > remaining {
        return Err(ParseError::Corrupt(
            "Read length is longer than data length",
        ));
    }
    if payload_len < 2 {
        return Err(ParseError::Corrupt("APP1 marker is too short"));
    }

    offset += 2;
    slice_range(
        source,
        offset,
        payload_len - 2,
        "EXIF APP1 payload extends past end of buffer",
    )
}

fn load_thumbnail(
    ctx: &mut LoadContext,
    tiff: &[u8],
    offset: usize,
    len: usize,
) -> Result<(), ParseError> {
    let data = ctx.data;
    let thumbnail = slice_range(tiff, offset, len, "Thumbnail extends past end of buffer")?;
    ctx.budget
        .charge_work(len.saturating_div(16).saturating_add(1), "EXIF parse-work budget exhausted")?;

    unsafe {
        if !(*data).data.is_null() {
            exif_mem_free_impl(exif_data_get_mem_impl(data), (*data).data.cast());
            (*data).data = ptr::null_mut();
            (*data).size = 0;
        }
    }

    let dest = unsafe {
        exif_mem_alloc_impl(exif_data_get_mem_impl(data), len as ExifLong).cast::<c_uchar>()
    };
    if dest.is_null() {
        return Err(ParseError::ResourceLimit("Out of memory allocating thumbnail"));
    }

    unsafe {
        ptr::copy_nonoverlapping(thumbnail.as_ptr(), dest, len);
        (*data).data = dest;
        (*data).size = len as c_uint;
    }
    Ok(())
}

fn load_entry(
    ctx: &mut LoadContext,
    tiff: &[u8],
    entry_offset: usize,
) -> Result<Option<*mut ExifEntry>, ParseError> {
    let tag = read_short(tiff, entry_offset, ctx.order)? as ExifTag;
    let format = read_short(tiff, entry_offset + 2, ctx.order)? as i32;
    let components = read_long(tiff, entry_offset + 4, ctx.order)? as usize;
    let format_size = exif_format_get_size_impl(format) as usize;
    let data_size = checked_mul(format_size, components, "EXIF entry size overflow")?;
    if data_size == 0 {
        return Ok(None);
    }

    let data_offset = if data_size > 4 {
        read_long(tiff, entry_offset + 8, ctx.order)? as usize
    } else {
        entry_offset + 8
    };
    let raw = slice_range(tiff, data_offset, data_size, "EXIF entry extends past end of buffer")?;

    ctx.budget.charge_work(
        data_size.saturating_div(16).saturating_add(1),
        "EXIF parse-work budget exhausted",
    )?;

    let entry = unsafe { exif_entry_new_mem_impl(exif_data_get_mem_impl(ctx.data)) };
    if entry.is_null() {
        return Err(ParseError::ResourceLimit("Out of memory allocating EXIF entry"));
    }

    let entry_data = unsafe {
        exif_mem_alloc_impl(exif_data_get_mem_impl(ctx.data), data_size as ExifLong).cast::<c_uchar>()
    };
    if entry_data.is_null() {
        unsafe { exif_entry_unref_impl(entry) };
        return Err(ParseError::ResourceLimit(
            "Out of memory allocating EXIF entry payload",
        ));
    }

    unsafe {
        ptr::copy_nonoverlapping(raw.as_ptr(), entry_data, data_size);
        (*entry).tag = tag;
        (*entry).format = format;
        (*entry).components = components as _;
        (*entry).data = entry_data;
        (*entry).size = data_size as c_uint;
        if tag == EXIF_TAG_MAKER_NOTE {
            exif_data_set_mnote_offset_impl(ctx.data, data_offset as c_uint);
        }
    }

    Ok(Some(entry))
}

fn parse_ifd(
    ctx: &mut LoadContext,
    ifd: ExifIfd,
    tiff: &[u8],
    offset: usize,
    depth: usize,
) -> Result<(), ParseError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(ParseError::ResourceLimit("Deep EXIF recursion detected"));
    }

    let index = ifd_index(ifd)?;
    if ctx.loaded_ifds[index] {
        return Ok(());
    }

    ctx.loaded_ifds[index] = true;
    ctx.budget.record_offset(offset as u32)?;

    let declared_entries = read_short(tiff, offset, ctx.order)? as usize;
    ctx.budget.charge_ifd(declared_entries)?;

    let entries_offset = checked_add(offset, 2, "IFD entry table overflow")?;
    if entries_offset > tiff.len() {
        return Err(ParseError::Corrupt("IFD entry table starts past end of buffer"));
    }

    let available_bytes = tiff.len() - entries_offset;
    let available_entries = available_bytes / 12;
    let entry_count = declared_entries.min(available_entries);
    let content = unsafe { (*ctx.data).ifd[index] };

    let mut thumbnail_offset = None;
    let mut thumbnail_length = None;

    for entry_index in 0..entry_count {
        let entry_offset = checked_add(
            entries_offset,
            checked_mul(entry_index, 12, "IFD entry offset overflow")?,
            "IFD entry offset overflow",
        )?;
        let tag = read_short(tiff, entry_offset, ctx.order)? as ExifTag;

        match tag {
            EXIF_TAG_EXIF_IFD_POINTER => {
                let target = read_long(tiff, entry_offset + 8, ctx.order)? as usize;
                if ifd != EXIF_IFD_EXIF {
                    parse_ifd(ctx, EXIF_IFD_EXIF, tiff, target, depth + 1)?;
                }
            }
            EXIF_TAG_GPS_INFO_IFD_POINTER => {
                let target = read_long(tiff, entry_offset + 8, ctx.order)? as usize;
                if ifd != EXIF_IFD_GPS {
                    parse_ifd(ctx, EXIF_IFD_GPS, tiff, target, depth + 1)?;
                }
            }
            EXIF_TAG_INTEROPERABILITY_IFD_POINTER => {
                let target = read_long(tiff, entry_offset + 8, ctx.order)? as usize;
                if ifd != EXIF_IFD_INTEROPERABILITY {
                    parse_ifd(ctx, EXIF_IFD_INTEROPERABILITY, tiff, target, depth + 1)?;
                }
            }
            EXIF_TAG_JPEG_INTERCHANGE_FORMAT => {
                thumbnail_offset = Some(read_long(tiff, entry_offset + 8, ctx.order)? as usize);
                if let (Some(offset), Some(length)) = (thumbnail_offset, thumbnail_length) {
                    load_thumbnail(ctx, tiff, offset, length)?;
                }
            }
            EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH => {
                thumbnail_length = Some(read_long(tiff, entry_offset + 8, ctx.order)? as usize);
                if let (Some(offset), Some(length)) = (thumbnail_offset, thumbnail_length) {
                    load_thumbnail(ctx, tiff, offset, length)?;
                }
            }
            _ => {
                let recorded = unsafe { exif_tag_get_name_in_ifd(tag, ifd) };
                if recorded.is_null() {
                    let prefix = slice_range(
                        tiff,
                        entry_offset,
                        4,
                        "Unknown EXIF entry header extends past end of buffer",
                    )?;
                    if prefix == [0, 0, 0, 0] {
                        continue;
                    }
                    if (unsafe { exif_data_get_options_impl(ctx.data) }
                        & crate::ffi::types::EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS)
                        != 0
                    {
                        continue;
                    }
                }

                if let Some(entry) = load_entry(ctx, tiff, entry_offset)? {
                    unsafe {
                        exif_content_add_entry_impl(content, entry);
                        exif_entry_unref_impl(entry);
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_payload(data: *mut ExifData, payload: &[u8]) -> Result<(), ParseError> {
    if payload.len() < EXIF_HEADER.len() || !payload.starts_with(&EXIF_HEADER) {
        return Err(ParseError::Corrupt("EXIF header not found"));
    }
    if payload.len() < 14 {
        return Err(ParseError::Corrupt(
            "Size of data too small to allow for EXIF data",
        ));
    }

    let capped_len = payload.len().min(MAX_APP1_LENGTH);
    let payload = &payload[..capped_len];
    let order = match &payload[6..8] {
        b"II" => EXIF_BYTE_ORDER_INTEL,
        b"MM" => EXIF_BYTE_ORDER_MOTOROLA,
        _ => return Err(ParseError::Corrupt("Unknown EXIF byte order")),
    };

    if read_short(payload, 8, order)? != 0x002a {
        return Err(ParseError::Corrupt("Invalid EXIF TIFF marker"));
    }

    unsafe {
        exif_data_set_byte_order_impl(data, order);
    }

    let ifd0_offset = read_long(payload, 10, order)? as usize;
    let tiff = &payload[6..];
    let mut ctx = LoadContext::new(data, order, payload.len());

    let ifd0_min_end = checked_add(ifd0_offset, 2, "IFD0 offset overflow")?;
    if ifd0_min_end > tiff.len() {
        return Err(ParseError::Corrupt("IFD0 starts past end of buffer"));
    }

    parse_ifd(&mut ctx, EXIF_IFD_0, tiff, ifd0_offset, 0)?;

    let entry_count = read_short(tiff, ifd0_offset, order)? as usize;
    let next_ifd_pos = checked_add(
        checked_add(ifd0_offset, 2, "IFD0 next-pointer overflow")?,
        checked_mul(entry_count, 12, "IFD0 next-pointer overflow")?,
        "IFD0 next-pointer overflow",
    )?;
    let next_ifd_end = checked_add(next_ifd_pos, 4, "IFD0 next-pointer overflow")?;
    if next_ifd_end <= tiff.len() {
        let ifd1_offset = read_long(tiff, next_ifd_pos, order)? as usize;
        if ifd1_offset != 0 {
            parse_ifd(&mut ctx, EXIF_IFD_1, tiff, ifd1_offset, 0)?;
        }
    }

    unsafe {
        crate::mnote::interpret_maker_note_impl(data, payload.as_ptr(), payload.len());
    }

    if (unsafe { exif_data_get_options_impl(data) }
        & crate::ffi::types::EXIF_DATA_OPTION_FOLLOW_SPECIFICATION)
        != 0
    {
        unsafe { exif_data_fix_impl(data) };
    }

    Ok(())
}

pub(crate) unsafe fn exif_data_load_data_impl(
    data: *mut ExifData,
    source: *const c_uchar,
    size: c_uint,
) {
    if data.is_null() || source.is_null() || size == 0 {
        return;
    }

    let source = unsafe { slice::from_raw_parts(source, size as usize) };
    unsafe { exif_data_reset_impl(data) };

    let payload = match resolve_exif_payload(source) {
        Ok(payload) => payload,
        Err(_) => return,
    };
    let _ = parse_payload(data, payload);
}

pub(crate) unsafe fn exif_data_new_from_file_impl(path: *const c_char) -> *mut ExifData {
    if path.is_null() {
        return ptr::null_mut();
    }

    let loader = unsafe { exif_loader_new_impl() };
    if loader.is_null() {
        return ptr::null_mut();
    }

    unsafe { exif_loader_write_file_impl(loader, path) };
    let data = unsafe { exif_loader_get_data_impl(loader) };
    unsafe { exif_loader_unref_impl(loader) };
    data
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_load_data(
    data: *mut ExifData,
    source: *const c_uchar,
    size: c_uint,
) {
    panic_boundary::call_void(|| unsafe { exif_data_load_data_impl(data, source, size) });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_new_from_file(path: *const c_char) -> *mut ExifData {
    panic_boundary::call_or(ptr::null_mut(), || unsafe { exif_data_new_from_file_impl(path) })
}
