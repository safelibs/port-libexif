# ABI, Headers, Packaging, and Reusable Package Artifacts

## Phase Name
`ABI, Headers, Packaging, and Reusable Package Artifacts`

## Implement Phase ID
`impl_01_abi_packaging`

## Preexisting Inputs
  - `original/libexif/Makefile.am`
  - `original/configure.ac`
  - `original/libexif/libexif.sym`
  - `original/debian/libexif12.symbols`
  - `original/libexif/exif-tag.c`
  - `safe/Cargo.toml`
  - `safe/Cargo.lock`
  - `safe/COPYING`
  - `safe/README`
  - `safe/NEWS`
  - `safe/SECURITY.md`
  - `safe/build.rs`
  - `safe/cshim/**`
  - `safe/src/ffi/{types.rs,panic_boundary.rs}`
  - `safe/src/lib.rs`
  - `safe/src/primitives/{byte_order.rs,format.rs,ifd.rs,utils.rs}`
  - `safe/src/runtime/{log.rs,mem.rs}`
  - `safe/src/object/{content.rs,data.rs,entry.rs}`
  - `safe/src/parser/{data_load.rs,data_save.rs,loader.rs}`
  - `safe/src/tables/{data_options.rs,tag_table.rs}`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `safe/include/**`
  - `safe/libexif.pc.in`
  - `safe/libexif-uninstalled.pc.in`
  - `safe/debian/**`
  - `safe/po/*.po`
  - `safe/po/*.gmo`
  - `safe/contrib/examples/**`
  - `safe/doc/libexif-api.html/**`
  - `safe/tests/abi_layout.rs`
  - `safe/tests/{run-package-build.sh,run-export-compare.sh,run-c-compile-smoke.sh,run-original-object-link-compat.sh,run-performance-compare.sh}`

## New Outputs
  - Updated Rust ABI/export implementation in `safe/src/ffi/{types.rs,panic_boundary.rs}`, `safe/src/lib.rs`, and the owning exported modules under `safe/src/{primitives,runtime,object,parser,tables,mnote}/` when `abi_layout`, export comparison, or C compile smoke exposes a layout or symbol mismatch
  - Updated `safe/build.rs` if the export map, SONAME flags, or build-time env propagation need correction
  - Updated `safe/debian/*` packaging metadata and install rules
  - Updated `safe/tests/abi_layout.rs`
  - Phase-local reusable package artifact root contract rooted at `safe/.artifacts/<implement-phase-id>/`, including `metadata/source-commit.txt`, `metadata/package-inputs.sha256`, `metadata/validated.ok`, and the non-mutating metadata-print path needed by verifiers
  - Deterministic compile-smoke outputs under `safe/.artifacts/<implement-phase-id>/compile-smoke/{shared,static}/`, including the forced-static `public-api-smoke-static` artifact that verifiers inspect directly
  - Updated package-backed wrapper scripts that reuse `PACKAGE_BUILD_ROOT` instead of rebuilding the same current-commit artifacts, and that turn `LIBEXIF_REQUIRE_REUSE=1` into checker-visible failures on accidental rebuilds

## File Changes
  - `safe/build.rs`
  - `safe/Cargo.toml` and `safe/Cargo.lock` only if build configuration or helper dependencies change
  - `safe/src/ffi/{types.rs,panic_boundary.rs}`
  - `safe/src/lib.rs`
  - `safe/src/primitives/{byte_order.rs,format.rs,ifd.rs,utils.rs}`
  - `safe/src/runtime/{log.rs,mem.rs}`
  - `safe/src/object/{content.rs,data.rs,entry.rs}`
  - `safe/src/parser/{data_load.rs,data_save.rs,loader.rs}`
  - `safe/src/tables/{data_options.rs,tag_table.rs}`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `safe/include/libexif/*.h`
  - `safe/libexif.pc.in`
  - `safe/libexif-uninstalled.pc.in`
  - `safe/debian/{rules,control,libexif12.symbols,libexif12.install,libexif-dev.install,libexif-dev.docs,libexif-doc.docs,libexif-doc.examples,libexif-doc.doc-base,changelog}`
  - `safe/tests/{abi_layout.rs,run-package-build.sh,run-export-compare.sh,run-c-compile-smoke.sh,run-original-object-link-compat.sh,run-performance-compare.sh}`

