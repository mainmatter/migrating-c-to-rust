# Structs in FFI

When C and Rust share a struct, they share raw bytes. C reads a field by adding
its offset to the struct's address, and it computes that offset from the struct
definition in the header. If Rust puts the field somewhere else, C reads the
wrong bytes, and neither compiler can tell you. Getting a struct across the
boundary means pinning down where each field goes, and checking that both sides
ended up with the same answer.

## `repr(C)`: the C rules

`#[repr(C)]` tells Rust to lay a struct out the way a C compiler would, which
takes four rules:

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
use std::mem::offset_of;

#[repr(C)]
pub struct Record {
    pub kind: u8,
    pub id: u64,
    pub flags: u16,
}

const _: () = {
    assert!(offset_of!(Record, kind) == 0); // 1 byte, then 7 bytes of padding
    assert!(offset_of!(Record, id) == 8); // 8 bytes
    assert!(offset_of!(Record, flags) == 16); // 2 bytes, then 6 bytes of padding
    assert!(align_of::<Record>() == 8); // from `id`
    assert!(size_of::<Record>() == 24); // 18 rounded up to a multiple of 8
};
```

```text
offset  0      1                      8                      16       18       24
        ├ kind ┼───────── pad ────────┼────────── id ────────┼─ flags ┼── pad ──┤
```

Eight of those 24 bytes are padding, and they're there because the order is
fixed. The same struct without `repr(C)` fits in 16, and ordering the fields
largest-first gets you to 16 while staying C-compatible.[^1]

Those `const` assertions are cheap insurance. They fail the build the moment
someone reorders a field or changes a type on one side only. `bindgen` emits the
same kind of checks for every struct it generates.

## `repr(transparent)`

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

A one-field `#[repr(C)]` struct has the same size and alignment as its field
too, so it's worth being clear about what `transparent` adds on top.

The first thing is the function-call ABI. As far as a target's ABI is concerned,
a `repr(C)` struct is a struct, and several ABIs pass or return a struct
differently from the bare value inside it, for example on the stack instead of
in a register. `repr(transparent)` guarantees that the wrapper is passed and
returned exactly like the field, so `extern "C" fn close(fd: Fd) -> c_int`
really does match C's `int close(int fd)` everywhere.

The second is that the field's representation carries over to types built on the
wrapper. A `#[repr(transparent)] struct Handle(NonNull<T>)` keeps the field's
niche, so `Option<Handle>` is still one pointer wide, which is guaranteed for a
`repr(transparent)` wrapper. Transmuting between the wrapper and the field stays
sound as well.

In practice the compiler usually treats a one-field `repr(C)` struct the same
way. `repr(transparent)` is what turns "usually" into a guarantee that holds
across targets and compiler versions.

## `repr(packed)`

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

Because a reference must always be aligned, Rust won't let you borrow a field of
a packed struct:

```rust,compile_fail
# #[repr(C, packed)]
# pub struct Header {
#     pub kind: u8,
#     pub len: u32,
# }
fn len(header: &Header) -> &u32 {
    // error[E0793]: reference to field of packed struct is unaligned
    &header.len
}
```

Copy the field out instead, with `let len = header.len;`. The compiler knows the
field may be misaligned and generates a safe load for it.

## `repr(align(N))`

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

You can't combine `packed` and `align` on the same type. The compiler rejects it
with error E0587.

## Bitfields

C can give a field a width in bits:

```c
struct Packet {
    unsigned version : 4;
    unsigned kind    : 4;
    unsigned length  : 24;
};
```

Rust has no equivalent, and no `repr` adds one. C also leaves much of this to
the implementation: which end of the storage unit the first field starts at,
whether a field may straddle a unit boundary, and whether a plain `int` bitfield
is signed or unsigned. Two compilers can lay the same declaration out
differently.

`bindgen` deals with it by generating one opaque storage field plus getters and
setters that do the shifting and masking for you. The struct works, but it is no
longer something you read field by field.

When you own both sides, keep the bit twiddling in one place: expose accessor
functions from C, or mirror the struct as plain integers and write the shifts
yourself.

## Zero-sized types

Rust has types that take up no space at all, such as `()` and a struct without
fields. C doesn't. An empty struct isn't valid standard C, and C compilers that
accept one as an extension disagree on its size. Keep zero-sized types off the
FFI boundary. The compiler warns when one ends up in an `extern` signature.

## Head to the exercise

The exercise lists three C declarations in comments, with the matching Rust
fields already written out. What's missing is the `repr` attribute on each type:
a struct that follows C's layout rules, a packed one, and a wrapper around an
`int`. The tests check size, alignment, and field offsets against what a C
compiler produces.

[^1]: You can watch the compiler do this in
    [Compiler Explorer](https://godbolt.org/z/bxh5eqM8z), which prints the
    layout rustc picked for each struct in the rightmost "Compiler Output" pane.
    The example there is a smaller one, where the compiler reorders the fields
    to get rid of an interior padding byte.
