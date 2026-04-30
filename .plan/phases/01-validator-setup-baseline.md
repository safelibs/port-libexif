# Phase Name

Validator Setup, Package Override, and Baseline Run

# Implement Phase ID

`impl_01_validator_setup_baseline`

# Preexisting Inputs

- `/home/yans/safelibs/pipeline/ports/port-libexif/safe`
- `/home/yans/safelibs/pipeline/ports/port-libexif/original`
- `dependents.json`, `relevant_cves.json`, `all_cves.json`, and `test-original.sh`; consume these existing artifacts in place and do not recollect, rediscover, regenerate, or replace them from scratch.
- `safe/PORT.md`; use only as existing background documentation and do not regenerate it for this goal.
- `safe/Cargo.toml` defines package `libexif-safe` version `0.6.24`, edition 2021, with library target `exif` emitted as `rlib`, `cdylib`, and `staticlib`.
- `safe/build.rs` reads `original/libexif/libexif.sym`, preprocesses `original/libexif/exif-tag.c`, writes generated Rust/linker artifacts into Cargo `OUT_DIR`, and compiles `safe/cshim/exif-log-shim.c`.
- Public C ABI surface inputs: `safe/include/libexif/*`, `safe/debian/*`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/src/ffi/types.rs`, and exported functions in `safe/src/**/*`.
- Existing local tests and harnesses: `safe/tests/*.rs`, `safe/tests/run-cve-regressions.sh`, `safe/tests/run-original-test-suite.sh`, `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, `safe/tests/run-package-build.sh`, copied original C/shell tests, smoke C tests, and MakerNote fixtures under `safe/tests/testdata/*`.
- Existing reusable package artifact root `safe/.artifacts/impl_09_final_release`; `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` records stale commit `2ed681a8f27bf33ab58718465ee44306ce142afe`, so this phase must refresh the package root before copying `.deb` files into the validator override root.
- Remote validator repository `https://github.com/safelibs/validator`.
- Validator reference `main` commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`.
- Validator README/code facts at that revision: Docker-based Ubuntu 24.04 matrix, `original` and `port` modes, package-oriented `port` mode, required `<override-deb-root>/<library>/*.deb`, required `--port-deb-lock`, and no direct library-path mode.
- Local repository facts: reference parent repo HEAD `7e641db0a231ea0acf64cdaed5497909e645b5d7`; unrelated tracked `workflow.yaml` modification; untracked `safe/.artifacts/` and `safe/target/`; no nested validator checkout at `/home/yans/safelibs/pipeline/ports/port-libexif/validator` at planning time; existing `.plan/phases/*` and `.plan/workflow-structure.yaml` were an older documentation workflow and are not inputs to this validator plan.
- Validator manifest facts at commit `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`: 5 source cases and 130 usage cases, 135 total; source case ids are `compile-link-smoke`, `invalid-data-handling`, `jpeg-exif-c-api-parse`, `maker-note-handling`, and `tag-lookup-value-formatting`; usage cases run the Ubuntu `exif` CLI against overridden `libexif12` and `libexif-dev`.
- Docker, `make`, `python3`, `jq`, `git`, `dpkg`, `dpkg-deb`, `dpkg-architecture`, and a working C toolchain.

# New Outputs

- `validator/` nested checkout, cloned or fast-forwarded only in this phase.
- Recorded cloned or pulled validator commit actually used.
- `validator/artifacts/debs/local/libexif/libexif12_*.deb`
- `validator/artifacts/debs/local/libexif/libexif-dev_*.deb`
- `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`
- `validator/artifacts/libexif-original/**` original Ubuntu-package baseline artifacts when Docker is available.
- `validator/artifacts/libexif-safe/**` local override validator artifacts and proof.
- `validator-report.md` initial report with validator commit, commands executed, package artifact provenance, testcase counts, original Ubuntu baseline, local override results, failure table, fix plan by class, validator bug waivers, package-build blocker fixes, and final status.
- Top-level `.gitignore` entries for `/validator/`, `/safe/.artifacts/`, and `/safe/target/` if missing.
- Minimal `safe/` package-build prerequisite fix only if the existing package build blocks all validator execution.
- One parent-repo commit for setup/report work.

# File Changes

- Create or update `.gitignore` with `/validator/`, `/safe/.artifacts/`, and `/safe/target/`.
- Create `validator-report.md`.
- If `safe/tests/run-package-build.sh` or package inputs fail before any validator testcase can run, make the smallest prerequisite fix under `safe/`, add a local smoke/regression check when practical, and record it under `Package-build blocker fixes:` in `validator-report.md`.
- Do not edit `safe/` for validator testcase failures in this phase.
- Do not modify the validator suite to make failures pass. The only exception is a clearly identified validator bug, which is not expected in this setup phase.
- Treat `validator/` and `validator/artifacts/` as run artifacts, not parent-repo deliverables. Do not stage or commit them in the parent repo.
- Do not stage unrelated existing work such as `workflow.yaml`, `safe/.artifacts/`, or `safe/target/`.

# Implementation Details

1. Execute linearly. Do not use `parallel_groups` or any other parallel topology.
2. Use self-contained inline-only YAML when this phase is encoded into workflow YAML. Do not use top-level `include`, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or any other YAML-source indirection. Define every verifier as an explicit top-level `check` phase with exactly one fixed `bounce_target` inside this implement block. Do not use `bounce_targets` lists.
3. Consume existing artifacts in place. Treat `safe/`, `original/`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `safe/PORT.md`, `safe/tests/**/*`, and `safe/.artifacts/impl_09_final_release` as inputs. Do not recollect, rediscover, regenerate, or replace them from scratch.
4. Refresh the package artifact root only with `safe/tests/run-package-build.sh` and `PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release`.
5. Check phases must not refresh package artifacts; any check phase invoking `safe/tests/run-package-build.sh` or a package-backed harness must use `LIBEXIF_REQUIRE_REUSE=1`.
6. Create or update the validator checkout exactly once in this phase at `validator/`. Later phases must use that checkout and must not run `git pull`, `git fetch`, reclone the validator, or run `make fetch-port-debs`.
7. Use validator `port` mode with local override packages. The local override directory is `validator/artifacts/debs/local/libexif/` and must contain only canonical local `libexif12_*.deb` and `libexif-dev_*.deb` unless a validator log proves another canonical package is required.
8. Write the local lock to `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`.
9. The local lock must use schema `1`, mode `port`, and one `libraries` entry for `libexif`.
10. The `libexif` lock entry must set `repository` to `local:/home/yans/safelibs/pipeline/ports/port-libexif/safe`, `commit` to the current parent repo `git rev-parse HEAD`, `release_tag` to `local-<short-parent-commit>`, `tag_ref` to `refs/tags/local-<short-parent-commit>`, `unported_original_packages` to `[]`, and `debs` entries for `libexif12` and `libexif-dev` in canonical apt package order.
11. Each lock `debs` entry must record the package name, exact filename, `Architecture` field from `dpkg-deb --field`, lowercase SHA256, and byte size for the copied file under `validator/artifacts/debs/local/libexif/`.
12. Include top-level lock fields `generated_at: "1970-01-01T00:00:00Z"`, `source_config: "repositories.yml"`, and `source_inventory: "local:/home/yans/safelibs/pipeline/ports/port-libexif/safe"` for clarity even though current validator `run_matrix.py` only requires `schema_version`, `mode`, and `libraries`.
13. Validator `port` mode can return success even when testcase failures are recorded, because failures after successful override installation are validation findings. Every checker that reruns the validator must inspect `*/port/results/libexif/summary.json` and fail explicitly for forbidden failures.
14. Every checker must assert provenance freshness: `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals current parent `HEAD`; the local lock commit equals current parent `HEAD`; copied override `.deb` filenames and SHA256 values match canonical files in `safe/.artifacts/impl_09_final_release/artifacts/`; every result JSON in the checked artifact root has `port_commit` equal to current parent `HEAD`.
15. Any checker that validates a proof must also assert the proof's `libraries[] | select(.library=="libexif") | .port_commit` equals current parent `HEAD`.
16. Record parent repo state with `git status --short --branch` and `git rev-parse HEAD`. Preserve unrelated tracked and untracked work.
17. Check Docker availability with `docker info`. If Docker is unavailable, write `validator-report.md` with the blocker, commit the report, perform the post-commit package/lock refresh if possible, and do not mark the goal complete.
18. Clone or update the validator only here:
    - `git -C validator pull --ff-only` if `validator/.git` exists.
    - `git clone https://github.com/safelibs/validator validator` otherwise.
    - Record `git -C validator rev-parse HEAD` and the validator README mode/override details in `validator-report.md`.
