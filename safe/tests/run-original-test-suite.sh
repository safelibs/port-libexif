#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

export LC_ALL=C
export LANG=
export LANGUAGE=

fixtures=(
    "$script_dir/testdata/canon_makernote_variant_1.jpg"
    "$script_dir/testdata/fuji_makernote_variant_1.jpg"
    "$script_dir/testdata/olympus_makernote_variant_2.jpg"
    "$script_dir/testdata/olympus_makernote_variant_3.jpg"
    "$script_dir/testdata/olympus_makernote_variant_4.jpg"
    "$script_dir/testdata/olympus_makernote_variant_5.jpg"
    "$script_dir/testdata/pentax_makernote_variant_2.jpg"
    "$script_dir/testdata/pentax_makernote_variant_3.jpg"
    "$script_dir/testdata/pentax_makernote_variant_4.jpg"
)

for fixture in "${fixtures[@]}"; do
    [[ -f "$fixture" ]] || {
        printf 'missing fixture: %s\n' "$fixture" >&2
        exit 1
    }
done

tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/libexif-original-suite.XXXXXX")
trap 'rm -rf "$tmpdir"' EXIT
extract_output="$tmpdir/extracted.exif"

"$script_dir/run-c-test.sh" \
    test-integers \
    test-tagtable \
    test-sorted \
    test-gps \
    test-value \
    test-null \
    test-data-content \
    test-mem

"$script_dir/run-c-test.sh" \
    test-extract \
    -o "$extract_output" \
    "$script_dir/testdata/fuji_makernote_variant_1.jpg"
[[ -s "$extract_output" ]] || {
    printf 'expected non-empty extract output: %s\n' "$extract_output" >&2
    exit 1
}

printf -v TEST_IMAGES '%s ' "${fixtures[@]}"
TEST_IMAGES=${TEST_IMAGES% }
export TEST_IMAGES
"$script_dir/run-c-test.sh" test-parse
"$script_dir/run-c-test.sh" test-parse-from-data
unset TEST_IMAGES

"$script_dir/run-test-mnote-matrix.sh"
"$script_dir/run-c-test.sh" test-apple-mnote
"$script_dir/run-c-test.sh" test-fuzzer "${fixtures[@]}"
"$script_dir/run-original-shell-test.sh" \
    parse-regression.sh \
    swap-byte-order.sh \
    extract-parse.sh \
    check-failmalloc.sh
"$script_dir/run-original-nls-test.sh"
