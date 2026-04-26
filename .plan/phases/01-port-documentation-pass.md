# Authoritative Port Documentation Pass

## Phase Name
`Authoritative Port Documentation Pass`

## Implement Phase ID
`impl_01_port_documentation_pass`

## Preexisting Inputs
- `safe/Cargo.toml`
- `safe/Cargo.lock`
- `safe/build.rs`
- `safe/src/lib.rs`
- `safe/src/ffi/mod.rs`
- `safe/src/ffi/types.rs`
- `safe/src/ffi/panic_boundary.rs`
- `safe/src/object/mod.rs`
- `safe/src/object/content.rs`
- `safe/src/object/data.rs`
- `safe/src/object/entry.rs`
- `safe/src/parser/mod.rs`
- `safe/src/parser/data_load.rs`
- `safe/src/parser/data_save.rs`
- `safe/src/parser/limits.rs`
- `safe/src/parser/loader.rs`
- `safe/src/primitives/mod.rs`
- `safe/src/primitives/byte_order.rs`
- `safe/src/primitives/format.rs`
- `safe/src/primitives/ifd.rs`
- `safe/src/primitives/utils.rs`
- `safe/src/runtime/mod.rs`
- `safe/src/runtime/cstdio.rs`
- `safe/src/runtime/log.rs`
- `safe/src/runtime/mem.rs`
- `safe/src/tables/mod.rs`
- `safe/src/tables/data_options.rs`
- `safe/src/tables/gps_ifd.rs`
- `safe/src/tables/tag_table.rs`
- `safe/src/mnote/mod.rs`
- `safe/src/mnote/base.rs`
- `safe/src/mnote/apple.rs`
- `safe/src/mnote/canon.rs`
- `safe/src/mnote/fuji.rs`
- `safe/src/mnote/olympus.rs`
- `safe/src/mnote/pentax.rs`
- `safe/src/i18n.rs`
- `safe/cshim/exif-log-shim.c`
- `safe/include/libexif/*`
- `safe/libexif.pc.in`
- `safe/libexif-uninstalled.pc.in`
- `safe/debian/*`
- `safe/doc/libexif-api.html/*`
- `safe/po/*`
- `safe/README`
- `safe/SAFETY.md`
- `safe/NEWS`
- `safe/SECURITY.md`
- `safe/tests/*.rs`
- `safe/tests/run-*.sh`
- `safe/tests/original-c/*`
- `safe/tests/original-sh/*`
- `safe/tests/perf/*`
- `safe/tests/smoke/*`
- `safe/tests/support/*`
- `safe/tests/testdata/*`
- `safe/tests/link-compat/object-manifest.txt`
- `safe/tests/link-compat/run-manifest.txt`
- `safe/.artifacts/impl_09_final_release` if valid
- `safe/target` as a preexisting untracked build-output tree that later git-state checks must tolerate but never stage
- `dependents.json`
- `relevant_cves.json`
- `all_cves.json`
- `test-original.sh`
- `original/libexif/*`
- `original/debian/libexif12.symbols`
- `original/test/*`
- `original/test/nls/*`
- `original/contrib/examples/*`

## New Outputs
- `safe/PORT.md` created or refreshed in place, with exactly the six required body `##` headings in the required order and no additional `##` headings anywhere else in the file
- One documentation-pass git commit
- Optional incidental fixes only within the exact tracked-file allowlist when verification proves that a listed file is the smallest in-scope repair needed for this documentation pass
- No persistent sidecar reports, unsafe ledgers, scratch markdown files, alternate deliverables, or package-root evidence files

## File Changes
- `safe/PORT.md`
- Conditional incidental-fix targets only when verification proves they are required:
  - `safe/README`
  - `safe/SAFETY.md`
  - `safe/tests/run-performance-compare.sh`
  - `safe/tests/run-original-object-link-compat.sh`
  - `safe/tests/link-compat/object-manifest.txt`
  - `safe/tests/link-compat/run-manifest.txt`
- No other tracked file is in scope. If accurate documentation would require widening the patch, leave the file unchanged and document the mismatch in `safe/PORT.md` section 4 instead.

