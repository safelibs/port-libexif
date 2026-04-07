# Safety Audit

This port still contains `unsafe`, but the remaining surface is confined to
explicit ABI, raw-pointer, and foreign-function boundaries. The final audit
command for this phase was:

```sh
rg -l '\bunsafe\b' src tests build.rs | LC_ALL=C sort
```

`build.rs` contains no `unsafe`. It generates the tag-table data from the
preprocessed upstream `exif-tag.c` input and compiles exactly one C
translation unit: `cshim/exif-log-shim.c`.

## MakerNote status

The vendor MakerNote helper C sources are fully gone from the safe overlay
build. Apple, Canon, Fuji, Olympus, and Pentax MakerNote parsing and
formatting now live in `src/mnote/*.rs`. The remaining foreign surface is
limited to the variadic logging shim plus libc/gettext calls.

## Interpreting the raw audit output

The `rg -l '\bunsafe\b' ...` command intentionally over-approximates. Some
hits are audit markers rather than additional unsafe implementation blocks.

### 0. FFI type definitions and safe export attributes

Files:

- `src/ffi/types.rs`
- `src/primitives/byte_order.rs`
- `src/primitives/format.rs`
- `src/primitives/ifd.rs`
- `src/tables/data_options.rs`
- `src/tables/tag_table.rs`

Why they appear:

- `src/ffi/types.rs` uses `unsafe extern "C" fn` in callback, allocator, and
  vtable type definitions because those function pointers describe C ABI
  slots whose implementations may dereference caller-controlled pointers.
- The getter modules still contain `#[unsafe(no_mangle)]` because Rust treats
  symbol export as an unsafe attribute even when the exported function itself
  is safe to call.

Current guardrails:

- `src/primitives/byte_order.rs`, `src/primitives/format.rs`,
  `src/primitives/ifd.rs`, `src/tables/data_options.rs`, and the enum/index
  getters in `src/tables/tag_table.rs` now expose safe `extern "C"` functions
  because they do not dereference caller-provided pointers.
- The pointer-bearing callback and allocator signatures in `src/ffi/types.rs`
  stay explicitly unsafe so the ABI contract remains visible at the type
  level.

## Remaining `unsafe` categories

### 1. Exported C ABI entry points that accept caller-owned pointers

Files:

- `src/primitives/utils.rs`
- `src/runtime/log.rs`
- `src/runtime/mem.rs`
- `src/tables/tag_table.rs`
- `src/object/content.rs`
- `src/object/data.rs`
- `src/object/entry.rs`
- `src/parser/data_load.rs`
- `src/parser/data_save.rs`
- `src/parser/loader.rs`
- `src/mnote/base.rs`
- `src/mnote/apple.rs`
- `src/mnote/canon.rs`
- `src/mnote/fuji.rs`
- `src/mnote/olympus.rs`
- `src/mnote/pentax.rs`

Why it remains:

- The libexif ABI accepts raw pointers, integers, function pointers, and
  caller-owned buffers.
- Rust must treat those entry points as unsafe when the caller can violate
  pointer validity, alignment, aliasing, size, or lifetime requirements.

Current guardrails:

- Every Rust-defined export still routes through `ffi::panic_boundary`, so
  Rust panics do not cross the ABI.
- Null checks, size checks, and bounds checks happen before dereferencing or
  copying when malformed input is possible.
- Pure enum and index getters that do not dereference caller-provided pointers
  are now safe `extern "C"` functions instead of `unsafe extern "C"` ones.

### 2. ABI-compatible raw object graph manipulation

Files:

- `src/object/content.rs`
- `src/object/data.rs`
- `src/object/entry.rs`
- `src/runtime/log.rs`
- `src/runtime/mem.rs`
- `src/mnote/base.rs`
- `src/mnote/mod.rs`
- `src/parser/loader.rs`
- `src/parser/data_save.rs`

Why it remains:

- The port preserves the published struct layouts from `src/ffi/types.rs`.
- That requires reading and writing fields through raw pointers owned by
  C-facing objects, including manual refcounts, parent pointers, and
  allocator-private payloads.

Current guardrails:

- Allocation and deallocation stay centralized through `ExifMem`.
- Pointer-chasing helpers check for null parents before dereferencing.
- Cast-only helper functions no longer carry `unsafe fn` markers; the
  remaining unsafe blocks are attached to the actual dereferences and writes.

### 3. Byte packing, unpacking, and slice bridging

Files:

- `src/primitives/utils.rs`
- `src/parser/data_load.rs`
- `src/parser/data_save.rs`
- `src/parser/loader.rs`
- `src/mnote/apple.rs`
- `src/mnote/canon.rs`
- `src/mnote/fuji.rs`
- `src/mnote/olympus.rs`
- `src/mnote/pentax.rs`

Why it remains:

- The public ABI still exposes raw byte buffers for integer and rational
  packing helpers.
- Parsing and serialization must bridge between caller-provided
  `*const u8` / `*mut u8` buffers and Rust slices.

Current guardrails:

- Length and offset arithmetic uses checked helpers in `src/parser/limits.rs`.
- Recursion, linked-offset, and parse-work budgets cover the surviving
  parser-logic CVE classes.
- Crafted regressions in `tests/cve_regressions.rs` cover cyclic IFD links,
  interoperability-budget exhaustion, thumbnail/content offset overflow,
  generic zero-denominator formatting, Olympus zero denominators, and Canon
  MakerNote expansion limits.

### 4. Narrow foreign-code boundaries

Files:

- `src/lib.rs`
- `src/i18n.rs`
- `src/runtime/cstdio.rs`
- `src/runtime/mem.rs`
- `src/mnote/apple.rs`
- `src/mnote/canon.rs`
- `src/mnote/fuji.rs`
- `src/mnote/olympus.rs`
- `src/mnote/pentax.rs`

Why it remains:

- Stable Rust cannot export C variadics, so the log/`va_list` edge stays in
  `cshim/exif-log-shim.c`.
- gettext integration (`bindtextdomain`, `bind_textdomain_codeset`,
  `dgettext`) and libc allocation/stdio calls are foreign code.
- MakerNote modules still call the exported variadic logging entry point, but
  the MakerNote parsing and formatting logic itself is now Rust.

Current guardrails:

- The foreign calls are narrow wrappers around specific libc/gettext/shim
  functions rather than broad foreign subsystems.
- `build.rs` keeps the compiled C surface to the single variadic log shim.
- The test suite still exercises MakerNote behavior through
  `run-test-mnote-matrix.sh`, `test-apple-mnote.c`, and the crafted Canon and
  Olympus regressions.

### 5. ABI and regression tests

Files:

- `tests/abi_layout.rs`
- `tests/cve_regressions.rs`
- `tests/object_model.rs`
- `tests/primitives_tables.rs`

Why it remains:

- The tests intentionally call the published C ABI directly and inspect
  ABI-visible structs.
- That requires FFI declarations, raw field reads, and deliberate malformed
  inputs that safe wrappers would reject.

Current guardrails:

- The unsafe test code is not shipped in the library.
- Each test keeps ownership and teardown local so probes do not leak across
  cases.

## Unsafe removed in this phase

- The phase removed unnecessary `unsafe` from ABI-stable getter exports that
  only operate on enum or index values.
- Cast-only helper functions in the logging, allocator, loader, and MakerNote
  modules no longer require `unsafe fn`.
- The final non-Rust implementation surface is now documented as the variadic
  log shim plus libc/gettext boundaries, with vendor MakerNote helper C
  sources fully absent from the safe overlay build.
