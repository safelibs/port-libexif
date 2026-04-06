# Source and Link Compatibility Against Existing Original Clients

## Phase Name
`Source and Link Compatibility Against Existing Original Clients`

## Implement Phase ID
`impl_04_source_link_compat`

## Preexisting Inputs
  - `safe/tests/run-package-build.sh`
  - `safe/tests/run-c-compile-smoke.sh`
  - `safe/tests/run-original-object-link-compat.sh`
  - `safe/tests/link-compat/object-manifest.txt`
  - `safe/tests/link-compat/run-manifest.txt`
  - `original/test/*.o`, `original/contrib/examples/*.o`, and `original/test/nls/print-localedir.o` when present
  - `original/test/Makefile.am`
  - `original/contrib/examples/Makefile.am`
  - `safe/contrib/examples/*.c`
  - `safe/tests/smoke/public-api-smoke.c`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_04_source_link_compat/` with fresh package metadata and validated overlay contents
  - Updated link-compat manifests or harness rules if new coverage is required
  - Updated library fixes exposed by relink or runtime mismatches
  - Updated compile-smoke source list if a missed public surface needs explicit coverage

## File Changes
  - `safe/tests/run-c-compile-smoke.sh`
  - `safe/tests/run-original-object-link-compat.sh`
  - `safe/tests/link-compat/{object-manifest.txt,run-manifest.txt}`
  - `safe/tests/smoke/public-api-smoke.c`
  - `safe/contrib/examples/*.c`
  - `safe/src/**` only as required by compatibility failures

## Implementation Details
  - Consume the existing object manifests. Do not weaken this phase into source-only recompilation.
  - If original object or wrapper artifacts are missing, regenerate them only via the existing `original/` build targets that the current harness already uses.
  - Reuse the fixed phase-local package root for every package-backed command in this phase. Do not create a new temp package build for each wrapper script.
  - Run `PACKAGE_BUILD_ROOT=... bash tests/run-package-build.sh` once, then rerun the compile and relink wrappers under `LIBEXIF_REQUIRE_REUSE=1`. Both wrappers must fail with the exact substring `reuse-required package root is missing or stale` when pointed at a deliberately missing root under `LIBEXIF_REQUIRE_REUSE=1`. Do not hide a rebuild behind the nested wrapper call.
  - Preserve the manifest-level comparison policy:
    - `exact_streams` unless a plan-documented reason requires pointer normalization
    - `normalize_hex_pointers_in_streams` only for known pointer-printing cases such as `test-fuzzer`
  - Preserve byte-for-byte output-file comparison for `test-extract`, `thumbnail`, `write-exif`, and any new file-producing entry added later.
  - If a relink failure reveals an ABI-visible behavior discrepancy, add the smallest possible new manifest entry or local regression in addition to the library fix.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_04_link_tester` (`check`, `bounce_target: impl_04_source_link_compat`)
    - Purpose: verify that source-compiled clients and already-built original object files still compile, relink, and behave identically against `libexif-safe` while reusing one phase-local package root, and that the relink wrapper honors reuse-only mode.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat
      missing_root_log=$(mktemp)
      missing_link_log=$(mktemp)
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      missing_root="${PACKAGE_BUILD_ROOT}.missing"
      trap 'rm -f "$missing_root_log" "$missing_link_log"' EXIT
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh
      test -x "$static_smoke"
      "$static_smoke"
      ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-original-object-link-compat.sh
      rm -rf "$missing_root"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_root" bash tests/run-c-compile-smoke.sh >"$missing_root_log" 2>&1; then
        cat "$missing_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_root" bash tests/run-original-object-link-compat.sh >"$missing_link_log" 2>&1; then
        cat "$missing_link_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_link_log"
      ```
  - `check_04_link_senior` (`check`, `bounce_target: impl_04_source_link_compat`)
    - Purpose: review manifest changes, confirm comparison modes were not weakened, and confirm the link-compat harness reuses the phase-local package root.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh
      test -x "$static_smoke"
      ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-original-object-link-compat.sh
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat bash tests/run-package-build.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat bash tests/run-c-compile-smoke.sh`
  - `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat/compile-smoke/static/public-api-smoke-static` must exist, execute successfully, and have no `NEEDED` entry for `libexif.so.12` in `readelf -d`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat bash tests/run-original-object-link-compat.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat.missing bash tests/run-c-compile-smoke.sh` must fail with `reuse-required package root is missing or stale`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_04_source_link_compat.missing bash tests/run-original-object-link-compat.sh` must fail with `reuse-required package root is missing or stale`

## Git Commit Requirement
The implementer must commit work to git before yielding.
