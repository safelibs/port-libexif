# Phase Name

Fix CLI Usage, Metadata Formatting, XML, Thumbnail, and Dependent-Client Failures

# Implement Phase ID

`impl_03_fix_cli_usage_metadata_failures`

# Preexisting Inputs

- Phase-1 and phase-2 validator artifacts and report entries, including `validator-report.md`, `validator/artifacts/libexif-safe/**`, and `validator/artifacts/libexif-safe-check02/**`.
- Existing `validator/` checkout created or updated only by phase 1. This phase must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
- `validator/tests/libexif/tests/cases/usage/*.sh` from the existing checkout.
- Validator sample fixtures under `validator/tests/libexif/tests/fixtures/samples/`.
- Validator logs/casts and result JSON showing the `exif` CLI usage behavior.
- `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- Local fixtures under `safe/tests/testdata/*`, copied original C/shell tests, smoke C tests, and MakerNote fixtures.
- Existing local harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, and `safe/tests/run-package-build.sh`.
- Public ABI and behavior inputs: `safe/src/**/*`, `safe/include/libexif/*`, `safe/debian/*`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/src/ffi/types.rs`, and exported functions in `safe/src/**/*`.
- Validator reference facts from phase 1: reference validator `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`; Docker-based Ubuntu 24.04 matrix; `original` and package-oriented `port` modes; `port` mode requires `<override-deb-root>/<library>/*.deb` and `--port-deb-lock`; this validator revision has no direct library-path mode.
- Validator libexif manifest facts: 5 source cases and 130 usage cases, 135 total; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`; usage failures are libexif ABI/behavior regressions unless evidence proves a validator bug.

# New Outputs

- Minimal local regressions for each usage-class failure.
- Fixed value formatting, save/load, thumbnail, MakerNote, tag, XML, machine-readable, or dependent-client behavior in `safe/src`.
- Refreshed `safe/.artifacts/impl_09_final_release` package artifacts for the phase commit.
- Re-copied canonical override `.deb`s under `validator/artifacts/debs/local/libexif/`.
- Regenerated `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the phase commit.
- Updated `validator-report.md` with usage-class dispositions and retest results.
- One parent-repo commit.

# File Changes

- Prefer `safe/tests/validator_regressions.rs` for direct ABI regressions that model `exif` CLI expectations.
- Use `safe/tests/cve_regressions.rs` only for malformed input or CVE-like safety regressions that also appear in usage logs.
- Likely implementation files: `safe/src/object/entry.rs`, `safe/src/tables/tag_table.rs`, `safe/src/parser/data_load.rs`, `safe/src/parser/data_save.rs`, `safe/src/mnote/*.rs`, and `safe/src/primitives/utils.rs`.
- Always update `validator-report.md`.
- Do not modify validator scripts for legitimate usage failures.
- Treat `validator/`, `validator/artifacts/`, `safe/.artifacts/`, and `safe/target/` as non-committed run/build artifacts.

# Implementation Details

- Execute linearly. Do not introduce `parallel_groups`, workflow indirection, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or `bounce_targets` lists. This phase's verifiers are explicit top-level `check` phases with fixed `bounce_target: impl_03_fix_cli_usage_metadata_failures`.
- Start only from failures assigned to this phase in `validator-report.md`. Consume existing prior artifacts instead of regenerating setup artifacts.
- Preserve the package artifact contract: refresh the package root only with `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`; check phases must use `LIBEXIF_REQUIRE_REUSE=1` when invoking `safe/tests/run-package-build.sh` or package-backed harnesses.
- Use validator `port` mode with local override packages. Do not run `make fetch-port-debs`; do not use direct library-path mode; do not add noncanonical `.deb`s unless a validator log proves another canonical package is required.
- Maintain the local lock at `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` with schema `1`, mode `port`, one `libexif` library entry, `repository: local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit: <current parent HEAD>`, `release_tag: local-<short-parent-commit>`, `tag_ref: refs/tags/local-<short-parent-commit>`, `unported_original_packages: []`, canonical apt-order `debs` entries for `libexif12` and `libexif-dev`, exact filenames, `Architecture` from `dpkg-deb --field`, lowercase SHA256, byte size, `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"`.
- Inspect validator logs for each usage failure and map the failing `exif` CLI behavior to underlying libexif ABI calls.
- Add direct Rust/C ABI regressions using the same fixture and tag/value expectation whenever practical. Do not add tests that shell out to the validator from `safe/`.
- Every new regression test name or comment must reference the validator testcase id or usage failure class it covers.
- Preserve upstream behavior by checking `original/` code and copied original tests before changing formatting strings or tag tables.
- For XML and machine-readable failures, test the underlying tag names, IFD names, values, and escaping-sensitive strings produced by libexif APIs.
- For thumbnail and set/remove/readback failures, add regressions around `exif_data_save_data`, `exif_data_new_from_data`, `exif_content_remove_entry`, and thumbnail fields as appropriate.
- If no usage-class failures exist, update `validator-report.md` with that no-op disposition and commit it. Use `git commit --allow-empty` only if the report already contains the exact disposition and no file changed.
- Commit before yielding, then refresh package root, recopy override `.deb`s, and regenerate the local lock for the new `HEAD`.
- Validator `port` mode can return success even when testcase failures are recorded. Inspect the relevant `*/port/results/libexif/summary.json` and result JSON files explicitly instead of trusting process status.
- Every post-commit checker must be able to assert: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every checked result JSON has `port_commit` equal to current parent `HEAD`.

# Verification Phases

## `check_03_usage_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_03_fix_cli_usage_metadata_failures`
- Purpose: Confirm usage-class failures are locally covered and no non-safety usage cases still fail.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `cargo test --manifest-path safe/Cargo.toml --release`
  - `bash safe/tests/run-original-test-suite.sh`
  - `bash safe/tests/run-cve-regressions.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-check03 --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
  - `cd validator && jq -e '.source_cases >= 5 and .usage_cases >= 130 and (.passed + .failed == .cases)' artifacts/libexif-safe-check03/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe-check03/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe-check03/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .status=="failed" and .kind=="usage" and (((.testcase_id // "") | test("invalid|truncated|corrupt|malformed|timeout|crash|no-exif-error"; "i")) | not)) | .testcase_id' artifacts/libexif-safe-check03/port/results/libexif/*.json)"`

