#!/usr/bin/env bash
set -euo pipefail

phase_id=impl_09_final_release
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

package_root=${PACKAGE_BUILD_ROOT:-"$safe_dir/.artifacts/$phase_id"}
metadata_dir="$package_root/metadata"
source_commit_path="$metadata_dir/source-commit.txt"
package_inputs_path="$metadata_dir/package-inputs.sha256"
validated_path="$metadata_dir/validated.ok"
artifacts_dir="$package_root/artifacts"
runtime_root="$package_root/libexif12"
dev_root="$package_root/libexif-dev"
doc_root="$package_root/libexif-doc"
overlay_root="$package_root/root"

fail() {
    printf 'run-package-build.sh: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: run-package-build.sh [--print-package-inputs-manifest]
EOF
}

print_package_input_paths() {
    printf '%s\n' \
        original/configure.ac \
        original/debian/libexif12.symbols \
        original/libexif/Makefile.am \
        original/libexif/libexif.sym \
        original/libexif/exif-tag.c \
        original/test/Makefile.am \
        original/contrib/examples/Makefile.am \
        safe/Cargo.toml \
        safe/Cargo.lock \
        safe/build.rs \
        safe/COPYING \
        safe/README \
        safe/NEWS \
        safe/SAFETY.md \
        safe/SECURITY.md \
        safe/libexif.pc.in \
        safe/libexif-uninstalled.pc.in \
        safe/tests/run-package-build.sh \
        safe/tests/run-performance-compare.sh \
        safe/tests/perf/bench-driver.c \
        safe/tests/perf/fixture-manifest.txt \
        safe/tests/perf/thresholds.env

    find "$safe_dir/src" -type f -printf 'safe/src/%P\n'
    find "$safe_dir/cshim" -type f -printf 'safe/cshim/%P\n'
    find "$safe_dir/include" -type f -printf 'safe/include/%P\n'
    find "$safe_dir/debian" -type f -printf 'safe/debian/%P\n'
    find "$safe_dir/po" -type f -printf 'safe/po/%P\n'
    find "$safe_dir/contrib/examples" -type f -printf 'safe/contrib/examples/%P\n'
    find "$safe_dir/doc/libexif-api.html" -type f -printf 'safe/doc/libexif-api.html/%P\n'
    find "$safe_dir/tests/support" -type f -printf 'safe/tests/support/%P\n'

    find "$repo_root/original/libexif" -maxdepth 1 -type f -name '*.h' -printf 'original/libexif/%f\n'
    while IFS= read -r vendor; do
        find "$repo_root/original/libexif/$vendor" -maxdepth 1 -type f \
            \( -name '*.c' -o -name '*.h' \) \
            -printf "original/libexif/$vendor/%f\n"
    done <<'EOF'
apple
canon
fuji
olympus
pentax
EOF
    find "$repo_root/original/test" -maxdepth 1 -type f -name '*.o' -printf 'original/test/%f\n'
    find "$repo_root/original/contrib/examples" -maxdepth 1 -type f -name '*.o' -printf 'original/contrib/examples/%f\n'
    find "$repo_root/original/test/nls" -maxdepth 1 -type f -name 'print-localedir.o' \
        -printf 'original/test/nls/%f\n'
}

move_artifact() {
    local path=$1

    [[ -e "$path" ]] || return 0
    mv "$path" "$artifacts_dir/"
}

print_package_inputs_manifest() {
    (
        cd "$repo_root"
        while IFS= read -r relpath; do
            [[ -n "$relpath" ]] || continue
            [[ -f "$relpath" ]] || fail "missing package input: $relpath"
            sha256sum "$relpath"
        done < <(print_package_input_paths | LC_ALL=C sort -u)
    )
}

