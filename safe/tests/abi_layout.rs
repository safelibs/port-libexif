use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::mem::{align_of, size_of, MaybeUninit};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use exif::ffi::types::*;

macro_rules! rust_offset {
    ($ty:ty, $($field:tt)+) => {{
        let uninit = MaybeUninit::<$ty>::uninit();
        let base = uninit.as_ptr();
        unsafe { (std::ptr::addr_of!((*base).$($field)+) as usize) - (base as usize) }
    }};
}

macro_rules! insert_size {
    ($map:ident, $ty:ty, $label:expr) => {{
        $map.insert($label.to_owned(), (size_of::<$ty>(), align_of::<$ty>()));
    }};
}

macro_rules! insert_offset {
    ($map:ident, $ty:ty, $label:expr, $($field:tt)+) => {{
        $map.insert(format!("{}|{}", stringify!($ty), $label), rust_offset!($ty, $($field)+));
    }};
}

#[test]
fn abi_layout_matches_headers() {
    let probe_output = run_probe();
    let (c_sizes, c_offsets) = parse_probe(&probe_output);

    assert_eq!(c_sizes, rust_sizes());
    assert_eq!(c_offsets, rust_offsets());
}

fn run_probe() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let include_dir = manifest_dir.join("include");
    let support_dir = manifest_dir.join("tests").join("support");
    let original_dir = manifest_dir
        .parent()
        .expect("safe crate should live below the project root")
        .join("original");
    let temp_root = temp_dir("abi-layout");
    let source = temp_root.join("abi-layout.c");
    let binary = temp_root.join("abi-layout");

    fs::write(&source, probe_source()).expect("failed to write ABI probe source");

    let compiler = match env::var("CC") {
        Ok(value) if !value.is_empty() => value,
        _ => String::from("cc"),
    };

    let compile_output = Command::new(&compiler)
        .arg("-std=c11")
        .arg("-I")
        .arg(&include_dir)
        .arg("-I")
        .arg(&support_dir)
        .arg("-I")
        .arg(&original_dir)
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("failed to run C compiler");
    assert!(
        compile_output.status.success(),
        "C ABI probe compilation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let run_output = Command::new(&binary)
        .output()
        .expect("failed to execute ABI probe");
    assert!(
        run_output.status.success(),
        "ABI probe failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );

    String::from_utf8(run_output.stdout).expect("ABI probe emitted invalid UTF-8")
}

fn probe_source() -> &'static str {
    r#"
#include <stddef.h>
#include <stdio.h>

#include <libexif/exif-byte-order.h>
#include <libexif/exif-content.h>
#include <libexif/exif-data-type.h>
#include <libexif/exif-data.h>
#include <libexif/exif-entry.h>
#include <libexif/exif-format.h>
#include <libexif/exif-ifd.h>
#include <libexif/exif-log.h>
#include <libexif/exif-mem.h>
#include <libexif/exif-mnote-data.h>
#include <libexif/exif-mnote-data-priv.h>
#include <libexif/exif-tag.h>
#include <libexif/exif-utils.h>

#define EMIT_SIZE(name, ty) printf("SIZE|%s|%zu|%zu\n", name, sizeof(ty), _Alignof(ty))
#define EMIT_OFFSET(name, ty, field) printf("OFFSET|%s|%s|%zu\n", name, #field, offsetof(ty, field))

int main(void) {
    EMIT_SIZE("ExifByteOrder", ExifByteOrder);
    EMIT_SIZE("ExifDataType", ExifDataType);
    EMIT_SIZE("ExifFormat", ExifFormat);
    EMIT_SIZE("ExifIfd", ExifIfd);
    EMIT_SIZE("ExifTag", ExifTag);
    EMIT_SIZE("ExifSupportLevel", ExifSupportLevel);
    EMIT_SIZE("ExifDataOption", ExifDataOption);
    EMIT_SIZE("ExifLogCode", ExifLogCode);
    EMIT_SIZE("ExifByte", ExifByte);
    EMIT_SIZE("ExifSByte", ExifSByte);
    EMIT_SIZE("ExifAscii", ExifAscii);
    EMIT_SIZE("ExifShort", ExifShort);
    EMIT_SIZE("ExifSShort", ExifSShort);
    EMIT_SIZE("ExifLong", ExifLong);
    EMIT_SIZE("ExifSLong", ExifSLong);
    EMIT_SIZE("ExifUndefined", ExifUndefined);
    EMIT_SIZE("ExifRational", ExifRational);
    EMIT_SIZE("ExifSRational", ExifSRational);
    EMIT_SIZE("ExifContent", ExifContent);
    EMIT_SIZE("ExifData", ExifData);
    EMIT_SIZE("ExifEntry", ExifEntry);
    EMIT_SIZE("ExifMnoteDataMethods", ExifMnoteDataMethods);
    EMIT_SIZE("ExifMnoteData", ExifMnoteData);

    EMIT_OFFSET("ExifContent", struct _ExifContent, entries);
    EMIT_OFFSET("ExifContent", struct _ExifContent, count);
    EMIT_OFFSET("ExifContent", struct _ExifContent, parent);
    EMIT_OFFSET("ExifContent", struct _ExifContent, priv);

    EMIT_OFFSET("ExifData", struct _ExifData, ifd);
    EMIT_OFFSET("ExifData", struct _ExifData, data);
    EMIT_OFFSET("ExifData", struct _ExifData, size);
    EMIT_OFFSET("ExifData", struct _ExifData, priv);

    EMIT_OFFSET("ExifEntry", struct _ExifEntry, tag);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, format);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, components);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, data);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, size);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, parent);
    EMIT_OFFSET("ExifEntry", struct _ExifEntry, priv);

    EMIT_OFFSET("ExifRational", ExifRational, numerator);
    EMIT_OFFSET("ExifRational", ExifRational, denominator);
    EMIT_OFFSET("ExifSRational", ExifSRational, numerator);
    EMIT_OFFSET("ExifSRational", ExifSRational, denominator);

    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, free);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, save);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, load);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, set_offset);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, set_byte_order);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, count);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, get_id);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, get_name);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, get_title);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, get_description);
    EMIT_OFFSET("ExifMnoteDataMethods", struct _ExifMnoteDataMethods, get_value);

    EMIT_OFFSET("ExifMnoteData", struct _ExifMnoteData, priv);
    EMIT_OFFSET("ExifMnoteData", struct _ExifMnoteData, methods);
    EMIT_OFFSET("ExifMnoteData", struct _ExifMnoteData, log);
    EMIT_OFFSET("ExifMnoteData", struct _ExifMnoteData, mem);

    return 0;
}
"#
}

