# FFI-safe types

We have passed data from C to Rust and from Rust to C in the previous exercises.
In doing so we have mostly constrained ourselves to pointers or `c_`-prefixed
primitives. At this point you may wonder: "What if I want to pass a `struct`, or
a `String`?"

## Primitives

All primitives (`i8`..`i64`, `u8`..`u64`, `f32`, `f64`, `bool`, and pointers)
are always safe to share across an FFI boundary, since they correspond 1:1 with
C types. `Option<NonNull<T>>` deserves special mention because it is _also_
FFI-safe and guaranteed to have the same representation as `*mut T`.

Note that C's `int` type and Rust's `i32`, for example, _almost always_ mean the
same thing. But because the C standard's definition of these types is loose,
there exist architectures for which this is not true. This is rare enough that
the Rust team decided Rust's integer types are FFI-safe. If you want to be sure,
though, you can use the `std::ffi::c_*` types such as `c_int` or `c_longlong`.

## `repr(C)`

Much like we need to use the C calling convention to make our Rust functions
interoperable, we need to use the `C` struct layout to make our structs
interoperable. This is because (just like functions) Rust reserves the right to
change the struct layout at any time (it is "undefined")[^1]. To have a stable
layout that other languages can understand, we need to mark our structs with the
`repr(C)` attribute.

```rust,no_run
// we know the layout of this struct is always (16 bytes in total, because the
// `usize` gives the whole struct an alignment of 8):
// - `a` - 8 bytes
// - `b` - 1 byte
// - 1 byte padding, so that `c` is 2-byte aligned
// - `c` - 2 bytes
// - 4 bytes trailing padding, so that the size is a multiple of the alignment
#[repr(C)]
struct Foo {
    a: usize,
    b: u8,
    c: u16,
}

// we DO NOT know the layout of this struct!
struct Bar {
    a: usize,
    b: u8,
    c: u16,
}
```

`repr(C)` also applies to _tuple structs_ where the layout is exactly the same
except the fields don't have names.

The `repr(..)` attribute can also be used on `enum`s:

```rust,no_run
// This corresponds to named u8 constants, where A = 0, B = 1, C = 2
#[repr(u8)]
enum Foo {
    A,
    B,
    C,
}

# mod explicit_tags {
// You can of course also assign explicit tags
#[repr(u8)]
enum Foo {
    A = 5,
    B = 2,
    C = 8,
}
# }
# mod c_abi {
// repr(C) also works and uses the "default enum size and sign for the target platform's C ABI"
#[repr(C)]
enum Foo {
    A,
    B,
    C,
}
# }
```

You can use enums with fields even though they don't have an inherent C
equivalent. Rust defines a stable mapping
[here](https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md).

```rust,no_run
// A definition like this...
#[repr(u8)]
enum TwoCases {
    A(u8, u16),
    B(u16),
}

//...is in essence just syntax sugar for this:
union TwoCasesRepr {
    A: TwoCasesVariantA,
    B: TwoCasesVariantB,
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum TwoCasesTag {
    A,
    B,
}

#[repr(C)]
#[derive(Clone, Copy)] // union fields have to be Copy
struct TwoCasesVariantA(TwoCasesTag, u8, u16);

#[repr(C)]
#[derive(Clone, Copy)]
struct TwoCasesVariantB(TwoCasesTag, u16);
```

So, essentially, an enum with fields decomposes into a tag enum and a union of
its fields.

### `repr(transparent)`

`repr(transparent)` doesn't appear as often but warrants special mention. It is
an attribute that can only be used on types with a single sized field. It
guarantees that the layout of the outer type will be exactly the same as that of
the inner type.

```rust,no_run
// Foo is guaranteed to have the same representation as `*const u8`!
#[repr(transparent)]
struct Foo(*const u8);

# mod generic {
# use std::marker::PhantomData;
// because this is concerned with _sized_ fields (fields that have a size)
// fields that have no size such as PhantomData can still be used!
#[repr(transparent)]
struct Foo<T>(*const u8, PhantomData<T>);
# }
```

`repr(transparent)` comes in handy if you need to cast pointers or `transmute`
between types.

## Types that cannot cross the FFI boundary

The list is long, but as a rule of thumb, two kinds of type cannot be shared:
anything generic, and any Rust type that is not explicitly FFI-safe, such as
`String`.

_Generics_ are not FFI-safe because the compiler will monomorphize a concrete
version of the struct for each type passed into the generic. If we pass the type
across the FFI boundary, the C compiler, which does not know about
monomorphization, cannot know which version to pick. There exists no ABI that
represents generics.

## Head to the exercise

There you will find an FFI function that attempts to pass types that are not
FFI-safe. Notice the compiler-generated warnings: it is your job to fix them by
using FFI-safe types.

[^1]: The compiler does this to be smart and optimize things. For example,
    [here](https://godbolt.org/z/bxh5eqM8z) is the code snippet you saw above in
    Compiler Explorer again. If you look at the rightmost "Compiler Output" pane
    you will see the actual layout of each struct. You can see that for `Bar`
    the compiler reordered the fields to remove the interior padding byte.
