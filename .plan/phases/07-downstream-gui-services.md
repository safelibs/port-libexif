# Downstream GUI and Service Runtime Matrix

## Phase Name
`Downstream GUI and Service Runtime Matrix`

## Implement Phase ID
`impl_07_downstream_gui_services`

## Preexisting Inputs
  - `dependents.json`
  - `test-original.sh`
  - existing helper or probe code embedded in `test-original.sh`
  - `safe/tests/testdata/*`
  - `safe/tests/run-package-build.sh`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_07_downstream_gui_services/` with fresh package metadata and validated overlay contents
  - Phase-local Docker image `libexif-original-test:impl_07_downstream_gui_services` with recorded image-input freshness metadata
  - A checker-visible downstream runtime summary at `safe/.artifacts/<implement-phase-id>/downstream/runtime-matrix.tsv` with one successful row per runtime-capable dependent in `dependents.json`
  - Updated or extended GUI or service probes in `test-original.sh`
  - Additional local regressions in `safe/tests/` where GUI or service failures can be reduced to a smaller reproducer
  - Library fixes required by GUI or service-visible runtime failures

## File Changes
  - `test-original.sh`
  - `safe/tests/*.rs`, `safe/tests/*.sh`, `safe/tests/original-c/**`, or `safe/tests/original-sh/**` for distilled regressions
  - `safe/tests/testdata/*` only if needed for a real service or plugin issue
  - `safe/src/**` as required by failures

## Implementation Details
  - Preserve and extend the existing inline probes in `test-original.sh` rather than replacing them with a new harness. The script already knows how to build fixtures, install packages in the container, and drive GUI or service apps under Xvfb or service-specific configs.
  - Convert any discovered issue into a smaller repo-local regression whenever possible; if only the full app integration can reproduce it, strengthen the existing Docker assertion instead of creating a redundant second harness.
  - Keep the Docker image definition inline in `test-original.sh`. Workflow YAML must remain inline-only and consume the existing script.
  - This phase provides the first checker-visible full runtime-matrix proof. Before any checker uses `downstream/runtime-matrix.tsv` as evidence, delete that file from the phase-local root so the exact `./test-original.sh --mode runtime` invocation recreates the 14-row summary in `dependents.json` runtime order.
  - `test-original.sh --mode runtime --only <name>` must not overwrite or truncate an existing 14-row `downstream/runtime-matrix.tsv`. If per-dependent logs or artifacts are useful for focused reruns, place them under a separate non-summary path such as `downstream/only/runtime/<name>/`.
  - Reuse the phase-local package root and phase-local Docker image tag for the full `--mode runtime` invocation and every `--only` rerun in this phase. Run one initial full runtime-matrix invocation without `LIBEXIF_REQUIRE_REUSE`, then reuse-only reruns for the GUI and service entries most likely to regress under Xvfb or service orchestration. Do not rebuild safe packages or the image between entries once the phase-local artifacts exist, and leave the phase-local package root valid for `LIBEXIF_REQUIRE_REUSE=1 bash safe/tests/run-package-build.sh` after the runtime reruns complete.
  - Ensure the GUI or service probes stay deterministic, avoid hidden user-profile state, and keep their assertions tied to observable EXIF behavior rather than brittle UI timing.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_07_downstream_tester` (`check`, `bounce_target: impl_07_downstream_gui_services`)
    - Purpose: verify the GUI, plugin, indexing, and media-service consumers in Docker, including the existing Xvfb- or dbus-driven probes, while reusing one phase-local package root and one phase-local image tag, and prove that the exact `--mode runtime` run recreates the full 14-row runtime summary.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv"
      expected_runtime_names=$(mktemp)
      actual_runtime_names=$(mktemp)
      trap 'rm -f "$expected_runtime_names" "$actual_runtime_names"' EXIT
      rm -f "$runtime_summary"
      ./test-original.sh --mode runtime
      test -f "$runtime_summary"
      test "$(head -n1 "$runtime_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      for app in eog-plugin-exif-display eog-plugin-map tracker-extract Shotwell FoxtrotGPS GTKam MiniDLNA Gerbera libexif-gtk3; do
        LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only "$app"
      done
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh >/dev/null
      ```
  - `check_07_downstream_senior` (`check`, `bounce_target: impl_07_downstream_gui_services`)
    - Purpose: review the heavier downstream probes, confirm that app fixes were not left as Docker-only tribal knowledge, ensure every added probe is stable and deterministic, and confirm that runtime-mode summary generation is not deferred to the final phase.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services
      runtime_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv"
      expected_runtime_names=$(mktemp)
      actual_runtime_names=$(mktemp)
      trap 'rm -f "$expected_runtime_names" "$actual_runtime_names"' EXIT
      rm -f "$runtime_summary"
      ./test-original.sh --mode runtime
      test -f "$runtime_summary"
      test "$(head -n1 "$runtime_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json >"$expected_runtime_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      for app in eog-plugin-exif-display eog-plugin-map tracker-extract Shotwell FoxtrotGPS GTKam MiniDLNA Gerbera libexif-gtk3; do
        LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only "$app"
      done
      awk -F'\t' 'NR > 1 { print $1 }' "$runtime_summary" >"$actual_runtime_names"
      diff -u "$expected_runtime_names" "$actual_runtime_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$runtime_summary")" = "14"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh >/dev/null
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services bash safe/tests/run-package-build.sh`
  - `rm -f /home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services/downstream/runtime-matrix.tsv && LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime`
  - `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services/downstream/runtime-matrix.tsv` must be recreated by that invocation with header `name<TAB>source_package<TAB>assertion<TAB>artifact<TAB>status`, its name column must diff cleanly against `jq -r '.dependents[] | select(.runtime_functionality != null) | .name' /home/yans/code/safelibs/ported/libexif/dependents.json`, and it must contain exactly 14 rows with `status=ok`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only eog-plugin-exif-display`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only eog-plugin-map`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only tracker-extract`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only Shotwell`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only FoxtrotGPS`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only GTKam`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only MiniDLNA`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only Gerbera`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_07_downstream_gui_services LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_07_downstream_gui_services ./test-original.sh --mode runtime --only libexif-gtk3`
## Git Commit Requirement
The implementer must commit work to git before yielding.
