use std::env;
use std::ffi::CStr;
use std::fs;
use std::os::raw::{c_char, c_void};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use exif::ffi::types::*;

unsafe extern "C" {
    safe fn exif_data_option_get_name(option: ExifDataOption) -> *const c_char;
    safe fn exif_data_option_get_description(option: ExifDataOption) -> *const c_char;
    fn exif_mem_new(
        alloc_func: ExifMemAllocFunc,
        realloc_func: ExifMemReallocFunc,
        free_func: ExifMemFreeFunc,
    ) -> *mut ExifMem;
    fn exif_mem_ref(mem: *mut ExifMem);
    fn exif_mem_unref(mem: *mut ExifMem);
    fn exif_mem_alloc(mem: *mut ExifMem, size: ExifLong) -> *mut c_void;
    fn exif_mem_realloc(mem: *mut ExifMem, ptr_: *mut c_void, size: ExifLong) -> *mut c_void;
    fn exif_mem_free(mem: *mut ExifMem, ptr_: *mut c_void);
    fn exif_log_new_mem(mem: *mut ExifMem) -> *mut ExifLog;
    fn exif_log_ref(log: *mut ExifLog);
    fn exif_log_unref(log: *mut ExifLog);
    fn exif_log_free(log: *mut ExifLog);
    fn exif_log_set_func(log: *mut ExifLog, func: ExifLogFunc, data: *mut c_void);
    safe fn exif_log_code_get_title(code: ExifLogCode) -> *const c_char;
    safe fn exif_log_code_get_message(code: ExifLogCode) -> *const c_char;
    fn calloc(nmemb: usize, size: usize) -> *mut c_void;
    fn realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static REALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static FREE_CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn counting_alloc(size: ExifLong) -> *mut c_void {
    ALLOC_CALLS.fetch_add(1, Ordering::SeqCst);
    unsafe { calloc(size as usize, 1) }
}

unsafe extern "C" fn counting_realloc(ptr_: *mut c_void, size: ExifLong) -> *mut c_void {
    REALLOC_CALLS.fetch_add(1, Ordering::SeqCst);
    unsafe { realloc(ptr_, size as usize) }
}

unsafe extern "C" fn counting_free(ptr_: *mut c_void) {
    FREE_CALLS.fetch_add(1, Ordering::SeqCst);
    unsafe { free(ptr_) };
}

#[test]
fn exif_data_option_strings_match_original_c_locale() {
    assert_eq!(
        c_string(exif_data_option_get_name(
            EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS
        )),
        Some("Ignore unknown tags".to_owned())
    );
    assert_eq!(
        c_string(exif_data_option_get_description(
            EXIF_DATA_OPTION_IGNORE_UNKNOWN_TAGS
        )),
        Some("Ignore unknown tags when loading EXIF data.".to_owned())
    );
    assert_eq!(
        c_string(exif_data_option_get_name(
            EXIF_DATA_OPTION_FOLLOW_SPECIFICATION
        )),
        Some("Follow specification".to_owned())
    );
    assert_eq!(
        c_string(exif_data_option_get_description(
            EXIF_DATA_OPTION_FOLLOW_SPECIFICATION
        )),
        Some(
            "Add, correct and remove entries to get EXIF data that follows the specification."
                .to_owned(),
        )
    );
    assert_eq!(
        c_string(exif_data_option_get_name(
            EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE
        )),
        Some("Do not change maker note".to_owned())
    );
    assert_eq!(
        c_string(exif_data_option_get_description(
            EXIF_DATA_OPTION_DONT_CHANGE_MAKER_NOTE,
        )),
        Some(
            "When loading and resaving Exif data, save the maker note unmodified. Be aware that the maker note can get corrupted.".to_owned(),
        )
    );
}

#[test]
fn unknown_data_option_returns_null() {
    assert!(exif_data_option_get_name(0).is_null());
    assert!(exif_data_option_get_description(0).is_null());
    assert!(exif_data_option_get_name(12345).is_null());
    assert!(exif_data_option_get_description(12345).is_null());
}

#[test]
fn log_mem_nulls() {
    assert!(unsafe { exif_mem_new(None, None, None) }.is_null());
    assert!(unsafe { exif_mem_alloc(std::ptr::null_mut(), 8) }.is_null());
    assert!(unsafe { exif_mem_realloc(std::ptr::null_mut(), std::ptr::null_mut(), 8) }.is_null());
    unsafe {
        exif_mem_free(std::ptr::null_mut(), std::ptr::null_mut());
        exif_mem_ref(std::ptr::null_mut());
        exif_mem_unref(std::ptr::null_mut());
    }

    assert!(unsafe { exif_log_new_mem(std::ptr::null_mut()) }.is_null());
    assert!(exif_log_code_get_title(EXIF_LOG_CODE_NONE).is_null());
    assert!(exif_log_code_get_message(EXIF_LOG_CODE_NONE).is_null());
    unsafe {
        exif_log_ref(std::ptr::null_mut());
        exif_log_unref(std::ptr::null_mut());
        exif_log_free(std::ptr::null_mut());
        exif_log_set_func(std::ptr::null_mut(), None, std::ptr::null_mut());
    }

    assert!(exif_data_option_get_name(0).is_null());
    assert!(exif_data_option_get_description(0).is_null());
}

#[test]
fn exif_mem_preserves_allocator_and_refcount_semantics() {
    reset_allocators();

    unsafe {
        assert!(exif_mem_new(None, None, None).is_null());

        let mem = exif_mem_new(
            Some(counting_alloc),
            Some(counting_realloc),
            Some(counting_free),
        );
        assert!(!mem.is_null());
        assert_eq!(ALLOC_CALLS.load(Ordering::SeqCst), 1);
        assert_eq!(REALLOC_CALLS.load(Ordering::SeqCst), 0);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 0);

        let block = exif_mem_alloc(mem, 32);
        assert!(!block.is_null());
        assert_eq!(ALLOC_CALLS.load(Ordering::SeqCst), 2);

        let block = exif_mem_realloc(mem, block, 64);
        assert!(!block.is_null());
        assert_eq!(REALLOC_CALLS.load(Ordering::SeqCst), 1);

        exif_mem_free(mem, block);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 1);

        exif_mem_ref(mem);
        exif_mem_unref(mem);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 1);

        exif_mem_unref(mem);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 2);
    }

    reset_allocators();

    unsafe {
        let mem = exif_mem_new(None, Some(counting_realloc), Some(counting_free));
        assert!(!mem.is_null());
        assert_eq!(ALLOC_CALLS.load(Ordering::SeqCst), 0);
        assert_eq!(REALLOC_CALLS.load(Ordering::SeqCst), 1);

        let block = exif_mem_alloc(mem, 16);
        assert!(!block.is_null());
        assert_eq!(REALLOC_CALLS.load(Ordering::SeqCst), 2);

        exif_mem_free(mem, block);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 1);

        exif_mem_unref(mem);
        assert_eq!(FREE_CALLS.load(Ordering::SeqCst), 2);
    }
}

