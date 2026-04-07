#!/usr/bin/env bash
set -euo pipefail

phase_id=impl_09_final_release
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

package_root=${PACKAGE_BUILD_ROOT:-"$safe_dir/.artifacts/$phase_id"}
PACKAGE_BUILD_ROOT="$package_root" "$script_dir/run-package-build.sh" >/dev/null

library="$package_root/root/usr/lib/$multiarch/libexif.so.12.3.4"
[[ -f "$library" ]] || {
    printf 'run-export-compare.sh: missing packaged library %s\n' "$library" >&2
    exit 1
}

normalize_symbols_file() {
    sed -E 's/^libexif\.so\.12 .*/libexif.so.12 #PACKAGE# #MINVER#/'
}

expected_names=$(mktemp)
expected_versions=$(mktemp)
actual_names=$(mktemp)
actual_versions=$(mktemp)
trap 'rm -f "$expected_names" "$expected_versions" "$actual_names" "$actual_versions"' EXIT

sed -n 's/^[[:space:]]*\([[:alnum:]_][[:alnum:]_]*\)$/\1/p' \
    "$repo_root/original/libexif/libexif.sym" \
    | LC_ALL=C sort -u >"$expected_names"

sed -n 's/^[[:space:]]*\([[:alnum:]_][[:alnum:]_]*@[[:alnum:]_.-][[:alnum:]_.-]*\)[[:space:]].*$/\1/p' \
    "$repo_root/original/debian/libexif12.symbols" \
    | LC_ALL=C sort -u >"$expected_versions"

objdump -T "$library" \
    | awk '$4 != "*UND*" && $6 != "" && $7 != "" { print $7 }' \
    | LC_ALL=C sort -u >"$actual_names"

objdump -T "$library" \
    | awk '$4 != "*UND*" && $6 != "" && $7 != "" { print $7 "@" $6 }' \
    | LC_ALL=C sort -u >"$actual_versions"

diff -u "$expected_names" "$actual_names"
diff -u "$expected_versions" "$actual_versions"

if ! diff -u \
    <(normalize_symbols_file <"$repo_root/original/debian/libexif12.symbols") \
    <(normalize_symbols_file <"$safe_dir/debian/libexif12.symbols"); then
    printf 'run-export-compare.sh: safe/debian/libexif12.symbols diverged from the copied original manifest\n' >&2
    exit 1
fi

while IFS= read -r generated; do
    if ! diff -u \
        <(normalize_symbols_file <"$repo_root/original/debian/libexif12.symbols") \
        <(normalize_symbols_file <"$generated"); then
        printf 'run-export-compare.sh: generated symbols file diverged from the copied original manifest: %s\n' \
            "$generated" >&2
        exit 1
    fi
done < <(find "$safe_dir/debian" -type f -name 'libexif12.symbols' | sort)
