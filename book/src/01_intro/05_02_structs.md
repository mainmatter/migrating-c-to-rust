# Structs in FFI

When C and Rust share a struct, they share raw bytes. C keeps the definition
order of struct fields. Rust is allowed to reorder fields to save space.[^1] If
Rust puts a field somewhere else, C reads the wrong bytes. Getting a struct
across the boundary means ensuring the layout is identical.

## Matching C's layout

`#[repr(C)]` tells Rust to lay out a struct the way a C compiler would,
following these rules:

1. Place the fields in declaration order.
2. Put each field at the next offset that is a multiple of its alignment, adding
   padding if necessary.
3. Give the struct the alignment of its most-aligned field.
4. Round the size up to a multiple of that alignment.

So to share a struct with C, write a `repr(C)` struct with the same fields, in
the same order, with types of the same size:

```c
struct Record {
    uint8_t kind;
    uint64_t id;
    uint16_t flags;
};
```

```rust,no_run
#[repr(C)]
pub struct Record {
    pub kind: u8,
    pub id: u64,
    pub flags: u16,
}
```

```text
offset  0      1                      8                      16       18           24
        ├ kind ┼───────── pad ────────┼────────── id ────────┼─ flags ┼──── pad ────┤
```

Of those 24 bytes, more than half are padding. With `repr(C)`, the declaration
order is the layout, so when you control both sides, putting the largest fields
first keeps the padding down.

## What can go in the fields

`repr(C)` fixes where each field goes, but not what goes in it. A struct can
only cross the boundary if all of its fields can, and these types are safe to
use as fields:

- **Primitives and raw pointers**, mapped as in
  [Primitive types in FFI](05_01_primitives.md).
- **Other `repr(C)` structs**, nested by value, as well as enums and unions with
  a C-compatible `repr`, which the next section covers.
- **Fixed-size arrays.** A C field `char name[16]` is `[c_char; 16]` in Rust,
  stored inline in the struct, exactly as in C.
- **Nullable pointers as an `Option`.** `Option<NonNull<T>>`, `Option<&T>`, and
  `Option<extern "C" fn(...)>` are guaranteed to be one pointer wide, with
  `None` as null. They describe a C pointer field that may be null more
  precisely than a raw pointer does.

Types whose layout Rust keeps to itself, such as `String`, `Vec<T>`,
`Box<dyn Trait>`, tuples, and slices, can't go in a shared struct. The
`improper_ctypes` lint helps here. When a struct appears in an `extern` block,
it warns if the struct lacks `repr(C)` or has a field that can't cross.

C code often zeroes a struct and then sets only the fields it cares about, as in
`struct options opts = {0};`. The Rust equivalent is `unsafe { mem::zeroed() }`,
which is sound only when all zeros is a valid value for every field. That holds
for integers, raw pointers, and an `Option` around a pointer, but not for a
`NonNull`, a `bool`, or a reference.

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
returns. Whether that copy travels in registers or on the stack is up to the
target's ABI. That suits small, plain data like a `Point`.

Pass a pointer instead when the struct is large, when the callee should modify
the caller's copy, as `translate` does, or when C holds on to the struct after
the call returns. A pointer also makes ownership a question again: who allocated
the struct, who frees it, and how long it stays valid.

## Wrapping a single value

`#[repr(transparent)]` is for structs with exactly one non-zero-sized field. It
guarantees that the struct has the same layout _and_ the same function-call ABI
as that field. That makes it the tool for type-safe wrappers around values C
knows as plain integers or pointers:

```rust,no_run
use std::ffi::c_int;

/// A file descriptor. C sees a plain `int`.
#[repr(transparent)]
pub struct Fd(c_int);
```

A one-field `repr(C)` struct has the same size and alignment as its field too,
but some ABIs pass a struct differently from the bare value inside it.
`repr(transparent)` guarantees they're passed the same way, so declaring
`fn close(fd: Fd) -> c_int;` in an `unsafe extern "C"` block matches C's
`int close(int fd)` on every target.

## Packed and over-aligned structs

`#[repr(packed)]` removes all padding, which means fields may end up misaligned.
It matches `#pragma pack(1)` and `__attribute__((packed))` in C, and you'll
mostly meet it in network protocols and file formats:

```rust,no_run
#[repr(C, packed)]
pub struct Header {
    pub kind: u8,
    pub len: u32, // at offset 1, misaligned
}

const _: () = assert!(size_of::<Header>() == 5 && align_of::<Header>() == 1);
```

Because a reference must always be aligned, Rust won't let you borrow a
misaligned field of a packed struct:

```rust,compile_fail
# #[repr(C, packed)]
# pub struct Header {
#     pub kind: u8,
#     pub len: u32,
# }
fn len(header: &Header) -> &u32 {
    // error: reference to field of packed struct is unaligned
    &header.len
}
```

Copy the field out instead, with `let len = header.len;`. The compiler knows the
field may be misaligned and generates an unaligned load for it.

`#[repr(align(N))]` raises a type's alignment to `N`, which must be a power of
two. It matches `__attribute__((aligned(N)))` in GCC and Clang. You need it when
hardware or a C API asks for a stricter alignment than the type itself has, such
as a buffer a device reads from:

```rust,no_run
#[repr(C, align(64))]
pub struct DmaBuffer {
    pub bytes: [u8; 64],
}

const _: () = assert!(align_of::<DmaBuffer>() == 64 && size_of::<DmaBuffer>() == 64);
```

You can't combine `packed` and `align` on the same type.

## C features Rust has no equivalent for

A few C struct declarations can't be mirrored in Rust at all:

- **Bitfields**, fields with a width in bits, as in `unsigned version : 4;`. C
  leaves their exact layout up to each compiler.
- **Flexible array members**, a trailing array without a length, as in
  `char data[];`. The struct's real size is only known at runtime, while a Rust
  struct's size is fixed at compile time.
- **Anonymous structs and unions**, nested without a name, whose fields C
  reaches as if they belonged to the outer struct. Rust requires every type to
  have a name.

`bindgen` can handle each of these, so when a header uses them, generate the
bindings rather than writing them by hand.

## Head to the exercise

The exercise gives you a C header, and you write the matching Rust types from
scratch. `Point` is a plain struct of two `double`s. `Shape` holds an array, a
nested `Point`, and a pointer that may be null. `WireHeader` is packed, and `Fd`
wraps an `int`. The tests compare the size, alignment, and field offsets of each
type against what the C compiler reports for the same header.

[^1]: You can watch the compiler do this in
    [Compiler Explorer](https://godbolt.org/z/bxh5eqM8z), which prints the
    layout rustc picked for each struct in the rightmost "Compiler Output" pane.
    The example there is a smaller one, where the compiler reorders the fields
    to get rid of an interior padding byte.
