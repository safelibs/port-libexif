# Safety Audit

This phase is an ABI scaffold. The crate keeps `unsafe` contained to the C ABI edge and to raw-byte integer packing helpers.

## Remaining `unsafe` categories

1. Exported `extern "C"` symbols in [src/lib.rs](/home/yans/code/safelibs/ported/libexif/safe/src/lib.rs).
   Every Rust-defined export routes through the shared unwind guard in [src/ffi/panic_boundary.rs](/home/yans/code/safelibs/ported/libexif/safe/src/ffi/panic_boundary.rs) so Rust panics do not cross the C ABI.

2. ABI-visible layout mirrors in [src/ffi/types.rs](/home/yans/code/safelibs/ported/libexif/safe/src/ffi/types.rs).
   These structs and scalar aliases intentionally mirror the vendored C headers, including opaque private-pointer fields that will later carry Rust-owned internals.

3. The `va_list` logging boundary in [cshim/exif-log-shim.c](/home/yans/code/safelibs/ported/libexif/safe/cshim/exif-log-shim.c).
   Stable Rust cannot export C variadics, so the scaffold keeps only the `ExifLog` variadic edge in C.

## Audit notes

- The installed header set under [include/libexif](/home/yans/code/safelibs/ported/libexif/safe/include/libexif) is copied byte-for-byte from the vendored upstream install list.
- The ELF export surface is constrained from the vendored [libexif.sym](/home/yans/code/safelibs/ported/libexif/original/libexif/libexif.sym) list in [build.rs](/home/yans/code/safelibs/ported/libexif/safe/build.rs).
- Layout regression coverage in [tests/abi_layout.rs](/home/yans/code/safelibs/ported/libexif/safe/tests/abi_layout.rs) compiles a C probe against the copied headers and the vendored private MakerNote header.
