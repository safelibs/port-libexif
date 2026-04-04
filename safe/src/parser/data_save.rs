use core::ffi::{c_uchar, c_uint};
use core::ptr;
use core::slice;

use crate::ffi::panic_boundary;
use crate::ffi::types::{
    ExifByteOrder, ExifContent, ExifData, ExifEntry, ExifIfd, ExifLong, ExifTag,
    EXIF_BYTE_ORDER_INTEL, EXIF_FORMAT_LONG, EXIF_IFD_0, EXIF_IFD_1, EXIF_IFD_EXIF,
    EXIF_IFD_GPS, EXIF_IFD_INTEROPERABILITY,
};
use crate::object::data::{exif_data_get_byte_order_impl, exif_data_get_mem_impl};
use crate::parser::limits::{checked_add, checked_mul, ParseError, MAX_SERIALIZED_BYTES};
use crate::primitives::format::exif_format_get_size_impl;
use crate::runtime::mem::exif_mem_alloc_impl;

const EXIF_HEADER: [u8; 6] = *b"Exif\0\0";

const EXIF_TAG_EXIF_IFD_POINTER: ExifTag = 0x8769;
const EXIF_TAG_GPS_INFO_IFD_POINTER: ExifTag = 0x8825;
const EXIF_TAG_INTEROPERABILITY_IFD_POINTER: ExifTag = 0xa005;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT: ExifTag = 0x0201;
const EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH: ExifTag = 0x0202;

fn write_short(buffer: &mut [u8], offset: usize, order: ExifByteOrder, value: u16) {
    let bytes = match order {
        EXIF_BYTE_ORDER_INTEL => value.to_le_bytes(),
        _ => value.to_be_bytes(),
    };
    buffer[offset..offset + 2].copy_from_slice(&bytes);
}

fn write_long(buffer: &mut [u8], offset: usize, order: ExifByteOrder, value: u32) {
    let bytes = match order {
        EXIF_BYTE_ORDER_INTEL => value.to_le_bytes(),
        _ => value.to_be_bytes(),
    };
    buffer[offset..offset + 4].copy_from_slice(&bytes);
}

fn ensure_len(buffer: &mut Vec<u8>, len: usize) -> Result<(), ParseError> {
    if len > MAX_SERIALIZED_BYTES {
        return Err(ParseError::Overflow("Serialized EXIF buffer exceeds size limit"));
    }
    if buffer.len() < len {
        buffer.resize(len, 0);
    }
    Ok(())
}

fn current_offset(buffer: &[u8]) -> Result<usize, ParseError> {
    buffer
        .len()
        .checked_sub(EXIF_HEADER.len())
        .ok_or(ParseError::Overflow("Serialized EXIF offset underflow"))
}

fn append_bytes(buffer: &mut Vec<u8>, bytes: &[u8], align_even: bool) -> Result<u32, ParseError> {
    let offset = current_offset(buffer)? as u32;
    let new_len = checked_add(buffer.len(), bytes.len(), "Serialized EXIF buffer overflow")?;
    ensure_len(buffer, new_len)?;
    let start = new_len - bytes.len();
    buffer[start..new_len].copy_from_slice(bytes);
    if align_even && (bytes.len() & 1) != 0 {
        let padded = checked_add(buffer.len(), 1, "Serialized EXIF padding overflow")?;
        ensure_len(buffer, padded)?;
    }
    Ok(offset)
}

fn sort_directory_records(
    buffer: &mut [u8],
    entries_offset: usize,
    entry_count: usize,
    order: ExifByteOrder,
) {
    let mut records = Vec::with_capacity(entry_count);
    for index in 0..entry_count {
        let start = entries_offset + index * 12;
        let mut record = [0u8; 12];
        record.copy_from_slice(&buffer[start..start + 12]);
        records.push(record);
    }

    records.sort_by(|left, right| {
        let lhs = match order {
            EXIF_BYTE_ORDER_INTEL => u16::from_le_bytes([left[0], left[1]]),
            _ => u16::from_be_bytes([left[0], left[1]]),
        };
        let rhs = match order {
            EXIF_BYTE_ORDER_INTEL => u16::from_le_bytes([right[0], right[1]]),
            _ => u16::from_be_bytes([right[0], right[1]]),
        };
        lhs.cmp(&rhs)
    });

    for (index, record) in records.iter().enumerate() {
        let start = entries_offset + index * 12;
        buffer[start..start + 12].copy_from_slice(record);
    }
}

fn visible_entries(content: *mut ExifContent) -> Vec<*mut ExifEntry> {
    if content.is_null() {
        return Vec::new();
    }

    let count = unsafe { (*content).count as usize };
    let entries = unsafe { (*content).entries };
    if count == 0 || entries.is_null() {
        return Vec::new();
    }

    let mut visible = Vec::with_capacity(count);
    for index in 0..count {
        let entry = unsafe { *entries.add(index) };
        if !entry.is_null() {
            visible.push(entry);
        }
    }
    visible
}

