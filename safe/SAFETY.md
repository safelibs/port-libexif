# Safety Audit

This port still contains `unsafe`, but it is now confined to explicit ABI, raw-pointer, and foreign-code boundaries. The remaining sites were audited with:

```sh
rg -l '\bunsafe\b' safe/src safe/tests safe/build.rs | LC_ALL=C sort
```

`safe/build.rs` no longer contains any `unsafe`.

## Remaining `unsafe` categories

### 1. Exported C ABI entry points

Files:

- `src/primitives/byte_order.rs`
- `src/primitives/format.rs`
- `src/primitives/ifd.rs`
- `src/primitives/utils.rs`
- `src/runtime/log.rs`
- `src/runtime/mem.rs`
- `src/tables/data_options.rs`
- `src/tables/tag_table.rs`
- `src/object/content.rs`
- `src/object/data.rs`
- `src/object/entry.rs`
- `src/parser/data_load.rs`
- `src/parser/data_save.rs`
- `src/parser/loader.rs`
- `src/mnote/base.rs`
- `src/mnote/canon.rs`
- `src/mnote/olympus.rs`
- `src/mnote/pentax.rs`

Why it remains:

- libexif exposes a C ABI that passes raw pointers, integers, and caller-owned buffers.
- Rust must model those entry points as `unsafe extern "C"` because the caller can violate pointer validity, alignment, aliasing, size, and lifetime requirements.

Current guardrails:

- Every Rust-defined exported function routes through `ffi::panic_boundary`, so Rust panics do not cross the ABI.
- Null-pointer and bounds checks happen before dereferencing or copying where the ABI allows malformed inputs.
- Parser entry points reject malformed offsets, recursive IFD links, oversized work budgets, and thumbnail/content range violations before copying bytes into Rust-owned objects.

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

- The port preserves libexif’s published struct layouts from `src/ffi/types.rs`.
- That requires reading and writing fields through raw pointers owned by C-facing objects, including manual refcounts, parent pointers, and allocator-private payloads.

Current guardrails:

- Allocation and deallocation are centralized through `ExifMem`.
- Pointer-chasing helpers check for null parents before dereferencing.
- Save/load code computes sizes with checked arithmetic and treats the serialized byte stream as untrusted input.

### 3. Byte packing, unpacking, and slice bridging

Files:

- `src/primitives/utils.rs`
- `src/parser/data_load.rs`
- `src/parser/data_save.rs`
- `src/parser/loader.rs`

Why it remains:

- The public ABI still exposes raw byte buffers for integer and rational packing helpers.
- Parsing and serialization must bridge between caller-provided `*const u8` / `*mut u8` buffers and Rust slices.

Current guardrails:

- Length/offset arithmetic uses checked helpers in `src/parser/limits.rs`.
- Recursion, linked-offset, and parse-work budgets cover the surviving parser-logic CVE classes.
- Crafted regressions in `tests/cve_regressions.rs` cover cyclic IFD links, interoperability-budget exhaustion, thumbnail/content offset overflow, generic zero-denominator formatting, Olympus zero denominators, and Canon MakerNote expansion limits.

### 4. Foreign helper and shim boundaries

Files:

- `src/lib.rs`
- `src/i18n.rs`
- `src/runtime/cstdio.rs`
- `src/mnote/apple.rs`
- `src/mnote/canon.rs`
- `src/mnote/fuji.rs`
- `src/mnote/olympus.rs`
- `src/mnote/pentax.rs`

Why it remains:

- Stable Rust cannot export C variadics, so the log/`va_list` edge stays in `cshim/exif-log-shim.c`.
- gettext integration (`dgettext`) is foreign code.
- Apple/Canon/Fuji/Olympus/Pentax MakerNote support still reuses the vendored upstream C helper sources compiled under renamed symbols by `build.rs`.

Current guardrails:

- These calls are narrow shims around specific foreign functions rather than ad hoc pointer arithmetic spread through the crate.
- The vendored helper code is isolated behind symbol-renamed wrapper functions, with Rust retaining ownership of the surrounding parser, allocator, and panic boundaries.
- The final test suite exercises MakerNote behavior through `run-test-mnote-matrix.sh`, `test-apple-mnote.c`, and the crafted Canon/Olympus regressions.

### 5. ABI and regression tests

Files:

- `tests/abi_layout.rs`
- `tests/cve_regressions.rs`
- `tests/object_model.rs`
- `tests/primitives_tables.rs`

Why it remains:

- The tests intentionally call the published C ABI directly and inspect ABI-visible structs.
- That requires FFI declarations, raw field reads, and deliberate construction of malformed inputs that safe wrappers would reject.

Current guardrails:

- The unsafe test code is not shipped in the library.
- Each test keeps ownership and teardown local so probes do not leak across cases.

## Unsafe removed in this phase

- The final hardening phase did not add any new library `unsafe`.
- Remaining hardening work focused on reducing risk around existing unsafe boundaries by tightening the test harnesses, adding crafted CVE regressions, and documenting the exact reason each unsafe category still exists.