19. Run validator setup checks:
    - `cd validator && make unit`
    - `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --list-summary`
    - `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --check --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
20. Refresh local safe packages:
    - `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
    If this fails before any validator testcase can run, fix the package-build prerequisite in this phase, rerun it, and report the fix.
21. Verify `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` equals `git rev-parse HEAD`.
22. Rebuild `validator/artifacts/debs/local/libexif/` by removing the old leaf, recreating it, and copying only `libexif12_*.deb` and `libexif-dev_*.deb` from `safe/.artifacts/impl_09_final_release/artifacts/`.
23. Generate `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` using the exact schema and fields above.
24. Run original baseline:
    - `cd validator && RECORD_CASTS=1 ARTIFACT_ROOT=artifacts/libexif-original LIBRARY=libexif make matrix-original`
    - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-original --proof-output artifacts/libexif-original/proof/original-validation-proof.json --mode original --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
25. Run local safe port matrix and proof:
    - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
    - `cd validator && jq . artifacts/libexif-safe/port/results/libexif/summary.json`
    - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --proof-output artifacts/libexif-safe/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
26. Populate `validator-report.md` with `Validator checkout`, `libexif-safe package under test`, `Commands executed`, `Original Ubuntu baseline`, `Local override validator run`, `Failure table`, `Fix plan by class`, `Validator bug waivers`, `Package-build blocker fixes`, and `Final status`. Make `validator/artifacts/libexif-safe/**` the authoritative post-commit artifact path and rely on checker assertions for final commit-specific provenance.
27. Build the failure table from `validator/artifacts/libexif-safe/port/results/libexif/*.json`. Classify source API/ABI/package failures to phase 2, ordinary CLI/metadata usage failures to phase 3, malformed/crash/timeout/safety failures to phase 4, and unclassified failures or validator-bug candidates to phase 5.
28. Stage only `.gitignore`, `validator-report.md`, and any phase-owned prerequisite safe fix. Do not stage `validator/`, `safe/.artifacts/`, `safe/target/`, or unrelated files.
29. Immediately after the parent-repo commit, rerun package build for the new `HEAD`, recopy the two canonical `.deb`s, and regenerate the local lock for the new `HEAD`.
30. Remove stale safe baseline run outputs while preserving the local lock:
    - `cd validator && rm -rf artifacts/libexif-safe/port artifacts/libexif-safe/proof/libexif-safe-validation-proof.json`