fn write_raw_entry(
    order: ExifByteOrder,
    buffer: &mut Vec<u8>,
    record_offset: usize,
    entry: *mut ExifEntry,
) -> Result<(), ParseError> {
    let components = usize::try_from(unsafe { (*entry).components })
        .map_err(|_| ParseError::Overflow("EXIF component count does not fit usize"))?;
    let format = unsafe { (*entry).format };
    let format_size = exif_format_get_size_impl(format) as usize;
    let data_size = checked_mul(format_size, components, "EXIF entry size overflow while saving")?;

    write_short(
        buffer,
        record_offset,
        order,
        unsafe { (*entry).tag as u16 },
    );
    write_short(buffer, record_offset + 2, order, format as u16);
    write_long(buffer, record_offset + 4, order, components as u32);

    if data_size > 4 {
        let payload = if unsafe { (*entry).data }.is_null() || unsafe { (*entry).size } == 0 {
            vec![0u8; data_size]
        } else {
            let available = unsafe { (*entry).size as usize }.min(data_size);
            let mut payload = vec![0u8; data_size];
            unsafe {
                let source = slice::from_raw_parts((*entry).data.cast_const(), available);
                payload[..available].copy_from_slice(source);
            }
            payload
        };
        let data_offset = append_bytes(buffer, &payload, true)?;
        write_long(buffer, record_offset + 8, order, data_offset);
    } else {
        buffer[record_offset + 8..record_offset + 12].fill(0);
        if !unsafe { (*entry).data }.is_null() && unsafe { (*entry).size } != 0 {
            let available = unsafe { (*entry).size as usize }.min(data_size);
            unsafe {
                let source = slice::from_raw_parts((*entry).data.cast_const(), available);
                buffer[record_offset + 8..record_offset + 8 + available].copy_from_slice(source);
            }
        }
    }

    Ok(())
}

fn write_pointer_entry(
    order: ExifByteOrder,
    buffer: &mut [u8],
    record_offset: usize,
    tag: ExifTag,
    value: u32,
) {
    write_short(buffer, record_offset, order, tag as u16);
    write_short(buffer, record_offset + 2, order, EXIF_FORMAT_LONG as u16);
    write_long(buffer, record_offset + 4, order, 1);
    write_long(buffer, record_offset + 8, order, value);
}

