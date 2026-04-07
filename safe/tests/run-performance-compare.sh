#!/usr/bin/env bash
set -euo pipefail

phase_id=impl_09_final_release
script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
safe_dir=$(cd "$script_dir/.." && pwd)
repo_root=$(cd "$safe_dir/.." && pwd)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)

source "$script_dir/perf/thresholds.env"

fixture_root="$script_dir/testdata"
manifest="$script_dir/perf/fixture-manifest.txt"
original_so="$repo_root/original/libexif/.libs/libexif.so.12.3.4"
package_root=${PACKAGE_BUILD_ROOT:-"$safe_dir/.artifacts/$phase_id"}
package_root=$(PACKAGE_BUILD_ROOT="$package_root" "$script_dir/run-package-build.sh")
perf_root="$package_root/perf"
driver="$perf_root/bench-driver"
safe_overlay="$package_root/root"
safe_lib_dir="$safe_overlay/usr/lib/$multiarch"
original_runtime_dir="$perf_root/original-lib"

export LC_ALL=C
export LANG=
export LANGUAGE=

fail() {
    printf 'run-performance-compare.sh: %s\n' "$*" >&2
    exit 1
}

ensure_original_library() {
    if [[ -f "$original_so" ]]; then
        return
    fi

    if make -C "$repo_root/original/libexif" all >/dev/null 2>&1; then
        :
    else
        make -C "$repo_root/original" all >/dev/null
    fi

    [[ -f "$original_so" ]] || fail "missing original baseline library after rebuild: $original_so"
}

compile_driver() {
    mkdir -p "$perf_root"
    export PKG_CONFIG_PATH="$safe_lib_dir/pkgconfig"
    export PKG_CONFIG_SYSROOT_DIR="$safe_overlay"

    cc -O2 -std=c11 \
        $(pkg-config --cflags libexif) \
        "$script_dir/perf/bench-driver.c" \
        $(pkg-config --libs libexif) \
        -o "$driver"
}

prepare_runtime_dirs() {
    mkdir -p "$original_runtime_dir"
    ln -sf "$original_so" "$original_runtime_dir/libexif.so.12.3.4"
    ln -sf "libexif.so.12.3.4" "$original_runtime_dir/libexif.so.12"
    ln -sf "libexif.so.12.3.4" "$original_runtime_dir/libexif.so"
}

median_file() {
    local times_file=$1
    LC_ALL=C sort -n "$times_file" | awk '
        { values[NR] = $1 }
        END {
            if (NR == 0) {
                exit 1
            }
            if ((NR % 2) == 1) {
                printf "%.6f", values[(NR + 1) / 2]
            } else {
                printf "%.6f", (values[NR / 2] + values[NR / 2 + 1]) / 2
            }
        }
    '
}

run_timed_sample() {
    local lib_dir=$1
    local workload=$2
    local iterations=$3
    local output_file=$4

    LD_LIBRARY_PATH="$lib_dir${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
    DYLD_LIBRARY_PATH="$lib_dir${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" \
    /usr/bin/time -f '%e' -o "$output_file" \
    "$driver" "$workload" "$fixture_root" "$manifest" "$iterations" >/dev/null
}

measure_workload() {
    local label=$1
    local lib_dir=$2
    local workload=$3
    local iterations=$4
    local times_file
    local sample_file
    local run
    local median

    times_file=$(mktemp)
    sample_file=$(mktemp)
    trap 'rm -f "$times_file" "$sample_file"' RETURN

    for ((run = 0; run < PERF_WARMUP_RUNS; run++)); do
        run_timed_sample "$lib_dir" "$workload" "$iterations" "$sample_file"
    done

    : >"$times_file"
    for ((run = 0; run < PERF_SAMPLE_RUNS; run++)); do
        run_timed_sample "$lib_dir" "$workload" "$iterations" "$sample_file"
        cat "$sample_file" >>"$times_file"
        printf '\n' >>"$times_file"
    done

    median=$(median_file "$times_file")
    printf '%-8s %-18s median=%ss iterations=%s\n' "$label" "$workload" "$median" "$iterations" >&2
    printf '%s' "$median"
}