## `check_03_usage_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_03_fix_cli_usage_metadata_failures`
- Purpose: Review whether usage fixes preserve upstream libexif semantics instead of hard-coding validator strings.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git show --stat --format=fuller HEAD`
  - `git diff HEAD~1..HEAD -- safe/src safe/tests validator-report.md`
  - `rg -n 'validator|usage-exif-cli|EXIF_TAG_|mnote_|thumbnail|xml|machine' safe/tests validator-report.md`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-export-compare.sh`
  - `jq -e '.source_cases >= 5 and .usage_cases >= 130 and (.passed + .failed == .cases)' validator/artifacts/libexif-safe-check03/port/results/libexif/summary.json`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-check03/port/results/libexif/*.json)"`
  - `test -z "$(jq -r 'select(.testcase_id and .status=="failed" and .kind=="usage" and (((.testcase_id // "") | test("invalid|truncated|corrupt|malformed|timeout|crash|no-exif-error"; "i")) | not)) | .testcase_id' validator/artifacts/libexif-safe-check03/port/results/libexif/*.json)"`

# Success Criteria

- Relevant local ABI/behavior regressions pass and reference the validator testcase id or usage failure class they cover.
- No usage failures assigned to this phase remain in `validator/artifacts/libexif-safe-check03/port/results/libexif/*.json`.
- Any remaining usage failures are only malformed/crash/timeout/safety-class cases intentionally left for phase 4 or documented validator-bug candidates left for phase 5.
- Fixes preserve upstream libexif semantics for tag names, values, IFD names, XML/machine-readable output, thumbnail behavior, and save/load behavior.
- Package/result/lock provenance is tied to current parent `HEAD`.
- Parent repo commit contains only `validator-report.md` and necessary `safe/` files; it excludes nested validator files and run artifacts.

# Git Commit Requirement

The implementer must commit work to git before yielding. If no usage-class failures exist, commit the `validator-report.md` no-op disposition, using `git commit --allow-empty` only when the exact disposition is already present and no file changed; immediately after the commit, refresh the package root, recopy canonical override `.deb`s, and regenerate the local lock for the new `HEAD`.
