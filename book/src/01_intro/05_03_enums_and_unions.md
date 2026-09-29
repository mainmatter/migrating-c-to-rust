# Enums and unions in FFI

C's enums are named integers, and C's unions are several types sharing the same
bytes. Rust has both concepts too, but a Rust enum is far stricter than a C
enum, and that difference is one of the most common sources of undefined
behavior in FFI code. This section covers how Rust lays out enums and unions,
and the rule that keeps C's integers out of Rust's enums.

## Field-less enums

An enum whose variants carry no data is, in memory, nothing more than its
_discriminant_: an integer that says which variant it is. Variants are numbered
from 0, unless you assign a value explicitly, and each variant without an
explicit value gets the previous one plus 1. You can read the discriminant with
an `as` cast:

```rust,no_run
enum Status {
    Ok,           // 0
    NotFound = 4, // 4
    Busy,         // 5
}

assert_eq!(Status::Ok as i32, 0);
assert_eq!(Status::Busy as i32, 5);
```

Without a `repr` attribute, the compiler picks the integer type for the
discriminant. To share an enum with C, choose it yourself:

- `#[repr(u8)]`, `#[repr(i32)]`, and the other integer types fix the
  discriminant to exactly that type.
- `#[repr(C)]` uses whatever size a C compiler would pick for an equivalent
  `enum` on the same target, which is usually 4 bytes, like an `int`.

Match what the C side actually uses. A `typedef enum` in a header maps to
`#[repr(C)]`. Values passed around as plain `int` constants map to
`#[repr(i32)]`. Avoid `#[repr(u32)]` for those: C's `int` is signed, and the
difference surfaces the first time someone adds a negative value.

## An invalid discriminant is undefined behavior

In C, any integer fits in an enum type. You can assign `42` to an `enum Color`,
and a newer version of a library can add variants your code has never heard of.
In Rust, a value of an enum type must be one of its declared variants. An enum
holding any other discriminant is an invalid value, and producing one is
undefined behavior, even if you never `match` on it.

That makes this binding dangerous:

```rust,no_run
#[repr(C)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

unsafe extern "C" {
    // If C ever returns 3, the behavior is undefined as soon as the value
    // arrives, before any of your code looks at it.
    fn current_color() -> Color;
}
```

Nothing in Rust can protect you here, because the invalid value exists before
Rust gets a chance to check it. The fix is to accept the integer C actually
sends, and convert it into the enum yourself:

```rust,no_run
use std::ffi::c_int;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl TryFrom<c_int> for Color {
    type Error = c_int;

    fn try_from(value: c_int) -> Result<Self, c_int> {
        match value {
            0 => Ok(Color::Red),
            1 => Ok(Color::Green),
            2 => Ok(Color::Blue),
            other => Err(other),
        }
    }
}

unsafe extern "C" {
    fn current_color() -> c_int;
}
```

We sum this up as **integers in, enums out**. When C sends a value to Rust,
accept the integer type and convert. When Rust sends a value to C, returning the
enum is fine, because a Rust enum always holds a valid discriminant. It's also
why `bindgen` doesn't turn C enums into Rust enums by default. For a C
`enum Color { RED, GREEN, BLUE }`, it generates a type alias and a constant per
variant instead:

```rust,no_run
# use std::ffi::c_uint;
pub type Color = c_uint;
pub const Color_RED: Color = 0;
pub const Color_GREEN: Color = 1;
pub const Color_BLUE: Color = 2;
```

Note the `c_uint` in there. A C compiler may pick an unsigned type for an enum
whose variants are all non-negative, and GCC and Clang do, while `#[repr(C)]` on
a Rust enum gives you a signed, `int`-sized discriminant. The two agree on size
but not always on signedness, which only surfaces for values above `INT_MAX` —
one more reason to read the header rather than assume.

## Enums with fields

For enums whose variants carry data, the default layout is unspecified, and the
compiler uses it aggressively. It hides the discriminant in invalid bit patterns
wherever it can, which is how `Option<&T>` ends up as small as `&T`.

