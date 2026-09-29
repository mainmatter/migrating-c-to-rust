# Types in FFI

We have passed data from C to Rust and from Rust to C in the previous exercises.
In doing so we have mostly constrained ourselves to pointers or `c_`-prefixed
primitives. At this point you may wonder: "What if I want to pass a `struct`, or
a `String`?"

The answer depends on the type. Scalars pair up almost one to one. Aggregates
need a `repr` attribute, so that both sides agree on where each field lives.
Some types can't cross at all, and for those a pointer crosses instead while the
type itself stays private. The sections that follow work through those cases,
one shape at a time, starting here with the scalars.

## Primitive types

A C header describes its parameters in C's own types: `int`, `long`, `char`,
`size_t`. Rust has `i32`, `i64`, `u8`, `usize`. Writing a binding means pairing
them up, and most of those pairs hold everywhere you're likely to build.

```c
int scale(int value, int factor);
```

```rust,no_run
unsafe extern "C" {
    fn scale(value: i32, factor: i32) -> i32;
}
```

That binding is correct on Windows, macOS, and Linux, on x86-64 and aarch64,
because C's `int` is 32 bits on all of them. Most of the mapping works out that
way, and those targets are what the table below assumes.

## The mapping

| C                        | Rust                           | Alias         |
| ------------------------ | ------------------------------ | ------------- |
| `char`                   | `i8`, or `u8` on aarch64 Linux | `c_char`      |
| `signed char`            | `i8`                           | `c_schar`     |
| `unsigned char`          | `u8`                           | `c_uchar`     |
| `short`                  | `i16`                          | `c_short`     |
| `unsigned short`         | `u16`                          | `c_ushort`    |
| `int`                    | `i32`                          | `c_int`       |
| `unsigned int`           | `u32`                          | `c_uint`      |
| `long`                   | `i64`, or `i32` on Windows     | `c_long`      |
| `unsigned long`          | `u64`, or `u32` on Windows     | `c_ulong`     |
| `long long`              | `i64`                          | `c_longlong`  |
| `unsigned long long`     | `u64`                          | `c_ulonglong` |
| `float`                  | `f32`                          | `c_float`     |
| `double`                 | `f64`                          | `c_double`    |
| `size_t`                 | `usize`                        | –             |
| `ptrdiff_t`              | `isize`                        | –             |
| `int32_t`, `uint64_t`, … | `i32`, `u64`, …                | –             |
| `void *`                 | `*mut c_void`                  | –             |
| `T *`                    | `*mut T`                       | –             |
| `const T *`              | `*const T`                     | –             |

The names in the last column are type aliases from `core::ffi`. `std::ffi`
re-exports them, so `std::ffi::c_int` and `core::ffi::c_int` are the same type,
and `core::ffi` is the one to reach for in a `no_std` crate. Two more spellings
show up in real code: `bindgen` writes `std::os::raw::c_int`, and the `libc`
crate offers `libc::c_int`. Both resolve to that same alias, so you can mix them
freely.

The aliases are not a more verbose way to write `i32`. Each one is defined per
target, and the standard library picks the definition that matches the C
compiler you're building against. On MSP430 or AVR, where C's `int` is 16 bits,
`c_int` is `i16`, and a binding written with the alias keeps describing the
header correctly without you touching it. Writing `i32` instead pins that
binding to the targets where the two happen to agree.

Which is often fine, and common in practice. The `scale` binding above writes
`i32`, and plenty of production code does the same, because the targets it will
ever be built for all agree that `int` is 32 bits. Do that deliberately rather
than by accident: know which targets you're claiming, and write the alias when
you can't answer that, as in a library other people will build.

That gives a rule of thumb for reading a header:

- **The width is up to the platform** (`int`, `long`, `unsigned short`): write
  the alias, unless you know every target you build for agrees on the width, in
  which case Rust's fixed-width type is a fair shortcut.
- **`char`:** write `c_char`, and take the shortcut here least of all. The next
  section is about why.
- **The width is already pinned** (`int32_t`, `uint64_t`): write Rust's
  fixed-width type, since the C type can't move either.
- **`size_t` and `ptrdiff_t`:** write `usize` and `isize`. They're pointer-sized
  in Rust, which is what `size_t` and `ptrdiff_t` are on every target Rust
  supports. That's why the table has no alias for them: `c_size_t` exists, but
  it is still unstable.
- **`void *`:** write `*mut c_void`, another alias from `core::ffi`. C uses it
  for "a pointer to something I'm not telling you about", and
  [Opaque types](05_06_opaque_types.md) covers what to do with such a pointer.

## `c_char` and `u8`

A C `char` is a byte, and Rust's byte type is `u8`, so writing `*const u8` in a
binding where the header says `const char *` looks harmless. It's the most
common mistake in hand-written bindings, so it's worth going through slowly.

`c_char` is not always `u8`. C lets each platform decide whether a plain `char`
is signed or unsigned, and `c_char` follows that decision. It is `i8` on x86-64,
and on macOS and Windows for aarch64. It is `u8` on aarch64 Linux, on 32-bit
ARM, and on RISC-V.

The binding still works on your own machine, which is what makes this easy to
miss. A byte is passed the same way whether it was declared signed or unsigned,
so the call links, runs, and moves the right bytes. Then someone builds for a
target where `c_char` is the other one, and the code stops compiling. Anything
that hands you a `c_char` no longer lines up with the `u8` you wrote:
`CStr::as_ptr` gives a `*const c_char`, and passing it where a `*const u8` is
expected is a type mismatch. Every call site that touched those bytes has to be
fixed on that target, or guarded with `cfg`.

So write `c_char` in the signature. It says what the header says, on every
target.

That leaves one thing to sort out, because `c_char` is not the type Rust's byte
methods work on. `b'a'` literals, `u8::to_ascii_lowercase`, `str::as_bytes` and
`CStr::to_bytes` all speak `u8`. Cast to `u8` as soon as you have the value, do
the work there, and cast back on the way out:

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
  valid for both. On a target where `c_char` is already `u8`, the casts do
  nothing at all, and they keep the code compiling there too.

When the bytes arrive as a buffer rather than one value, the same reasoning
applies to the pointer, and [Slices in FFI](05_04_slices.md) picks that up.

## Head to the exercise

The exercise gives you a C library over bytes with its bindings already written,
and three small wrappers to finish. They're in `u8` and `bool`, the way the rest
of Rust wants them, so each one converts at the boundary: a byte in, a byte out,
C's `int` as a `bool`, and a `&mut [u8]` whose pointer C takes as a `char *`.

The interesting part is what counts as done. Before running the tests, `wr`
type-checks the crate for two targets, one where `c_char` is `i8` and one where
it is `u8`. Cast to whichever of those your own compiler asks for today, as the
code you inherit there does, and one of the two checks rejects it. Only `c_char`
gets past both. Neither check compiles or links anything, so no cross compiler
is involved.
