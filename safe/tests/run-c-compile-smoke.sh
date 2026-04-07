#!/usr/bin/env bash
set -euo pipefail

phase_id=impl_09_final_release
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

package_root=${PACKAGE_BUILD_ROOT:-"$safe_dir/.artifacts/$phase_id"}
PACKAGE_BUILD_ROOT="$package_root" "$script_dir/run-package-build.sh" >/dev/null

overlay_root="$package_root/root"
overlay_lib_dir="$overlay_root/usr/lib/$multiarch"
build_root="$package_root/compile-smoke"
shared_root="$build_root/shared"
static_root="$build_root/static"

export PKG_CONFIG_PATH="$overlay_root/usr/lib/$multiarch/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="$overlay_root"

fail() {
    printf 'run-c-compile-smoke.sh: %s\n' "$*" >&2
    exit 1
}

rm -rf "$shared_root" "$static_root"
mkdir -p "$shared_root" "$static_root"

mapfile -t sources <<EOF
$safe_dir/tests/smoke/public-api-smoke.c
$repo_root/original/test/test-integers.c
$repo_root/original/test/test-extract.c
$repo_root/original/test/test-sorted.c
$repo_root/original/contrib/examples/photographer.c
$repo_root/original/contrib/examples/thumbnail.c
$repo_root/original/contrib/examples/write-exif.c
EOF

for source in "${sources[@]}"; do
    output="$shared_root/$(basename "${source%.c}")"
    cc -std=c11 $(pkg-config --cflags libexif) "$source" \
        $(pkg-config --libs libexif) \
        -Wl,-rpath,"$overlay_lib_dir" \
        -o "$output"
done

"$shared_root/public-api-smoke"

static_output="$static_root/public-api-smoke-static"
cc -std=c11 $(pkg-config --cflags libexif) \
    "$safe_dir/tests/smoke/public-api-smoke.c" \
    "$overlay_lib_dir/libexif.a" \
    -lm \
    -o "$static_output"

"$static_output"
if readelf -d "$static_output" 2>/dev/null | grep -q 'Shared library: \[libexif\.so\.12\]'; then
    fail "forced static smoke binary still depends on libexif.so.12"
fi
