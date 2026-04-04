#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

package_root=${PACKAGE_BUILD_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/libexif-compile-smoke.XXXXXX")}
PACKAGE_BUILD_ROOT="$package_root" "$script_dir/run-package-build.sh" >/dev/null

overlay_root="$package_root/root"
build_root="$package_root/compile-smoke"
mkdir -p "$build_root"

export PKG_CONFIG_PATH="$overlay_root/usr/lib/$multiarch/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="$overlay_root"

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
    output="$build_root/$(basename "${source%.c}")"
    cc -std=c11 $(pkg-config --cflags libexif) "$source" \
        $(pkg-config --libs libexif) -o "$output"
done