#[test]
fn exif_log_codes_and_null_handling_match_original() {
    assert_eq!(
        c_string(exif_log_code_get_title(EXIF_LOG_CODE_DEBUG)),
        Some("Debugging information".to_owned())
    );
    assert_eq!(
        c_string(exif_log_code_get_message(EXIF_LOG_CODE_DEBUG)),
        Some("Debugging information is available.".to_owned())
    );
    assert_eq!(
        c_string(exif_log_code_get_title(EXIF_LOG_CODE_NO_MEMORY)),
        Some("Not enough memory".to_owned())
    );
    assert_eq!(
        c_string(exif_log_code_get_message(EXIF_LOG_CODE_CORRUPT_DATA)),
        Some("The data provided does not follow the specification.".to_owned())
    );
    assert!(exif_log_code_get_title(EXIF_LOG_CODE_NONE).is_null());
    assert!(exif_log_code_get_message(EXIF_LOG_CODE_NONE).is_null());

    assert!(unsafe { exif_log_new_mem(std::ptr::null_mut()) }.is_null());

    unsafe {
        exif_log_ref(std::ptr::null_mut());
        exif_log_unref(std::ptr::null_mut());
        exif_log_free(std::ptr::null_mut());
        exif_log_set_func(std::ptr::null_mut(), None, std::ptr::null_mut());
    }
}

