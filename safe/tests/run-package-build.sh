#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

package_root=${PACKAGE_BUILD_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/libexif-package.XXXXXX")}
artifacts_dir="$package_root/artifacts"
runtime_root="$package_root/libexif12"
dev_root="$package_root/libexif-dev"
doc_root="$package_root/libexif-doc"
overlay_root="$package_root/root"

fail() {
    printf 'run-package-build.sh: %s\n' "$*" >&2
    exit 1
}

require_file() {
    local path=$1
    [[ -f "$path" ]] || fail "missing file: $path"
}

require_dir() {
    local path=$1
    [[ -d "$path" ]] || fail "missing directory: $path"
}

move_artifact() {
    local path=$1

    [[ -e "$path" ]] || return 0
    mv "$path" "$artifacts_dir/"
}

rm -rf "$artifacts_dir" "$runtime_root" "$dev_root" "$doc_root" "$overlay_root"
mkdir -p "$artifacts_dir" "$runtime_root" "$dev_root" "$doc_root" "$overlay_root"

rm -f "$repo_root"/libexif*.deb "$repo_root"/libexif*.buildinfo "$repo_root"/libexif*.changes

(
    cd "$safe_dir"
    LC_ALL=C LANG= LANGUAGE= dpkg-buildpackage -us -uc -b >/dev/null
)

runtime_deb=$(find "$repo_root" -maxdepth 1 -type f -name 'libexif12_*_*.deb' | sort | tail -n 1)
dev_deb=$(find "$repo_root" -maxdepth 1 -type f -name 'libexif-dev_*_*.deb' | sort | tail -n 1)
doc_deb=$(find "$repo_root" -maxdepth 1 -type f -name 'libexif-doc_*_*.deb' | sort | tail -n 1)

[[ -n "${runtime_deb:-}" ]] || fail "did not produce libexif12 .deb"
[[ -n "${dev_deb:-}" ]] || fail "did not produce libexif-dev .deb"
[[ -n "${doc_deb:-}" ]] || fail "did not produce libexif-doc .deb"

move_artifact "$runtime_deb"
move_artifact "$dev_deb"
move_artifact "$doc_deb"
while IFS= read -r extra_deb; do
    move_artifact "$extra_deb"
done < <(find "$repo_root" -maxdepth 1 -type f -name 'libexif*.deb' | sort)
move_artifact "$(find "$repo_root" -maxdepth 1 -type f -name 'libexif*.buildinfo' | sort | tail -n 1)"
move_artifact "$(find "$repo_root" -maxdepth 1 -type f -name 'libexif*.changes' | sort | tail -n 1)"

runtime_deb="$artifacts_dir/$(basename "$runtime_deb")"
dev_deb="$artifacts_dir/$(basename "$dev_deb")"
doc_deb="$artifacts_dir/$(basename "$doc_deb")"

dpkg-deb -x "$runtime_deb" "$runtime_root"
dpkg-deb -x "$dev_deb" "$dev_root"
dpkg-deb -x "$doc_deb" "$doc_root"

cp -a "$runtime_root"/. "$overlay_root"/
cp -a "$dev_root"/. "$overlay_root"/
cp -a "$doc_root"/. "$overlay_root"/

runtime_lib_dir="$runtime_root/usr/lib/$multiarch"
dev_lib_dir="$dev_root/usr/lib/$multiarch"
overlay_lib_dir="$overlay_root/usr/lib/$multiarch"

require_file "$runtime_lib_dir/libexif.so.12.3.4"
[[ $(readlink "$runtime_lib_dir/libexif.so.12") == "libexif.so.12.3.4" ]] \
    || fail "runtime SONAME symlink is wrong"
[[ $(readlink "$dev_lib_dir/libexif.so") == "libexif.so.12.3.4" ]] \
    || fail "development symlink is wrong"
[[ $(readlink -f "$overlay_lib_dir/libexif.so.12") == "$overlay_lib_dir/libexif.so.12.3.4" ]] \
    || fail "libexif.so.12 does not resolve directly to libexif.so.12.3.4"
[[ $(readlink -f "$overlay_lib_dir/libexif.so") == "$overlay_lib_dir/libexif.so.12.3.4" ]] \
    || fail "libexif.so does not resolve directly to libexif.so.12.3.4"

require_file "$dev_root/usr/share/doc/libexif-dev/NEWS"
require_file "$dev_root/usr/share/doc/libexif-dev/README"
require_file "$dev_root/usr/share/doc/libexif-dev/SECURITY.md"
require_file "$doc_root/usr/share/doc/libexif-dev/libexif-api.html/index.html"

while IFS= read -r example; do
    require_file "$doc_root/usr/share/doc/libexif-dev/examples/$(basename "$example")"
done < <(find "$safe_dir/contrib/examples" -maxdepth 1 -type f -name '*.c' | sort)

docbase_dir="$doc_root/usr/share/doc-base"
require_dir "$docbase_dir"
docbase_file=$(grep -R -l '/usr/share/doc/libexif-dev/libexif-api.html/index.html' "$docbase_dir" || true)
[[ -n "$docbase_file" ]] || fail "missing installed doc-base entry"
grep -q '^Index: /usr/share/doc/libexif-dev/libexif-api.html/index.html$' "$docbase_file" \
    || fail "doc-base entry has the wrong Index"
grep -q '^Files: /usr/share/doc/libexif-dev/libexif-api.html/\*\.html$' "$docbase_file" \
    || fail "doc-base entry has the wrong Files glob"

while IFS= read -r gmo; do
    lang=${gmo##*/}
    lang=${lang%.gmo}
    require_file "$runtime_root/usr/share/locale/$lang/LC_MESSAGES/libexif-12.mo"
done < <(find "$safe_dir/po" -maxdepth 1 -type f -name '*.gmo' | sort)

(
    cd "$safe_dir"
    LC_ALL=C LANG= LANGUAGE= debian/rules clean >/dev/null
)

printf '%s\n' "$package_root"
