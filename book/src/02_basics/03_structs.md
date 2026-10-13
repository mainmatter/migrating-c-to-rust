# Structs in FFI

[Memory layout](../01_foundations/04_layout.md) showed that a Rust struct has no
specified layout unless you ask for one, and that `repr(C)` asks for the one a C
compiler would pick. This section covers a few more things to know about structs
in FFI.

## What can go in the fields

A struct can only cross the FFI boundary if all of its fields can. These types
are safe to use as fields:

- **Primitives and raw pointers**, mapped as in
  [Primitive types in FFI](02_primitives.md).
- **Other `repr(C)` structs**, nested by value, as well as enums and unions with
  a C-compatible `repr`, which the next section covers.
- **Fixed-size arrays.** A C field `char name[16]` is `[c_char; 16]` in Rust,
  stored inline in the struct, exactly as in C.
- **Nullable pointers as an `Option`.** `Option<NonNull<T>>`, `Option<&T>`, and
  `Option<extern "C" fn(...)>` are guaranteed to be one pointer wide, with
  `None` as null.

Types whose layout Rust keeps to itself, such as `String`, `Vec<T>`,
`Box<dyn Trait>`, tuples, and slices, can't go in a shared struct. The
`improper_ctypes` lint warns when a struct used in an `extern` block lacks
`repr(C)` or has a field that can't cross.

## By value or by pointer

A C function can take or return a struct by value, or work through a pointer to
one:

```c
struct Point {
    double x;
    double y;
};

struct Point midpoint(struct Point a, struct Point b);
void translate(struct Point *point, double dx, double dy);
```

```rust,no_run
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

unsafe extern "C" {
    fn midpoint(a: Point, b: Point) -> Point;
    fn translate(point: *mut Point, dx: f64, dy: f64);
}
```

By value, the callee gets its own copy, and nothing is shared once the call
returns. That suits small, plain data like a `Point`. Pass a pointer instead
when the struct is large, when the callee should modify the caller's copy, or
when C holds on to the struct after the call returns.

## Wrapping a single value

`#[repr(transparent)]` is for structs with exactly one field. It guarantees that
the struct has the same layout _and_ the same function-call ABI as that field.
That makes it useful for type-safe wrappers around values C knows as plain
integers or pointers:

```rust,no_run
use std::ffi::c_int;

/// A file descriptor. C sees a plain `int`.
#[repr(transparent)]
pub struct Fd(c_int);
```

A one-field `repr(C)` struct has the same size and alignment as its field too,
but some ABIs pass a struct differently from the bare value inside it.
`repr(transparent)` guarantees they're passed the same way.

## Head to the exercise

The exercise gives you a C header, and you write the matching Rust types from
scratch. `Point` is a plain struct of two `double`s. `Shape` holds an array, a
nested `Point`, and a pointer that may be null. `Fd` wraps an `int`. The tests
compare the size, alignment, and field offsets of each type against what the C
compiler reports for the same header.
