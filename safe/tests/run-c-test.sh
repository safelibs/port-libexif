#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd "$script_dir/.." && pwd)
target_dir=${CARGO_TARGET_DIR:-"$crate_dir/target"}
profile=${PROFILE:-debug}
deps_dir="$target_dir/$profile/deps"
compiler=${CC:-cc}
soname_link="$deps_dir/libexif.so.12"

export LC_ALL=C
export LANG=
export LANGUAGE=

cargo build --manifest-path "$crate_dir/Cargo.toml" --lib >/dev/null

if [[ -f "$deps_dir/libexif.so" ]]; then
    ln -sf "libexif.so" "$soname_link"
fi

sources=()
binary_args=()

resolve_source() {
    local source=$1
    local candidate

    if [[ -f "$source" && "$source" == *.c ]]; then
        printf '%s\n' "$source"
        return 0
    fi

    candidate="$script_dir/original-c/$source"
    if [[ -f "$candidate" && "$candidate" == *.c ]]; then
        printf '%s\n' "$candidate"
        return 0
    fi

    if [[ "$source" != *.c ]]; then
        candidate="$script_dir/original-c/${source}.c"
        if [[ -f "$candidate" ]]; then
            printf '%s\n' "$candidate"
            return 0
        fi
    fi

    return 1
}

if [[ $# -eq 0 ]]; then
    while IFS= read -r -d '' source; do
        sources+=("$source")
    done < <(find "$script_dir/original-c" -maxdepth 1 -name '*.c' ! -name 'test-null.c' -print0 | sort -z)
else
    while [[ $# -gt 0 ]]; do
        if candidate=$(resolve_source "$1"); then
            sources+=("$candidate")
            shift
            continue
        fi

        binary_args=("$@")
        break
    done
fi

if [[ ${#sources[@]} -eq 0 ]]; then
    echo "no C tests selected" >&2
    exit 1
fi

mkdir -p "$target_dir/c-tests"

for source in "${sources[@]}"; do
    base=$(basename "${source%.c}")
    binary="$target_dir/c-tests/$base"

    "$compiler" \
        -std=c11 \
        -I"$crate_dir/include" \
        -I"$crate_dir/tests/support" \
        -I"$crate_dir/../original" \
        -L"$deps_dir" \
        "$source" \
        -lexif \
        -o "$binary"

    LD_LIBRARY_PATH="$deps_dir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
    DYLD_LIBRARY_PATH="$deps_dir${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" \
    PATH="$deps_dir${PATH:+:$PATH}" \
    "$binary" "${binary_args[@]}"
done
