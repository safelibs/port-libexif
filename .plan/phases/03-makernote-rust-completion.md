# MakerNote Rust Completion

## Phase Name
`MakerNote Rust Completion`

## Implement Phase ID
`impl_03_makernote_rust_completion`

## Preexisting Inputs
  - `safe/build.rs`
  - `safe/cshim/exif-log-shim.c`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `original/libexif/{apple,canon,fuji,olympus,pentax}/*`
  - `safe/tests/run-original-test-suite.sh`
  - `safe/tests/run-test-mnote-matrix.sh`
  - `safe/tests/original-c/{test-apple-mnote.c,test-fuzzer.c,test-fuzzer-persistent.c,test-mnote.c}`
  - `safe/tests/testdata/*`
  - `safe/tests/cve_regressions.rs`

## New Outputs
  - Rust implementations of the vendor MakerNote helpers that currently come from `build.rs`
  - Simplified `safe/build.rs` with vendor-helper C compilation removed after the Rust replacements are complete
  - Additional MakerNote regressions or fixtures if downstream behavior requires them
  - Updated copied upstream MakerNote-sensitive runners or focused C tests only when preserving their original regression intent requires harness maintenance alongside the Rust port
  - Updated `test-original.sh` usage or help text only if the reason it stages `original/` changes

## File Changes
  - `safe/build.rs`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `safe/src/object/data.rs` and `safe/src/parser/{data_load.rs,data_save.rs}` if MakerNote integration points need adjustment
  - `safe/tests/{cve_regressions.rs,run-original-test-suite.sh,run-test-mnote-matrix.sh,run-c-test.sh}`
  - `safe/tests/original-c/{test-apple-mnote.c,test-fuzzer.c,test-fuzzer-persistent.c,test-mnote.c}`
  - `safe/tests/testdata/*` only if new MakerNote fixtures are truly needed
  - `test-original.sh` only if its usage or staging comments must describe a narrower remaining `original/` dependency