## Implementation Details
- Preserve the generated workflow contract while splitting this plan:
  - Linear execution only. Do not emit `parallel_groups` or any other parallel topology.
  - The phase order is fixed: `impl_01_port_documentation_pass`, `check_01_port_static_evidence`, `check_01_port_harness_status`, `check_01_port_final_sanity`, `check_01_port_commit_review`.
  - The workflow YAML must be fully self-contained and inline-only. Do not use top-level `include`, phase-level `prompt_file`, `workflow_file`, `workflow_dir`, `checks`, or any other YAML indirection.
  - Every verifier must be an explicit top-level `check` phase with fixed `bounce_target: impl_01_port_documentation_pass`.
  - Every implement prompt in the generated workflow must instruct the agent to commit work to git before yielding.
  - Do not create persistent sidecar outputs in the repo or reusable package root. Checkers may use shell variables, pipes, and `mktemp` files outside the repo and `PACKAGE_BUILD_ROOT`, provided they clean them with `trap`.
- Preserve these planning-time facts explicitly enough that the implementer and verifiers can rely on them without rediscovery:
  - `safe/PORT.md` does not exist at planning commit `9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c`, so the implementation should create it unless it already exists by the time the generated workflow runs.
  - `cargo metadata --manifest-path safe/Cargo.toml --no-deps` reports one package, that same package as the only workspace member, and the expected library, build-script, and integration-test targets.
  - `cargo tree --manifest-path safe/Cargo.toml` currently shows only direct build-dependency `cc v1.2.59` plus transitive build helpers `find-msvc-tools v0.1.9` and `shlex v1.3.0`.
  - `cargo geiger` is not installed in this environment, so the workflow must probe for it and report its absence rather than assuming it exists.
  - `cargo test --manifest-path safe/Cargo.toml --release`, `bash safe/tests/run-cve-regressions.sh`, `bash safe/tests/run-original-test-suite.sh`, `bash safe/tests/run-package-build.sh`, `bash safe/tests/run-export-compare.sh`, and `bash safe/tests/run-c-compile-smoke.sh` currently pass.
  - `bash safe/tests/run-performance-compare.sh` currently fails with `make: *** No rule to make target 'all'.  Stop.` while trying to ensure the upstream baseline library exists.
  - `bash safe/tests/run-original-object-link-compat.sh` currently fails with `fatal error: config.h: No such file or directory` while upstream object-link test builds look for generated headers.
  - `objdump -p safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4` currently shows `NEEDED` entries for `libgcc_s.so.1`, `libm.so.6`, `libc.so.6`, and the loader.
  - `grep -RIn '\bunsafe\b' safe` currently traverses the preexisting `safe/.artifacts/impl_09_final_release` package cache, duplicates `safe/NEWS` hits from extracted doc packages, and emits a broken-symlink error below `safe/.artifacts/impl_09_final_release/libexif-dev/usr/lib/x86_64-linux-gnu/libexif.so`; treat the raw full-tree sweep as evidence capture, not as a zero-stderr assertion, then derive the authoritative section-2 inventory from tracked source files.
  - `rg -n 'TODO|FIXME' safe -S` currently finds markers in `safe/tests/original-sh/check-failmalloc.sh`, `safe/tests/original-sh/extract-parse.sh`, and `safe/po/sk.po`; section 4 must resolve all of them.
  - `safe/tests/link-compat/object-manifest.txt` and `safe/tests/link-compat/run-manifest.txt` embed stale absolute paths to another checkout; section 4 must cite those manifest files and describe the portability defect without repeating the stale path literal.
  - `relevant_cves.json` embeds a stale historical `source_file` string that points outside this checkout; cite the checked-in JSON files, not that stale path literal.
  - `git status --short --untracked-files=all` is not empty at planning time because preexisting untracked `safe/.artifacts/` and `safe/target/` trees already exist; commit-review checks must allow those exact baseline trees, including descendants, while still rejecting new untracked paths elsewhere and any tracked worktree or index dirt.
  - The package-build and package-backed harnesses already use `safe/.artifacts/impl_09_final_release` as the reusable package root. Consume that artifact root through the existing scripts instead of inventing a new packaging flow.
  - `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` currently matches `git rev-parse HEAD`, and `safe/.artifacts/impl_09_final_release/metadata/package-inputs.sha256` currently matches `bash safe/tests/run-package-build.sh --print-package-inputs-manifest`. Because `safe/PORT.md` is intentionally not part of that package-input manifest, any documentation-only commit changes `HEAD` without changing package inputs; the implement phase must therefore refresh the package root after every `HEAD`-changing commit so later checks reuse it instead of refreshing it themselves.
