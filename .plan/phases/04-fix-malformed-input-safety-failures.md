# Phase Name

Fix Malformed Input, Crash, Timeout, and Memory-Safety Validator Failures

# Implement Phase ID

`impl_04_fix_malformed_input_safety_failures`

# Preexisting Inputs

- Failure table entries in `validator-report.md` classified as malformed input, crash, timeout, resource exhaustion, panic, no-EXIF error, MakerNote safety, or memory-safety regressions.
- Phase-1 through phase-3 validator artifacts and report entries, including `validator/artifacts/libexif-safe/**`, `validator/artifacts/libexif-safe-check02/**`, and `validator/artifacts/libexif-safe-check03/**`.
- Existing `validator/` checkout created or updated only by phase 1. This phase must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
- Validator logs/casts and result JSON for failing malformed-input cases.
- `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- `safe/tests/cve_regressions.rs` and `safe/tests/cve-regressions-manifest.txt`.
- Parser, object, runtime, and MakerNote implementation files: `safe/src/parser/*`, `safe/src/object/*`, `safe/src/runtime/*`, and `safe/src/mnote/*.rs`.
- Existing local harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, `safe/tests/run-package-build.sh`, copied original C/shell tests, smoke C tests, and MakerNote fixtures under `safe/tests/testdata/*`.
- Validator reference facts from phase 1: reference validator `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`; Docker-based Ubuntu 24.04 matrix; `original` and package-oriented `port` modes; `port` mode requires `<override-deb-root>/<library>/*.deb` and `--port-deb-lock`; this validator revision has no direct library-path mode.
- Validator libexif manifest facts: 5 source cases and 130 usage cases, 135 total; source case ids `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting`; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`.

# New Outputs

- Minimal malformed-input regression tests.
- Parser/object/MakerNote/runtime fixes that reject or safely handle the inputs.
- Refreshed `safe/.artifacts/impl_09_final_release` package artifacts for the phase commit.
- Re-copied canonical override `.deb`s under `validator/artifacts/debs/local/libexif/`.
- Regenerated `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the phase commit.
- Updated `validator-report.md` with safety-class dispositions and retest results.
- One parent-repo commit.

# File Changes

- Prefer `safe/tests/cve_regressions.rs` for CVE-like malformed input cases.
- Use `safe/tests/validator_regressions.rs` for non-CVE validator safety cases.
- Likely implementation files: `safe/src/parser/*`, `safe/src/object/*`, `safe/src/mnote/*.rs`, and `safe/src/runtime/mem.rs`.
- Always update `validator-report.md`.
- Do not modify validator scripts for legitimate malformed/crash/timeout/safety failures.
- Treat `validator/`, `validator/artifacts/`, `safe/.artifacts/`, and `safe/target/` as non-committed run/build artifacts.

# Implementation Details

- Execute linearly. Do not introduce `parallel_groups`, workflow indirection, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or `bounce_targets` lists. This phase's verifiers are explicit top-level `check` phases with fixed `bounce_target: impl_04_fix_malformed_input_safety_failures`.
- Start only from failures assigned to this phase in `validator-report.md`. Consume existing prior artifacts instead of regenerating setup artifacts.
- Preserve the package artifact contract: refresh the package root only with `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`; check phases must use `LIBEXIF_REQUIRE_REUSE=1` when invoking `safe/tests/run-package-build.sh` or package-backed harnesses.
- Use validator `port` mode with local override packages. Do not run `make fetch-port-debs`; do not use direct library-path mode; do not add noncanonical `.deb`s unless a validator log proves another canonical package is required.
- Maintain the local lock at `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` with schema `1`, mode `port`, one `libexif` library entry, `repository: local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit: <current parent HEAD>`, `release_tag: local-<short-parent-commit>`, `tag_ref: refs/tags/local-<short-parent-commit>`, `unported_original_packages: []`, canonical apt-order `debs` entries for `libexif12` and `libexif-dev`, exact filenames, `Architecture` from `dpkg-deb --field`, lowercase SHA256, byte size, `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"`.
- Reproduce each failure with the smallest local byte fixture or constructed buffer. Do not rely only on Docker logs.
- Every new regression test name or comment must reference the validator testcase id or malformed/crash/timeout/safety failure class it covers.
- For parser offset/length defects, use checked arithmetic, bounded slice helpers, `ParseBudget`, and explicit IFD bounds rather than wrapping arithmetic.
- For loader failures, verify both incremental `exif_loader_write` and `exif_loader_get_data` behavior.
- For MakerNote malformed data, bound count, component, offset, and payload calculations before allocating or copying.
- If a failure exposes a panic crossing an FFI boundary, fix the underlying unwrap/panic source and ensure exported functions still route through `panic_boundary`.
- If no safety-class failures exist, update `validator-report.md` with that no-op disposition and commit it. Use `git commit --allow-empty` only if the report already contains the exact disposition and no file changed.
- Commit before yielding, then refresh package root, recopy override `.deb`s, and regenerate the local lock for the new `HEAD`.
- Validator `port` mode can return success even when testcase failures are recorded. Inspect the relevant `*/port/results/libexif/summary.json` and result JSON files explicitly instead of trusting process status.
- Every post-commit checker must be able to assert: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every checked result JSON has `port_commit` equal to current parent `HEAD`.

# Verification Phases

## `check_04_safety_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_04_fix_malformed_input_safety_failures`
- Purpose: Verify malformed-input fixes with focused safety regressions and a full validator rerun.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `cargo test --manifest-path safe/Cargo.toml --test cve_regressions -- --test-threads=1`
  - `test ! -f safe/tests/validator_regressions.rs || cargo test --manifest-path safe/Cargo.toml --test validator_regressions -- --test-threads=1`
  - `cargo test --manifest-path safe/Cargo.toml --release`
  - `bash safe/tests/run-cve-regressions.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-check04 --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
  - `cd validator && jq -e '.source_cases >= 5 and .usage_cases >= 130 and (.passed + .failed == .cases)' artifacts/libexif-safe-check04/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe-check04/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe-check04/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .status=="failed" and (((.testcase_id // "") | test("invalid|truncated|corrupt|malformed|timeout|crash|maker-note|no-exif-error"; "i")) or (((.error // "") | test("invalid|truncated|corrupt|malformed|timed out|crash|panic|segmentation"; "i"))))) | .testcase_id' artifacts/libexif-safe-check04/port/results/libexif/*.json)"`

