# Phase Name

Fix Remaining Failures or Document Validator Bugs

# Implement Phase ID

`impl_05_fix_remaining_or_validator_bugs`

# Preexisting Inputs

- All prior reports and validator artifacts, including `validator-report.md`, `validator/artifacts/libexif-safe/**`, `validator/artifacts/libexif-safe-check02/**`, `validator/artifacts/libexif-safe-check03/**`, and `validator/artifacts/libexif-safe-check04/**`.
- Any validator failures not resolved by phases 2 through 4.
- Existing `validator/` checkout created or updated only by phase 1. This phase must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
- Validator testcase scripts and logs for remaining failures, especially `validator/tests/libexif/tests/cases/**`.
- `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- Existing local harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, `safe/tests/run-package-build.sh`, copied original C/shell tests, smoke C tests, and MakerNote fixtures under `safe/tests/testdata/*`.
- Root-cause code and package surfaces under `safe/src/**/*`, `safe/include/libexif/*`, `safe/debian/*`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, and `safe/build.rs`.
- Validator reference facts from phase 1: reference validator `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`; Docker-based Ubuntu 24.04 matrix; `original` and package-oriented `port` modes; `port` mode requires `<override-deb-root>/<library>/*.deb` and `--port-deb-lock`; this validator revision has no direct library-path mode.
- Validator libexif manifest facts: 5 source cases and 130 usage cases, 135 total; source case ids `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting`; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`; usage failures are libexif ABI/behavior regressions unless evidence proves a validator bug.

# New Outputs

- Additional minimal local regressions and safe fixes for remaining legitimate libexif-safe failures.
- Or a narrowly documented validator bug waiver with evidence and, only if unavoidable, a local scoped skip for a single validator testcase.
- Updated `validator-report.md`.
- Refreshed `safe/.artifacts/impl_09_final_release` package artifacts for the phase commit.
- Re-copied canonical override `.deb`s under `validator/artifacts/debs/local/libexif/`.
- Regenerated `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the phase commit.
- One parent-repo commit.

# File Changes

- Conditional `safe/tests/validator_regressions.rs`, `safe/tests/cve_regressions.rs`, `safe/tests/smoke/*`, and root-cause `safe/src/**/*` files.
- `validator-report.md`.
- Local nested-validator changes are allowed only for a documented validator bug and must be limited to one affected testcase under `validator/tests/libexif/tests/cases/**`; do not stage them in the parent repo.
- Treat `validator/`, `validator/artifacts/`, `safe/.artifacts/`, and `safe/target/` as non-committed run/build artifacts.

# Implementation Details

- Execute linearly. Do not introduce `parallel_groups`, workflow indirection, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or `bounce_targets` lists. This phase's verifiers are explicit top-level `check` phases with fixed `bounce_target: impl_05_fix_remaining_or_validator_bugs`.
- Rerun or inspect the latest validator artifact root and list every remaining failure.
- Preserve the package artifact contract: refresh the package root only with `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`; check phases must use `LIBEXIF_REQUIRE_REUSE=1` when invoking `safe/tests/run-package-build.sh` or package-backed harnesses.
- Use validator `port` mode with local override packages. Do not run `make fetch-port-debs`; do not use direct library-path mode; do not add noncanonical `.deb`s unless a validator log proves another canonical package is required.
- Maintain the local lock at `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` with schema `1`, mode `port`, one `libexif` library entry, `repository: local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit: <current parent HEAD>`, `release_tag: local-<short-parent-commit>`, `tag_ref: refs/tags/local-<short-parent-commit>`, `unported_original_packages: []`, canonical apt-order `debs` entries for `libexif12` and `libexif-dev`, exact filenames, `Architecture` from `dpkg-deb --field`, lowercase SHA256, byte size, `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"`.
- For each failure, decide whether it is a legitimate libexif-safe compatibility/safety issue or a validator bug.
- For legitimate libexif-safe issues, add a minimal regression under `safe/`, fix `safe/`, refresh packages, rerun tests, and report the fix.
- Every new regression test name or comment must reference the validator testcase id or failure class it covers.
- For validator bugs, document original libexif behavior, Ubuntu baseline evidence, upstream source evidence, or testcase-script evidence. Record testcase id, validator commit, script path, observed command, expected behavior, actual behavior, evidence, justification, and why this is a validator bug in `validator-report.md`.
- Do not use broad filters, global skips, or result rewriting. If a skip is required, skip only the single proven-bug testcase, preserve the testcase id and overall case count, and implement the local skip inside that one validator script with waiver output rather than deleting or renaming the case.
- Never stage the nested validator patch in the parent repo.
- This is the final phase allowed to make `libexif-safe` behavior changes for validator failures. It must reach a clean validator summary after any documented single-testcase validator-bug waiver.
- If no remaining failures exist, update `validator-report.md` with that no-op disposition and commit it. Use `git commit --allow-empty` only if the report already contains the exact disposition and no file changed.
- Commit before yielding, then refresh package root, recopy override `.deb`s, and regenerate the local lock for the new `HEAD`.
- Validator `port` mode can return success even when testcase failures are recorded. Inspect the relevant `*/port/results/libexif/summary.json` and result JSON files explicitly instead of trusting process status.
- Every post-commit checker must be able to assert: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every checked result JSON has `port_commit` equal to current parent `HEAD`.

# Verification Phases

## `check_05_remaining_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_05_fix_remaining_or_validator_bugs`
- Purpose: Verify every remaining validator failure is fixed or explicitly waived as a validator bug.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `cargo test --manifest-path safe/Cargo.toml --release`
  - `bash safe/tests/run-cve-regressions.sh`
  - `bash safe/tests/run-original-test-suite.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-c-compile-smoke.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-export-compare.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-check05 --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
  - `cd validator && jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' artifacts/libexif-safe-check05/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe-check05/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe-check05/port/results/libexif/*.json)"`

## `check_05_remaining_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_05_fix_remaining_or_validator_bugs`
- Purpose: Review catch-all fixes and any validator bug waiver for rigor, scope, and traceability.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git show --stat --format=fuller HEAD`
  - `git diff HEAD~1..HEAD`
  - `rg -n 'validator|usage-exif-cli|compile-link-smoke|invalid-data-handling|jpeg-exif-c-api-parse|maker-note-handling|tag-lookup-value-formatting|truncated|malformed|waiver' safe/tests validator-report.md`
  - `grep -n 'Validator bug waivers' -A80 validator-report.md`
  - `git -C validator diff -- tests/libexif || true`
  - `jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' validator/artifacts/libexif-safe-check05/port/results/libexif/summary.json`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-check05/port/results/libexif/*.json)"`

# Success Criteria

- Full local validator matrix passes after any explicitly documented single-testcase validator-bug waiver.
- Every remaining failure has a cited validator log path and disposition in `validator-report.md`.
- Any validator local patch is scoped to one testcase, preserves testcase identity/case count, and is documented with validator commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e` or the actual recorded checkout commit if different.
- Parent repo commit does not include nested validator files.
- Package/result/lock provenance is tied to current parent `HEAD`.
- This phase leaves no known validator failures for phase 6 to fix in `safe/`.

# Git Commit Requirement

The implementer must commit work to git before yielding. If no remaining failures exist, commit the `validator-report.md` no-op disposition, using `git commit --allow-empty` only when the exact disposition is already present and no file changed; immediately after the commit, refresh the package root, recopy canonical override `.deb`s, and regenerate the local lock for the new `HEAD`.