- Preserve the consume-existing-artifacts contract:
  - Use the current `safe/` tree as authoritative and `original/` only as comparison evidence when it clarifies ABI, behavior, or packaging intent.
  - Consume existing artifacts in place instead of refetching, recollecting, or regenerating them from scratch. This includes `safe/README`, `safe/SAFETY.md`, `safe/NEWS`, `safe/SECURITY.md`, `safe/tests/**/*`, `dependents.json`, `relevant_cves.json`, `all_cves.json`, `test-original.sh`, `original/libexif/libexif.sym`, `original/debian/libexif12.symbols`, `original/libexif/exif-tag.c`, and the rest of the `original/` comparison tree that the build and tests already reference.
  - Preserve the consume-existing-artifacts contract for prepared package outputs. `safe/tests/run-package-build.sh` is the only approved refresher for `safe/.artifacts/impl_09_final_release`, but check phases must consume a package root that the implement phase has already refreshed.
  - Because `safe/PORT.md` is not a package input, rerun `PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-package-build.sh` immediately after every `HEAD`-changing commit, including the first required documentation commit and any later amend.
  - Check phases must first assert that the stored source commit still matches `git rev-parse HEAD` and that the stored input manifest still matches `bash safe/tests/run-package-build.sh --print-package-inputs-manifest`; only then may they run package-backed harnesses, and they must do so with both `PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release` and `LIBEXIF_REQUIRE_REUSE=1`.
  - Treat `dependents.json` and `test-original.sh` as the authoritative downstream inventory and workflow description, and do not regenerate either file.
