# Safety, Performance, and Documentation Hardening

## Phase Name
`Safety, Performance, and Documentation Hardening`

## Implement Phase ID
`impl_08_safety_perf_docs`

## Preexisting Inputs
  - `safe/SAFETY.md`
  - `safe/README`
  - `safe/NEWS`
  - `safe/build.rs`
  - `safe/cshim/exif-log-shim.c`
  - `safe/src/**`
  - `safe/tests/run-package-build.sh`
  - `safe/tests/run-performance-compare.sh`
  - `safe/tests/perf/{bench-driver.c,fixture-manifest.txt,thresholds.env}`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_08_safety_perf_docs/` with fresh package metadata and validated overlay contents
  - Updated `safe/SAFETY.md`
  - Updated public-facing documentation that no longer claims the implementation is plain C
  - Performance-driven code or harness adjustments if the Rust completion work regresses throughput
  - Further unsafe reductions if low-risk cleanup remains after earlier phases

## File Changes
  - `safe/SAFETY.md`
  - `safe/README`
  - `safe/NEWS`
  - `safe/build.rs`
  - `safe/cshim/exif-log-shim.c`
  - `safe/src/**`
  - `safe/tests/run-performance-compare.sh`
  - `safe/tests/perf/{bench-driver.c,fixture-manifest.txt,thresholds.env}`

## Implementation Details
  - Remove any newly unnecessary `unsafe` introduced during compatibility fixes. The final unsafe surface should be limited to true ABI or FFI boundaries and other clearly unavoidable cases.
  - Update `safe/SAFETY.md` so it matches the actual post-phase code, including whether the vendor MakerNote helper C sources are fully gone.
  - Update shipped docs such as `safe/README` so they accurately describe the Rust implementation and its remaining narrow foreign boundaries. The current wording that the library is written in plain C is stale.
  - Use the existing performance harness and thresholds as the contract. Reuse the phase-local package root for performance runs instead of rebuilding the same packaged library again.
  - Run one explicit `run-package-build.sh` call followed by `LIBEXIF_REQUIRE_REUSE=1 bash tests/run-performance-compare.sh`, plus a deliberate missing-root rerun of the performance wrapper that must fail with `reuse-required package root is missing or stale`. Do not let the performance wrapper silently rebuild the package root.
  - If a threshold needs to change, document why in both the code review and `safe/NEWS`, and keep the change backed by measured data rather than convenience.
  - Preserve or improve the current thresholds encoded in `safe/tests/perf/thresholds.env`; do not relax them without measured justification captured in both the code review and `safe/NEWS`.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_08_hardening_tester` (`check`, `bounce_target: impl_08_safety_perf_docs`)
    - Purpose: verify that the remaining `unsafe` is justified, performance remains within thresholds, the shipped documentation describes the Rust implementation accurately, and the performance wrapper honors reuse-only mode.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_08_safety_perf_docs
      missing_perf_log=$(mktemp)
      missing_root="${PACKAGE_BUILD_ROOT}.missing"
      trap 'rm -f "$missing_perf_log"' EXIT
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      rg -n "\\bunsafe\\b" src tests build.rs || true
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-performance-compare.sh
      rm -rf "$missing_root"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_root" bash tests/run-performance-compare.sh >"$missing_perf_log" 2>&1; then
        cat "$missing_perf_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_perf_log"
      cargo test --release
      ! rg -n "written in plain C|does not require any additional library" README SAFETY.md NEWS
      ```
  - `check_08_hardening_senior` (`check`, `bounce_target: impl_08_safety_perf_docs`)
    - Purpose: audit each remaining `unsafe` category against `safe/SAFETY.md`, confirm that the MakerNote port reduced foreign implementation code, and review any performance-threshold or documentation changes.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_08_safety_perf_docs
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-performance-compare.sh
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_08_safety_perf_docs bash tests/run-package-build.sh`
  - `rg -n "\\bunsafe\\b" src tests build.rs || true`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_08_safety_perf_docs bash tests/run-performance-compare.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_08_safety_perf_docs.missing bash tests/run-performance-compare.sh` must fail with `reuse-required package root is missing or stale`
  - `cargo test --release`
  - `! rg -n "written in plain C|does not require any additional library" README SAFETY.md NEWS`

## Git Commit Requirement
The implementer must commit work to git before yielding.
