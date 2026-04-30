# Phase Name

Fix Source API, ABI, Packaging, and C-Level Validator Failures

# Implement Phase ID

`impl_02_fix_source_api_validator_failures`

# Preexisting Inputs

- `validator-report.md` with the phase-1 failure table and source-class assignments.
- Existing `validator/` checkout created or updated only by phase 1. This phase must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
- Phase-1 validator artifacts, including `validator/artifacts/libexif-safe/**`, `validator/artifacts/libexif-original/**` when Docker was available, result JSON logs/casts, and `validator/artifacts/libexif-safe/proof/libexif-safe-validation-proof.json`.
- `validator/artifacts/debs/local/libexif/*.deb` containing the canonical local `libexif12_*.deb` and `libexif-dev_*.deb`.
- `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`.
- `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- `safe/src/**/*`, `safe/include/libexif/*`, `safe/debian/*`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/build.rs`, `safe/Cargo.toml`, and `safe/Cargo.lock`.
- Existing local harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, `safe/tests/run-package-build.sh`, copied original C/shell tests, smoke C tests, and MakerNote fixtures under `safe/tests/testdata/*`.
- Upstream ABI evidence in `original/libexif/libexif.sym`, `original/debian/libexif12.symbols`, `original/test/*`, and generated artifacts produced by `safe/build.rs`.
- Validator reference facts from phase 1: reference validator `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`; Docker-based Ubuntu 24.04 matrix; `original` and package-oriented `port` modes; `port` mode requires `<override-deb-root>/<library>/*.deb` and `--port-deb-lock`; this validator revision has no direct library-path mode.
- Validator libexif manifest facts: 5 source cases and 130 usage cases, 135 total; source case ids `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting`; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`.

# New Outputs

- Minimal local regression tests for each fixed source-class failure.
- Fixed `libexif-safe` source, header, pkg-config, symbol, or packaging behavior as needed.
- Refreshed `safe/.artifacts/impl_09_final_release` package artifacts for the phase commit.
- Re-copied canonical override `.deb`s under `validator/artifacts/debs/local/libexif/`.
- Regenerated `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the phase commit.
- Updated `validator-report.md` with source-class dispositions and retest results.
- One parent-repo commit.

# File Changes

- Prefer source-class regressions in `safe/tests/validator_regressions.rs`.
- For compile/link or header/pkg-config failures, add `safe/tests/smoke/validator-compile-link.c` when needed and wire it into `safe/tests/run-c-compile-smoke.sh`.
- Conditional implementation files: `safe/include/libexif/*.h`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/debian/*`, `safe/build.rs`, `safe/src/tables/tag_table.rs`, `safe/src/object/*`, `safe/src/parser/*`, `safe/src/primitives/utils.rs`, and `safe/src/mnote/*.rs`.
- Always update `validator-report.md`.
- Do not modify the validator suite to make legitimate libexif-safe failures pass.
- Treat `validator/`, `validator/artifacts/`, `safe/.artifacts/`, and `safe/target/` as non-committed run/build artifacts.

# Implementation Details

- Execute linearly. Do not introduce `parallel_groups`, workflow indirection, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or `bounce_targets` lists. This phase's verifiers are explicit top-level `check` phases with fixed `bounce_target: impl_02_fix_source_api_validator_failures`.
- Start only from failures assigned to this phase in `validator-report.md`. Consume existing phase-1 artifacts instead of rediscovering or regenerating setup artifacts.
- Preserve the package artifact contract: refresh the package root only with `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`; check phases must use `LIBEXIF_REQUIRE_REUSE=1` when invoking `safe/tests/run-package-build.sh` or package-backed harnesses.
- Use validator `port` mode with local override packages. Do not run `make fetch-port-debs`; do not use direct library-path mode; do not add noncanonical `.deb`s unless a validator log proves another canonical package is required.
- Maintain the local lock at `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` with schema `1`, mode `port`, one `libexif` library entry, `repository: local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit: <current parent HEAD>`, `release_tag: local-<short-parent-commit>`, `tag_ref: refs/tags/local-<short-parent-commit>`, `unported_original_packages: []`, canonical apt-order `debs` entries for `libexif12` and `libexif-dev`, exact filenames, `Architecture` from `dpkg-deb --field`, lowercase SHA256, byte size, `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"`.
- Reproduce each source failure locally before fixing when practical.
- Add traceable regression coverage for `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting` as applicable.
- Every new regression test name or comment must reference the validator testcase id or failure class it covers.
- Fix the underlying ABI, parser, object, table, MakerNote, header, or package boundary. Do not edit validator scripts for legitimate libexif-safe failures.
- If no source-class failures exist, update `validator-report.md` with that no-op disposition and commit it. Use `git commit --allow-empty` only if the report already contains the exact disposition and no file changed.
- After edits, run relevant local tests, refresh packages, recopy override `.deb`s, rerun the full libexif validator matrix, update `validator-report.md`, and commit before yielding.
- Validator `port` mode can return success even when testcase failures are recorded. Inspect the relevant `*/port/results/libexif/summary.json` and result JSON files explicitly instead of trusting process status.
- Immediately after the parent-repo commit, refresh `safe/.artifacts/impl_09_final_release`, recopy canonical override `.deb`s, and regenerate the local lock for the new `HEAD`. This applies to report-only commits because the package script keys freshness to `HEAD`.
- Every post-commit checker must be able to assert: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every checked result JSON has `port_commit` equal to current parent `HEAD`.

# Verification Phases

## `check_02_source_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_02_fix_source_api_validator_failures`
- Purpose: Confirm source-class validator failures have local regressions, pass locally, and no source cases still fail.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `test ! -f safe/tests/validator_regressions.rs || cargo test --manifest-path safe/Cargo.toml --test validator_regressions -- --test-threads=1`
  - `cargo test --manifest-path safe/Cargo.toml --release`
  - `bash safe/tests/run-cve-regressions.sh`
  - `bash safe/tests/run-original-test-suite.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-c-compile-smoke.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-check02 --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
  - `cd validator && jq -e '.source_cases >= 5 and .usage_cases >= 130 and (.passed + .failed == .cases)' artifacts/libexif-safe-check02/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe-check02/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe-check02/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .status=="failed" and .kind=="source") | .testcase_id' artifacts/libexif-safe-check02/port/results/libexif/*.json)"`