- Build the document exactly as the source plan requires:
  - Create `safe/PORT.md` because it is absent at the planning commit. If it already exists when the workflow runs, update it in place and preserve any still-accurate prose.
  - If `safe/PORT.md` uses a title, keep it as a single `#` heading above the body. The six required sections must be the only `##` headings in the file, in this exact order: `High-level architecture`, `Where the unsafe Rust lives`, `Remaining unsafe FFI beyond the original ABI/API boundary`, `Remaining issues`, `Dependencies and other libraries used`, `How this document was produced`.
  - Build section 1 from the live code and packaging surface using `safe/Cargo.toml`, `cargo metadata`, `safe/src/lib.rs`, `safe/src/*/mod.rs`, `safe/src/ffi/types.rs`, `safe/src/ffi/panic_boundary.rs`, `safe/src/object/{data.rs,content.rs,entry.rs}`, `safe/src/parser/{data_load.rs,loader.rs,data_save.rs}`, `safe/src/mnote/mod.rs`, `safe/build.rs`, `safe/cshim/exif-log-shim.c`, `safe/include/libexif/*`, `safe/libexif.pc.in`, `safe/libexif-uninstalled.pc.in`, `safe/debian/{control,rules}`, `safe/doc/libexif-api.html/*`, `safe/po/*`, and `safe/tests/run-package-build.sh`.
  - State explicitly that the current tree is not an explicit multi-member Cargo workspace, `cargo metadata` reports only `libexif-safe` as the sole workspace member, and the build does not use `cbindgen` or `bindgen`.
  - Build section 2 from a full-tree unsafe scan that begins with raw `grep -RIn '\bunsafe\b' safe`, accepts exit status `0` or `2`, preserves the known broken-symlink stderr, and refines with `rg -n '\bunsafe\s+extern\b|\bunsafe\s+fn\b|\bunsafe\s+impl\b|\bunsafe\s*\{|#\[unsafe\(no_mangle\)\]' safe/src safe/tests safe/build.rs`.
  - Derive the authoritative section-2 inventory from tracked source files only: `safe/src/**/*`, `safe/tests/*.rs`, and `safe/build.rs`. Enumerate every real `unsafe` block, `unsafe fn`, `unsafe extern`, and `unsafe impl` as an individual cited site with `file:line` and a one-sentence justification. Test-only Rust sites under `safe/tests/*.rs` must appear as their own subgroup. State explicitly if no `unsafe impl` sites exist.
  - Explicitly classify non-section-2 raw hits such as `#[unsafe(no_mangle)]`, prose in `safe/SAFETY.md`, prose in `safe/NEWS`, changelog text in `safe/debian/changelog`, generated package-copy text under `safe/.artifacts/**`, and any other explained false-positive bucket.
  - Build section 3 from current foreign declarations in `safe/src/lib.rs`, `safe/src/i18n.rs`, `safe/src/runtime/{mem.rs,cstdio.rs}`, `safe/src/object/entry.rs`, and `safe/src/mnote/{canon,apple,fuji,olympus,pentax}.rs`, plus `objdump -p` on the packaged `libexif.so.12.3.4`.
  - Classify the MakerNote `exif_log` redeclarations as intended libexif ABI self-reentry. Mention them only as exclusions or implementation detail; do not count them as extra foreign-library dependencies in section 3.
  - For each remaining non-ABI FFI surface, record the exact symbol name, `file:line`, provider crate or system library, why it is needed, and a plausible future safe-Rust replacement or explicit statement that no realistic safe replacement is currently available.
  - Use `safe/src/i18n.rs` for `bindtextdomain`, `bind_textdomain_codeset`, and `dgettext`; `safe/src/runtime/mem.rs` for `calloc`, `realloc`, and `free`; `safe/src/runtime/cstdio.rs` for `printf`; `safe/src/object/entry.rs` for `time` and `localtime_r`; and `safe/src/lib.rs` plus `safe/cshim/exif-log-shim.c` for the local variadic logging bridge.
  - Build section 4 from `cargo test --release`, the shell harnesses under `safe/tests/`, `dependents.json`, `test-original.sh`, `relevant_cves.json`, `all_cves.json`, `safe/src/parser/{limits.rs,data_load.rs}`, and `safe/tests/cve_regressions.rs`.
  - Record the current pass or fail state of `run-cve-regressions.sh`, `run-original-test-suite.sh`, the post-commit `run-package-build.sh` refresh, `run-export-compare.sh`, `run-c-compile-smoke.sh`, `run-performance-compare.sh`, and `run-original-object-link-compat.sh`.
  - Treat the post-commit `run-package-build.sh` refresh as the authoritative package-build execution for this workflow. Later checks must validate and reuse the refreshed root under `LIBEXIF_REQUIRE_REUSE=1`; they must not refresh it themselves.
  - Resolve every live `TODO` and `FIXME` marker, including the exact current hits in `safe/tests/original-sh/check-failmalloc.sh`, `safe/tests/original-sh/extract-parse.sh`, and `safe/po/sk.po`, into a concrete caveat, harness limitation, or non-code localization note.
  - Mention the dead-code warnings from `safe/src/mnote/canon.rs` and `safe/src/mnote/olympus.rs` if they are still present, and the `dwz` and `dpkg-shlibdeps` warnings if they are still present.
  - Describe the Docker-based downstream matrix using `dependents.json` and `test-original.sh`, and state clearly whether it was rerun during the documentation refresh. Note that `dependents.json` is representative rather than exhaustive because its own metadata says so.
  - Call out the stale absolute-path coupling in `safe/tests/link-compat/object-manifest.txt` and `safe/tests/link-compat/run-manifest.txt` unless it is fixed incidentally, and describe the defect by citing those existing files instead of repeating the stale literal path. Likewise, cite `relevant_cves.json` and `all_cves.json` without copying the stale historical `source_file` literal out of `relevant_cves.json`.
  - Build section 5 from `safe/Cargo.toml`, `cargo tree`, `safe/debian/{control,rules}`, the packaging harnesses, and `objdump -p` on the packaged shared library. State explicitly that the only direct Cargo dependency is build-dependency `cc = "1.2"`, that `find-msvc-tools` and `shlex` appear only transitively behind `cc`, that the tree does not use `cbindgen` or `bindgen`, and that `pkg-config` appears only in the compile-smoke and performance harnesses.
  - Build section 6 as a short reproducibility note listing the exact commands actually used for metadata, tree, unsafe scan, TODO scan, `run-cve-regressions.sh`, `run-original-test-suite.sh`, package build, export comparison, compile smoke, failing harness capture, `nm`, and `objdump`, plus the key source and metadata files consulted.
  - Before committing, confirm that every path named in `safe/PORT.md` exists, every cited symbol is findable with `rg`, `nm -D`, or `objdump -T`, section 2 matches the full-tree unsafe sweep modulo explicitly classified noise, section 3 lists every remaining non-ABI FFI surface with symbol, `file:line`, provider, why, and replacement fields, section 4 matches the full-tree `TODO` and `FIXME` sweep, section 5 matches `safe/Cargo.toml`, `cargo tree`, Debian metadata, and the packaged library, no stale non-existent absolute path literal from `safe/tests/link-compat/*` or `relevant_cves.json` appears in `safe/PORT.md`, and no tracked file outside the explicit allowlist is modified.
