# Phase Name

Final Validator Run, Proof, Site Check, and Report

# Implement Phase ID

`impl_06_final_validator_report`

# Preexisting Inputs

- All safe fixes and local regressions from prior phases.
- Refreshed `safe/.artifacts/impl_09_final_release`.
- Existing `validator/` checkout created or updated only by phase 1. This phase must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
- Existing local override root `validator/artifacts/debs/local/libexif/`.
- Existing local lock `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`.
- Phase-5 clean validator artifact root `validator/artifacts/libexif-safe-check05/`.
- `validator-report.md`.
- `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- Existing local harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, and `safe/tests/run-package-build.sh`.
- Validator reference facts from phase 1: reference validator `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`; Docker-based Ubuntu 24.04 matrix; `original` and package-oriented `port` modes; `port` mode requires `<override-deb-root>/<library>/*.deb` and `--port-deb-lock`; this validator revision has no direct library-path mode.
- Validator libexif manifest facts: 5 source cases and 130 usage cases, 135 total; source case ids `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting`; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`.

# New Outputs

- Final `validator-report.md` summary committed before the post-commit final run.
- `validator/artifacts/libexif-safe-final/**` post-commit final run artifacts.
- `validator/artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json`.
- `validator/artifacts/libexif-safe-final-site/**` rendered site if proof succeeds.
- One parent-repo commit.

# File Changes

- `validator-report.md`.
- No `safe/` source changes in this phase. `impl_05_fix_remaining_or_validator_bugs` was the last phase allowed to make libexif-safe behavior changes for validator failures.
- If the final run finds a libexif-safe behavior failure, document the failure in `validator-report.md`, commit the report, and let the fixed-bounce final checker fail. Do not bounce to an earlier implement phase.
- Treat `validator/`, `validator/artifacts/`, `safe/.artifacts/`, and `safe/target/` as non-committed run/build artifacts.

# Implementation Details

1. Execute linearly. Do not introduce `parallel_groups`, workflow indirection, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or `bounce_targets` lists. This phase's verifiers are explicit top-level `check` phases with fixed `bounce_target: impl_06_final_validator_report`.
2. Preserve the package artifact contract: refresh the package root only with `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`; check phases must use `LIBEXIF_REQUIRE_REUSE=1` when invoking `safe/tests/run-package-build.sh` or package-backed harnesses.
3. Use validator `port` mode with local override packages. Do not run `make fetch-port-debs`; do not use direct library-path mode; do not add noncanonical `.deb`s unless a validator log proves another canonical package is required.
4. Maintain the local lock at `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` with schema `1`, mode `port`, one `libexif` library entry, `repository: local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit: <current parent HEAD>`, `release_tag: local-<short-parent-commit>`, `tag_ref: refs/tags/local-<short-parent-commit>`, `unported_original_packages: []`, canonical apt-order `debs` entries for `libexif12` and `libexif-dev`, exact filenames, `Architecture` from `dpkg-deb --field`, lowercase SHA256, byte size, `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"`.
5. Confirm phase 5 produced a clean validator result and current package/lock provenance.
6. Run a candidate final matrix and proof into `validator/artifacts/libexif-safe-final-candidate` using the current lock. If it fails, update `validator-report.md` with the failure evidence and do not claim final success.
7. Update `validator-report.md` with stable final content: validator commit, safe fix commits, package artifact filenames and SHA256 values, commands executed, original baseline summary, candidate/final artifact paths, failures found, fixes applied, validator bug waivers or `None`, and a final clean-run statement that names `validator/artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json` as the post-commit proof path.
8. Final success requires a clean full libexif validator run against the local override packages, a proof artifact for that same run, proof/result `port_commit` equal to current parent `HEAD`, and a `validator-report.md` summary containing validator commit, commands executed, testcase counts, failures found, fixes applied, skipped validator bugs if any, and final status.
9. Commit `validator-report.md` before yielding any final result.
10. Immediately after that commit, refresh `safe/.artifacts/impl_09_final_release`, recopy canonical `.deb`s, and regenerate `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the final parent `HEAD`.
11. Remove stale final artifacts:
    - `cd validator && rm -rf artifacts/libexif-safe-final artifacts/libexif-safe-final-site`
12. Rerun the final local override matrix after the report commit:
    - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
    - `cd validator && jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' artifacts/libexif-safe-final/port/results/libexif/summary.json`
13. Generate the post-commit proof:
    - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --proof-output artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
14. Render and verify the review site when proof succeeds:
    - `cd validator && python3 tools/render_site.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --proof-path artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --output-root artifacts/libexif-safe-final-site`
    - `cd validator && bash scripts/verify-site.sh --config repositories.yml --tests-root tests --artifacts-root artifacts/libexif-safe-final --proof-path artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --site-root artifacts/libexif-safe-final-site --library libexif`
15. Do not edit or recommit `validator-report.md` after the post-commit final run. Step 7 must word the report so the post-commit artifact path and checker assertions complete provenance without another commit cycle.
16. Validator `port` mode can return success even when testcase failures are recorded. Inspect `validator/artifacts/libexif-safe-final/port/results/libexif/summary.json` and result JSON files explicitly.
17. Every post-commit checker must be able to assert: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every checked result JSON has `port_commit` equal to current parent `HEAD`; proof has `libraries[] | select(.library=="libexif") | .port_commit` equal to current parent `HEAD`.

# Verification Phases

## `check_06_final_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_06_final_validator_report`
- Purpose: Independently reproduce the final validator run and proof against current `HEAD`.
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
  - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final-check --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
  - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final-check --proof-output artifacts/libexif-safe-final-check/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
  - `cd validator && jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' artifacts/libexif-safe-final-check/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe-final-check/port/results/libexif/*.json)"`
  - `jq -e --arg head "$(git rev-parse HEAD)" '.libraries[] | select(.library=="libexif") | .port_commit == $head' validator/artifacts/libexif-safe-final-check/proof/libexif-safe-validation-proof.json`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe-final-check/port/results/libexif/*.json)"`

## `check_06_final_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_06_final_validator_report`
- Purpose: Final review of report completeness, committed scope, post-commit final artifacts, and clean run claims.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git log --oneline --decorate -n 12`
  - `git status --short --ignored`
  - `git diff --name-only HEAD~6..HEAD || true`
  - `grep -E 'Validator commit|Checks executed|Failures found|Fixes applied|Final clean run|Validator bug waivers' validator-report.md`
  - `git -C validator rev-parse HEAD`
  - `git -C validator status --short`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' validator/artifacts/libexif-safe-final/port/results/libexif/summary.json`
  - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-final/port/results/libexif/*.json)"`
  - `test -f validator/artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json`
  - `jq -e --arg head "$(git rev-parse HEAD)" '.libraries[] | select(.library=="libexif") | .port_commit == $head' validator/artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json`

# Success Criteria

- Final post-commit result summary reports zero failures and all expected cases passed.
- Final proof exists and has `port_commit` equal to current parent `HEAD`.
- Final rendered site is produced by `python3 tools/render_site.py` and validated by `bash scripts/verify-site.sh`.
- Final report includes validator commit, checks executed, failures found, fixes applied, validator bug waivers, and final status.
- No nested validator artifacts are committed in the parent repo.

# Critical Files

- `.gitignore`: must ignore `/validator/`, `/safe/.artifacts/`, and `/safe/target/`.
- `validator-report.md`: primary persistent report for setup, validator commit, commands, testcase counts, original baseline, local override results, failure classes, regression tests, fixes, validator bug waivers, package artifacts, proof path, and final status.
- `safe/tests/validator_regressions.rs`: preferred integration test file for validator-derived ABI and behavior regressions that do not belong to existing CVE tests.
- `safe/tests/cve_regressions.rs`: extend only for malformed-input or CVE-like validator regressions.
- `safe/tests/smoke/public-api-smoke.c`, `safe/tests/smoke/validator-compile-link.c` if added, and `safe/tests/run-c-compile-smoke.sh`: extend only for compile/link/header/pkg-config regressions.
- `safe/src/parser/limits.rs`, `safe/src/parser/data_load.rs`, `safe/src/parser/loader.rs`, and `safe/src/parser/data_save.rs`: parser resource limits, JPEG/EXIF payload discovery, loader behavior, serialization, and malformed input handling.
- `safe/src/object/data.rs`, `safe/src/object/content.rs`, and `safe/src/object/entry.rs`: object graph ownership, entry lookup/removal, formatting, fix-up, thumbnail state, and public ABI behavior.
- `safe/src/tables/tag_table.rs` and `safe/src/primitives/utils.rs`: tag metadata, IFD-specific lookup behavior, and byte-order scalar helpers.
- `safe/src/mnote/*.rs`: MakerNote load/save/count/name/title/description/value behavior.
- `safe/src/runtime/mem.rs` and `safe/src/runtime/log.rs`: allocator and logging behavior if validator exposes ownership, callback, or variadic logging regressions.
- `safe/include/libexif/*.h`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/debian/*`, and `safe/build.rs`: package, header, generated symbol, and install surface for compile/link and validator Docker override failures.
- `safe/.artifacts/impl_09_final_release`: reusable package artifact root, refreshed by script but never committed.
- `validator/`: nested external validator checkout. It is used for runs and may contain local artifacts or one narrowly scoped local skip for a proven validator bug, but it is never committed in the parent repo.

# Final Verification

Run from `/home/yans/safelibs/pipeline/ports/port-libexif` after all implementation phases.

1. Local safe validation:
   - `cargo test --manifest-path safe/Cargo.toml --release`
   - `bash safe/tests/run-cve-regressions.sh`
   - `bash safe/tests/run-original-test-suite.sh`
   - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-c-compile-smoke.sh`
   - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-export-compare.sh`
   - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
2. Provenance assertions:
   - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
   - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
   - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
3. Validator final run:
   - `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --check --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
   - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
   - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --proof-output artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
   - `cd validator && python3 tools/render_site.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe-final --proof-path artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --output-root artifacts/libexif-safe-final-site`
   - `cd validator && bash scripts/verify-site.sh --config repositories.yml --tests-root tests --artifacts-root artifacts/libexif-safe-final --proof-path artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json --site-root artifacts/libexif-safe-final-site --library libexif`
4. Result assertions:
   - `jq -e '.failed == 0 and .passed == .cases and .source_cases >= 5 and .usage_cases >= 130' validator/artifacts/libexif-safe-final/port/results/libexif/summary.json`
   - `test -z "$(jq -r --arg head "$(git rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' validator/artifacts/libexif-safe-final/port/results/libexif/*.json)"`
   - `jq -e --arg head "$(git rev-parse HEAD)" '.libraries[] | select(.library=="libexif") | .port_commit == $head' validator/artifacts/libexif-safe-final/proof/libexif-safe-validation-proof.json`
   - `grep -E 'Validator commit|Checks executed|Failures found|Fixes applied|Final clean run|Validator bug waivers' validator-report.md`
5. Git hygiene:
   - `git status --short --ignored`
   - Confirm parent commits include only `.gitignore`, `validator-report.md`, and the safe files needed for tested fixes.

# Git Commit Requirement

The implementer must commit work to git before yielding. Commit `validator-report.md` before the post-commit final run, then refresh the package root, recopy canonical override `.deb`s, regenerate the local lock for the new `HEAD`, run the final matrix/proof/site commands, and do not edit or recommit `validator-report.md` afterward.