fn temp_dir(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_nanos();
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-artifacts")
        .join(format!("{prefix}-{nonce}"));
    fs::create_dir_all(&dir).expect("failed to create temp directory");
    dir
}

fn parse_probe(output: &str) -> (BTreeMap<String, (usize, usize)>, BTreeMap<String, usize>) {
    let mut sizes = BTreeMap::new();
    let mut offsets = BTreeMap::new();

    for line in output.lines() {
        let parts: Vec<_> = line.split('|').collect();
        match parts.as_slice() {
            ["SIZE", name, size, align] => {
                sizes.insert(
                    (*name).to_owned(),
                    (
                        size.parse::<usize>().expect("invalid size entry"),
                        align.parse::<usize>().expect("invalid align entry"),
                    ),
                );
            }
            ["OFFSET", ty, field, offset] => {
                offsets.insert(
                    format!("{ty}|{field}"),
                    offset.parse::<usize>().expect("invalid offset entry"),
                );
            }
            _ => panic!("unexpected ABI probe output line: {line}"),
        }
    }

    (sizes, offsets)
}

fn rust_sizes() -> BTreeMap<String, (usize, usize)> {
    let mut sizes = BTreeMap::new();

    insert_size!(sizes, ExifByteOrder, "ExifByteOrder");
    insert_size!(sizes, ExifDataType, "ExifDataType");
    insert_size!(sizes, ExifFormat, "ExifFormat");
    insert_size!(sizes, ExifIfd, "ExifIfd");
    insert_size!(sizes, ExifTag, "ExifTag");
    insert_size!(sizes, ExifSupportLevel, "ExifSupportLevel");
    insert_size!(sizes, ExifDataOption, "ExifDataOption");
    insert_size!(sizes, ExifLogCode, "ExifLogCode");
    insert_size!(sizes, ExifByte, "ExifByte");
    insert_size!(sizes, ExifSByte, "ExifSByte");
    insert_size!(sizes, ExifAscii, "ExifAscii");
    insert_size!(sizes, ExifShort, "ExifShort");
    insert_size!(sizes, ExifSShort, "ExifSShort");
    insert_size!(sizes, ExifLong, "ExifLong");
    insert_size!(sizes, ExifSLong, "ExifSLong");
    insert_size!(sizes, ExifUndefined, "ExifUndefined");
    insert_size!(sizes, ExifRational, "ExifRational");
    insert_size!(sizes, ExifSRational, "ExifSRational");
    insert_size!(sizes, ExifContent, "ExifContent");
    insert_size!(sizes, ExifData, "ExifData");
    insert_size!(sizes, ExifEntry, "ExifEntry");
    insert_size!(sizes, ExifMnoteDataMethods, "ExifMnoteDataMethods");
    insert_size!(sizes, ExifMnoteData, "ExifMnoteData");

    sizes
}