- Commit behavior:
  - Do all repo edits in this single implement phase.
  - Commit before yielding with a documentation-pass message such as `docs: add authoritative libexif port report`.
  - Immediately after the first required documentation-pass commit, rerun `PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-package-build.sh` before yielding so the reusable package root is refreshed for the new `HEAD`.
  - If a later checker bounces back, update the work and amend that same commit rather than creating a second commit.
  - After any later amend that changes `HEAD`, rerun that same `PACKAGE_BUILD_ROOT=... bash safe/tests/run-package-build.sh` command before yielding again so `safe/.artifacts/impl_09_final_release/metadata/source-commit.txt` and the validated root stay aligned with the amended commit.
  - Never stage or commit `safe/.artifacts/` or `safe/target/`.

## Verification Phases
### `check_01_port_static_evidence`
- Phase ID: `check_01_port_static_evidence`
- Type: `check`
- `bounce_target`: `impl_01_port_documentation_pass`
- Purpose: validate the linear inline-only workflow assumptions that the document describes, the crate and package architecture claims, the per-site `unsafe` ledger with `file:line` citations and one-sentence justifications, the classification of raw `.artifacts` sweep noise including the known broken-symlink stderr, the classification of internal self-ABI `extern` declarations versus true non-ABI FFI, the exact current `TODO` or `FIXME` locations, and the dependency and runtime-library claims directly against the live tree and reusable package root.
- Commands:
  - `cargo metadata --format-version 1 --manifest-path safe/Cargo.toml --no-deps`
  - `cargo tree --manifest-path safe/Cargo.toml`
  - Probe `cargo geiger --manifest-path safe/Cargo.toml` and report absence if the command is unavailable
  - Raw `grep -RIn '\bunsafe\b' safe` with acceptance of exit status `0` or `2`, and stdout and stderr captured only inline or in `mktemp` files outside the repo and `PACKAGE_BUILD_ROOT`, cleaned with `trap`
  - `rg -n '\bunsafe\s+extern\b|\bunsafe\s+fn\b|\bunsafe\s+impl\b|\bunsafe\s*\{|#\[unsafe\(no_mangle\)\]' safe/src safe/tests safe/build.rs`
  - `rg -n 'TODO|FIXME' safe -S`
  - `rg -n 'check-failmalloc\.sh|extract-parse\.sh|sk\.po|bindtextdomain|bind_textdomain_codeset|dgettext|calloc|realloc\(|free\(|printf\(|localtime_r|time\(|exif_log\(' safe safe/cshim original -S`
  - `dpkg-architecture -qDEB_HOST_MULTIARCH`
  - `objdump -p safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4`

