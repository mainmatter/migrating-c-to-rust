# `unsafe` and safety contracts

Safe Rust comes with a guarantee: however you combine safe code, it can't cause
[undefined behavior](#undefined-behavior). No use-after-free, no read past the
end of a buffer, no two threads writing the same value at once. The compiler
enforces that by checking every borrow, every index, and every conversion it can
reason about. Some things are beyond what it can check, though. It can't tell
whether a pointer handed over by a C library still points to live memory, or
whether that library expects a buffer of 16 bytes or 16 kilobytes.

For those operations Rust has the `unsafe` keyword. It marks the places where
you, instead of the compiler, are responsible for keeping the program sound.
Mixed C-Rust codebases use it a lot, so it's worth being precise about what it
does.

## What `unsafe` unlocks

`unsafe` doesn't switch off the borrow checker or any other check. It lets you
perform a small set of extra operations the compiler can't verify:

- dereferencing a raw pointer,
- calling an `unsafe` function, which includes every function implemented in C,
- reading or writing a `static mut` or an extern static,
- reading a field of a `union`,
- implementing an `unsafe` trait such as `Send` or `Sync`.

Everything else inside an `unsafe` block is checked exactly as it would be
outside of one.

## Undefined behavior

The rules these operations can break all lead to _undefined behavior_ (UB). Most
of those rules protect two properties. Memory safety means a program only ever
accesses memory it owns, that is still alive, and that holds a value of the type
it is read as. Freedom from data races means no two threads touch the same
memory at the same time with at least one of them writing, unless they
synchronize.

Break either one and the program has UB, and UB is worse than a crash. It
doesn't mean "the program crashes" or "you get a wrong value". It means the
program no longer has any defined meaning. The compiler optimizes on the
assumption that UB never happens, so a single violation can change code far away
from where it occurred: a null check gets removed, a loop runs one time too
many, or a value that "can't change" changes. A crash would be the good case,
because at least it points at something.

The Rust Reference keeps the
[full list](https://doc.rust-lang.org/reference/behavior-considered-undefined.html).
The entries you'll meet most often in FFI code are:

- dereferencing a null, dangling, or misaligned pointer,
- producing an invalid value, such as a `bool` holding `3` or an enum with a
  discriminant that doesn't match any variant,
- breaking the aliasing rules, for example mutating memory while a shared
  reference to it exists,
- reading uninitialized memory,
- data races.

## Safety contracts

An `unsafe` operation always comes with conditions that make it sound. We call
those conditions the operation's _safety contract_. Rust splits the contract
into two halves.

An `unsafe fn` says "calling me is only sound if you, the caller, uphold these
conditions". The conditions go into a numbered `# Safety` section of the doc
comment so that the code relying on them can point at one:

```rust,no_run
/// Reads the `u32` that `ptr` points to.
///
/// # Safety
///
/// 1. `ptr` must be non-null and aligned for `u32`.
/// 2. `ptr` must point to an initialized `u32` that stays alive and unmodified
///    for the duration of the call.
pub unsafe fn read_u32(ptr: *const u32) -> u32 {
    // SAFETY: ...
    unsafe { *ptr }
}
```

An `unsafe` block says "I checked the conditions here". The justification goes
into a `// SAFETY:` comment right above it:[^1]

```rust,no_run
# /// # Safety
# /// 1. `ptr` must be non-null and aligned for `u32`.
# /// 2. `ptr` must point to an initialized `u32`.
# unsafe fn read_u32(ptr: *const u32) -> u32 { unsafe { *ptr } }
let x = 41_u32;
// SAFETY: `&x` is non-null and aligned (1.), and `x` is an initialized `u32`
// that outlives the call (2.).
let y = unsafe { read_u32(&x) };
```

Each `// SAFETY:` comment then explains why the code upholds each condition, or
names the numbered invariant its caller already promised.
[High-quality FFI APIs](../01_intro/07_high_quality_ffi_apis.md) returns to this
convention once a real `extern "C"` function has several invariants to keep
apart.

## Safe abstractions and soundness

Most `unsafe` code ends up wrapped in a safe function. The safe function checks
whatever the contract requires, then performs the `unsafe` operation. That
wrapper is only correct if no input, however strange, can cause UB. A safe
function that can cause UB is called _unsound_, and it's a bug even if no caller
triggers it today.

Here's an unsound one:

```rust,no_run
/// Returns the first byte of `bytes`.
pub fn first(bytes: &[u8]) -> u8 {
    // SAFETY: ??? An empty slice has no index 0.
    unsafe { *bytes.get_unchecked(0) }
}
```

Calling `first(&[])` reads out of bounds, and nothing in the signature warns the
caller about it. The sound version checks the condition before relying on it:

```rust,no_run
/// Returns the first byte of `bytes`, or `None` if `bytes` is empty.
pub fn first(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }
    // SAFETY: `bytes` is not empty, so index 0 is in bounds.
    Some(unsafe { *bytes.get_unchecked(0) })
}
```

In real code you would write `bytes.first().copied()`. The pattern is what
matters: check, then justify, then perform the operation. Writing the
`// SAFETY:` comment is often how you find out that a check is missing. If you
can't finish the sentence, the code isn't sound yet.

## `unsafe` in FFI code

`unsafe` shows up in three places when you work across a language boundary.

```rust,no_run
use std::ffi::{c_char, c_double, c_int};

unsafe extern "C" {
    // Rust can't check what `strlen` does with the pointer, so calling it is
    // `unsafe`.
    fn strlen(s: *const c_char) -> usize;

    // `sqrt` accepts every `double`: a negative one gives NaN rather than
    // undefined behavior. With no condition to uphold, it can be declared
    // `safe` and called from safe code.
    safe fn sqrt(x: c_double) -> c_double;
}

#[unsafe(no_mangle)]
pub extern "C" fn root_of_two() -> c_double {
    sqrt(2.0)
}
```

- **`unsafe extern` blocks.** Declaring a foreign function is itself a promise
  that the signature is correct, which is why the block is marked `unsafe`.
  Every function in it is `unsafe` to call, unless you mark it `safe fn` because
  it has no conditions for the caller to uphold. These declarations are what
  [What is FFI?](../01_intro/01_what_is_ffi.md) calls bindings, and it covers
  what the compiler does and doesn't check about them.
- **`unsafe` attributes.** `#[unsafe(no_mangle)]` exports a function under its
  plain name. That name can collide with another symbol, such as a C library
  function, and the linker won't always tell you. The attribute is `unsafe`
  because you are promising that it doesn't.
- **`unsafe` functions you export.** An `extern "C" fn` that dereferences
  pointers from C has a safety contract like any other `unsafe fn`, and C
  callers need it documented just as much.

All three come up as soon as you write bindings of your own.

## Head to the exercise

The exercise is one function: return the last byte of a slice, or `None` if
there isn't one, reading the byte with `get_unchecked`.

[^1]: `read_u32` needs its own `unsafe` block even though the whole function is
    `unsafe`. Since the 2024 edition, the body of an `unsafe fn` is treated like
    any other function body, and the `unsafe_op_in_unsafe_fn` lint warns when
    you perform an `unsafe` operation without a block. That keeps every `unsafe`
    operation visible, with its own justification next to it.
