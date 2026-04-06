# Core Parser, Tables, Object Model, and Existing Regression Surface

## Phase Name
`Core Parser, Tables, Object Model, and Existing Regression Surface`

## Implement Phase ID
`impl_02_core_behavior`

## Preexisting Inputs
  - `safe/build.rs`
  - `safe/src/lib.rs`
  - `safe/src/ffi/types.rs`
  - `safe/src/i18n.rs`
  - `safe/po/*.po`
  - `safe/po/*.gmo`
  - `safe/src/primitives/*`
  - `safe/src/runtime/*`
  - `safe/src/object/*`
  - `safe/src/parser/*`
  - `safe/src/tables/*`
  - `safe/src/mnote/*`
  - `safe/tests/{primitives_tables.rs,object_model.rs,cve_regressions.rs,ported_c.rs,run-original-test-suite.sh,run-c-test.sh,run-original-shell-test.sh,run-original-nls-test.sh,run-test-mnote-matrix.sh}`
  - `safe/tests/original-c/**`
  - `safe/tests/original-sh/**`
  - `safe/tests/support/**`
  - `safe/tests/testdata/*.jpg`
  - `safe/tests/testdata/*.parsed`
  - `original/libexif/exif-tag.c`
  - `relevant_cves.json`
  - `safe/tests/cve-regressions-manifest.txt`

## New Outputs
  - Updated Rust logic in primitives, runtime, object, parser, table, i18n, and current helper-backed MakerNote modules when the copied upstream suite exposes a compatibility bug
  - Updated or additional minimal regressions in `safe/tests/`
  - Updated copied original test harnesses, shell or NLS runners, nested NLS fixtures, or generated tag-metadata wiring only if needed to preserve the intended original behavior contract

## File Changes
  - `safe/build.rs`
  - `safe/src/lib.rs`
  - `safe/src/i18n.rs`
  - `safe/src/primitives/{byte_order.rs,format.rs,ifd.rs,utils.rs}`
  - `safe/src/runtime/{mem.rs,log.rs,cstdio.rs}`
  - `safe/src/object/{content.rs,data.rs,entry.rs}`
  - `safe/src/parser/{data_load.rs,data_save.rs,loader.rs,limits.rs,mod.rs}`
  - `safe/src/tables/{mod.rs,data_options.rs,gps_ifd.rs,tag_table.rs}`
  - `safe/src/mnote/{mod.rs,base.rs,apple.rs,canon.rs,fuji.rs,olympus.rs,pentax.rs}`
  - `safe/tests/{primitives_tables.rs,object_model.rs,cve_regressions.rs,ported_c.rs,run-original-test-suite.sh,run-c-test.sh,run-cve-regressions.sh,run-original-shell-test.sh,run-original-nls-test.sh,run-test-mnote-matrix.sh}`
  - `safe/tests/original-c/**`
  - `safe/tests/original-sh/**`
  - `safe/tests/support/**`

## Implementation Details
  - Preserve the ABI-visible public-struct semantics. C callers are still allowed to observe and mutate fields such as `ExifData::ifd`, `ExifData::data`, `ExifContent::entries`, `ExifEntry::data`, and `ExifEntry::parent`.
  - Use the existing regression harnesses as the driver for fixes: null handling, default-tag insertion, byte-order conversion, `exif_loader_get_buf`, `exif_data_fix`, `exif_content_fix`, `exif_entry_get_value`, parse or save symmetry, table-string behavior, tag-table ordering or lookup, current helper-backed MakerNote behavior exercised by `run-original-test-suite.sh`, NLS or localedir behavior, and shell-script compatibility.
  - Handle any compatibility failure exposed by `bash tests/run-original-test-suite.sh` in this phase. Do not defer such failures to phase 3 just because the exercised path touches `safe/src/mnote/*`; phase 3 is reserved for removing vendor-helper C and making those MakerNote paths Rust-owned after they are behaviorally correct.
  - If the copied-suite failure comes from generated tag metadata or gettext or localedir wiring, fix the extractor or env propagation in `safe/build.rs` and keep the resulting behavior covered by `primitives_tables`, `test-tagtable`, or the NLS runner rather than inventing a separate phase.
  - Preserve checked arithmetic and parser budgets in `safe/src/parser/limits.rs`; expand them only when a legitimate compatibility case requires it and a regression test proves the new bound.
  - Preserve parser-fixture contracts exactly. Parser tests that currently rely on explicit fixture argv or `TEST_IMAGES` must keep those explicit inputs instead of being loosened into rediscovery, directory scans, or regenerated fixture selection.
  - When a bug is found via an upstream-style C test or shell script, add or tighten the smallest possible regression under `safe/tests/` so the issue becomes locally reproducible without depending solely on the full suite.
  - Checks that compare safe and original behavior must keep `LC_ALL=C`, `LANG=`, and `LANGUAGE=` pinned exactly as the existing scripts do.
  - Preserve the copied shell and NLS fixtures in place. Update nested `safe/tests/original-c/**` or `safe/tests/original-sh/**` only when that preserves the original test intent rather than papering over a library bug.
  - Prefer safe helper functions over widening raw-pointer manipulation or adding new `unsafe`.
  - Commit requirement: commit phase changes to git before yielding.

## Verification Phases
  - `check_02_core_tester` (`check`, `bounce_target: impl_02_core_behavior`)
    - Purpose: verify the Rust object model, parser, scalar helpers, table metadata and strings, helper-backed MakerNote behavior exercised by the copied upstream suite, shell regressions, the NLS surface, and current CVE regressions.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      cargo test --release --test primitives_tables
      cargo test --release --test object_model
      cargo test --release --test cve_regressions
      bash tests/run-original-test-suite.sh
      ```
  - `check_02_core_senior` (`check`, `bounce_target: impl_02_core_behavior`)
    - Purpose: review fixes for parser, table, i18n, helper-backed MakerNote, and object correctness, ensure each failure has a minimal regression, and confirm that checks comparing safe and original behavior still keep `LC_ALL=C`, `LANG=`, and `LANGUAGE=` pinned exactly as the existing scripts do.
    - Commands:
      ```bash
      cd /home/yans/code/safelibs/ported/libexif/safe
      bash tests/run-original-test-suite.sh
      git show --stat --format=fuller HEAD
      ```

## Success Criteria
  - `cargo test --release --test primitives_tables`
  - `cargo test --release --test object_model`
  - `cargo test --release --test cve_regressions`
  - `bash tests/run-original-test-suite.sh`

## Git Commit Requirement
The implementer must commit work to git before yielding.