### `check_01_port_harness_status`
- Phase ID: `check_01_port_harness_status`
- Type: `check`
- `bounce_target`: `impl_01_port_documentation_pass`
- Purpose: validate the remaining-issues section, package and runtime dependency section, reproducibility section, and package-root freshness against live harness outcomes and packaged artifacts without allowing the checker to refresh the package root on its own.
- Commands:
  - `cargo test --manifest-path safe/Cargo.toml --release`
  - `bash safe/tests/run-cve-regressions.sh`
  - `bash safe/tests/run-original-test-suite.sh`
  - Inline freshness assertions mirroring the current `ensure_package_root_is_valid` gate in `safe/tests/run-package-build.sh`, including `metadata/source-commit.txt`, `metadata/package-inputs.sha256`, `metadata/validated.ok`, the packaged `.deb` files, the packaged shared and static libraries and symlinks, `pkgconfig/libexif.pc`, installed headers, packaged docs, and locale `.mo` files
  - `cmp -s safe/.artifacts/impl_09_final_release/metadata/package-inputs.sha256 <(bash safe/tests/run-package-build.sh --print-package-inputs-manifest | sha256sum | cut -d' ' -f1)` or equivalent inline comparison against `bash safe/tests/run-package-build.sh --print-package-inputs-manifest`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-export-compare.sh`
  - `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-c-compile-smoke.sh`
  - Best-effort `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-performance-compare.sh` under `set +e`, with exit code and stderr kept inline or in `mktemp` files cleaned with `trap`
  - Best-effort `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-original-object-link-compat.sh` under `set +e`, with exit code and stderr kept inline or in `mktemp` files cleaned with `trap`
  - `objdump -p safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4`
  - Optional `nm -D --defined-only safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4`

### `check_01_port_final_sanity`
- Phase ID: `check_01_port_final_sanity`
- Type: `check`
- `bounce_target`: `impl_01_port_documentation_pass`
- Purpose: confirm that every path cited in `safe/PORT.md` exists, every cited symbol is grep-able or export-visible, every live `unsafe` and `TODO` or `FIXME` sweep is accounted for, stale absolute-path caveats are described through existing files rather than pasted literals, and the only `##` headings in `safe/PORT.md` are the six required headings in the required order.
- Commands:
  - `test -f safe/PORT.md`
  - `grep -n '^## ' safe/PORT.md`
  - An exact-heading `sed`/array or `awk` check that the only `##` headings are `High-level architecture`, `Where the unsafe Rust lives`, `Remaining unsafe FFI beyond the original ABI/API boundary`, `Remaining issues`, `Dependencies and other libraries used`, and `How this document was produced` in that order
  - Raw `grep -RIn '\bunsafe\b' safe` with acceptance of exit status `0` or `2` and without writing persistent evidence files
  - `rg -n 'TODO|FIXME' safe -S`
  - `objdump -T safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4`
  - `nm -D --defined-only safe/.artifacts/impl_09_final_release/root/usr/lib/$(dpkg-architecture -qDEB_HOST_MULTIARCH)/libexif.so.12.3.4`
  - Targeted `rg -n '<symbol>' safe original` and `test -e '<path>'` lookups for each symbol or path cited in the finished document

### `check_01_port_commit_review`
- Phase ID: `check_01_port_commit_review`
- Type: `check`
- `bounce_target`: `impl_01_port_documentation_pass`
- Purpose: confirm that no tracked worktree or index dirt remains, the only allowed untracked baseline is the preexisting `safe/.artifacts/` and `safe/target/` trees including descendants, the final diff is limited to `safe/PORT.md` plus the explicit incidental-fix allowlist, every non-`safe/PORT.md` committed file is individually justified by a concrete in-scope verification failure, and `HEAD` is the single documentation-pass commit relative to pinned start hash `9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c`.
- Commands:
  - `git status --porcelain=v1 --untracked-files=all`
  - An explicit allowlist check that rejects any status line other than `??` entries under `safe/.artifacts/` or `safe/target/`
  - `test -z "$(git diff --name-only)"`
  - `test -z "$(git diff --cached --name-only)"`
  - `git show --stat --format=fuller HEAD`
  - `git diff --name-only 9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c..HEAD`
  - An explicit committed-path allowlist limited to `safe/PORT.md`, `safe/README`, `safe/SAFETY.md`, `safe/tests/run-performance-compare.sh`, `safe/tests/run-original-object-link-compat.sh`, `safe/tests/link-compat/object-manifest.txt`, and `safe/tests/link-compat/run-manifest.txt`
  - Per-file `git diff --unified=0 9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c..HEAD -- <path>` inspection for every committed non-`safe/PORT.md` path to reject unrelated cleanup or broad refactors
  - `test "$(git rev-list --count 9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c..HEAD)" = 1`

