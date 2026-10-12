# `unsafe` and safety contracts

Safe Rust can't cause [undefined behavior](#undefined-behavior), but some things
are beyond what the compiler can check, such as whether a pointer handed over by
a C library still points to live memory. For those, Rust has the `unsafe`
keyword. It marks the places where you, instead of the compiler, are responsible
for keeping the program sound.

## What `unsafe` unlocks

`unsafe` doesn't switch off the borrow checker or any other check. It lets you
perform a small set of extra operations the compiler can't verify:

- dereferencing a raw pointer
- calling an `unsafe` function, which includes most functions implemented in C
- reading or writing a `static mut` or an `extern static`
- reading a field of a `union`
- implementing an `unsafe` trait such as `Send` or `Sync`

Everything else inside an `unsafe` block is checked exactly as it would be
outside of one.

## Undefined behavior

The rules these operations can break all lead to _undefined behavior_ (UB). If a
program has UB, Rust no longer defines what it does: it might crash, give wrong
results, or seem to work fine. The compiler assumes UB never happens and
optimizes based on that, so a program that seems fine today can break with the
next compiler version.

The Rust Reference keeps the
[full list](https://doc.rust-lang.org/reference/behavior-considered-undefined.html)
of what counts as UB. The entries you'll meet most often in FFI code are:

- dereferencing a null, dangling, or misaligned pointer
- producing an invalid value, such as a `bool` holding `3` or an enum with a
  discriminant that doesn't match any variant
- breaking the aliasing rules, for example mutating memory while a shared
  reference to it exists
- reading uninitialized memory
- data races

## Safety contracts

An `unsafe` operation always comes with conditions that make it sound. We call
those conditions the operation's _safety contract_. Rust splits the contract
into two halves.

An `unsafe fn` declaration says "calling me is only sound if you, the caller,
uphold these conditions". The conditions should go into a numbered `# Safety`
section of the doc comment, so that we can refer back to them at the relevant
call sites:

```rust,no_run
/// Reads the `u32` that `ptr` points to.
///
/// # Safety
///
/// 1. `ptr` must be non-null and aligned for `u32`.
/// 2. `ptr` must point to an initialized `u32` that stays alive and remains
///    unmodified for the duration of the call.
pub unsafe fn read_u32(ptr: *const u32) -> u32 {
    // SAFETY: ...
    unsafe { *ptr }
}
```

An `unsafe` block says "I checked the conditions here". The justification goes
into a `// SAFETY:` comment right above it:

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

A `// SAFETY:` comment explains why the code upholds each condition, or names
the numbered condition its caller already promised to uphold.

## Safe abstractions and soundness

It's common for `unsafe` code to end up wrapped in a safe function. The safe
function checks whatever the contract requires, then performs the `unsafe`
operation. That wrapper is only correct if no input, however strange, can cause
UB. A safe function that can cause UB is called _unsound_, and it's a bug even
if no caller triggers it today.

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

In real code you would write `bytes.first().copied()`, which does the same check
without any `unsafe`. The example uses `get_unchecked` only to show the pattern:
check, then justify, then perform the operation. Writing the `// SAFETY:`
comment is often how you find out that a check is missing. If you can't finish
the sentence, the code isn't sound yet.

## Head to the exercise

The exercise is one function: return the last two bytes of a slice, or `None` if
it has fewer than two, reading each byte with `get_unchecked`. Work out the
condition that makes both indexes valid before you rely on it, and say why in a
`// SAFETY:` comment.