## `check_02_source_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_02_fix_source_api_validator_failures`
- Purpose: Review source fixes for ABI compatibility, package surface correctness, regression traceability, and non-overfitting.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git show --stat --format=fuller HEAD`
  - `git diff HEAD~1..HEAD -- safe/src safe/tests safe/include safe/debian safe/Cargo.toml safe/build.rs validator-report.md`
  - `rg -n 'validator|compile-link-smoke|invalid-data-handling|jpeg-exif-c-api-parse|maker-note-handling|tag-lookup-value-formatting' safe/tests validator-report.md`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-export-compare.sh`
  - `objdump -T safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4 | rg 'exif_data_new|exif_loader_write|exif_entry_get_value|exif_tag_get_name_in_ifd|exif_mnote_data_canon_new'`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-check02/port/results/libexif/*.json)"`
  - `test -z "$(jq -r 'select(.testcase_id and .status=="failed" and .kind=="source") | .testcase_id' validator/artifacts/libexif-safe-check02/port/results/libexif/*.json)"`

# Success Criteria

- New local regressions pass and reference the relevant validator testcase id or source failure class.
- All source-class failures assigned in `validator-report.md` are fixed or explicitly reported as no-op because none existed.
- No result in `validator/artifacts/libexif-safe-check02/port/results/libexif/*.json` has `status=="failed"` and `kind=="source"`.
- Header, pkg-config, symbol, ABI, parser, object, table, and MakerNote fixes preserve upstream libexif behavior and do not overfit validator scripts.
- Package/result/lock provenance is tied to current parent `HEAD`.
- Parent repo commit contains only `validator-report.md` and necessary `safe/` files; it excludes nested validator files and run artifacts.

# Git Commit Requirement

The implementer must commit work to git before yielding. If no source-class failures exist, commit the `validator-report.md` no-op disposition, using `git commit --allow-empty` only when the exact disposition is already present and no file changed; immediately after the commit, refresh the package root, recopy canonical override `.deb`s, and regenerate the local lock for the new `HEAD`.
