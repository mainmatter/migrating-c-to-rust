# Primitive types in FFI

So far, the exercises have passed data between C and Rust mostly as pointers and
plain integers. The following sections cover how other types cross the FFI
boundary, starting with primitive types, then structs, enums and unions, slices,
and opaque types.

A C header describes its parameters in C's own types: `int`, `long`, `char`,
`size_t`. Rust has `i32`, `i64`, `u8`, `usize`. Writing a binding means pairing
them up, and most of those pairs work on every platform you're likely to target.

Take this function:

```c
int scale(int value, int factor);
```

On Windows, macOS, and Linux, on x86-64 and aarch64, an `int` is the same as an
`i32` in Rust. If you only target those platforms, the Rust code looks like
this:

```rust,no_run
unsafe extern "C" {
    fn scale(value: i32, factor: i32) -> i32;
}
```

## The mapping

Here is how the common primitive types map from C to Rust when targeting
Windows, macOS, or Linux on x86-64 or aarch64:

| C                        | Rust                            | Alias         |
| ------------------------ | ------------------------------- | ------------- |
| `char`                   | `i8` (or `u8` on aarch64 Linux) | `c_char`      |
| `signed char`            | `i8`                            | `c_schar`     |
| `unsigned char`          | `u8`                            | `c_uchar`     |
| `short`                  | `i16`                           | `c_short`     |
| `unsigned short`         | `u16`                           | `c_ushort`    |
| `int`                    | `i32`                           | `c_int`       |
| `unsigned int`           | `u32`                           | `c_uint`      |
| `long`                   | `i64` (or `i32` on Windows)     | `c_long`      |
| `unsigned long`          | `u64` (or `u32` on Windows)     | `c_ulong`     |
| `long long`              | `i64`                           | `c_longlong`  |
| `unsigned long long`     | `u64`                           | `c_ulonglong` |
| `float`                  | `f32`                           | `c_float`     |
| `double`                 | `f64`                           | `c_double`    |
| `size_t`                 | `usize`                         | –             |
| `ptrdiff_t`              | `isize`                         | –             |
| `int32_t`, `uint64_t`, … | `i32`, `u64`, …                 | –             |
| `void *`                 | `*mut c_void`                   | –             |
| `T *`                    | `*mut T`                        | –             |
| `const T *`              | `*const T`                      | –             |

The names in the last column are type aliases from `core::ffi`. `std::ffi`
re-exports them so `std::ffi::c_int` and `core::ffi::c_int` are the same type.
`core::ffi` is the one to reach for in a `no_std` crate. You'll also see
`std::os::raw::c_int` in `bindgen` output and `libc::c_int` from the `libc`
crate. All of them resolve to the same alias, so you can mix them freely.

The aliases are for code that targets platforms where a fixed-width Rust type
doesn't match. On MSP430 or AVR, for example, C's `int` is 16 bits, so `c_int`
is `i16`. If your code has to build there as well as on x86-64, where `c_int` is
an `i32`, no single fixed-width type fits the signature, and the alias is the
only option.

That gives a rule of thumb for reading a header:

- **The width is up to the platform** (`int`, `long`, `unsigned short`): write
  the alias, unless you know every target you build for agrees on the width, in
  which case Rust's fixed-width type is a fair shortcut.
- **`char`:** always write `c_char`, even where you take the shortcut for other
  types. The next section explains why.
- **The width is already pinned** (`int32_t`, `uint64_t`): write Rust's
  fixed-width type, since the C type can't move either.
- **`size_t` and `ptrdiff_t`:** write `usize` and `isize`. They're pointer-sized
  in Rust, which is what `size_t` and `ptrdiff_t` are on every target Rust
  supports.
- **`void *`:** write `*mut c_void`, another alias from `core::ffi`. C uses it
  for "a pointer to something I'm not telling you about", and
  [Opaque types](05_05_opaque_types.md) covers what to do with such a pointer.

## `c_char` and `u8`

A C `char` is a byte, and Rust's byte type is `u8`, so writing `*const u8` in a
binding where the header says `const char *` looks harmless, and it's the most
common mistake in hand-written bindings: `c_char` is not always `u8`. It is `i8`
on x86-64, and on macOS and Windows for aarch64. It is `u8` on aarch64 Linux, on
32-bit ARM, and on RISC-V.

The `u8` binding might just work on your own machine, which is what makes this
easy to miss. Then someone builds for a target where `c_char` is an `i8` and the
code stops compiling. Anything that hands you a `c_char` no longer lines up with
the `u8` you wrote: `CStr::as_ptr` gives a `*const c_char`, and passing it where
a `*const u8` is expected is now a type mismatch. So write `c_char` in the
signature.

The second issue is that `c_char` is not the type Rust's byte methods work on.
`b'a'` literals, `u8::to_ascii_lowercase`, `str::as_bytes`, and `CStr::to_bytes`
all speak `u8`. So cast a `c_char` to `u8` as soon as you have the value, do the
work there, and cast back on the way out:

```rust,no_run
use std::ffi::c_char;

/// Lowercases `byte` if it is an ASCII letter, and returns it unchanged
/// otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn lower_byte(byte: c_char) -> c_char {
    (byte as u8).to_ascii_lowercase() as c_char
}
```

Two details about those casts are worth naming:

- **They change the type, not the value.** `as` between `c_char` and `u8` keeps
  the bit pattern exactly, and casting back gives you the same byte again.
  Nothing is lost in either direction, so the byte that reaches C is the byte
  the function produced.
- **They are allowed because the layouts match.** `c_char` and `u8` have the
  same size and the same alignment on every target, and every bit pattern is
  valid for both. That makes the cast purely a change of type: at runtime it is
  a no-op, and the compiler emits no instruction for it.

## Head to the exercise

The exercise gives you a C library over bytes with its bindings already written,
and two small wrappers to finish. They're in `u8` and `bool`, the way the rest
of Rust wants them, so each one converts at the boundary: a byte in and a byte
out, and C's `int` as a `bool`.

The interesting part is what counts as done. Before running the tests, `wr`
type-checks the crate for two targets, one where `c_char` is `i8` and one where
it is `u8`. If you cast to whichever type your own compiler accepts today, as
the existing code does, one of the two checks rejects it. Only `c_char` gets
past both. Neither check compiles or links anything, so no cross-compiler is
involved.