Adding an explicit `repr` gives the enum a defined layout, as specified by
[RFC 2195](https://github.com/rust-lang/rfcs/blob/master/text/2195-really-tagged-unions.md).
There are three variants, and they differ in where the discriminant sits and how
much padding follows it. Take this enum:

```rust,no_run
#[repr(u8)] // or `#[repr(C)]`, or `#[repr(C, u8)]`
pub enum TwoCases {
    A(u8, u16),
    B(u16),
}
```

**`#[repr(u8)]`** (or any other integer type) lays the enum out as a union of
`repr(C)` structs, one per variant, each starting with the `u8` tag. Each
variant's fields follow the tag directly, so small fields fill the space next to
it:

```text
A:  │ tag │ u8  │    u16    │   4 bytes
B:  │ tag │ pad │    u16    │
```

**`#[repr(C)]`** lays it out as a `repr(C)` struct with two fields: a tag, sized
like a C enum, and a union of the variants' fields. This is what you'd write by
hand in C:

```text
│         tag           │ A: u8 │ pad │    u16    │   8 bytes
                        │ B:    u16   │    pad    │
```

**`#[repr(C, u8)]`** is the same struct-plus-union layout, with a `u8` tag. The
union is still aligned to 2 bytes, so a byte of padding follows the tag:

```text
│ tag │ pad │ A: u8 │ pad │    u16    │   6 bytes
            │ B:    u16   │    pad    │
```

The same enum takes 4, 8, or 6 bytes depending on the `repr`. The default layout
also comes out at 4 bytes here, but that's the compiler's choice and it can
change. When you mirror a C struct-plus-union, `#[repr(C)]` is the one that
matches. The invalid discriminant rule applies here too: if the tag can come
from C, don't let C write a Rust enum directly. We'll see what to do instead at
the end of this section.

## Unions

A Rust `union` works like a C union: all fields start at offset 0, and the union
is as large as its largest field, rounded up to its alignment. Always mark
unions that cross the FFI boundary `#[repr(C)]`, since the default layout isn't
guaranteed to match C's:

```rust,no_run
#[repr(C)]
pub union IntOrFloat {
    pub i: u32,
    pub f: f32,
}

let value = IntOrFloat { f: 1.0 };
// SAFETY: every bit pattern is a valid `u32`.
let bits = unsafe { value.i };
assert_eq!(bits, 0x3f80_0000);
```

A union doesn't know which of its fields holds a value. Writing a field is safe,
but reading one is `unsafe`, because you're asserting that the bytes are valid
for that field's type. For `u32` and `f32` every bit pattern is valid, so
reading the "wrong" field gives you a meaningless number but no undefined
behavior. For a `bool`, an enum, or a reference, reading the wrong field can
produce an invalid value, which is undefined behavior.

A union field's type also has to be one that Rust never has to clean up, which
in practice means a `Copy` type: integers, floats, raw pointers, and `repr(C)`
structs of those. Since the union can't tell which field is active, it can't
know which destructor to run, so Rust doesn't let you put a type that needs one
into a union. The compiler says so when you try.

## Tagged unions from C

C code often combines an integer tag with a union, and relies on convention to
keep them in sync:

```c
#define SHAPE_CIRCLE 0
#define SHAPE_RECT 1

struct Shape {
    int kind;
    union {
        struct { float radius; } circle;
        struct { float width, height; } rect;
    } data;
};
```

Mirror that layout exactly, with the tag as an integer, and convert into a Rust
enum in one place:

```rust,no_run
use std::ffi::c_int;

pub const SHAPE_CIRCLE: c_int = 0;
pub const SHAPE_RECT: c_int = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Circle {
    pub radius: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rect {
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union ShapeData {
    pub circle: Circle,
    pub rect: Rect,
}

#[repr(C)]
pub struct CShape {
    pub kind: c_int,
    pub data: ShapeData,
}

pub enum Shape {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
}

impl TryFrom<&CShape> for Shape {
    type Error = c_int;

    fn try_from(shape: &CShape) -> Result<Self, c_int> {
        match shape.kind {
            SHAPE_CIRCLE => {
                // SAFETY: `kind` says `circle` is the active field.
                let Circle { radius } = unsafe { shape.data.circle };
                Ok(Shape::Circle { radius })
            }
            SHAPE_RECT => {
                // SAFETY: `kind` says `rect` is the active field.
                let Rect { width, height } = unsafe { shape.data.rect };
                Ok(Shape::Rect { width, height })
            }
            other => Err(other),
        }
    }
}
```

An unknown `kind` becomes an error instead of undefined behavior, and the rest
of your Rust code works with a `Shape` that can't be out of sync.

## Head to the exercise

The exercise mirrors a C header for you: an enum, and a struct with an integer
tag next to a union. Both conversions are missing. Write `TryFrom` for the enum,
and one that turns the tagged struct into a Rust enum by matching on the tag
before it reads a union field. The tests pass in every valid value, and a few
invalid ones.