ratio_value() {
    local numerator=$1
    local denominator=$2
    awk -v num="$numerator" -v den="$denominator" 'BEGIN {
        if (den == 0) {
            if (num == 0) {
                printf "%.6f", 1
            } else {
                printf "%.6f", 999999
            }
            exit 0
        }
        printf "%.6f", num / den
    }'
}

sum_values() {
    printf '%s\n' "$@" | awk '
        { sum += $1 }
        END { printf "%.6f", sum }
    '
}

ensure_original_library
[[ -f "$safe_lib_dir/libexif.so.12.3.4" ]] || fail "missing packaged safe library"
compile_driver
prepare_runtime_dirs

declare -A iterations=(
    [parse_file]="$PERF_ITERATIONS_PARSE_FILE"
    [parse_memory]="$PERF_ITERATIONS_PARSE_MEMORY"
    [save_data]="$PERF_ITERATIONS_SAVE_DATA"
    [swap_byte_order]="$PERF_ITERATIONS_SWAP_BYTE_ORDER"
    [format_entries]="$PERF_ITERATIONS_FORMAT_ENTRIES"
    [format_makernotes]="$PERF_ITERATIONS_FORMAT_MAKERNOTES"
)
workloads=(
    parse_file
    parse_memory
    save_data
    swap_byte_order
    format_entries
    format_makernotes
)

declare -A original_medians
declare -A safe_medians

for workload in "${workloads[@]}"; do
    original_medians["$workload"]=$(
        measure_workload original "$original_runtime_dir" "$workload" "${iterations[$workload]}"
    )
    safe_medians["$workload"]=$(
        measure_workload safe "$safe_lib_dir" "$workload" "${iterations[$workload]}"
    )
done

printf '\n'
for workload in "${workloads[@]}"; do
    ratio=$(ratio_value "${safe_medians[$workload]}" "${original_medians[$workload]}")
    printf '%-18s original=%ss safe=%ss ratio=%sx\n' \
        "$workload" \
        "${original_medians[$workload]}" \
        "${safe_medians[$workload]}" \
        "$ratio"
done

parse_save_swap_original=$(sum_values \
    "${original_medians[parse_file]}" \
    "${original_medians[parse_memory]}" \
    "${original_medians[save_data]}" \
    "${original_medians[swap_byte_order]}")
parse_save_swap_safe=$(sum_values \
    "${safe_medians[parse_file]}" \
    "${safe_medians[parse_memory]}" \
    "${safe_medians[save_data]}" \
    "${safe_medians[swap_byte_order]}")
makernote_original=$(sum_values "${original_medians[format_makernotes]}")
makernote_safe=$(sum_values "${safe_medians[format_makernotes]}")

parse_save_swap_ratio=$(ratio_value "$parse_save_swap_safe" "$parse_save_swap_original")
makernote_ratio=$(ratio_value "$makernote_safe" "$makernote_original")

printf '\n'
printf 'aggregate parse/save/swap original=%ss safe=%ss ratio=%sx threshold=%sx\n' \
    "$parse_save_swap_original" \
    "$parse_save_swap_safe" \
    "$parse_save_swap_ratio" \
    "$PERF_MAX_PARSE_SAVE_SWAP_RATIO"
printf 'aggregate makernote format original=%ss safe=%ss ratio=%sx threshold=%sx\n' \
    "$makernote_original" \
    "$makernote_safe" \
    "$makernote_ratio" \
    "$PERF_MAX_MAKERNOTE_FORMAT_RATIO"

awk -v ratio="$parse_save_swap_ratio" -v max="$PERF_MAX_PARSE_SAVE_SWAP_RATIO" 'BEGIN {
    exit !(ratio <= max)
}' || fail "aggregate parse/save/swap regression exceeded threshold"

awk -v ratio="$makernote_ratio" -v max="$PERF_MAX_MAKERNOTE_FORMAT_RATIO" 'BEGIN {
    exit !(ratio <= max)
}' || fail "aggregate makernote formatting regression exceeded threshold"