#[test]
fn exif_log_callback_receives_varargs_exactly() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let include_dir = manifest_dir.join("include");
    let support_dir = manifest_dir.join("tests").join("support");
    let original_dir = manifest_dir
        .parent()
        .expect("safe crate should live below the project root")
        .join("original");
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target"));
    let profile_dir = target_dir.join("debug").join("deps");
    let temp_root = temp_dir("log-callback");
    let source = temp_root.join("log-callback.c");
    let binary = temp_root.join("log-callback");

    fs::write(&source, callback_probe_source()).expect("failed to write logging probe");

    let compiler = env::var("CC").unwrap_or_else(|_| String::from("cc"));
    let compile_output = Command::new(&compiler)
        .arg("-std=c11")
        .arg("-I")
        .arg(&include_dir)
        .arg("-I")
        .arg(&support_dir)
        .arg("-I")
        .arg(&original_dir)
        .arg("-L")
        .arg(&profile_dir)
        .arg(&source)
        .arg("-lexif")
        .arg("-o")
        .arg(&binary)
        .output()
        .expect("failed to compile logging probe");
    assert!(
        compile_output.status.success(),
        "logging probe compilation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let run_output = Command::new(&binary)
        .env("LC_ALL", "C")
        .env("LANG", "")
        .env("LANGUAGE", "")
        .env("LD_LIBRARY_PATH", &profile_dir)
        .env("DYLD_LIBRARY_PATH", &profile_dir)
        .output()
        .expect("failed to run logging probe");
    assert!(
        run_output.status.success(),
        "logging probe failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
}

fn c_string(ptr_: *const c_char) -> Option<String> {
    if ptr_.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(ptr_) }
                .to_string_lossy()
                .into_owned(),
        )
    }
}

fn reset_allocators() {
    ALLOC_CALLS.store(0, Ordering::SeqCst);
    REALLOC_CALLS.store(0, Ordering::SeqCst);
    FREE_CALLS.store(0, Ordering::SeqCst);
}

fn callback_probe_source() -> &'static str {
    r#"
#include <libexif/exif-log.h>

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

static int callback_seen = 0;
static int cookie = 17;

static void
logfunc(ExifLog *log, ExifLogCode code, const char *domain, const char *format, va_list args, void *data)
{
    char buffer[64];

    vsnprintf(buffer, sizeof(buffer), format, args);
    if (log && code == EXIF_LOG_CODE_CORRUPT_DATA &&
        !strcmp(domain, "rust-test") &&
        !strcmp(format, "%s %d") &&
        data == &cookie &&
        !strcmp(buffer, "value 7")) {
        callback_seen = 1;
    }
}

int
main(void)
{
    ExifLog *log = exif_log_new();
    if (!log) {
        return 10;
    }

    exif_log_set_func(log, logfunc, &cookie);
    exif_log(log, EXIF_LOG_CODE_CORRUPT_DATA, "rust-test", "%s %d", "value", 7);
    exif_log_set_func(log, NULL, NULL);
    exif_log(log, EXIF_LOG_CODE_CORRUPT_DATA, "rust-test", "%s", "ignored");
    exif_log_unref(log);

    return callback_seen ? 0 : 11;
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