fn save_ifd(
    data: *mut ExifData,
    ifd_index: ExifIfd,
    buffer: &mut Vec<u8>,
    offset: usize,
) -> Result<(), ParseError> {
    let order = unsafe { exif_data_get_byte_order_impl(data) };
    let content = unsafe { (*data).ifd[ifd_index as usize] };
    let entries = visible_entries(content);

    let mut pointer_count = 0usize;
    let mut thumbnail_count = 0usize;
    match ifd_index {
        EXIF_IFD_0 => {
            if unsafe { (*(*data).ifd[EXIF_IFD_EXIF as usize]).count } != 0
                || unsafe { (*(*data).ifd[EXIF_IFD_INTEROPERABILITY as usize]).count } != 0
            {
                pointer_count += 1;
            }
            if unsafe { (*(*data).ifd[EXIF_IFD_GPS as usize]).count } != 0 {
                pointer_count += 1;
            }
        }
        EXIF_IFD_EXIF => {
            if unsafe { (*(*data).ifd[EXIF_IFD_INTEROPERABILITY as usize]).count } != 0 {
                pointer_count += 1;
            }
        }
        EXIF_IFD_1 => {
            if unsafe { (*data).size } != 0 {
                thumbnail_count = 2;
            }
        }
        _ => {}
    }

    let entry_count = entries.len() + pointer_count + thumbnail_count;
    if entry_count > u16::MAX as usize {
        return Err(ParseError::Overflow("Too many EXIF entries to serialize"));
    }

    let directory_size = checked_add(
        2,
        checked_add(
            checked_mul(entry_count, 12, "EXIF directory size overflow")?,
            4,
            "EXIF directory size overflow",
        )?,
        "EXIF directory size overflow",
    )?;

    let directory_offset = checked_add(EXIF_HEADER.len(), offset, "EXIF directory offset overflow")?;
    ensure_len(buffer, checked_add(directory_offset, directory_size, "EXIF directory overflow")?)?;
    write_short(buffer, directory_offset, order, entry_count as u16);

    let entries_offset = directory_offset + 2;
    let mut cursor = 0usize;
    for entry in entries {
        write_raw_entry(order, buffer, entries_offset + cursor * 12, entry)?;
        cursor += 1;
    }

    match ifd_index {
        EXIF_IFD_0 => {
            let exif_content = unsafe { (*data).ifd[EXIF_IFD_EXIF as usize] };
            let interoperability_content = unsafe { (*data).ifd[EXIF_IFD_INTEROPERABILITY as usize] };
            if unsafe { (*exif_content).count } != 0 || unsafe { (*interoperability_content).count } != 0 {
                let target = current_offset(buffer)? as u32;
                write_pointer_entry(
                    order,
                    buffer,
                    entries_offset + cursor * 12,
                    EXIF_TAG_EXIF_IFD_POINTER,
                    target,
                );
                save_ifd(data, EXIF_IFD_EXIF, buffer, target as usize)?;
                cursor += 1;
            }

            let gps_content = unsafe { (*data).ifd[EXIF_IFD_GPS as usize] };
            if unsafe { (*gps_content).count } != 0 {
                let target = current_offset(buffer)? as u32;
                write_pointer_entry(
                    order,
                    buffer,
                    entries_offset + cursor * 12,
                    EXIF_TAG_GPS_INFO_IFD_POINTER,
                    target,
                );
                save_ifd(data, EXIF_IFD_GPS, buffer, target as usize)?;
            }
        }
        EXIF_IFD_EXIF => {
            let interoperability_content = unsafe { (*data).ifd[EXIF_IFD_INTEROPERABILITY as usize] };
            if unsafe { (*interoperability_content).count } != 0 {
                let target = current_offset(buffer)? as u32;
                write_pointer_entry(
                    order,
                    buffer,
                    entries_offset + cursor * 12,
                    EXIF_TAG_INTEROPERABILITY_IFD_POINTER,
                    target,
                );
                save_ifd(data, EXIF_IFD_INTEROPERABILITY, buffer, target as usize)?;
            }
        }
        EXIF_IFD_1 => {
            if unsafe { (*data).size } != 0 && !unsafe { (*data).data }.is_null() {
                let thumbnail = unsafe {
                    slice::from_raw_parts((*data).data.cast_const(), (*data).size as usize)
                };
                let thumbnail_offset = append_bytes(buffer, thumbnail, false)?;
                write_pointer_entry(
                    order,
                    buffer,
                    entries_offset + cursor * 12,
                    EXIF_TAG_JPEG_INTERCHANGE_FORMAT,
                    thumbnail_offset,
                );
                cursor += 1;
                write_pointer_entry(
                    order,
                    buffer,
                    entries_offset + cursor * 12,
                    EXIF_TAG_JPEG_INTERCHANGE_FORMAT_LENGTH,
                    unsafe { (*data).size },
                );
            }
        }
        _ => {}
    }

    sort_directory_records(buffer, entries_offset, entry_count, order);

    let next_offset_pos = entries_offset + entry_count * 12;
    if ifd_index == EXIF_IFD_0
        && (unsafe { (*(*data).ifd[EXIF_IFD_1 as usize]).count } != 0 || unsafe { (*data).size } != 0)
    {
        let target = current_offset(buffer)? as u32;
        write_long(buffer, next_offset_pos, order, target);
        save_ifd(data, EXIF_IFD_1, buffer, target as usize)?;
    } else {
        write_long(buffer, next_offset_pos, order, 0);
    }

    Ok(())
}

fn serialize_exif(data: *mut ExifData) -> Result<Vec<u8>, ParseError> {
    let order = unsafe { exif_data_get_byte_order_impl(data) };
    let mut buffer = vec![0u8; 14];
    buffer[..6].copy_from_slice(&EXIF_HEADER);
    buffer[6..8].copy_from_slice(match order {
        EXIF_BYTE_ORDER_INTEL => b"II",
        _ => b"MM",
    });
    write_short(&mut buffer, 8, order, 0x002a);
    write_long(&mut buffer, 10, order, 8);
    save_ifd(data, EXIF_IFD_0, &mut buffer, 8)?;
    Ok(buffer)
}

pub(crate) unsafe fn exif_data_save_data_impl(
    data: *mut ExifData,
    out_buffer: *mut *mut c_uchar,
    out_size: *mut c_uint,
) {
    if !out_buffer.is_null() {
        unsafe { *out_buffer = ptr::null_mut() };
    }
    if !out_size.is_null() {
        unsafe { *out_size = 0 };
    }
    if data.is_null() || out_buffer.is_null() || out_size.is_null() {
        return;
    }

    let serialized = match serialize_exif(data) {
        Ok(serialized) => serialized,
        Err(_) => return,
    };

    let dest = unsafe {
        exif_mem_alloc_impl(exif_data_get_mem_impl(data), serialized.len() as ExifLong).cast::<c_uchar>()
    };
    if dest.is_null() {
        return;
    }

    unsafe {
        ptr::copy_nonoverlapping(serialized.as_ptr(), dest, serialized.len());
        *out_buffer = dest;
        *out_size = serialized.len() as c_uint;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn exif_data_save_data(
    data: *mut ExifData,
    out_buffer: *mut *mut c_uchar,
    out_size: *mut c_uint,
) {
    panic_boundary::call_void(|| unsafe { exif_data_save_data_impl(data, out_buffer, out_size) });
}
