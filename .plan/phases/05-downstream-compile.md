# Downstream Compile Compatibility Matrix

## Phase Name
`Downstream Compile Compatibility Matrix`

## Implement Phase ID
`impl_05_downstream_compile`

## Preexisting Inputs
  - `dependents.json`
  - `test-original.sh`
  - `safe/tests/run-package-build.sh`
  - current Docker inline image definition embedded in `test-original.sh`
  - current smaller compile probes for `libexif-gtk3`, `CamlImages`, and `ImageMagick`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_05_downstream_compile/` with fresh package metadata and validated overlay contents
  - Phase-local Docker image `libexif-original-test:impl_05_downstream_compile` with recorded image-input freshness metadata and an inspectable `io.safelibs.libexif.image-inputs-sha256` label
  - Updated `test-original.sh` with explicit `compile`, `runtime`, and `all` modes, full-matrix downstream summary writers, and a non-mutating `--print-image-inputs-sha256` helper for verifiers
  - A concrete compile-mode assertion for every current downstream name in `dependents.json`
  - A checker-visible downstream compile summary at `safe/.artifacts/<implement-phase-id>/downstream/compile-matrix.tsv` with one successful row per dependent name in `dependents.json`
  - Checked-in helper manifests or probe sources under `safe/tests/downstream/**` only if the per-dependent compile assertions become too large to maintain inline in `test-original.sh`
  - New minimal local regressions under `safe/tests/` for any library bug discovered through the downstream compile matrix
  - Library fixes required by compile-time failures

## File Changes
  - `test-original.sh`
  - `dependents.json` only if a factual inventory error must be corrected in place
  - `safe/tests/downstream/**` only if helper manifests or probe sources are truly needed
  - `safe/tests/*.rs`, `safe/tests/*.sh`, or `safe/tests/original-c/**` for distilled regressions
  - `safe/tests/testdata/*` only if a new fixture is required to reproduce a real downstream bug
  - `safe/src/**` as required by failures

## Implementation Details
  - Extend `test-original.sh` with `--mode compile`, `--mode runtime`, and `--mode all`; keep `all` as the default broadest end-to-end entry point.
  - Preserve the exact downstream name contract in `test-original.sh`: `REQUIRED_DEPENDENTS` must stay synchronized with `dependents.json`, and `--only` must continue using those exact case-sensitive names.
  - Preserve the existing downstream Docker harness in `test-original.sh`. Extend it in place; do not replace it with a separate workflow, external Dockerfile, or rediscovered app list.
  - `--mode compile` must cover all 15 downstream names in `dependents.json`. The inventory already records compile evidence via `dependency_paths`, but compile-mode membership is still the full fixed dependent-name list. It must not silently skip a dependent just because the existing script only had a runtime probe for it before, and it must not collapse shared source-package builds into fewer named results.
  - For each unique source package from the existing inventory, build the actual Ubuntu source package inside the Docker container using the already selected source-package identity from `dependents.json`: `apt-get build-dep -y --no-install-recommends <source_package>`, `apt-get source <source_package>`, and `DEB_BUILD_OPTIONS='nocheck noautodbgsym nostrip' dpkg-buildpackage -us -uc -b`.
  - The required source-package build coverage is: `exif`, `fbi`, `eog-plugins`, `tracker-miners`, `shotwell`, `foxtrotgps`, `gphoto2`, `gtkam`, `minidlna`, `gerbera`, `ruby-exif`, `libexif-gtk`, `camlimages`, and `imagemagick`.
  - For duplicate downstream names that share a source package, one source build is acceptable inside a single `--mode compile` invocation only if each downstream name still has an explicit post-build assertion. `eog-plugin-exif-display` and `eog-plugin-map` are the main required shared-build case.
  - Each compile assertion must check for the relevant build product by dependent name, such as the produced `.deb`, shared object, or built executable. When the built product is an ELF consumer of `libexif.so.12`, also assert that `ldd` resolves it to the active safe library installed from `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT`.
  - `test-original.sh --mode compile` must refresh `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv` with the exact header `name<TAB>source_package<TAB>assertion<TAB>artifact<TAB>status`, one row per dependent name in the exact `dependents.json` order, and `status=ok` for every successful row. The `assertion` field must describe the dependent-specific post-build check, and the `artifact` field must identify the built file or package that satisfied that check.
  - `test-original.sh --mode compile --only <name>` must not overwrite or truncate an existing 15-row `downstream/compile-matrix.tsv`. If per-dependent logs or artifacts are useful for focused reruns, place them under a separate non-summary path such as `downstream/only/compile/<name>/`.
  - A full-matrix invocation of `test-original.sh --mode runtime` without `--only`, and the runtime half of `test-original.sh --mode all`, must refresh `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/runtime-matrix.tsv` with the exact header `name<TAB>source_package<TAB>assertion<TAB>artifact<TAB>status`, one row per runtime-capable dependent in the exact `jq -r '.dependents[] | select(.runtime_functionality != null) | .name' dependents.json` order, and `status=ok` for every successful row. The `assertion` field must describe the dependent-specific runtime probe, and the `artifact` field must identify the runtime log, inspected output, or generated file that satisfied that probe. Compile-only dependents must never appear in this file.
  - Treat `ImageMagick` as compile-only for mode membership because `dependents.json` records `"runtime_functionality": null`. Preserve the existing `identify -verbose` execution as `ImageMagick`'s compile-mode post-build assertion after the source package finishes building, but do not schedule `ImageMagick` inside runtime mode.
  - The existing smaller compile probes for `libexif-gtk3`, `CamlImages`, and `ImageMagick` may remain as faster debugging loops, but they are supplemental. They do not replace the full source-package build contract.
  - `--mode all` must run the complete compile matrix first and then the runtime matrix, reusing the same validated `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT` and the same validated `LIBEXIF_ORIGINAL_TEST_IMAGE` within that invocation.
  - `test-original.sh` must install prebuilt safe packages from `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT`, must reject that package root if its freshness metadata does not match the current workspace, must expose `--print-image-inputs-sha256`, and must reuse the already-existing `LIBEXIF_ORIGINAL_TEST_IMAGE` tag only when its recorded image-input digest still matches the current harness inputs. It must not rebuild `safe/` inside the container once the phase-local package root exists.
  - The image-input digest printed by `./test-original.sh --print-image-inputs-sha256` and stored on `libexif-original-test:impl_05_downstream_compile` must cover `test-original.sh`, `dependents.json`, and any checked-in `safe/tests/downstream/**` helpers used by this phase.
  - `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/` is reserved for host-visible downstream summaries and per-run logs. Keep it outside `metadata/package-inputs.sha256` and `validated.ok`; only the validated package tree above `downstream/` participates in package-root freshness checks.
  - The downstream harness must keep `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT` reusable after each compile-mode run. Summary or per-dependent log writes may happen only under `downstream/`; the validated package contents and metadata must remain untouched so `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh` still succeeds after `./test-original.sh --mode compile`.
  - After `test-original.sh` validates `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT`, treat every path under that root except `downstream/` as read-only. If containerized probes need writable host-visible output, route it through `downstream/` or another separate writable mount rather than mutating the validated package tree.
  - With `LIBEXIF_REQUIRE_REUSE=1`, `test-original.sh` must emit `reuse-required docker image is missing or stale` for a deliberately missing or stale `LIBEXIF_ORIGINAL_TEST_IMAGE`, and `reuse-required package root is missing or stale` for a deliberately missing or stale `LIBEXIF_DOWNSTREAM_PACKAGE_ROOT`.
  - Before any phase-5 checker invocation that uses `downstream/compile-matrix.tsv` as proof, delete that file from the phase-local root so the exact `./test-original.sh --mode compile` run must recreate it. A leftover 15-row summary from an earlier bounce is not acceptable evidence.
  - First delete any preexisting `downstream/compile-matrix.tsv`. Then `./test-original.sh --mode compile` may build the phase-local image if needed and must recreate the 15-row compile summary. Then `LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode compile --only exif` must succeed without rebuilding the image or repackaging `libexif-safe`, must leave the 15-row compile summary intact, and must leave the phase-local package root valid for `LIBEXIF_REQUIRE_REUSE=1 bash safe/tests/run-package-build.sh`. Deliberate missing-image, stale-image, missing-package-root, and stale-package-root reruns must fail with the exact reuse-only substrings above.
  - Use `./test-original.sh --mode compile --only <name>` as the tight failure reproduction loop for a single dependent.
  - For each consumer-visible failure, first reproduce it through the Docker harness, then create the smallest possible non-Docker regression under `safe/tests/` whenever feasible.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_05_downstream_compile_tester` (`check`, `bounce_target: impl_05_downstream_compile`)
    - Purpose: verify that every dependent in `dependents.json` still builds against the packaged safe headers or libraries inside the Ubuntu 24.04 Docker harness, reusing one phase-local package root and one phase-local image tag, and that the exact `--mode compile` run recreates a 15-row checker-visible success summary instead of reusing a stale partial subset.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv"
      expected_compile_names=$(mktemp)
      actual_compile_names=$(mktemp)
      missing_image_log=$(mktemp)
      missing_root_log=$(mktemp)
      stale_image_log=$(mktemp)
      stale_root_log=$(mktemp)
      missing_image_tag="${LIBEXIF_ORIGINAL_TEST_IMAGE}-missing"
      stale_image_tag="${LIBEXIF_ORIGINAL_TEST_IMAGE}-stale"
      missing_package_root="${LIBEXIF_DOWNSTREAM_PACKAGE_ROOT}.missing"
      stale_package_root="${LIBEXIF_DOWNSTREAM_PACKAGE_ROOT}.stale"
      trap 'rm -f "$expected_compile_names" "$actual_compile_names" "$missing_image_log" "$missing_root_log" "$stale_image_log" "$stale_root_log"; rm -rf "$stale_package_root"; docker image rm -f "$stale_image_tag" >/dev/null 2>&1 || true' EXIT
      rm -f "$compile_summary"
      ./test-original.sh --mode compile
      test -f "$compile_summary"
      test "$(head -n1 "$compile_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[].name' dependents.json >"$expected_compile_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode compile --only exif
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
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
      rm -rf "$missing_package_root"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="$missing_package_root" ./test-original.sh --mode compile --only exif >"$missing_root_log" 2>&1; then
        cat "$missing_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_root_log"
      rm -rf "$stale_package_root"
      cp -a "$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" "$stale_package_root"
      printf 'stale\n' >"$stale_package_root/metadata/source-commit.txt"
      if LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_DOWNSTREAM_PACKAGE_ROOT="$stale_package_root" ./test-original.sh --mode compile --only exif >"$stale_root_log" 2>&1; then
        cat "$stale_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$stale_root_log"
      ```
  - `check_05_downstream_compile_senior` (`check`, `bounce_target: impl_05_downstream_compile`)
    - Purpose: review the downstream compile additions, confirm the harness uses the existing inventory rather than rediscovering apps, confirm the container consumes prebuilt safe packages instead of rebuilding them, and confirm that the exact verifier run recreates a per-dependent compile summary naming all 15 dependents.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      compile_summary="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT/downstream/compile-matrix.tsv"
      expected_compile_names=$(mktemp)
      actual_compile_names=$(mktemp)
      missing_image_log=$(mktemp)
      stale_image_log=$(mktemp)
      stale_image_tag="${LIBEXIF_ORIGINAL_TEST_IMAGE}-stale"
      trap 'rm -f "$expected_compile_names" "$actual_compile_names" "$missing_image_log" "$stale_image_log"; docker image rm -f "$stale_image_tag" >/dev/null 2>&1 || true' EXIT
      rm -f "$compile_summary"
      ./test-original.sh --mode compile
      test -f "$compile_summary"
      test "$(head -n1 "$compile_summary")" = $'name\tsource_package\tassertion\tartifact\tstatus'
      jq -r '.dependents[].name' dependents.json >"$expected_compile_names"
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode compile --only exif
      awk -F'\t' 'NR > 1 { print $1 }' "$compile_summary" >"$actual_compile_names"
      diff -u "$expected_compile_names" "$actual_compile_names"
      test "$(awk -F'\t' 'NR > 1 && $5 == "ok" { count++ } END { print count + 0 }' "$compile_summary")" = "15"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$LIBEXIF_DOWNSTREAM_PACKAGE_ROOT" bash safe/tests/run-package-build.sh >/dev/null
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
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile bash safe/tests/run-package-build.sh`
  - `cd /home/yans/code/safelibs/ported/libexif && ./test-original.sh --print-image-inputs-sha256` must match `docker image inspect libexif-original-test:impl_05_downstream_compile --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}'`
  - `rm -f /home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile/downstream/compile-matrix.tsv && LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile ./test-original.sh --mode compile`
  - `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile/downstream/compile-matrix.tsv` must be recreated by that invocation with header `name<TAB>source_package<TAB>assertion<TAB>artifact<TAB>status`, its name column must diff cleanly against `jq -r '.dependents[].name' /home/yans/code/safelibs/ported/libexif/dependents.json`, and it must contain exactly 15 rows with `status=ok`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile ./test-original.sh --mode compile --only exif`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile-missing LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile ./test-original.sh --mode compile --only exif` must fail with `reuse-required docker image is missing or stale`
  - An intentionally stale Docker tag such as `docker tag ubuntu:24.04 libexif-original-test:impl_05_downstream_compile-stale` must cause `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile-stale LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile ./test-original.sh --mode compile --only exif` to fail with `reuse-required docker image is missing or stale`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile.missing ./test-original.sh --mode compile --only exif` must fail with `reuse-required package root is missing or stale`
  - A copied stale root made from `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_05_downstream_compile` with mismatched `metadata/source-commit.txt` must cause `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_05_downstream_compile LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=<stale-copy> ./test-original.sh --mode compile --only exif` to fail with `reuse-required package root is missing or stale`

## Git Commit Requirement
The implementer must commit work to git before yielding.
