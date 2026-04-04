#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd "$script_dir/.." && pwd)
target_dir=${CARGO_TARGET_DIR:-"$crate_dir/target"}
profile=${PROFILE:-debug}
deps_dir="$target_dir/$profile/deps"
compiler=${CC:-cc}

export LC_ALL=C
export LANG=
export LANGUAGE=

cargo build --manifest-path "$crate_dir/Cargo.toml" --lib >/dev/null

sources=()
if [[ $# -eq 0 ]]; then
    while IFS= read -r -d '' source; do
        sources+=("$source")
    done < <(find "$script_dir/original-c" -maxdepth 1 -name '*.c' -print0 | sort -z)
else
    for source in "$@"; do
        candidate="$source"
        if [[ ! -f "$candidate" ]]; then
            candidate="$script_dir/original-c/$source"
        fi
        if [[ ! -f "$candidate" && "$candidate" != *.c ]]; then
            candidate="${candidate}.c"
        fi
        if [[ ! -f "$candidate" ]]; then
            echo "unknown C test: $source" >&2
            exit 1
        fi
        sources+=("$candidate")
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
    "$binary"
done
