# Final Catch-All and Release Gate

## Phase Name
`Final Catch-All and Release Gate`

## Implement Phase ID
`impl_09_final_release`

## Preexisting Inputs
  - `original/libexif/Makefile.am`
  - `original/configure.ac`
  - `original/libexif/libexif.sym`
  - `original/debian/libexif12.symbols`
  - `original/libexif/exif-tag.c`
  - `original/libexif/{apple,canon,fuji,olympus,pentax}/*`
  - `original/test/Makefile.am`
  - `original/contrib/examples/Makefile.am`
  - `original/test/*.o`, `original/contrib/examples/*.o`, and `original/test/nls/print-localedir.o` when present
  - `dependents.json`
  - `test-original.sh`
  - `safe/Cargo.toml`
  - `safe/Cargo.lock`
  - `safe/COPYING`
  - `safe/README`
  - `safe/NEWS`
  - `safe/build.rs`
  - `safe/SECURITY.md`
  - `safe/SAFETY.md`
  - `safe/cshim/exif-log-shim.c`
  - `safe/src/ffi/{types.rs,panic_boundary.rs}`
  - `safe/src/lib.rs`
  - `safe/src/i18n.rs`
  - `safe/src/primitives/{byte_order.rs,format.rs,ifd.rs,utils.rs}`
  - `safe/src/runtime/{mem.rs,log.rs,cstdio.rs}`
  - `safe/src/object/{content.rs,data.rs,entry.rs}`
  - `safe/src/parser/{data_load.rs,data_save.rs,loader.rs,limits.rs,mod.rs}`
  - `safe/src/tables/{mod.rs,data_options.rs,gps_ifd.rs,tag_table.rs}`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `safe/include/libexif/*.h`
  - `safe/libexif.pc.in`
  - `safe/libexif-uninstalled.pc.in`
  - `safe/debian/{rules,control,libexif12.symbols,libexif12.install,libexif-dev.install,libexif-dev.docs,libexif-doc.docs,libexif-doc.examples,libexif-doc.doc-base,changelog}`
  - `safe/po/*.po`
  - `safe/po/*.gmo`
  - `safe/contrib/examples/*.c`
  - `safe/doc/libexif-api.html/**`
  - `safe/tests/{abi_layout.rs,primitives_tables.rs,object_model.rs,cve_regressions.rs,ported_c.rs}`
  - `safe/tests/{run-package-build.sh,run-export-compare.sh,run-c-compile-smoke.sh,run-original-object-link-compat.sh,run-performance-compare.sh,run-original-test-suite.sh,run-c-test.sh,run-cve-regressions.sh,run-original-shell-test.sh,run-original-nls-test.sh,run-test-mnote-matrix.sh}`
  - `safe/tests/original-c/**`
  - `safe/tests/original-sh/**`
  - `safe/tests/support/**`
  - `safe/tests/link-compat/{object-manifest.txt,run-manifest.txt}`
  - `safe/tests/perf/{bench-driver.c,fixture-manifest.txt,thresholds.env}`
  - `safe/tests/testdata/*.jpg`
  - `safe/tests/testdata/*.parsed`
  - `safe/tests/smoke/public-api-smoke.c`
  - `relevant_cves.json`
  - `all_cves.json`
  - `safe/tests/cve-regressions-manifest.txt`
  - `safe/.artifacts/impl_01_abi_packaging/`
  - `safe/.artifacts/impl_01_abi_packaging/compile-smoke/static/public-api-smoke-static`
  - `safe/.artifacts/impl_04_source_link_compat/`
  - `safe/.artifacts/impl_04_source_link_compat/compile-smoke/static/public-api-smoke-static`
  - `safe/.artifacts/impl_05_downstream_compile/`
  - `safe/.artifacts/impl_05_downstream_compile/downstream/compile-matrix.tsv`
  - `safe/.artifacts/impl_06_downstream_cli_library/`
  - `safe/.artifacts/impl_07_downstream_gui_services/`
  - `safe/.artifacts/impl_07_downstream_gui_services/downstream/runtime-matrix.tsv`
  - `safe/.artifacts/impl_08_safety_perf_docs/`
  - `libexif-original-test:impl_05_downstream_compile`
  - `libexif-original-test:impl_06_downstream_cli_library`
  - `libexif-original-test:impl_07_downstream_gui_services`
  - `.plan/goal.md`
  - `.plan/plan.md`
  - `.plan/findings.md`
  - `.plan/phases/*.md`
  - `.plan/workflow-structure.yaml`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_09_final_release/` with fresh package metadata and validated overlay contents
  - Phase-local Docker image `libexif-original-test:impl_09_final_release` with recorded image-input freshness metadata
  - Final checker-visible downstream compile summary at `safe/.artifacts/impl_09_final_release/downstream/compile-matrix.tsv`
  - Final checker-visible downstream runtime summary at `safe/.artifacts/impl_09_final_release/downstream/runtime-matrix.tsv`
  - The final catch-all code, harness, packaging, and documentation fixes needed for a release candidate
  - Final in-repo regressions for any late-found issue
  - Final package or changelog adjustments for the release-ready state

## File Changes
  - Any previously listed compatibility, test, packaging, or documentation file as required by residual issues
  - `safe/debian/changelog` if the final release candidate needs a packaging note

## Implementation Details
  - This is the only catch-all phase. Use it to absorb residual failures discovered by earlier dedicated phases, not to invent new test classes.
  - Keep fixes focused and traceable. Every late fix should still come with a local regression or a strengthened existing harness assertion.
  - The final release gate must rerun `bash tests/run-original-test-suite.sh` after `cargo test --release`, even if earlier phases already used it. The end-of-work proof has to explicitly revalidate the copied original C, shell, NLS, parser, and MakerNote-sensitive suite at the final commit rather than relying on stale earlier evidence.
  - Reuse the phase-local package root and phase-local Docker image tag for every package-backed command in this phase. The final release checker must prove that contract both positively and negatively for every wrapper named in the workflow contract: `run-package-build.sh`, `run-export-compare.sh`, `run-c-compile-smoke.sh`, `run-original-object-link-compat.sh`, and `run-performance-compare.sh` must all succeed against the existing root under `LIBEXIF_REQUIRE_REUSE=1`; each must fail with `reuse-required package root is missing or stale` when pointed at a deliberately missing root; and at least one local wrapper must also be exercised against a deliberately stale copied root that fails with the same substring. `test-original.sh` must likewise reject deliberately missing and deliberately stale image or downstream-package-root inputs with the exact reuse-only substrings. Do not treat the initial package-root creation as sufficient evidence for `run-package-build.sh`; rerun it in explicit reuse-only mode against the populated final root and against a deliberately missing root. Do not rebuild the same final current-commit package artifacts between local scripts and the downstream Docker harness.
  - The final phase-local `metadata/package-inputs.sha256` must preserve the exact sorted `sha256sum`-style manifest contract from phase 1 over the full package-affecting input set, including `safe/po/**`, `safe/contrib/examples/**`, `safe/doc/libexif-api.html/**`, and any consumed `original/**` build inputs. Late fixes must not drop existing copied locales, docs, or examples from that contract.
  - The final release checker must verify the static-link proof directly by executing `safe/.artifacts/impl_09_final_release/compile-smoke/static/public-api-smoke-static` and confirming that `readelf -d` contains no `NEEDED` entry for `libexif.so.12`.
  - The final release checker must diff `safe/.artifacts/impl_09_final_release/downstream/compile-matrix.tsv` against `jq -r '.dependents[].name' dependents.json` and require exactly 15 `status=ok` rows so a partial compile matrix cannot pass.
  - The final release checker must diff `safe/.artifacts/impl_09_final_release/downstream/runtime-matrix.tsv` against `jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json` and require exactly 14 `status=ok` rows so `--mode all` cannot silently skip runtime-capable dependents after catch-all edits.
  - Before the final `./test-original.sh --mode all` invocation, the checker must delete any preexisting `downstream/compile-matrix.tsv` and `downstream/runtime-matrix.tsv` files so both summaries are forced to be recreated by that exact final run instead of being inherited from an earlier phase.
  - Focused reuse-only reruns after the final full-matrix pass must leave the full 15-row compile summary and full 14-row runtime summary intact, and must leave `safe/.artifacts/impl_09_final_release/` valid for `LIBEXIF_REQUIRE_REUSE=1 bash safe/tests/run-package-build.sh`.
  - The final release checker must assert the exact compile-only rejection string `compile-only dependent: ImageMagick only supports --mode compile` rather than treating any nonzero exit as sufficient.
  - Preserve earlier comparison policies during late fixes: do not weaken parser tests that depend on explicit fixture argv or `TEST_IMAGES`, and do not relax existing link-compat comparison modes or file-content checks just to get the final reruns green.
  - Do not parallelize the final verification commands. The shared package-build scripts and Docker harnesses must remain serial.
  - The final commit should represent the release-candidate state that all checkers reason about.
  - If any canonical planning artifact under `.plan/` needs adjustment while executing this phase, rewrite `.plan/goal.md`, `.plan/plan.md`, `.plan/findings.md`, existing `.plan/phases/*.md`, and `.plan/workflow-structure.yaml` in place rather than creating a second planning tree.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_09_release_tester` (`check`, `bounce_target: impl_09_final_release`)
    - Purpose: run the full serial end-to-end verification matrix and prove that the safe package remains a drop-in replacement across the copied original C/shell/NLS/parser suite, ABI, source, link, runtime, safety, packaging, performance, MakerNote, static-archive, and downstream-app surfaces, with artifact reuse and final compile-plus-runtime summary proof after `--mode all`.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release
      expected_manifest=$(mktemp)
      expected_compile_names=$(mktemp)
      actual_compile_names=$(mktemp)
      expected_runtime_names=$(mktemp)
      actual_runtime_names=$(mktemp)
      missing_build_root_log=$(mktemp)
      missing_export_root_log=$(mktemp)
      missing_compile_root_log=$(mktemp)
      missing_link_root_log=$(mktemp)
      missing_perf_root_log=$(mktemp)
      stale_local_root_log=$(mktemp)
      missing_image_log=$(mktemp)
      stale_image_log=$(mktemp)
      missing_downstream_root_log=$(mktemp)
      stale_downstream_root_log=$(mktemp)
      imagemagick_runtime_log=$(mktemp)
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      missing_local_root="${PACKAGE_BUILD_ROOT}.missing"
      stale_local_root="${PACKAGE_BUILD_ROOT}.stale"
      missing_downstream_root="${PACKAGE_BUILD_ROOT}.downstream-missing"
      stale_downstream_root="${PACKAGE_BUILD_ROOT}.downstream-stale"
      missing_image_tag="libexif-original-test:impl_09_final_release-missing"
      stale_image_tag="libexif-original-test:impl_09_final_release-stale"
      trap 'rm -f "$expected_manifest" "$expected_compile_names" "$actual_compile_names" "$expected_runtime_names" "$actual_runtime_names" "$missing_build_root_log" "$missing_export_root_log" "$missing_compile_root_log" "$missing_link_root_log" "$missing_perf_root_log" "$stale_local_root_log" "$missing_image_log" "$stale_image_log" "$missing_downstream_root_log" "$stale_downstream_root_log" "$imagemagick_runtime_log"; rm -rf "$stale_local_root" "$stale_downstream_root"; docker image rm -f "$stale_image_tag" >/dev/null 2>&1 || true' EXIT
      cargo test --release
      bash tests/run-original-test-suite.sh
      bash tests/run-test-mnote-matrix.sh
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
      test -f "$PACKAGE_BUILD_ROOT/libexif-dev/usr/lib/$multiarch/libexif.a"
      test -f "$PACKAGE_BUILD_ROOT/root/usr/lib/$multiarch/libexif.a"
      test "$(cat "$PACKAGE_BUILD_ROOT/metadata/source-commit.txt")" = "$(git -C /home/yans/code/safelibs/ported/libexif rev-parse HEAD)"
      bash tests/run-package-build.sh --print-package-inputs-manifest >"$expected_manifest"
      diff -u "$expected_manifest" "$PACKAGE_BUILD_ROOT/metadata/package-inputs.sha256"
      test -f "$PACKAGE_BUILD_ROOT/metadata/validated.ok"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh >/dev/null
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-export-compare.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh
      test -x "$static_smoke"
      "$static_smoke"
      ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-original-object-link-compat.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-performance-compare.sh
      rm -rf "$missing_local_root"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-package-build.sh >"$missing_build_root_log" 2>&1; then
        cat "$missing_build_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_build_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-export-compare.sh >"$missing_export_root_log" 2>&1; then
        cat "$missing_export_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_export_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-c-compile-smoke.sh >"$missing_compile_root_log" 2>&1; then
        cat "$missing_compile_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_compile_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-original-object-link-compat.sh >"$missing_link_root_log" 2>&1; then
        cat "$missing_link_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_link_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-performance-compare.sh >"$missing_perf_root_log" 2>&1; then
        cat "$missing_perf_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_perf_root_log"
      rm -rf "$stale_local_root"
      cp -a "$PACKAGE_BUILD_ROOT" "$stale_local_root"
      printf 'stale\n' >"$stale_local_root/metadata/source-commit.txt"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$stale_local_root" bash tests/run-export-compare.sh >"$stale_local_root_log" 2>&1; then
        cat "$stale_local_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$stale_local_root_log"
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv"
      runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv"
      rm -f "$compile_summary" "$runtime_summary"
      ./test-original.sh --mode all
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      test -f "$compile_summary"
      test "$(head -n1 "$compile_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[].name' dependents.json >"$expected_compile_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      test -f "$runtime_summary"
      test "$(head -n1 "$runtime_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode compile --only exif
      LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only exif
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh >/dev/null
      docker image rm -f "$missing_image_tag" >/dev/null 2>&1 || true
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="$missing_image_tag" ./test-original.sh --mode compile --only exif >"$missing_image_log" 2>&1; then
        cat "$missing_image_log" >&2
        exit 1
      fi
      grep -F "reuse-required docker image is missing or stale" "$missing_image_log"
      docker tag ubuntu:24.04 "$stale_image_tag"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="$stale_image_tag" ./test-original.sh --mode compile --only exif >"$stale_image_log" 2>&1; then
        cat "$stale_image_log" >&2
        exit 1
      fi
      grep -F "reuse-required docker image is missing or stale" "$stale_image_log"
      rm -rf "$missing_downstream_root"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="$missing_downstream_root" ./test-original.sh --mode compile --only exif >"$missing_downstream_root_log" 2>&1; then
        cat "$missing_downstream_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_downstream_root_log"
      rm -rf "$stale_downstream_root"
      cp -a "$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" "$stale_downstream_root"
      printf 'stale\n' >"$stale_downstream_root/metadata/source-commit.txt"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="$stale_downstream_root" ./test-original.sh --mode compile --only exif >"$stale_downstream_root_log" 2>&1; then
        cat "$stale_downstream_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$stale_downstream_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only ImageMagick >"$imagemagick_runtime_log" 2>&1; then
        cat "$imagemagick_runtime_log" >&2
        exit 1
      fi
      grep -F "compile-only dependent: ImageMagick only supports --mode compile" "$imagemagick_runtime_log"
      ```
  - `check_09_release_senior` (`check`, `bounce_target: impl_09_final_release`)
    - Purpose: perform the final senior review, confirm that every earlier checker issue has a corresponding fix or regression, and verify that the final git state is release-ready, including the copied original C/shell/NLS/parser suite rerun, the static archive, all-wrapper reuse-only, stale-artifact, and full compile-plus-runtime matrix contracts.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif
      git show --stat --format=fuller HEAD
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release
      expected_manifest=$(mktemp)
      expected_compile_names=$(mktemp)
      actual_compile_names=$(mktemp)
      expected_runtime_names=$(mktemp)
      actual_runtime_names=$(mktemp)
      missing_build_root_log=$(mktemp)
      missing_local_root_log=$(mktemp)
      missing_image_log=$(mktemp)
      stale_image_log=$(mktemp)
      imagemagick_runtime_log=$(mktemp)
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      missing_local_root="${PACKAGE_BUILD_ROOT}.missing"
      stale_image_tag="libexif-original-test:impl_09_final_release-stale"
      trap 'rm -f "$expected_manifest" "$expected_compile_names" "$actual_compile_names" "$expected_runtime_names" "$actual_runtime_names" "$missing_build_root_log" "$missing_local_root_log" "$missing_image_log" "$stale_image_log" "$imagemagick_runtime_log"; docker image rm -f "$stale_image_tag" >/dev/null 2>&1 || true' EXIT
      cargo test --release
      bash tests/run-original-test-suite.sh
      bash tests/run-test-mnote-matrix.sh
      multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
      test -f "$PACKAGE_BUILD_ROOT/libexif-dev/usr/lib/$multiarch/libexif.a"
      bash tests/run-package-build.sh --print-package-inputs-manifest >"$expected_manifest"
      diff -u "$expected_manifest" "$PACKAGE_BUILD_ROOT/metadata/package-inputs.sha256"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh >/dev/null
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-export-compare.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh
      test -x "$static_smoke"
      "$static_smoke"
      ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-original-object-link-compat.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-performance-compare.sh
      rm -rf "$missing_local_root"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-package-build.sh >"$missing_build_root_log" 2>&1; then
        cat "$missing_build_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_build_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-c-compile-smoke.sh >"$missing_local_root_log" 2>&1; then
        cat "$missing_local_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_local_root_log"
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv"
      runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv"
      rm -f "$compile_summary" "$runtime_summary"
      ./test-original.sh --mode all
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      test -f "$compile_summary"
      test "$(head -n1 "$compile_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[].name' dependents.json >"$expected_compile_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      test -f "$runtime_summary"
      test "$(head -n1 "$runtime_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only Shotwell
      docker image rm -f "${LIBEXIF_ORIGINAL_TEST_IMAGE}-missing" >/dev/null 2>&1 || true
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="${LIBEXIF_ORIGINAL_TEST_IMAGE}-missing" ./test-original.sh --mode compile --only exif >"$missing_image_log" 2>&1; then
        cat "$missing_image_log" >&2
        exit 1
      fi
      grep -F "reuse-required docker image is missing or stale" "$missing_image_log"
      docker tag ubuntu:24.04 "$stale_image_tag"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="$stale_image_tag" ./test-original.sh --mode compile --only exif >"$stale_image_log" 2>&1; then
        cat "$stale_image_log" >&2
        exit 1
      fi
      grep -F "reuse-required docker image is missing or stale" "$stale_image_log"
      if LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only ImageMagick >"$imagemagick_runtime_log" 2>&1; then
        cat "$imagemagick_runtime_log" >&2
        exit 1
      fi
      grep -F "compile-only dependent: ImageMagick only supports --mode compile" "$imagemagick_runtime_log"
      ```

## Success Criteria
  - The safe build still exports the exact symbol set and symbol versions required by the original manifests.
  - The packaged library, headers, static archive, docs, examples, doc-base entry, and locales install under the expected Debian paths and produce `libexif12`, `libexif-dev`, and `libexif-doc`.
  - The final package root's `metadata/source-commit.txt`, `metadata/package-inputs.sha256`, and `metadata/validated.ok` match the current workspace, and `bash tests/run-package-build.sh --print-package-inputs-manifest` diffs cleanly against the stored manifest.
  - The final gate reruns `bash tests/run-original-test-suite.sh`, so the copied original C, shell, NLS, parser, and MakerNote-sensitive suite is revalidated at the release-candidate commit; all copied original C, shell, NLS, CVE, object-model, and primitive tests pass, and the dedicated MakerNote matrix passes.
  - Previously compiled original object files relink against `libexif-safe` and match original runtime behavior and output files according to the manifest comparison policy.
  - Source-compiled smoke clients still build against the packaged headers and pkg-config data, and the static-link smoke path proves that the packaged `libexif.a` is usable by executing `safe/.artifacts/impl_09_final_release/compile-smoke/static/public-api-smoke-static` and showing that `readelf -d` contains no `NEEDED` entry for `libexif.so.12`.
  - Every current downstream name in `dependents.json` successfully completes its explicit compile-mode check against the installed safe packages, including the entries that previously had only runtime coverage and the compile-only `ImageMagick` source-build-plus-`identify` assertion, and `safe/.artifacts/impl_09_final_release/downstream/compile-matrix.tsv` proves that coverage with exactly 15 `status=ok` rows whose name column matches `dependents.json`.
  - The full downstream runtime matrix passes for every dependent whose `runtime_functionality` is non-null in `dependents.json`, `safe/.artifacts/impl_09_final_release/downstream/runtime-matrix.tsv` proves that coverage with exactly 14 `status=ok` rows whose name column matches the runtime-capable subset of `dependents.json`, and compile-only dependents are rejected from runtime mode with the exact substring `compile-only dependent: ImageMagick only supports --mode compile` while still passing compile mode.
  - Focused reuse-only downstream reruns leave the full-matrix `downstream/compile-matrix.tsv` and `downstream/runtime-matrix.tsv` files intact and leave `safe/.artifacts/impl_09_final_release/` valid for `LIBEXIF_REQUIRE_REUSE=1 bash safe/tests/run-package-build.sh`.
  - Reuse-only reruns are checker-visible: every package-backed local wrapper passes with `LIBEXIF_REQUIRE_REUSE=1` against the existing root, including `safe/tests/run-package-build.sh`; deliberately missing local package roots fail with `reuse-required package root is missing or stale` for every wrapper, including `safe/tests/run-package-build.sh`; at least one deliberately stale copied local root fails with the same substring; the final Docker image label matches `./test-original.sh --print-image-inputs-sha256`; targeted downstream reruns pass with `LIBEXIF_REQUIRE_REUSE=1`; deliberately missing and deliberately stale Docker images fail with `reuse-required docker image is missing or stale`; and deliberately missing and deliberately stale downstream package roots fail with `reuse-required package root is missing or stale`.
  - Performance remains within `safe/tests/perf/thresholds.env`.
  - `safe/SAFETY.md` matches the actual remaining `unsafe` or FFI surface, and any residual non-Rust implementation code is explicitly justified as unavoidable.

## Critical Files
  - `safe/build.rs`: central build glue for version-script generation, SONAME injection, gettext env propagation, the log shim, and the remaining MakerNote helper C compilation that phase 3 should remove; the current high-value seams are `safe/build.rs:13-57` and `safe/build.rs:62-278`.
  - `safe/Cargo.toml` and `safe/Cargo.lock`: crate type, dependency, and build-script configuration if ABI or helper removal requires adjustment.
  - `safe/src/ffi/types.rs` and `safe/src/ffi/panic_boundary.rs`: ABI layout mirrors and panic containment for exported C entry points; `safe/src/ffi/types.rs:101-167` is the core public-struct contract checked by `safe/tests/abi_layout.rs`.
  - `safe/src/primitives/{byte_order.rs,format.rs,ifd.rs,utils.rs}`: scalar conversion helpers and public low-level ABI functions.
  - `safe/src/i18n.rs`: gettext or localedir initialization used by translated table strings and the copied NLS checks.
  - `safe/src/tables/{mod.rs,data_options.rs,gps_ifd.rs,tag_table.rs}`: tag metadata, option strings, GPS-table data, and lookup compatibility.
  - `safe/src/runtime/{mem.rs,log.rs,cstdio.rs}` plus `safe/cshim/exif-log-shim.c`: allocator semantics, logging callbacks or varargs behavior, and the remaining unavoidable C boundary if it still exists after hardening; the current varargs bridge lives in `safe/cshim/exif-log-shim.c:1-30`.
  - `safe/src/object/{content.rs,data.rs,entry.rs}`: core `ExifContent`, `ExifData`, and `ExifEntry` semantics, including refcounts, parent pointers, default-tag creation, and value formatting; `safe/src/object/data.rs:21-220` and `safe/src/object/content.rs:21-240` are the main ownership and graph-manipulation seams.
  - `safe/src/parser/{data_load.rs,data_save.rs,loader.rs,limits.rs,mod.rs}`: load or save behavior, incremental loader state machine, checked arithmetic, and parse-budget enforcement; `safe/src/parser/data_load.rs:92-260`, `safe/src/parser/data_save.rs:22-260`, and `safe/src/parser/loader.rs:34-260` are the key compatibility and CVE-sensitive paths.
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`: the remaining MakerNote implementation surface. `safe/src/mnote/mod.rs:44-132` is the MakerNote construction and save bridge. `apple.rs` and `fuji.rs` are currently internal helper trampolines, while `canon.rs`, `olympus.rs`, and `pentax.rs` are still part of the exported ABI and therefore must keep their public symbol and formatting behavior while becoming genuinely Rust-owned.
  - `safe/include/libexif/*.h`, `safe/libexif.pc.in`, and `safe/libexif-uninstalled.pc.in`: installed public development surface; keep compatible with the original package.
  - `safe/debian/{rules,control,libexif12.symbols,libexif12.install,libexif-dev.install,libexif-dev.docs,libexif-doc.docs,libexif-doc.examples,libexif-doc.doc-base,changelog}`: Debian package assembly, symbol manifest, docs or examples ownership, changelog, and exact installed file layout.
  - `safe/README`, `safe/NEWS`, and `safe/SAFETY.md`: shipped documentation that must accurately describe the Rust port, remaining foreign boundaries, and release notes.
  - `safe/tests/{abi_layout.rs,primitives_tables.rs,object_model.rs,cve_regressions.rs,ported_c.rs}`: local ABI and behavior regression coverage; `safe/tests/abi_layout.rs:31-220`, `safe/tests/object_model.rs:35-133`, `safe/tests/primitives_tables.rs:56-260`, and `safe/tests/cve_regressions.rs:210-314` are the current high-signal regression anchors.
  - `safe/tests/{run-c-test.sh,run-original-test-suite.sh,run-original-shell-test.sh,run-original-nls-test.sh,run-export-compare.sh,run-package-build.sh,run-c-compile-smoke.sh,run-original-object-link-compat.sh,run-performance-compare.sh,run-cve-regressions.sh,run-test-mnote-matrix.sh}`: serial orchestration of the compatibility matrix, now including phase-local package-root reuse, `--print-package-inputs-manifest`, `LIBEXIF_REQUIRE_REUSE=1` reuse-only checks, the dedicated MakerNote matrix that phase 3 and phase 9 must run explicitly, and the fixed `compile-smoke/static/public-api-smoke-static` artifact that proves static linking. The most important current scripts are `safe/tests/run-package-build.sh:9-106`, `safe/tests/run-export-compare.sh:9-62`, `safe/tests/run-c-compile-smoke.sh:9-33`, `safe/tests/run-original-object-link-compat.sh:88-248`, and `safe/tests/run-performance-compare.sh:145-224`.
  - `safe/tests/original-c/**`, `safe/tests/original-sh/**`, and `safe/tests/support/**`: copied upstream tests, nested NLS fixtures, shell helpers, and support-only headers that must be updated in place rather than recopied wholesale.
  - `safe/tests/link-compat/{object-manifest.txt,run-manifest.txt}`: object-relink coverage and exact output-comparison policy.
  - `safe/tests/perf/{bench-driver.c,fixture-manifest.txt,thresholds.env}`: performance workloads, fixture inventory, and acceptance thresholds.
  - `safe/tests/downstream/**`: checked-in helper manifests or probe sources only if the downstream compile matrix becomes too large to maintain inline in `test-original.sh`.
  - `safe/tests/testdata/*` and `safe/contrib/examples/*.c`: fixtures and example programs used by compile, relink, runtime, and downstream checks.
  - `dependents.json`: existing 15-app downstream inventory that already satisfies the "dozen applications" requirement and should only be corrected in place if factually wrong.
  - `test-original.sh`: existing Docker-based downstream-app harness that must gain explicit compile or runtime modes, `--print-image-inputs-sha256`, freshness-checked phase-local image reuse, freshness-checked consumption of prebuilt safe package roots, `LIBEXIF_REQUIRE_REUSE=1` reuse-only enforcement with exact reuse-failure substrings, checker-visible `downstream/compile-matrix.tsv` and `downstream/runtime-matrix.tsv` artifacts for the full compile and runtime inventories, and a fixed compile-only `ImageMagick` rule. The current inventory and probe seams are `test-original.sh:149-165`, `test-original.sh:259-321`, `test-original.sh:564-1793`, and the per-app runtime probes at `test-original.sh:1413-1757`.

## Final Verification
Run final verification serially, in this exact order, with no other package-backed script running:

1. `cd /home/yans/code/safelibs/ported/libexif/safe && cargo test --release`
2. `cd /home/yans/code/safelibs/ported/libexif/safe && bash tests/run-original-test-suite.sh`
3. `cd /home/yans/code/safelibs/ported/libexif/safe && bash tests/run-test-mnote-matrix.sh`
4. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh`
5. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && expected_manifest=$(mktemp) && trap 'rm -f "$expected_manifest"' EXIT && multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH) && test -f "$PACKAGE_BUILD_ROOT/libexif-dev/usr/lib/$multiarch/libexif.a" && test -f "$PACKAGE_BUILD_ROOT/root/usr/lib/$multiarch/libexif.a" && test "$(cat "$PACKAGE_BUILD_ROOT/metadata/source-commit.txt")" = "$(git -C /home/yans/code/safelibs/ported/libexif rev-parse HEAD)" && bash tests/run-package-build.sh --print-package-inputs-manifest >"$expected_manifest" && diff -u "$expected_manifest" "$PACKAGE_BUILD_ROOT/metadata/package-inputs.sha256" && test -f "$PACKAGE_BUILD_ROOT/metadata/validated.ok" && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh >/dev/null`
6. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-export-compare.sh`
7. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static" && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh && test -x "$static_smoke" && "$static_smoke" && ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'`
8. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-original-object-link-compat.sh`
9. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-performance-compare.sh`
10. `cd /home/yans/code/safelibs/ported/libexif/safe && export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && missing_local_root="$PACKAGE_BUILD_ROOT.missing" && stale_local_root="$PACKAGE_BUILD_ROOT.stale" && missing_build_log=$(mktemp) && missing_export_log=$(mktemp) && missing_compile_log=$(mktemp) && missing_link_log=$(mktemp) && missing_perf_log=$(mktemp) && stale_local_root_log=$(mktemp) && trap 'rm -f "$missing_build_log" "$missing_export_log" "$missing_compile_log" "$missing_link_log" "$missing_perf_log" "$stale_local_root_log"; rm -rf "$stale_local_root"' EXIT && rm -rf "$missing_local_root" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-package-build.sh >"$missing_build_log" 2>&1; then cat "$missing_build_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_build_log" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-export-compare.sh >"$missing_export_log" 2>&1; then cat "$missing_export_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_export_log" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-c-compile-smoke.sh >"$missing_compile_log" 2>&1; then cat "$missing_compile_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_compile_log" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-original-object-link-compat.sh >"$missing_link_log" 2>&1; then cat "$missing_link_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_link_log" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_local_root" bash tests/run-performance-compare.sh >"$missing_perf_log" 2>&1; then cat "$missing_perf_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_perf_log" && rm -rf "$stale_local_root" && cp -a "$PACKAGE_BUILD_ROOT" "$stale_local_root" && printf 'stale\n' >"$stale_local_root/metadata/source-commit.txt" && if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$stale_local_root" bash tests/run-export-compare.sh >"$stale_local_root_log" 2>&1; then cat "$stale_local_root_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$stale_local_root_log"`
11. `cd /home/yans/code/safelibs/ported/libexif && export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)" && compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv" && runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv" && expected_compile_names=$(mktemp) && actual_compile_names=$(mktemp) && expected_runtime_names=$(mktemp) && actual_runtime_names=$(mktemp) && trap 'rm -f "$expected_compile_names" "$actual_compile_names" "$expected_runtime_names" "$actual_runtime_names"' EXIT && rm -f "$compile_summary" "$runtime_summary" && ./test-original.sh --mode all && test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest" && test -f "$compile_summary" && test "$(head -n1 "$compile_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus' && jq -r '.dependents[].name' dependents.json >"$expected_compile_names" && awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names" && diff -u "$expected_compile_names" "$actual_compile_names" && test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15" && test -f "$runtime_summary" && test "$(head -n1 "$runtime_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus' && jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names" && awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names" && diff -u "$expected_runtime_names" "$actual_runtime_names" && test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"`
12. `cd /home/yans/code/safelibs/ported/libexif && export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv" && runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv" && expected_compile_names=$(mktemp) && actual_compile_names=$(mktemp) && expected_runtime_names=$(mktemp) && actual_runtime_names=$(mktemp) && trap 'rm -f "$expected_compile_names" "$actual_compile_names" "$expected_runtime_names" "$actual_runtime_names"' EXIT && jq -r '.dependents[].name' dependents.json >"$expected_compile_names" && jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names" && LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode compile --only exif && LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only exif && awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names" && diff -u "$expected_compile_names" "$actual_compile_names" && test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15" && awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names" && diff -u "$expected_runtime_names" "$actual_runtime_names" && test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14" && LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh >/dev/null`
13. `cd /home/yans/code/safelibs/ported/libexif && export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && missing_image_log=$(mktemp) && stale_image_log=$(mktemp) && missing_root_log=$(mktemp) && stale_root_log=$(mktemp) && stale_image_tag="${LIBEXIF_ORIGINAL_TEST_IMAGE}-stale" && stale_root="${LIBEXIF_DOWNSTREAM_PACKAGE_ROOT}.stale" && trap 'rm -f "$missing_image_log" "$stale_image_log" "$missing_root_log" "$stale_root_log"; rm -rf "$stale_root"; docker image rm -f "$stale_image_tag" >/dev/null 2>&1 || true' EXIT && docker image rm -f "${LIBEXIF_ORIGINAL_TEST_IMAGE}-missing" >/dev/null 2>&1 || true && if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="${LIBEXIF_ORIGINAL_TEST_IMAGE}-missing" ./test-original.sh --mode compile --only exif >"$missing_image_log" 2>&1; then cat "$missing_image_log" >&2; exit 1; fi && grep -F "reuse-required docker image is missing or stale" "$missing_image_log" && docker tag ubuntu:24.04 "$stale_image_tag" && if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE="$stale_image_tag" ./test-original.sh --mode compile --only exif >"$stale_image_log" 2>&1; then cat "$stale_image_log" >&2; exit 1; fi && grep -F "reuse-required docker image is missing or stale" "$stale_image_log" && rm -rf "${LIBEXIF_DOWNSTREAM_PACKAGE_ROOT}.missing" && if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="${LIBEXIF_DOWNSTREAM_PACKAGE_ROOT}.missing" ./test-original.sh --mode compile --only exif >"$missing_root_log" 2>&1; then cat "$missing_root_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$missing_root_log" && rm -rf "$stale_root" && cp -a "$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" "$stale_root" && printf 'stale\n' >"$stale_root/metadata/source-commit.txt" && if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="$stale_root" ./test-original.sh --mode compile --only exif >"$stale_root_log" 2>&1; then cat "$stale_root_log" >&2; exit 1; fi && grep -F "reuse-required package root is missing or stale" "$stale_root_log"`
14. `cd /home/yans/code/safelibs/ported/libexif && export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_09_final_release LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_09_final_release && imagemagick_runtime_log=$(mktemp) && trap 'rm -f "$imagemagick_runtime_log"' EXIT && if LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only ImageMagick >"$imagemagick_runtime_log" 2>&1; then cat "$imagemagick_runtime_log" >&2; exit 1; fi && grep -F "compile-only dependent: ImageMagick only supports --mode compile" "$imagemagick_runtime_log"`

The final result is acceptable only if all of the following are true:

  - The safe build still exports the exact symbol set and symbol versions required by the original manifests.
  - The packaged library, headers, static archive, docs, examples, doc-base entry, and locales install under the expected Debian paths and produce `libexif12`, `libexif-dev`, and `libexif-doc`.
  - The final package root's `metadata/source-commit.txt`, `metadata/package-inputs.sha256`, and `metadata/validated.ok` match the current workspace, and `bash tests/run-package-build.sh --print-package-inputs-manifest` diffs cleanly against the stored manifest.
  - The final gate reruns `bash tests/run-original-test-suite.sh`, so the copied original C, shell, NLS, parser, and MakerNote-sensitive suite is revalidated at the release-candidate commit; all copied original C, shell, NLS, CVE, object-model, and primitive tests pass, and the dedicated MakerNote matrix passes.
  - Previously compiled original object files relink against `libexif-safe` and match original runtime behavior and output files according to the manifest comparison policy.
  - Source-compiled smoke clients still build against the packaged headers and pkg-config data, and the static-link smoke path proves that the packaged `libexif.a` is usable by executing `safe/.artifacts/impl_09_final_release/compile-smoke/static/public-api-smoke-static` and showing that `readelf -d` contains no `NEEDED` entry for `libexif.so.12`.
  - Every current downstream name in `dependents.json` successfully completes its explicit compile-mode check against the installed safe packages, including the entries that previously had only runtime coverage and the compile-only `ImageMagick` source-build-plus-`identify` assertion, and `safe/.artifacts/impl_09_final_release/downstream/compile-matrix.tsv` proves that coverage with exactly 15 `status=ok` rows whose name column matches `dependents.json`.
  - The full downstream runtime matrix passes for every dependent whose `runtime_functionality` is non-null in `dependents.json`, `safe/.artifacts/impl_09_final_release/downstream/runtime-matrix.tsv` proves that coverage with exactly 14 `status=ok` rows whose name column matches the runtime-capable subset of `dependents.json`, and compile-only dependents are rejected from runtime mode with the exact substring `compile-only dependent: ImageMagick only supports --mode compile` while still passing compile mode.
  - Focused reuse-only downstream reruns leave the full-matrix `downstream/compile-matrix.tsv` and `downstream/runtime-matrix.tsv` files intact and leave `safe/.artifacts/impl_09_final_release/` valid for `LIBEXIF_REQUIRE_REUSE=1 bash safe/tests/run-package-build.sh`.
  - Reuse-only reruns are checker-visible: every package-backed local wrapper passes with `LIBEXIF_REQUIRE_REUSE=1` against the existing root, including `safe/tests/run-package-build.sh`; deliberately missing local package roots fail with `reuse-required package root is missing or stale` for every wrapper, including `safe/tests/run-package-build.sh`; at least one deliberately stale copied local root fails with the same substring; the final Docker image label matches `./test-original.sh --print-image-inputs-sha256`; targeted downstream reruns pass with `LIBEXIF_REQUIRE_REUSE=1`; deliberately missing and deliberately stale Docker images fail with `reuse-required docker image is missing or stale`; and deliberately missing and deliberately stale downstream package roots fail with `reuse-required package root is missing or stale`.
  - Performance remains within `safe/tests/perf/thresholds.env`.
  - `safe/SAFETY.md` matches the actual remaining `unsafe` or FFI surface, and any residual non-Rust implementation code is explicitly justified as unavoidable.

## Git Commit Requirement
The implementer must commit work to git before yielding.
