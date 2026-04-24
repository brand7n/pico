//! Pico Runtime — native runtime for the picohp self-hosting compiler.
//!
//! All functions are `extern "C"` with `#[no_mangle]` for direct FFI
//! from LLVM IR or ARM64 assembly. Strings are null-terminated C strings.
//! Collections are opaque pointers to heap-allocated Rust structs.
//!
//! This crate compiles to a static library (`libpico_runtime.a`) that
//! gets linked into the final binary by clang or ld.
//!
//! Integer type: Pico `int` is i64 (pointer-sized on 64-bit platforms).
//! All FFI functions use i64 for int-typed parameters and returns.
//! Boolean returns use i32 (0 or 1).

mod alloc;
mod collection;
mod file;
mod regex;
mod string;
mod value;