fn rust_offsets() -> BTreeMap<String, usize> {
    let mut offsets = BTreeMap::new();

    insert_offset!(offsets, ExifContent, "entries", entries);
    insert_offset!(offsets, ExifContent, "count", count);
    insert_offset!(offsets, ExifContent, "parent", parent);
    insert_offset!(offsets, ExifContent, "priv", priv_);

    insert_offset!(offsets, ExifData, "ifd", ifd);
    insert_offset!(offsets, ExifData, "data", data);
    insert_offset!(offsets, ExifData, "size", size);
    insert_offset!(offsets, ExifData, "priv", priv_);

    insert_offset!(offsets, ExifEntry, "tag", tag);
    insert_offset!(offsets, ExifEntry, "format", format);
    insert_offset!(offsets, ExifEntry, "components", components);
    insert_offset!(offsets, ExifEntry, "data", data);
    insert_offset!(offsets, ExifEntry, "size", size);
    insert_offset!(offsets, ExifEntry, "parent", parent);
    insert_offset!(offsets, ExifEntry, "priv", priv_);

    insert_offset!(offsets, ExifRational, "numerator", numerator);
    insert_offset!(offsets, ExifRational, "denominator", denominator);
    insert_offset!(offsets, ExifSRational, "numerator", numerator);
    insert_offset!(offsets, ExifSRational, "denominator", denominator);

    insert_offset!(offsets, ExifMnoteDataMethods, "free", free);
    insert_offset!(offsets, ExifMnoteDataMethods, "save", save);
    insert_offset!(offsets, ExifMnoteDataMethods, "load", load);
    insert_offset!(offsets, ExifMnoteDataMethods, "set_offset", set_offset);
    insert_offset!(
        offsets,
        ExifMnoteDataMethods,
        "set_byte_order",
        set_byte_order
    );
    insert_offset!(offsets, ExifMnoteDataMethods, "count", count);
    insert_offset!(offsets, ExifMnoteDataMethods, "get_id", get_id);
    insert_offset!(offsets, ExifMnoteDataMethods, "get_name", get_name);
    insert_offset!(offsets, ExifMnoteDataMethods, "get_title", get_title);
    insert_offset!(
        offsets,
        ExifMnoteDataMethods,
        "get_description",
        get_description
    );
    insert_offset!(offsets, ExifMnoteDataMethods, "get_value", get_value);

    insert_offset!(offsets, ExifMnoteData, "priv", priv_);
    insert_offset!(offsets, ExifMnoteData, "methods", methods);
    insert_offset!(offsets, ExifMnoteData, "log", log);
    insert_offset!(offsets, ExifMnoteData, "mem", mem);

    offsets
}
