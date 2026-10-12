# Enums and unions in FFI

A Rust enum is a sum type, with variants that can carry data. A C enum is just a
set of named integers. Only field-less Rust enums map onto C enums. For variants
with data, C pairs an integer tag with a union. This section covers how to
handle each at the FFI boundary.

## Field-less enums

In memory, a field-less Rust enum is just its discriminant: the integer that
says which variant it is. `#[repr(C)]` gives the discriminant the size a C
compiler would pick for the same enum.

```rust,no_run
#[repr(C)]
pub enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}
```

In C, any integer fits in an enum type. You can assign `42` to an `enum Color`,
and a newer version of a library can add variants your code has never heard of.
In Rust, a value of an enum type must be one of its declared variants. An enum
holding any other discriminant is undefined behavior. So when C sends you an
enum value, declare the binding with the integer type, and convert it using
`TryFrom`:

```rust,no_run
# use std::ffi::c_int;
# #[repr(C)]
# pub enum Color { Red = 0, Green = 1, Blue = 2 }
unsafe extern "C" {
    fn current_color() -> c_int;
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
```

## Unions

A Rust `union` works like a C union: all fields start at offset 0, and the union
is as large as its largest field. Mark any union that crosses the boundary
`#[repr(C)]`, since the default layout isn't guaranteed to match C's:

```rust,no_run
#[repr(C)]
pub union IntOrFloat {
    pub i: u32,
    pub f: f32,
}
```

A union doesn't know which of its fields holds a value. Writing a field is safe,
but reading one is `unsafe`, because you're asserting that the bytes are valid
for that field's type. Reading a `bool`, an enum, or a reference from bytes that
hold something else can produce an invalid value, which is undefined behavior.

Union fields are also limited to types without a destructor, which in practice
means `Copy` types, because the union can't tell which field's destructor to
run.

## Tagged unions from C

C often pairs an integer tag with a union, and relies on convention to keep the
two in sync:

```c
struct Value {
    int kind; /* VALUE_INT or VALUE_FLOAT */
    union {
        int64_t i;
        double f;
    } data;
};
```

Mirror that layout as it is, with a `repr(C)` struct and a `repr(C)` union, and
keep the tag a `c_int`. Then convert into a Rust enum in one place, reading only
the union field the tag points to:

```rust,no_run
# use std::ffi::c_int;
# pub const VALUE_INT: c_int = 0;
# pub const VALUE_FLOAT: c_int = 1;
# #[repr(C)] pub union CValueData { pub i: i64, pub f: f64 }
# #[repr(C)] pub struct CValue { pub kind: c_int, pub data: CValueData }
pub enum Value {
    Int(i64),
    Float(f64),
}

fn read_value(value: &CValue) -> Option<Value> {
    // SAFETY: each arm reads only the field that `kind` marks as active.
    match value.kind {
        VALUE_INT => Some(Value::Int(unsafe { value.data.i })),
        VALUE_FLOAT => Some(Value::Float(unsafe { value.data.f })),
        _ => None,
    }
}
```

An unknown `kind` becomes `None` instead of undefined behavior, and the rest of
your code works with a `Value` whose tag and data can't disagree.

To send a Rust enum to C in this shape, convert the other way: set the tag, and
write the variant's data into the matching union field. Writing a union field is
safe, so that direction needs no `unsafe` at all.

## Head to the exercise

The exercise mirrors a C tagged union for you, next to the Rust enum the rest of
the code works with. Convert that enum into the C struct, setting the tag and
the matching union field for each variant.