31. Rerun the local safe port matrix and proof into `validator/artifacts/libexif-safe` using the post-commit lock:
    - `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
    - `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --proof-output artifacts/libexif-safe/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
32. Do not edit or recommit `validator-report.md` after the post-commit setup rerun. The report must point to the artifact path; checker assertions establish exact post-commit provenance.

# Verification Phases

## `check_01_setup_software_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_01_validator_setup_baseline`
- Purpose: Verify validator checkout, package override layout, reuse-only package freshness, lock/result/proof provenance, baseline artifacts, and report structure.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `test -d validator/.git`
  - `git -C validator rev-parse HEAD`
  - `test -f validator-report.md`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `cmp -s safe/.artifacts/impl_09_final_release/metadata/package-inputs.sha256 <(bash safe/tests/run-package-build.sh --print-package-inputs-manifest)`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test -d validator/artifacts/debs/local/libexif`
  - `ls validator/artifacts/debs/local/libexif/libexif12_*.deb validator/artifacts/debs/local/libexif/libexif-dev_*.deb`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --check --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
  - `cd validator && jq -e '.cases >= 135 and .source_cases >= 5 and .usage_cases >= 130 and (.passed + .failed == .cases)' artifacts/libexif-safe/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe/port/results/libexif/*.json)"`
  - `cd validator && test -z "$(jq -r 'select(.testcase_id and .override_debs_installed != true) | .testcase_id' artifacts/libexif-safe/port/results/libexif/*.json)"`
  - `jq -e --arg head "$(git rev-parse HEAD)" '.libraries[] | select(.library=="libexif") | .port_commit == $head' validator/artifacts/libexif-safe/proof/libexif-safe-validation-proof.json`

## `check_01_setup_senior_tester`

- Type: `check`
- Fixed `bounce_target`: `impl_01_validator_setup_baseline`
- Purpose: Review setup scope, artifact contract, no staged validator clone, baseline classification, and any phase-1 prerequisite package-build fix.
- Commands:
  - `cd /home/yans/safelibs/pipeline/ports/port-libexif`
  - `git status --short --ignored`
  - `git show --stat --format=fuller HEAD`
  - `git diff --name-only HEAD~1..HEAD`
  - `test -z "$(git diff --name-only HEAD~1..HEAD | rg -v '^(\.gitignore|validator-report\.md|safe/)' || true)"`
  - `if git diff --name-only HEAD~1..HEAD | rg '^safe/' >/dev/null; then grep -q '^Package-build blocker fixes: ' validator-report.md; fi`
  - `git -C validator status --short`
  - `grep -E 'Validator commit|libexif-safe package|Commands executed|Failure table|Fix plan by class|Final status' validator-report.md`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
  - `test "$(cat safe/.artifacts/impl_09_final_release/metadata/source-commit.txt)" = "$(git rev-parse HEAD)"`
  - `for package in libexif12 libexif-dev; do srcs=$(find safe/.artifacts/impl_09_final_release/artifacts -maxdepth 1 -type f -name "${package}_*.deb" | sort); test "$(printf '%s\n' "$srcs" | sed '/^$/d' | wc -l)" -eq 1; src=$(printf '%s\n' "$srcs"); dst="validator/artifacts/debs/local/libexif/$(basename "$src")"; test -f "$dst"; test "$(sha256sum "$src" | cut -d' ' -f1)" = "$(sha256sum "$dst" | cut -d' ' -f1)"; done`
  - `test "$(jq -r '.libraries[] | select(.library=="libexif") | .commit' validator/artifacts/libexif-safe/proof/local-port-debs-lock.json)" = "$(git rev-parse HEAD)"`
  - `cd validator && jq -e '.cases >= 135 and (.passed + .failed == .cases)' artifacts/libexif-safe/port/results/libexif/summary.json`
  - `cd validator && test -z "$(jq -r --arg head "$(git -C .. rev-parse HEAD)" 'select(.testcase_id and .port_commit != $head) | .testcase_id' artifacts/libexif-safe/port/results/libexif/*.json)"`
  - `jq -e --arg head "$(git rev-parse HEAD)" '.libraries[] | select(.library=="libexif") | .port_commit == $head' validator/artifacts/libexif-safe/proof/libexif-safe-validation-proof.json`

# Success Criteria

- Validator checkout exists at `validator/` and the actual checked-out commit is recorded in `validator-report.md`.
- The validator commit used is compatible with reference `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`, including Ubuntu 24.04 Docker matrix behavior, package-oriented `port` mode, and no direct library-path mode.
- `safe/.artifacts/impl_09_final_release`, copied override `.deb`s, local lock, result JSON, and proof all point to the current parent `HEAD`.
- `validator/artifacts/debs/local/libexif/` contains only canonical local `libexif12_*.deb` and `libexif-dev_*.deb` unless a validator log proves another canonical package is required.
- The local lock contains exact required fields: schema `1`, mode `port`, `repository`, `commit`, `release_tag`, `tag_ref`, `unported_original_packages`, `debs`, `generated_at`, `source_config`, and `source_inventory`.
- `make unit`, testcase summary/check commands, original Ubuntu baseline, local port matrix, and proof verification have run or blockers are explicitly documented.
- Baseline result set contains all current libexif cases unless the report documents a current validator count change.
- Every failed result has a failure class and owning phase.
- Parent commit excludes the nested validator checkout/artifacts, `safe/.artifacts/`, `safe/target/`, and unrelated work.

# Git Commit Requirement

The implementer must commit work to git before yielding. Stage only `.gitignore`, `validator-report.md`, and any phase-owned prerequisite `safe/` fix; after the commit, refresh the package root, recopy canonical override `.deb`s, regenerate `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json` for the new `HEAD`, and rerun the post-commit local port matrix and proof without making a second report commit.