## Success Criteria
- The generated split remains linear, inline-only, and explicit-phase only: one implement phase followed by four top-level `check` phases in the fixed order, with no YAML indirection and no parallel topology.
- The phase file preserves the consume-existing-artifacts contract, including reuse of `dependents.json`, `test-original.sh`, `original/`, and `safe/.artifacts/impl_09_final_release` instead of rediscovery or regeneration work.
- `safe/.artifacts/impl_09_final_release` is treated as a required preexisting input only if valid, and later verifiers reuse the package root under freshness checks plus `LIBEXIF_REQUIRE_REUSE=1` instead of rebuilding it themselves.
- The plan records the exact current runtime-library facts from `objdump -p`: `libgcc_s.so.1`, `libm.so.6`, `libc.so.6`, and the loader.
- The plan records the known raw `grep -RIn '\bunsafe\b' safe` caveat, including `.artifacts` package-copy noise, duplicated `safe/NEWS` hits from extracted docs, and the broken-symlink stderr beneath `safe/.artifacts/impl_09_final_release/libexif-dev/usr/lib/x86_64-linux-gnu/libexif.so`.
- The plan records the exact current `TODO` and `FIXME` locations in `safe/tests/original-sh/check-failmalloc.sh`, `safe/tests/original-sh/extract-parse.sh`, and `safe/po/sk.po`.
- `safe/PORT.md` exists and its only `##` headings, in order, are exactly `High-level architecture`, `Where the unsafe Rust lives`, `Remaining unsafe FFI beyond the original ABI/API boundary`, `Remaining issues`, `Dependencies and other libraries used`, and `How this document was produced`.
- Section 2 matches the full-tree raw unsafe sweep modulo explicitly classified non-code hits and generated `.artifacts` package-copy noise, and every real tracked-source unsafe site under `safe/src/**/*`, `safe/tests/*.rs`, and `safe/build.rs` is cited with `file:line` plus a one-sentence justification.
- Section 3 inventories every remaining non-ABI FFI surface with symbol, `file:line`, provider, why, and replacement fields, while treating the MakerNote `exif_log` redeclarations as self-ABI exclusions rather than extra dependencies.
- `cargo metadata --manifest-path safe/Cargo.toml --no-deps` and `cargo tree --manifest-path safe/Cargo.toml` match section 5 exactly, and `cargo geiger` is probed and reported accurately if unavailable.
- `rg -n 'TODO|FIXME' safe -S` is fully resolved in section 4.
- `cargo test --manifest-path safe/Cargo.toml --release`, `bash safe/tests/run-cve-regressions.sh`, and `bash safe/tests/run-original-test-suite.sh` succeed, and later package-backed checks consume the already refreshed `safe/.artifacts/impl_09_final_release` root under `LIBEXIF_REQUIRE_REUSE=1` instead of rebuilding it.
- `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-export-compare.sh` and `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-c-compile-smoke.sh` succeed after inline freshness validation.
- `LIBEXIF_REQUIRE_REUSE=1 PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-performance-compare.sh` and `... bash safe/tests/run-original-object-link-compat.sh` are rerun best-effort, and section 4 either records their exact blocker classes or removes the caveats if they pass.
- `nm -D --defined-only`, `objdump -T`, and `objdump -p` against the packaged `libexif.so.12.3.4` support the documented export-surface and runtime-library claims.
- Every path and symbol cited in `safe/PORT.md` is validated with `test -e`, `rg`, `nm`, or `objdump`, and `safe/PORT.md` does not copy any stale non-existent absolute path literal from `safe/tests/link-compat/*` or `relevant_cves.json`.
- Git review confirms no tracked worktree or index dirt remains, no new untracked path exists outside the preexisting `safe/.artifacts/**` and `safe/target/**` trees, the committed diff is limited to the allowed files, and exactly one commit exists in `9f7fcf07e3702f97d61ccb0d0b5eb48591d85b5c..HEAD`.

## Git Commit Requirement
The implementer must commit work to git before yielding. Use one documentation-pass commit, amend that same commit on any bounce-back, rerun `PACKAGE_BUILD_ROOT=/home/yans/safelibs/pipeline/ports/port-libexif/safe/.artifacts/impl_09_final_release bash safe/tests/run-package-build.sh` after every `HEAD`-changing commit, and never stage or commit `safe/.artifacts/` or `safe/target/`.
