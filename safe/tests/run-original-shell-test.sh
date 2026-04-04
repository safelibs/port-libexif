#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
crate_dir=$(cd "$script_dir/.." && pwd)
target_dir=${CARGO_TARGET_DIR:-"$crate_dir/target"}
profile=${PROFILE:-debug}
deps_dir="$target_dir/$profile/deps"
binary_dir="$target_dir/original-shell"
compiler=${CC:-cc}
soname_link="$deps_dir/libexif.so.12"
exeext=${EXEEXT:-}
diff_cmd=${DIFF:-diff}
diff_u_cmd=${DIFF_U:-"diff -u"}
failmalloc_path=${FAILMALLOC_PATH:-}

export LC_ALL=C
export LANG=
export LANGUAGE=

sources=(
    test-parse
    test-extract
    test-parse-from-data
    test-value
    test-mem
)

scripts=("$@")
if [[ ${#scripts[@]} -eq 0 ]]; then
    scripts=(
        parse-regression.sh
        swap-byte-order.sh
        extract-parse.sh
        check-failmalloc.sh
    )
fi

cargo build --manifest-path "$crate_dir/Cargo.toml" --lib >/dev/null

if [[ -f "$deps_dir/libexif.so" ]]; then
    ln -sf "libexif.so" "$soname_link"
fi

mkdir -p "$binary_dir"
cp "$script_dir/original-sh/inc-comparetool.sh" "$binary_dir/inc-comparetool.sh"
ln -sfn "$script_dir/testdata" "$binary_dir/testdata"

for source in "${sources[@]}"; do
    "$compiler" \
        -std=c11 \
        -I"$crate_dir/include" \
        -I"$crate_dir/tests/support" \
        -I"$crate_dir/../original" \
        -L"$deps_dir" \
        "$script_dir/original-c/${source}.c" \
        -lexif \
        -o "$binary_dir/${source}${exeext}"
done

run_script() {
    local script_name=$1
    local status=0

    set +e
    (
        cd "$binary_dir"
        export srcdir="$binary_dir"
        export EXEEXT="$exeext"
        export DIFF="$diff_cmd"
        export DIFF_U="$diff_u_cmd"
        export FAILMALLOC_PATH="$failmalloc_path"
        export LD_LIBRARY_PATH="$deps_dir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        export DYLD_LIBRARY_PATH="$deps_dir${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}"
        export PATH="$deps_dir${PATH:+:$PATH}"
        sh "$script_dir/original-sh/$script_name"
    )
    status=$?
    set -e

    if [[ "$script_name" == "check-failmalloc.sh" && $status -eq 77 ]]; then
        printf 'skipping %s because FAILMALLOC_PATH is unavailable\n' "$script_name"
        return 0
    fi

    return "$status"
}

for script_name in "${scripts[@]}"; do
    run_script "$script_name"
done
