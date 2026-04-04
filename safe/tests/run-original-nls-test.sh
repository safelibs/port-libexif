#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
build_root=${PACKAGE_BUILD_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/libexif-nls.XXXXXX")}
binary="$build_root/print-localedir"

export LC_ALL=C
export LANG=
export LANGUAGE=

cc -std=c11 \
    -I"$safe_dir/tests/support" \
    "$safe_dir/tests/original-c/nls/print-localedir.c" \
    -o "$binary"

cp "$safe_dir/tests/original-sh/nls/check-localedir.sh" "$build_root/check-localedir.sh"
chmod +x "$build_root/check-localedir.sh"

(
    cd "$build_root"
    LOCALEDIR="/usr/share/locale" \
    PRINT_LOCALEDIR_BIN="$binary" \
    ./check-localedir.sh
)