## Implementation Details
  - Preserve the existing ABI mirror strategy in `safe/src/ffi/types.rs`; fix layout or naming drift only if `abi_layout` or export checks prove a mismatch.
  - Handle public ABI and export-surface fixes in this phase. If `cargo test --release --test abi_layout` fails, update `safe/src/ffi/{types.rs,panic_boundary.rs}`. If `run-export-compare.sh` or `run-c-compile-smoke.sh` reveals missing, extra, or mis-exported public symbols, fix the owning Rust module and any `safe/src/lib.rs` export glue here instead of deferring the change.
  - Preserve the original header install set from `original/libexif/Makefile.am`. Do not replace copied headers with generated Rust output.
  - Preserve the original symbol or version contract from `original/libexif/libexif.sym` and `original/debian/libexif12.symbols`. If a symbol mismatch is found, fix the Rust export surface or packaging; do not silently edit the manifest unless the original manifest itself is provably wrong for this workspace.
  - Preserve Debian package ownership and install paths: runtime library under `libexif12`, headers, pkg-config data, and static archive under `libexif-dev`, docs, examples, and doc-base metadata under `libexif-doc` with files still living under `/usr/share/doc/libexif-dev/`.
  - Keep the real shared object file `libexif.so.12.3.4` plus direct `libexif.so.12` and `libexif.so` symlinks.
  - Make `safe/tests/run-package-build.sh` idempotent for a populated `PACKAGE_BUILD_ROOT`. It must record `metadata/source-commit.txt`, `metadata/package-inputs.sha256`, and `metadata/validated.ok`, it must expose `--print-package-inputs-manifest` for non-mutating checker diffs, and it may reuse an existing root only when those files still match the current workspace and the extracted package validation checks still pass.
  - `metadata/package-inputs.sha256` must be the exact sorted `sha256sum`-style manifest emitted by `bash tests/run-package-build.sh --print-package-inputs-manifest`, with one `<sha256><two spaces><repo-relative-path>` line per input file.
  - That manifest must cover every package-affecting file still consumed by the safe build: `safe/{Cargo.toml,Cargo.lock,build.rs,COPYING,README,NEWS,SECURITY.md,libexif.pc.in,libexif-uninstalled.pc.in}`, `safe/src/**`, `safe/cshim/**`, `safe/include/**`, `safe/debian/**`, `safe/po/**`, `safe/contrib/examples/**`, `safe/doc/libexif-api.html/**`, `safe/tests/run-package-build.sh`, and any `original/**` file still consumed by the build such as `original/libexif/libexif.sym` and `original/libexif/exif-tag.c`.
  - Preserve the consume-existing-artifacts contract for copied headers, docs, examples, and locales. When `safe/po/*.po`, `safe/po/*.gmo`, `safe/contrib/examples/**`, or `safe/doc/libexif-api.html/**` already exist, keep them as in-place package inputs rather than rediscovering or regenerating replacements.
  - `safe/tests/run-package-build.sh` validation must explicitly include `libexif-dev/usr/lib/$multiarch/libexif.a` and `root/usr/lib/$multiarch/libexif.a`. Do not treat the static archive as implied by the `.deb` build alone.
  - `safe/tests/run-c-compile-smoke.sh` must prove both shared-link and static-link compatibility from the packaged root. At least one smoke target must be forced to link against the packaged `libexif.a`, and the resulting binary must be written to `safe/.artifacts/<implement-phase-id>/compile-smoke/static/public-api-smoke-static`, must execute successfully, and must have no `NEEDED` entry for `libexif.so.12` in `readelf -d` output.
  - If the metadata or extracted-package validation no longer matches, `safe/tests/run-package-build.sh` must rebuild that root in place before any wrapper continues. If `LIBEXIF_REQUIRE_REUSE=1` is set, it must fail instead of rebuilding. This invalidation behavior is part of this phase.
  - When `LIBEXIF_REQUIRE_REUSE=1` blocks a missing or stale phase-local package root, `safe/tests/run-package-build.sh` must emit the exact substring `reuse-required package root is missing or stale`. Package-backed wrappers that delegate to it must preserve that substring so verifiers can assert the contract without inferring why the command failed.
  - `safe/tests/run-export-compare.sh`, `safe/tests/run-c-compile-smoke.sh`, `safe/tests/run-original-object-link-compat.sh`, and `safe/tests/run-performance-compare.sh` must preserve inherited `LIBEXIF_REQUIRE_REUSE=1` so later verifiers can prove wrapper-level reuse instead of a silent nested rebuild through `run-package-build.sh`. This phase covers missing-root and stale-root negative coverage for `run-export-compare.sh`.
  - Later phases may not overwrite or repurpose an earlier phase's package root. A later phase that changes code may create its own new phase-local package root for its own phase, but it must still reuse that root for all remaining package-backed commands in that phase.
  - Treat serial execution as part of the implementation. If a failure only reproduces under concurrent package builds, fix the workflow or harness ordering rather than adding retry loops.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_01_abi_tester` (`check`, `bounce_target: impl_01_abi_packaging`)
    - Purpose: verify the original ABI, the installed development surface including `libexif.a`, Debian packages and docs, and the reusable phase-local package-root contract for the root builder and export wrapper.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging
      expected_manifest=$(mktemp)
      missing_root_log=$(mktemp)
      missing_export_log=$(mktemp)
      stale_export_log=$(mktemp)
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      missing_root="${PACKAGE_BUILD_ROOT}.missing"
      stale_root="${PACKAGE_BUILD_ROOT}.stale"
      trap 'rm -f "$expected_manifest" "$missing_root_log" "$missing_export_log" "$stale_export_log"; rm -rf "$stale_root"' EXIT
      cargo test --release --test abi_layout
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
      rm -rf "$missing_root"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_root" bash tests/run-package-build.sh >"$missing_root_log" 2>&1; then
        cat "$missing_root_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_root_log"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$missing_root" bash tests/run-export-compare.sh >"$missing_export_log" 2>&1; then
        cat "$missing_export_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$missing_export_log"
      rm -rf "$stale_root"
      cp -a "$PACKAGE_BUILD_ROOT" "$stale_root"
      printf 'stale\n' >"$stale_root/metadata/source-commit.txt"
      if LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$stale_root" bash tests/run-export-compare.sh >"$stale_export_log" 2>&1; then
        cat "$stale_export_log" >&2
        exit 1
      fi
      grep -F "reuse-required package root is missing or stale" "$stale_export_log"
      ```
  - `check_01_abi_senior` (`check`, `bounce_target: impl_01_abi_packaging`)
    - Purpose: review ABI or packaging drift against the original manifests, confirm the static archive remains packaged and linkable, and confirm wrapper reuse of the fixed phase-local package root.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      export PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging
      expected_manifest=$(mktemp)
      static_smoke="$PACKAGE_BUILD_ROOT/compile-smoke/static/public-api-smoke-static"
      trap 'rm -f "$expected_manifest"' EXIT
      multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
      test -f "$PACKAGE_BUILD_ROOT/libexif-dev/usr/lib/$multiarch/libexif.a"
      bash tests/run-package-build.sh --print-package-inputs-manifest >"$expected_manifest"
      diff -u "$expected_manifest" "$PACKAGE_BUILD_ROOT/metadata/package-inputs.sha256"
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-export-compare.sh
      LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT="$PACKAGE_BUILD_ROOT" bash tests/run-c-compile-smoke.sh
      test -x "$static_smoke"
      "$static_smoke"
      ! readelf -d "$static_smoke" | grep -F 'libexif.so.12'
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `cargo test --release --test abi_layout`
  - `PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging bash tests/run-package-build.sh`
  - `bash tests/run-package-build.sh --print-package-inputs-manifest` must diff cleanly against `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging/metadata/package-inputs.sha256`
  - `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging/metadata/package-inputs.sha256` must remain the exact sorted `sha256sum`-style manifest over the required package-affecting inputs, including `safe/po/**`, `safe/contrib/examples/**`, `safe/doc/libexif-api.html/**`, and any consumed `original/**` build inputs
  - `test -f /home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging/libexif-dev/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.a`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging bash tests/run-export-compare.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging bash tests/run-c-compile-smoke.sh`
  - `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging/compile-smoke/static/public-api-smoke-static` must exist, execute successfully, and have no `NEEDED` entry for `libexif.so.12` in `readelf -d`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging.missing bash tests/run-package-build.sh` must fail with `reuse-required package root is missing or stale`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging.missing bash tests/run-export-compare.sh` must fail with `reuse-required package root is missing or stale`
  - A copied stale root made from `/home/yans/code/safelibs/ported/libexif/safe/.artifacts/impl_01_abi_packaging` with mismatched `metadata/source-commit.txt` must cause `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=<stale-copy> bash tests/run-export-compare.sh` to fail with `reuse-required package root is missing or stale`

## Git Commit Requirement
The implementer must commit work to git before yielding.
