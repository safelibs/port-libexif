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
relink_dir="$package_root/relinked"
mkdir -p "$relink_dir"

fail() {
    printf 'run-original-object-link-compat.sh: %s\n' "$*" >&2
    exit 1
}

validate_relative_path() {
    local path=$1

    [[ -n "$path" ]] || fail "empty relative path"
    [[ "$path" != /* ]] || fail "path escapes run directory: $path"
    [[ "$path" != *".."* ]] || fail "path escapes run directory: $path"
}

normalize_streams() {
    sed -E 's/0x[0-9a-fA-F]+/0xPTR/g' "$1"
}

run_program() {
    local mode=$1
    local command_path=$2
    local run_dir=$3
    local stdout_file=$4
    local stderr_file=$5
    shift 5
    local -a argv=("$@")
    local status=0

    set +e
    (
        cd "$run_dir"
        export LC_ALL=C
        export LANG=
        export LANGUAGE=
        case "$mode" in
            libtool_wrapper|direct_original_binary)
                "$command_path" "${argv[@]}"
                ;;
            direct_with_ld_library_path)
                LD_LIBRARY_PATH="$repo_root/original/libexif/.libs${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" \
                    "$command_path" "${argv[@]}"
                ;;
            *)
                exit 125
                ;;
        esac
    ) >"$stdout_file" 2>"$stderr_file"
    status=$?
    set -e

    [[ $status -ne 125 ]] || fail "unsupported baseline mode: $mode"
    return "$status"
}

compare_streams() {
    local mode=$1
    local left=$2
    local right=$3

    case "$mode" in
        exact_streams)
            cmp -s "$left" "$right"
            ;;
        normalize_hex_pointers_in_streams)
            cmp -s <(normalize_streams "$left") <(normalize_streams "$right")
            ;;
        *)
            fail "unsupported comparison_mode: $mode"
            ;;
    esac
}

declare -A object_paths
while IFS='|' read -r name path; do
    [[ -n "${name:-}" && -n "${path:-}" ]] || continue
    object_paths["$name"]=$path
done <"$safe_dir/tests/link-compat/object-manifest.txt"

make -C "$repo_root/original/test" \
    test-mem test-mnote test-value test-integers test-parse test-parse-from-data \
    test-data-content test-tagtable test-sorted test-fuzzer test-extract test-null test-gps >/dev/null
make -C "$repo_root/original/test/nls" print-localedir >/dev/null
make -C "$repo_root/original/contrib/examples" photographer thumbnail write-exif >/dev/null

flush_entry() {
    local -a argv=("${current_argv[@]}")
    local -a fixtures=("${current_fixtures[@]}")
    local -a outputs=("${current_outputs[@]}")

    local name=$current_name
    local baseline_cmd=$current_baseline_cmd
    local baseline_mode=$current_baseline_mode
    local comparison_mode=$current_comparison_mode

    [[ -n "$name" ]] || return 0
    [[ -n "$baseline_cmd" ]] || fail "manifest entry $name is missing baseline_cmd"
    [[ -n "$comparison_mode" ]] || fail "manifest entry $name is missing comparison_mode"
    [[ "$baseline_cmd" == /* ]] || fail "manifest entry $name has a non-absolute baseline_cmd"
    [[ -e "$baseline_cmd" ]] || fail "manifest entry $name baseline_cmd does not exist: $baseline_cmd"
    [[ "$comparison_mode" == "exact_streams" || "$comparison_mode" == "normalize_hex_pointers_in_streams" ]] \
        || fail "manifest entry $name has an invalid comparison_mode"
    [[ "$baseline_mode" == "libtool_wrapper" || "$baseline_mode" == "direct_original_binary" || "$baseline_mode" == "direct_with_ld_library_path" ]] \
        || fail "manifest entry $name has an invalid baseline_mode"
    if [[ "$baseline_cmd" == *"/.libs/"* && "$baseline_mode" != "direct_with_ld_library_path" ]]; then
        fail "manifest entry $name uses a direct .libs baseline without direct_with_ld_library_path mode"
    fi
    if [[ "$baseline_mode" == "direct_with_ld_library_path" && "$baseline_cmd" != *"/.libs/"* ]]; then
        fail "manifest entry $name uses direct_with_ld_library_path without a .libs baseline"
    fi

    object_path=${object_paths["$name"]:-}
    [[ -n "$object_path" ]] || fail "no object-manifest entry for $name"
    [[ -f "$object_path" ]] || fail "missing object file for $name: $object_path"

    relinked_binary="$relink_dir/$name"
    cc "$object_path" \
        -Wl,--no-as-needed \
        -Wl,-rpath,"$overlay_lib_dir" \
        -L"$overlay_lib_dir" \
        -lexif \
        -Wl,--as-needed \
        -o "$relinked_binary"

    resolved_lib=$(ldd "$relinked_binary" | awk '/libexif\.so\.12/ {print $3; exit}')
    [[ -n "$resolved_lib" ]] || fail "$name does not resolve libexif.so.12 to the safe package"
    [[ $(readlink -f "$resolved_lib") == "$overlay_lib_dir/libexif.so.12.3.4" ]] \
        || fail "$name resolved libexif.so.12 to the wrong library"

    baseline_run_dir=$(mktemp -d "$package_root/${name}.baseline.XXXXXX")
    relink_run_dir=$(mktemp -d "$package_root/${name}.relinked.XXXXXX")
    baseline_stdout="$baseline_run_dir/stdout"
    baseline_stderr="$baseline_run_dir/stderr"
    relink_stdout="$relink_run_dir/stdout"
    relink_stderr="$relink_run_dir/stderr"

    for fixture in "${fixtures[@]}"; do
        src=${fixture%%=>*}
        dest=${fixture#*=>}
        [[ -e "$src" ]] || fail "$name fixture source does not exist: $src"
        validate_relative_path "$dest"
        install -d "$baseline_run_dir/$(dirname "$dest")" "$relink_run_dir/$(dirname "$dest")"
        cp -a "$src" "$baseline_run_dir/$dest"
        cp -a "$src" "$relink_run_dir/$dest"
    done

    baseline_status=0
    relink_status=0

    if run_program "$baseline_mode" "$baseline_cmd" "$baseline_run_dir" \
        "$baseline_stdout" "$baseline_stderr" "${argv[@]}"; then
        baseline_status=0
    else
        baseline_status=$?
    fi

    set +e
    (
        cd "$relink_run_dir"
        export LC_ALL=C
        export LANG=
        export LANGUAGE=
        "$relinked_binary" "${argv[@]}"
    ) >"$relink_stdout" 2>"$relink_stderr"
    relink_status=$?
    set -e

    [[ $baseline_status -eq $relink_status ]] || fail "$name exit status mismatch"
    compare_streams "$comparison_mode" "$baseline_stdout" "$relink_stdout" \
        || fail "$name stdout mismatch"
    compare_streams "$comparison_mode" "$baseline_stderr" "$relink_stderr" \
        || fail "$name stderr mismatch"

    for output in "${outputs[@]}"; do
        validate_relative_path "$output"
        [[ -e "$baseline_run_dir/$output" ]] || fail "$name baseline is missing output $output"
        [[ -e "$relink_run_dir/$output" ]] || fail "$name relinked run is missing output $output"
        cmp -s "$baseline_run_dir/$output" "$relink_run_dir/$output" \
            || fail "$name output file mismatch: $output"
    done
}

current_name=
current_baseline_cmd=
current_baseline_mode=
current_comparison_mode=
declare -a current_argv=()
declare -a current_fixtures=()
declare -a current_outputs=()

while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ -z "$line" ]]; then
        flush_entry
        current_name=
        current_baseline_cmd=
        current_baseline_mode=
        current_comparison_mode=
        current_argv=()
        current_fixtures=()
        current_outputs=()
        continue
    fi

    key=${line%%=*}
    value=${line#*=}
    case "$key" in
        name)
            current_name=$value
            ;;
        baseline_cmd)
            current_baseline_cmd=$value
            ;;
        baseline_mode)
            current_baseline_mode=$value
            ;;
        comparison_mode)
            current_comparison_mode=$value
            ;;
        argv)
            current_argv+=("$value")
            ;;
        fixture)
            current_fixtures+=("$value")
            ;;
        output_path)
            current_outputs+=("$value")
            ;;
        *)
            fail "unknown manifest key: $key"
            ;;
    esac
done <"$safe_dir/tests/link-compat/run-manifest.txt"

flush_entry
