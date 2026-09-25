# References vs raw pointers

C has one kind of pointer. Whether a `T *` may be null, who owns the memory
behind it, and how long it stays valid are all conventions, written down in
documentation if you're lucky. Rust has two kinds: references, which carry those
guarantees in the type system, and raw pointers, which carry none of them.
Everything that crosses the FFI boundary arrives as a raw pointer, so you need
to know exactly what you give up and what you take on when you convert between
the two.

## What a reference guarantees

A reference, `&T` or `&mut T`, is always:

- **non-null**,
- **aligned** for `T`,
- pointing to a **live, initialized, valid** `T`,
- valid for its **lifetime**, which the borrow checker enforces,
- subject to the **aliasing rules**: either any number of `&T`, or exactly one
  `&mut T`. While a `&T` exists, the value behind it doesn't change.

These guarantees aren't only there to protect you. The compiler optimizes based
on them. Because a `&mut T` is the only way to reach its value, the compiler can
keep that value in a register instead of re-reading it from memory. Because a
`&T` is never null, `Option<&T>` can use null to represent `None`.

That's also why creating a reference that breaks one of these guarantees is
undefined behavior, even if you never read through it.

## Raw pointers

Raw pointers come in two flavors: `*const T` and `*mut T`. They are Rust's
equivalent of C's `const T *` and `T *`, and they guarantee nothing. A raw
pointer may be null, dangling, misaligned, or pointing to memory that is being
mutated through another pointer at the same time.

Creating one is safe. Dereferencing one is `unsafe`, because that's the moment
all those possibilities matter:

```rust,no_run
let mut x = 10_i32;

// Creating raw pointers is safe.
let p: *mut i32 = &mut x;
let null: *const i32 = std::ptr::null();
assert!(!p.is_null());
assert!(null.is_null());

// SAFETY: `p` comes from `&mut x`, so it's non-null, aligned, and points to a
// live `i32`. Nothing else accesses `x` while we use `p`.
unsafe {
    *p += 1;
    assert_eq!(p.read(), 11);
}
```

Raw pointers have a small API of their own. The methods you'll use most are:

- `is_null` to check for null,
- `cast` to change the pointee type, the equivalent of a C pointer cast,
- `add` for pointer arithmetic, which is `unsafe` because the result must stay
  within the same allocation,
- `read` and `write` to copy a value out or in without creating a reference.

### Mutability is about where a pointer came from

In C, casting away `const` is legal as long as the memory itself isn't `const`.
Rust is stricter. Whether you may write through a pointer depends on how it was
created, not only on its type. A `*mut T` that you got by casting a `&T` must
never be written through, because a `&T` guarantees that the value doesn't
change.

The compiler catches the most obvious version of this:

```rust,compile_fail
let x = 10_i32;
let p = &x as *const i32 as *mut i32;
// error: assigning to `&T` is undefined behavior, consider using an `UnsafeCell`
unsafe { *p = 20 };
```

It doesn't catch the less obvious ones. This example compiles, runs, and prints
`20`, but it is undefined behavior all the same:

```rust,no_run
let mut x = 10_i32;
let p = &mut x as *mut i32;
let r = &x;
// SAFETY: none! `r` is a shared reference, so `x` must not change while it
// is alive.
unsafe { *p = 20 };
println!("{r}");
```

The borrow checker only tracks references, so it has no idea `p` and `r` point
to the same value. Miri, a dynamic checker for unsafe Rust, does catch it and
reports undefined behavior.

## From raw pointer to reference

Sooner or later you'll want to turn a raw pointer from C into a reference, so
the rest of your Rust code can work with it safely. There are two ways to do
that:

- `&*ptr` or `&mut *ptr` dereference the pointer and borrow the result,
- `ptr.as_ref()` and `ptr.as_mut()` do the same, but return `None` for a null
  pointer.

Both are `unsafe`, and both are the point where you take on everything in the
list at the top of this section. One more condition comes with them: the
lifetime. A reference created from a raw pointer can have any lifetime the
caller asks for, because the compiler has nothing to derive it from. It's up to
you to tie it to something meaningful:

```rust,no_run
pub struct Config {
    pub verbose: bool,
}

/// Borrows the configuration a C library handed us.
///
/// # Safety
///
/// `ptr` must be null, or point to a valid `Config` that stays alive and
/// unmodified for as long as the returned reference is in use.
pub unsafe fn config_from_c<'a>(ptr: *const Config) -> Option<&'a Config> {
    // SAFETY: the caller guarantees that `ptr` is null or valid for `'a`.
    unsafe { ptr.as_ref() }
}
```

The `# Safety` section spells out what `'a` means in practice. Without it, a
caller could keep the reference around long after the C library freed the
`Config`, and nothing would stop them.

## Mapping C pointers to Rust

| C                  | Rust                                        |
| ------------------ | ------------------------------------------- |
| `const T *`        | `*const T`                                  |
| `T *`              | `*mut T`                                    |
| `T **` (out param) | `*mut *mut T`                               |
| `void *`           | `*mut c_void` (or `*const c_void`)          |
| `NULL`             | `std::ptr::null()` / `std::ptr::null_mut()` |

Keep in mind that C's `const` is a convention. A C function that takes a
`const T *` can still cast it away and write through it. If you pass it a
pointer derived from a `&T`, you are trusting that it doesn't.[^1]

## Head to the exercise

The exercise is one function: swap the two `i32`s that a pair of raw pointers
point to.

[^1]: Raw pointers carry more than an address. Each pointer also has a
    _provenance_: the allocation and permissions it was derived from. Two
    pointers with the same address can differ in what they're allowed to access.
    You rarely need to think about this in day-to-day FFI code, but it's the
    reason integer-to-pointer casts are subtle. The
    [`std::ptr` documentation](https://doc.rust-lang.org/std/ptr/index.html#provenance)
    covers the details.
