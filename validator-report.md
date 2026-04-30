# Validator Setup Baseline Report

Phase: `impl_01_validator_setup_baseline`

Validator checkout:
- Path: `validator/`
- Remote: `https://github.com/safelibs/validator`
- Validator commit: `5d908be26e33f071e119ffe1a52e3149f1e5ec4e`
- Compatibility notes: this commit uses Docker-based Ubuntu 24.04 library harnesses, supports `original` and package-oriented `port` modes, requires local override packages at `<override-deb-root>/<library>/*.deb`, requires `--port-deb-lock` for `port` mode, and has no direct library-path mode.

libexif-safe package under test:
- Package root: `safe/.artifacts/impl_09_final_release`
- Baseline package source commit before the report commit: `5eeaf29e1826f9a3eae45b511313830d6c1c17f1`
- Authoritative post-commit artifact path: `validator/artifacts/libexif-safe/**`
- Local override root: `validator/artifacts/debs/local/libexif/`
- Local lock path: `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`
- Override packages copied from `safe/.artifacts/impl_09_final_release/artifacts/`:
  - `libexif12_0.6.24-1safelibs1_amd64.deb`, architecture `amd64`, SHA256 `e9dc8fea1bf1055824e2489bff41917cb62bdeb3b58cf8a07d366b716482b270`, size `617792`
  - `libexif-dev_0.6.24-1safelibs1_amd64.deb`, architecture `amd64`, SHA256 `07969d08815db422782ea0e7b4cf7d8cfb96f11e747cf36ad0af3caa6b4df6af`, size `3567480`

Commands executed:
- `git status --short --branch`
- `git rev-parse HEAD`
- `docker info`
- `git clone https://github.com/safelibs/validator validator`
- `git -C validator rev-parse HEAD`
- `cd validator && make unit`
- `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --list-summary`
- `cd validator && python3 tools/testcases.py --config repositories.yml --tests-root tests --library libexif --check --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
- `PACKAGE_BUILD_ROOT="$PWD/safe/.artifacts/impl_09_final_release" bash safe/tests/run-package-build.sh`
- Rebuilt `validator/artifacts/debs/local/libexif/` from canonical `libexif12` and `libexif-dev` `.deb` files.
- Generated `validator/artifacts/libexif-safe/proof/local-port-debs-lock.json`.
- `cd validator && RECORD_CASTS=1 ARTIFACT_ROOT=artifacts/libexif-original LIBRARY=libexif make matrix-original`
- `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-original --proof-output artifacts/libexif-original/proof/original-validation-proof.json --mode original --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`
- `cd validator && bash test.sh --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --mode port --override-deb-root artifacts/debs/local --port-deb-lock artifacts/libexif-safe/proof/local-port-debs-lock.json --library libexif --record-casts`
- `cd validator && jq . artifacts/libexif-safe/port/results/libexif/summary.json`
- `cd validator && python3 tools/verify_proof_artifacts.py --config repositories.yml --tests-root tests --artifact-root artifacts/libexif-safe --proof-output artifacts/libexif-safe/proof/libexif-safe-validation-proof.json --mode port --library libexif --require-casts --min-source-cases 5 --min-usage-cases 130 --min-cases 135`

Original Ubuntu baseline:
- Artifact root: `validator/artifacts/libexif-original`
- Proof: `validator/artifacts/libexif-original/proof/original-validation-proof.json`
- Result: 135 cases, 5 source, 130 usage, 135 passed, 0 failed, 135 casts.

Local override validator run:
- Artifact root: `validator/artifacts/libexif-safe`
- Proof: `validator/artifacts/libexif-safe/proof/libexif-safe-validation-proof.json`
- Result: 135 cases, 5 source, 130 usage, 129 passed, 6 failed, 135 casts.
- Override install status: every result JSON recorded `override_debs_installed: true`.
- Provenance status before the report commit: every result JSON and proof library entry recorded parent commit `5eeaf29e1826f9a3eae45b511313830d6c1c17f1`.
- Post-commit requirement: package root, copied override packages, local lock, local result JSON, and local proof are refreshed after the parent commit; checker assertions establish the final commit-specific provenance.

Failure table:

| Testcase | Class | Owning phase | Evidence |
| --- | --- | --- | --- |
| `usage-exif-cli-debug-ifd-gps-trace` | Ordinary CLI/metadata usage failure | Phase 3 | `--debug --ifd=GPS` output lacks the expected `ExifLoader: Scanning` debug trace. |
| `usage-exif-cli-debug-loader-trace` | Ordinary CLI/metadata usage failure | Phase 3 | `--debug` output lacks the expected loader trace around IFD 0 entry loading. |
| `usage-exif-cli-debug-machine-readable-combo` | Ordinary CLI/metadata usage failure | Phase 3 | `--debug --machine-readable` preserves tab-delimited metadata but lacks the expected debug trace. |
| `usage-exif-cli-debug-no-fixup-loader-trace` | Ordinary CLI/metadata usage failure | Phase 3 | `--debug --no-fixup` output lacks the expected loader trace. |
| `usage-exif-cli-remove-decrements-ifd-zero-entries` | Ordinary CLI/metadata usage failure | Phase 3 | The remove flow's verification expects debug output showing `ExifData: Loading 9 entries...`, which is absent. |
| `usage-exif-cli-remove-missing-copyright` | Ordinary CLI/metadata usage failure | Phase 3 | The absent-tag remove/no-op verification expects debug output showing IFD 0 entry loading, which is absent. |

Fix plan by class:
- Phase 2 source API/ABI/package failures: none in this baseline.
- Phase 3 ordinary CLI/metadata usage failures: implement the libexif debug/loader trace compatibility expected by the Ubuntu `exif` CLI behavior and then rerun the six failed usage cases.
- Phase 4 malformed/crash/timeout/safety failures: none in this baseline.
- Phase 5 unclassified failures or validator-bug candidates: none in this baseline.

Validator bug waivers:
- None.

Package-build blocker fixes:
- None. `safe/tests/run-package-build.sh` completed successfully before validator execution, so no `safe/` prerequisite fix was made in this phase.

Final status:
- Docker was available.
- Validator checkout, unit tests, testcase inventory checks, package refresh, original baseline, local override baseline, and proof verification all ran.
- The setup baseline is complete after the required post-commit package/lock/artifact refresh into `validator/artifacts/libexif-safe/**`.
