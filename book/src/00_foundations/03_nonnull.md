# `NonNull<T>` and niches

References and raw pointers sit at two ends of a scale: references guarantee
everything, raw pointers guarantee nothing. In FFI code you often want something
in between, a pointer you know isn't null even though you can't vouch for the
memory behind it. That's what `NonNull<T>` is for.

## `NonNull<T>`

`NonNull<T>` behaves like a `*mut T` with one extra guarantee: it is never null.
You create one from a raw pointer with `NonNull::new`, which returns `None` for
null, or from a reference, which is never null to begin with:

```rust,no_run
use std::ptr::NonNull;

let mut x = 5_u32;
let p: NonNull<u32> = NonNull::from(&mut x);

// SAFETY: `p` comes from `&mut x`, which is alive and not borrowed elsewhere.
unsafe { *p.as_ptr() += 1 };
assert_eq!(x, 6);

let from_c: *mut u32 = std::ptr::null_mut();
assert!(NonNull::new(from_c).is_none());
```

A few more methods are worth knowing:

- `NonNull::new_unchecked` skips the null check. It's `unsafe`, because passing
  a null pointer is undefined behavior.
- `as_ptr` turns it back into a `*mut T`, for example to hand it to C.
- `as_ref` and `as_mut` turn it into a reference. They're `unsafe` for the same
  reasons as with raw pointers: non-null doesn't mean aligned, alive, or valid.
- `NonNull::dangling` creates a non-null, aligned pointer that doesn't point to
  anything. The standard library uses it for empty collections.

`NonNull<T>` also differs from `*mut T` in its _variance_, the rule that decides
when the compiler accepts one generic type in place of another. `NonNull<T>` is
covariant in `T`, like `&T`: where a `NonNull<&'short Config>` is expected, a
`NonNull<&'long Config>` will do, because a reference that lives longer is
usable anywhere a shorter-lived one is. `*mut T` is invariant, so it only
accepts the exact type it was declared with. Invariance is the careful default
for a pointer you can write through: if the compiler let you shorten the
lifetime inside a `*mut T`, you could write a short-lived reference into a place
that is later read as a long-lived one, and that reference would outlive what it
points to. The difference rarely shows up in FFI signatures. It matters when you
build your own pointer types, which is why the standard library's `Box` and
`Vec` are built on `NonNull`.

## Niches

A type that can't be null frees up a bit pattern. A `NonNull<T>`, a `&T`, or a
`Box<T>` can never be all zeros, so the compiler can use all zeros to represent
`None` in an `Option` around them. Invalid bit patterns like these are called
_niches_, and they let `Option<&T>` be exactly as big as `&T`:

```rust,no_run
use std::mem::size_of;
use std::ptr::NonNull;

const _: () = {
    // Types with a niche: the `Option` is free.
    assert!(size_of::<Option<&u8>>() == size_of::<&u8>());
    assert!(size_of::<Option<Box<u8>>>() == size_of::<Box<u8>>());
    assert!(size_of::<Option<NonNull<u8>>>() == size_of::<NonNull<u8>>());
    assert!(size_of::<Option<extern "C" fn()>>() == size_of::<extern "C" fn()>());
};
```

A raw pointer is the counter-example. Null is an ordinary value for a
`*const T`, and so is every other bit pattern, so there's no niche left to mark
`None` with. The compiler has to store that separately, and pad the result back
out to the pointer's alignment. On a 64-bit target that doubles the size:

```rust,no_run
use std::mem::size_of;

const _: () = {
    assert!(size_of::<*const u8>() == 8);
    assert!(size_of::<Option<*const u8>>() == 16);
};
```

That size is a symptom rather than the rule: `Option<*const T>` has no specified
layout, so it wouldn't be a C pointer even if it happened to be eight bytes
wide. Which is the practical reason `NonNull` exists. It lets you say "may be
null" in Rust's type system and get a value that is one pointer wide, with a
layout you're allowed to rely on.

## What's guaranteed

Niches are an optimization, and in general the compiler is free to decide how to
lay out an `Option`. For a short list of types, though, the standard library
[guarantees](https://doc.rust-lang.org/std/option/index.html#representation)
that `Option<T>` has the same size, alignment, and function-call ABI as `T`,
with `None` represented as all zeros:

- `&T` and `&mut T`,
- `Box<T>`,
- `NonNull<T>`,
- `NonZero<u8>`, `NonZero<u32>`, and the other `NonZero` integers,
- function pointers, including `extern "C" fn`,
- `#[repr(transparent)]` structs around one of the above.

For the pointer types, the all-zeros part only holds when `T` is sized. A
pointer to an unsized type, such as `&[u8]` or `&str`, is a pointer plus a
length, and can't cross the FFI boundary as a single C pointer anyway.

That guarantee is what makes these types usable in FFI signatures. For anything
else, such as `Option<bool>` or `Option<u32>`, the layout is a compiler
decision, not a contract C can rely on.

## Head to the exercise

The exercise is one function: take a raw pointer the way C would hand it over,
read a field out of what it points to, and return `None` when there is nothing
to read. Let the type carry the null check rather than writing one by hand.