## `check_04_safety_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_04_fix_malformed_input_safety_failures`
- Purpose: Review bounds checks, allocator behavior, panic boundaries, and absence of validator-only bypasses.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git diff HEAD~1..HEAD -- safe/src/parser safe/src/object safe/src/mnote safe/src/runtime safe/tests validator-report.md`
  - `rg -n 'validator|invalid|truncated|corrupt|malformed|timeout|crash|maker-note|no-exif-error|CVE' safe/tests validator-report.md`
  - `rg -n 'checked_add|checked_mul|ParseBudget|MAX_|slice_range|copy_nonoverlapping|from_raw_parts|unwrap\(|panic!' safe/src/parser safe/src/object safe/src/mnote safe/tests`
  - `bash safe/tests/run-original-test-suite.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-check04/port/results/libexif/*.json)"`
  - `test -z "$(jq -r 'select(.testcase_id and .status=="failed" and (((.testcase_id // "") | test("invalid|truncated|corrupt|malformed|timeout|crash|maker-note|no-exif-error"; "i")) or (((.error // "") | test("invalid|truncated|corrupt|malformed|timed out|crash|panic|segmentation"; "i"))))) | .testcase_id' validator/artifacts/libexif-safe-check04/port/results/libexif/*.json)"`

# Success Criteria

- New safety regressions pass and reference the relevant validator testcase id or malformed/crash/timeout/safety failure class.
- No malformed-input, crash, timeout, panic, no-EXIF error, MakerNote safety, or memory-safety failures assigned to this phase remain in `validator/artifacts/libexif-safe-check04/port/results/libexif/*.json`.
- Bounds, allocation, slice, loader, MakerNote, and panic-boundary changes address root causes rather than bypassing validator cases.
- Package/result/lock provenance is tied to current parent `HEAD`.
- Parent repo commit contains only `validator-report.md` and necessary `safe/` files; it excludes nested validator files and run artifacts.

# Git Commit Requirement

The implementer must commit work to git before yielding. If no safety-class failures exist, commit the `validator-report.md` no-op disposition, using `git commit --allow-empty` only when the exact disposition is already present and no file changed; immediately after the commit, refresh the package root, recopy canonical override `.deb`s, and regenerate the local lock for the new `HEAD`.
