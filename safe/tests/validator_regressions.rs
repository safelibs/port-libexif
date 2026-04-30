use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn validator_usage_debug_trace_callbacks_cover_cli_debug_failures() {
    // Covers validator testcase ids:
    // usage-exif-cli-debug-loader-trace,
    // usage-exif-cli-debug-ifd-gps-trace,
    // usage-exif-cli-debug-machine-readable-combo,
    // usage-exif-cli-debug-no-fixup-loader-trace,
    // usage-exif-cli-remove-decrements-ifd-zero-entries,
    // usage-exif-cli-remove-missing-copyright.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest_dir
        .join("tests")
        .join("testdata")
        .join("canon_makernote_variant_1.jpg");
    assert!(fixture.is_file(), "missing validator fixture: {fixture:?}");

    let temp_root = temp_dir("validator-usage-debug");
    let source = temp_root.join("validator-usage-debug.c");
    let binary = temp_root.join("validator-usage-debug");
    fs::write(&source, debug_trace_probe_source()).expect("failed to write debug trace probe");

    let include_dir = manifest_dir.join("include");
    let support_dir = manifest_dir.join("tests").join("support");
    let original_dir = manifest_dir
        .parent()
        .expect("safe crate should live below the project root")
        .join("original");
    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("target"));
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let profile_dir = target_dir.join(profile).join("deps");

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
        .expect("failed to compile debug trace probe");
    assert!(
        compile_output.status.success(),
        "debug trace probe compilation failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile_output.stdout),
        String::from_utf8_lossy(&compile_output.stderr)
    );

    let run_output = Command::new(&binary)
        .arg(&fixture)
        .env("LC_ALL", "C")
        .env("LANG", "")
        .env("LANGUAGE", "")
        .env("LD_LIBRARY_PATH", &profile_dir)
        .env("DYLD_LIBRARY_PATH", &profile_dir)
        .output()
        .expect("failed to run debug trace probe");
    assert!(
        run_output.status.success(),
        "debug trace probe failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        run_output.status.code(),
        String::from_utf8_lossy(&run_output.stdout),
        String::from_utf8_lossy(&run_output.stderr)
    );
}

fn debug_trace_probe_source() -> &'static str {
    r#"
#include <libexif/exif-content.h>
#include <libexif/exif-data.h>
#include <libexif/exif-loader.h>
#include <libexif/exif-log.h>
#include <libexif/exif-tag.h>

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

struct Seen {
    int scanning;
    int found_header;
    int ifd0_at_8;
    int entries9;
    int entries8;
    int make_entry;
    int model_entry;
};

static struct Seen *current_seen;

static void
logfunc(ExifLog *log, ExifLogCode code, const char *domain, const char *format, va_list args, void *data)
{
    char message[256];

    (void) log;
    (void) data;
    if (!current_seen || code != EXIF_LOG_CODE_DEBUG || !domain || !format) {
        return;
    }

    vsnprintf(message, sizeof(message), format, args);
    if (!strcmp(domain, "ExifLoader") && strstr(message, "Scanning ")) {
        current_seen->scanning = 1;
    }
    if (!strcmp(domain, "ExifData") && strstr(message, "Found EXIF header")) {
        current_seen->found_header = 1;
    }
    if (!strcmp(domain, "ExifData") && !strcmp(message, "IFD 0 at 8.")) {
        current_seen->ifd0_at_8 = 1;
    }
    if (!strcmp(domain, "ExifData") && !strcmp(message, "Loading 9 entries...")) {
        current_seen->entries9 = 1;
    }
    if (!strcmp(domain, "ExifData") && !strcmp(message, "Loading 8 entries...")) {
        current_seen->entries8 = 1;
    }
    if (!strcmp(domain, "ExifData") && strstr(message, "Loading entry 0x10f ('Make')")) {
        current_seen->make_entry = 1;
    }
    if (!strcmp(domain, "ExifData") && strstr(message, "Loading entry 0x110 ('Model')")) {
        current_seen->model_entry = 1;
    }
}

static int
load_file_with_log(const char *path, ExifData **out, struct Seen *seen)
{
    ExifLoader *loader = exif_loader_new();
    ExifLog *log = exif_log_new();
    ExifData *data;

    if (!loader || !log) {
        return 10;
    }

    current_seen = seen;
    exif_log_set_func(log, logfunc, NULL);
    exif_loader_log(loader, log);
    exif_loader_write_file(loader, path);
    data = exif_loader_get_data(loader);
    current_seen = NULL;

    exif_loader_unref(loader);
    exif_log_unref(log);

    if (!data) {
        return 11;
    }

    *out = data;
    return 0;
}

static int
load_raw_with_log(const unsigned char *raw, unsigned int raw_size, struct Seen *seen)
{
    ExifData *data = exif_data_new();
    ExifLog *log = exif_log_new();

    if (!data || !log) {
        return 20;
    }

    current_seen = seen;
    exif_log_set_func(log, logfunc, NULL);
    exif_data_log(data, log);
    exif_data_load_data(data, raw, raw_size);
    current_seen = NULL;

    exif_log_unref(log);
    exif_data_unref(data);
    return 0;
}

int
main(int argc, char **argv)
{
    ExifData *data = NULL;
    ExifContent *ifd0;
    ExifEntry *make;
    unsigned int before_ifd0_count;
    unsigned char *raw = NULL;
    unsigned int raw_size = 0;
    struct Seen before = {0};
    struct Seen after = {0};
    int rc;

    if (argc != 2) {
        return 2;
    }

    rc = load_file_with_log(argv[1], &data, &before);
    if (rc) {
        return rc;
    }
    if (!before.scanning || !before.found_header || !before.ifd0_at_8 ||
        !before.entries9 || !before.make_entry || !before.model_entry) {
        fprintf(stderr,
                "missing original debug trace scanning=%d found=%d ifd0=%d entries9=%d make=%d model=%d\n",
                before.scanning, before.found_header, before.ifd0_at_8,
                before.entries9, before.make_entry, before.model_entry);
        return 12;
    }

    ifd0 = data->ifd[EXIF_IFD_0];
    if (!ifd0 || ifd0->count == 0) {
        fprintf(stderr, "expected public IFD 0 entries before removal\n");
        return 13;
    }
    before_ifd0_count = ifd0->count;
    if (exif_content_get_entry(ifd0, EXIF_TAG_COPYRIGHT)) {
        fprintf(stderr, "validator fixture unexpectedly contains Copyright\n");
        return 14;
    }

    make = exif_content_get_entry(ifd0, EXIF_TAG_MAKE);
    if (!make) {
        return 15;
    }
    exif_content_remove_entry(ifd0, make);
    if (ifd0->count != before_ifd0_count - 1) {
        fprintf(stderr, "expected public IFD 0 count to decrement from %u to %u, got %u\n",
                before_ifd0_count, before_ifd0_count - 1, ifd0->count);
        return 16;
    }

    exif_data_save_data(data, &raw, &raw_size);
    exif_data_unref(data);
    if (!raw || raw_size == 0) {
        return 17;
    }

    rc = load_raw_with_log(raw, raw_size, &after);
    free(raw);
    if (rc) {
        return rc;
    }
    if (!after.found_header || !after.ifd0_at_8 || !after.entries8) {
        fprintf(stderr,
                "missing stripped debug trace found=%d ifd0=%d entries8=%d\n",
                after.found_header, after.ifd0_at_8, after.entries8);
        return 18;
    }

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
