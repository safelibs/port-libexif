# Downstream CLI and Library Runtime Matrix

## Phase Name
`Downstream CLI and Library Runtime Matrix`

## Implement Phase ID
`impl_06_downstream_cli_library`

## Preexisting Inputs
  - `dependents.json`
  - `test-original.sh`
  - `safe/tests/testdata/*`
  - `safe/tests/*.rs`
  - `safe/tests/*.sh`
  - `safe/tests/original-c/**`
  - `safe/tests/original-sh/**`
  - `safe/tests/support/**`
  - `safe/tests/run-package-build.sh`

## New Outputs
  - Phase-local package root `safe/.artifacts/impl_06_downstream_cli_library/` with fresh package metadata and validated overlay contents
  - Phase-local Docker image `libexif-original-test:impl_06_downstream_cli_library` with recorded image-input freshness metadata
  - Updated downstream Docker assertions for these consumers
  - New minimal local regressions under `safe/tests/` for issues found through Docker
  - Library fixes required by consumer-visible runtime failures

## File Changes
  - `test-original.sh`
  - `dependents.json` only if a factual inventory error must be corrected in place
  - `safe/tests/*.rs`, `safe/tests/*.sh`, or `safe/tests/original-c/**` for distilled regressions
  - `safe/tests/testdata/*` only if a new fixture is required to reproduce a real downstream bug
  - `safe/src/**` as required by failures

## Implementation Details
  - Use the existing 15-entry `dependents.json` inventory. Do not rediscover a new app list.
  - Use `./test-original.sh --mode runtime --only <name>` for tight failure reproduction loops.
  - Runtime mode in this phase must cover only dependents whose `runtime_functionality` is non-null. `ImageMagick` is therefore intentionally excluded here and remains compile-mode-only; `./test-original.sh --mode runtime --only ImageMagick` must fail fast with the exact substring `compile-only dependent: ImageMagick only supports --mode compile` instead of being treated as a runtime probe.
  - Reuse the phase-local package root and phase-local Docker image tag for every `--only` invocation in this phase. Run one initial runtime invocation without `LIBEXIF_REQUIRE_REUSE`, then reuse-only reruns for the remaining apps and the compile-only negative case. Do not rebuild safe packages or the image once the phase-local artifacts exist.
  - Keep fixture generation inside `test-original.sh` as the source of truth for synthetic images; extend it in place only when a real app scenario needs a new EXIF shape.
  - Preserve the contract that the container installs locally built `safe/` Debian packages and that downstream binaries must resolve `libexif.so.12` to the safe package.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_06_downstream_tester` (`check`, `bounce_target: impl_06_downstream_cli_library`)
    - Purpose: verify the lower-friction downstream consumers in Docker, ensure every discovered runtime issue gets a local regression or tighter harness assertion, and reuse one phase-local package root and one phase-local image tag.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library
      expected_image_digest="$(./test-original.sh --print-image-inputs-sha256)"
      imagemagick_runtime_log=$(mktemp)
      trap 'rm -f "$imagemagick_runtime_log"' EXIT
      ./test-original.sh --mode runtime --only exif
      test "$(docker image inspect "$LIBEXIF_ORIGINAL_TEST_IMAGE" --format '{{ index .Config.Labels "io.safelibs.libexif.image-inputs-sha256" }}')" = "$expected_image_digest"
      for app in exiftran gphoto2 ruby-exif CamlImages; do
        LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only "$app"
      done
      if LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only ImageMagick >"$imagemagick_runtime_log" 2>&1; then
        cat "$imagemagick_runtime_log" >&2
        exit 1
      fi
      grep -F "compile-only dependent: ImageMagick only supports --mode compile" "$imagemagick_runtime_log"
      ```
  - `check_06_downstream_senior` (`check`, `bounce_target: impl_06_downstream_cli_library`)
    - Purpose: review each downstream runtime fix, confirm that Docker-only failures were converted into the smallest possible repo-local regression when feasible, and confirm that the harness still consumes `dependents.json` rather than rediscovering apps.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library
      PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-package-build.sh
      cd /home/yans/code/safelibs/ported/libexif
      export LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library
      export LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library
      imagemagick_runtime_log=$(mktemp)
      trap 'rm -f "$imagemagick_runtime_log"' EXIT
      ./test-original.sh --mode runtime --only exif
      for app in exiftran gphoto2 ruby-exif CamlImages; do
        LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only "$app"
      done
      if LIBEXIF_REQUIRE_REUSE=1 ./test-original.sh --mode runtime --only ImageMagick >"$imagemagick_runtime_log" 2>&1; then
        cat "$imagemagick_runtime_log" >&2
        exit 1
      fi
      grep -F "compile-only dependent: ImageMagick only supports --mode compile" "$imagemagick_runtime_log"
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library bash safe/tests/run-package-build.sh`
  - `LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only exif`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only exiftran`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only gphoto2`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only ruby-exif`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only CamlImages`
  - `LIBEXIF_REQUIRE_REUSE=1 LIBEXIF_ORIGINAL_TEST_IMAGE=libexif-original-test:impl_06_downstream_cli_library LIBEXIF_DOWNSTREAM_PACKAGE_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_06_downstream_cli_library ./test-original.sh --mode runtime --only ImageMagick` must fail with `compile-only dependent: ImageMagick only supports --mode compile`

## Git Commit Requirement
The implementer must commit work to git before yielding.