## Implementation Details
  - Port the remaining vendor-specific MakerNote identification, load or save, enumeration, and formatting logic from the vendored C helper sources into Rust.
  - Replace the concrete helper seams already visible in the current tree: `safe/src/mnote/apple.rs` and `safe/src/mnote/fuji.rs` are thin helper trampolines only, while `safe/src/mnote/canon.rs`, `safe/src/mnote/olympus.rs`, and `safe/src/mnote/pentax.rs` expose public ABI entry points that still call `safe_helper_*` foreign functions. Phase 3 removes those helper dependencies while keeping the Canon, Olympus, and Pentax exported symbol surface unchanged.
  - Preserve the exported ABI and visible behavior of `exif_mnote_data_canon_new`, `exif_mnote_data_olympus_new`, `exif_mnote_data_pentax_new`, `mnote_canon_entry_get_value`, `mnote_olympus_entry_get_value`, `mnote_pentax_entry_get_value`, and the associated tag metadata functions.
  - Keep `ExifMnoteDataMethods` and `ExifMnoteData` layout-compatible with the original `exif-mnote-data-priv.h` contract.
  - Keep the existing copied upstream MakerNote-sensitive checks in scope because helper removal can regress parser-adjacent MakerNote behavior: `run-test-mnote-matrix.sh`, `test-apple-mnote`, and `test-fuzzer` as orchestrated by `bash tests/run-original-test-suite.sh`. Do not defer that coverage to phase 9.
  - Phase-3 verifiers must execute `bash tests/run-test-mnote-matrix.sh` explicitly in addition to `bash tests/run-original-test-suite.sh`. Do not rely on the wrapper suite alone to prove the dedicated MakerNote matrix still runs after helper-removal edits.
  - Delete `build.rs` MakerNote helper compilation once the Rust replacements are in place. The only planned remaining C shim should be the narrow variadic logging boundary, unless that can also be removed safely.
  - The helper-removal proof in this phase must target the real vendored build inputs and the surviving C-build structure, not just today's helper symbol names. The phase-3 verifiers must reject any remaining `build.rs` reference to `../original/libexif/{apple,canon,fuji,olympus,pentax}/*.c`, any `cargo:rerun-if-changed` for those files, and any helper-builder scaffolding such as `compile_mnote_helper`, `build_mnote_helpers`, `emit_mnote_rerun_hints`, or `safe_helper_*` wrapper names. They must also prove that `build.rs` compiles exactly one C source file via `cc::Build`, namely `cshim/exif-log-shim.c`; no extra `.file(...)` or `.files(...)` calls may remain.
  - Removing vendor-helper compilation does not authorize removing every `../original` build input. `build.rs` may continue consuming `original/libexif/libexif.sym` and `original/libexif/exif-tag.c` until there is an already-committed replacement for the symbol map and tag-table source of truth.
  - If phase 3 narrows the reason `test-original.sh` stages `original/`, update its usage or help text to describe the remaining dependency accurately rather than deleting the staging step prematurely.
  - Preserve existing CVE protections such as Olympus zero-denominator formatting and Canon size limits while moving the implementation into Rust.
  - Add focused regressions for any newly ported MakerNote path that previously relied only on the helper C code.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_03_mnote_tester` (`check`, `bounce_target: impl_03_makernote_rust_completion`)
    - Purpose: verify Apple, Canon, Fuji, Olympus, and Pentax MakerNote behavior after helper removal, keep copied upstream MakerNote-sensitive regressions passing, and confirm that `safe/build.rs` no longer compiles or tracks vendored MakerNote helper C sources from `original/libexif/{apple,canon,fuji,olympus,pentax}` or any MakerNote C.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      bash tests/run-original-test-suite.sh
      bash tests/run-test-mnote-matrix.sh
      cargo test --release --test cve_regressions
      ! rg -n '\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/[^"]+\.c' build.rs
      ! rg -n 'cargo:rerun-if-changed=\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/' build.rs
      rg -n 'cshim/exif-log-shim\.c' build.rs
      test "$(rg -c '\.file\(' build.rs)" = "1"
      ! rg -n '\.files\(' build.rs
      ! rg -n 'compile_mnote_helper|build_mnote_helpers|emit_mnote_rerun_hints|safe_helper_exif_mnote_data_|safe_helper_mnote_' build.rs src/mnote
      ```
  - `check_03_mnote_senior` (`check`, `bounce_target: impl_03_makernote_rust_completion`)
    - Purpose: review the MakerNote porting diff, confirm exported MakerNote symbols and outputs stay compatible, keep copied upstream MakerNote-sensitive checks passing, and confirm that the only remaining compiled C source in `build.rs` is the log shim rather than MakerNote helper code.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      bash tests/run-original-test-suite.sh
      bash tests/run-test-mnote-matrix.sh
      ! rg -n '\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/[^"]+\.c' build.rs
      ! rg -n 'cargo:rerun-if-changed=\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/' build.rs
      rg -n 'cshim/exif-log-shim\.c' build.rs
      test "$(rg -c '\.file\(' build.rs)" = "1"
      ! rg -n '\.files\(' build.rs
      ! rg -n 'compile_mnote_helper|build_mnote_helpers|emit_mnote_rerun_hints|safe_helper_exif_mnote_data_|safe_helper_mnote_' build.rs src/mnote
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `bash tests/run-original-test-suite.sh`
  - `bash tests/run-test-mnote-matrix.sh`
  - `cargo test --release --test cve_regressions`
  - `! rg -n '\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/[^"]+\.c' build.rs`
  - `! rg -n 'cargo:rerun-if-changed=\.\./original/libexif/(apple|canon|fuji|olympus|pentax)/' build.rs`
  - `rg -n 'cshim/exif-log-shim\.c' build.rs`
  - `test "$(rg -c '\.file\(' build.rs)" = "1"`
  - `! rg -n '\.files\(' build.rs`
  - `! rg -n 'compile_mnote_helper|build_mnote_helpers|emit_mnote_rerun_hints|safe_helper_exif_mnote_data_|safe_helper_mnote_' build.rs src/mnote`

## Git Commit Requirement
The implementer must commit work to git before yielding.