case "${1:-}" in
    "")
        ;;
    --print-package-inputs-manifest)
        [[ $# -eq 1 ]] || fail "unexpected extra arguments"
        print_package_inputs_manifest
        exit 0
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        fail "unsupported argument: $1"
        ;;
esac

current_source_commit=$(git -C "$repo_root" rev-parse HEAD)
current_manifest=$(mktemp)
trap 'rm -f "$current_manifest"' EXIT
print_package_inputs_manifest >"$current_manifest"

path_exists() {
    [[ -e "$1" ]]
}

ensure_package_root_is_valid() {
    local docbase_dir
    local docbase_file

    if [[ ! -d "$package_root" ]]; then
        return 1
    fi
    if [[ ! -f "$source_commit_path" || ! -f "$package_inputs_path" || ! -f "$validated_path" ]]; then
        return 1
    fi
    if [[ $(<"$source_commit_path") != "$current_source_commit" ]]; then
        return 1
    fi
    if ! cmp -s "$package_inputs_path" "$current_manifest"; then
        return 1
    fi

    if ! path_exists "$artifacts_dir"; then
        return 1
    fi
    if ! find "$artifacts_dir" -maxdepth 1 -type f -name 'libexif12_*_*.deb' | grep -q .; then
        return 1
    fi
    if ! find "$artifacts_dir" -maxdepth 1 -type f -name 'libexif-dev_*_*.deb' | grep -q .; then
        return 1
    fi
    if ! find "$artifacts_dir" -maxdepth 1 -type f -name 'libexif-doc_*_*.deb' | grep -q .; then
        return 1
    fi

    if [[ ! -f "$runtime_root/usr/lib/$multiarch/libexif.so.12.3.4" ]]; then
        return 1
    fi
    if [[ ! -f "$dev_root/usr/lib/$multiarch/libexif.a" ]]; then
        return 1
    fi
    if [[ ! -f "$overlay_root/usr/lib/$multiarch/libexif.a" ]]; then
        return 1
    fi
    if [[ ! -L "$runtime_root/usr/lib/$multiarch/libexif.so.12" ]]; then
        return 1
    fi
    if [[ $(readlink "$runtime_root/usr/lib/$multiarch/libexif.so.12") != "libexif.so.12.3.4" ]]; then
        return 1
    fi
    if [[ ! -L "$dev_root/usr/lib/$multiarch/libexif.so" ]]; then
        return 1
    fi
    if [[ $(readlink "$dev_root/usr/lib/$multiarch/libexif.so") != "libexif.so.12.3.4" ]]; then
        return 1
    fi
    if [[ $(readlink -f "$overlay_root/usr/lib/$multiarch/libexif.so.12") != "$overlay_root/usr/lib/$multiarch/libexif.so.12.3.4" ]]; then
        return 1
    fi
    if [[ $(readlink -f "$overlay_root/usr/lib/$multiarch/libexif.so") != "$overlay_root/usr/lib/$multiarch/libexif.so.12.3.4" ]]; then
        return 1
    fi
    if [[ ! -f "$dev_root/usr/lib/$multiarch/pkgconfig/libexif.pc" ]]; then
        return 1
    fi

    while IFS= read -r header; do
        [[ -f "$dev_root/usr/include/libexif/$header" ]] || return 1
    done <<'EOF'
_stdint.h
exif-byte-order.h
exif-content.h
exif-data-type.h
exif-data.h
exif-entry.h
exif-format.h
exif-ifd.h
exif-loader.h
exif-log.h
exif-mem.h
exif-mnote-data.h
exif-tag.h
exif-utils.h
EOF

    [[ -f "$dev_root/usr/share/doc/libexif-dev/NEWS" ]] || return 1
    [[ -f "$dev_root/usr/share/doc/libexif-dev/README" ]] || return 1
    [[ -f "$dev_root/usr/share/doc/libexif-dev/SECURITY.md" ]] || return 1
    [[ -f "$doc_root/usr/share/doc/libexif-dev/libexif-api.html/index.html" ]] || return 1

    while IFS= read -r example; do
        [[ -f "$doc_root/usr/share/doc/libexif-dev/examples/$(basename "$example")" ]] || return 1
    done < <(find "$safe_dir/contrib/examples" -maxdepth 1 -type f -name '*.c' | LC_ALL=C sort)

    docbase_dir="$doc_root/usr/share/doc-base"
    [[ -d "$docbase_dir" ]] || return 1
    docbase_file=$(grep -R -l '/usr/share/doc/libexif-dev/libexif-api.html/index.html' "$docbase_dir" || true)
    [[ -n "$docbase_file" ]] || return 1
    grep -q '^Index: /usr/share/doc/libexif-dev/libexif-api.html/index.html$' "$docbase_file" || return 1
    grep -q '^Files: /usr/share/doc/libexif-dev/libexif-api.html/\*\.html$' "$docbase_file" || return 1

    while IFS= read -r gmo; do
        local lang
        lang=${gmo##*/}
        lang=${lang%.gmo}
        [[ -f "$runtime_root/usr/share/locale/$lang/LC_MESSAGES/libexif-12.mo" ]] || return 1
    done < <(find "$safe_dir/po" -maxdepth 1 -type f -name '*.gmo' | LC_ALL=C sort)

    return 0
}

mkdir -p "$(dirname "$package_root")"
exec 9>"$(dirname "$package_root")/.${phase_id}.package-build.lock"
flock 9

if ensure_package_root_is_valid; then
    printf '%s\n' "$package_root"
    exit 0
fi

if [[ ${LIBEXIF_REQUIRE_REUSE:-0} == 1 ]]; then
    fail "reuse-required package root is missing or stale"
fi

mkdir -p "$package_root"
rm -rf \
    "$artifacts_dir" \
    "$runtime_root" \
    "$dev_root" \
    "$doc_root" \
    "$overlay_root" \
    "$package_root/compile-smoke" \
    "$package_root/perf" \
    "$package_root/relinked" \
    "$metadata_dir"
mkdir -p "$artifacts_dir" "$runtime_root" "$dev_root" "$doc_root" "$overlay_root" "$metadata_dir"

rm -f "$repo_root"/libexif*.deb "$repo_root"/libexif*.ddeb "$repo_root"/libexif*.buildinfo "$repo_root"/libexif*.changes

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
while IFS= read -r extra_ddeb; do
    move_artifact "$extra_ddeb"
done < <(find "$repo_root" -maxdepth 1 -type f -name 'libexif*.ddeb' | sort)
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

(
    cd "$safe_dir"
    LC_ALL=C LANG= LANGUAGE= debian/rules clean >/dev/null
)

printf '%s\n' "$current_source_commit" >"$source_commit_path"
cp "$current_manifest" "$package_inputs_path"
printf 'validated\n' >"$validated_path"

if ! ensure_package_root_is_valid; then
    fail "fresh package root failed validation"
fi

printf '%s\n' "$package_root"
