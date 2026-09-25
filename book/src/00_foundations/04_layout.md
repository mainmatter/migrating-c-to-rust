# Memory layout: size, alignment, and `repr`

When C and Rust share data, they share bytes. Both sides have to agree on how
many bytes a value takes, and on where inside those bytes each field sits. C
fixes both: for a given target, the standard and the platform ABI determine the
width of every type and the offset of every struct field. Rust fixes neither for
its own types. The compiler chooses, and it is free to choose differently in the
next release.

This section covers the vocabulary for talking about that, and `repr`, the
attribute that pins a type's layout down.

## Size and alignment

Every type has a size and an alignment. The _size_ is how many bytes a value
takes up. The _alignment_ is a number that the address of a value has to be a
multiple of. Both are known at compile time:

```rust,no_run
assert_eq!((size_of::<u8>(), align_of::<u8>()), (1, 1));
assert_eq!((size_of::<u32>(), align_of::<u32>()), (4, 4));
assert_eq!((size_of::<[u32; 3]>(), align_of::<[u32; 3]>()), (12, 4));
```

For primitives, alignment usually equals size. Alignment comes from the
hardware: an aligned value can be loaded with a single instruction, and some
architectures raise a fault for a load that isn't aligned. Rust doesn't
distinguish between architectures here. Accessing a value through a misaligned
pointer is undefined behavior on every target, including those whose
instructions handle misaligned access without complaint.

An array's size is its element size times its length, with nothing in between,
which is what turns indexing into a multiplication.

Two cases are worth knowing, because they don't follow from the above:

- **Some types have size 0.** The unit type `()` and a struct with no fields
  occupy no bytes, and have an alignment of 1.
- **Not every reference is one word.** A `&u8` is a single address, 8 bytes on a
  64-bit target. A `&[u8]` is 16: an address plus a length. The same holds for
  `&str`. Those extra bytes are the reason a slice cannot be handed to C as a
  plain pointer.

## `Layout`

The two numbers travel together often enough that the standard library has a
type for the pair. `std::alloc::Layout` carries a size and an alignment,
describing either a single value or an array of them:

```rust,no_run
use std::alloc::Layout;

let one = Layout::new::<u64>();
assert_eq!((one.size(), one.align()), (8, 8));

let three = Layout::array::<u64>(3).unwrap();
assert_eq!((three.size(), three.align()), (24, 8));
```

`Layout::array` returns a `Result`, because the total size of a large array can
overflow.

Allocation APIs are written in terms of `Layout` rather than a type parameter:
an allocator has to know how many bytes to hand out and how strictly they need
to be aligned, and nothing else about the value going into them. That's also the
form in which you'll meet it, as the argument to an allocator's methods rather
than as something you construct by hand.

## Padding

Put fields of different alignments in one struct and they can't all sit next to
each other, because each one needs an address that matches its own alignment.
The bytes left in the gaps are called _padding_.

Here is a C struct and the bytes a C compiler gives it:

```c
struct sample {
    bool flag;
    uint32_t value;
    uint8_t tag;
};
```

```text
offset  0      1              4             8      9          12
        ├ flag ┼──── pad ─────┼─── value ───┼ tag ─┼─── pad ───┤
```

Three bytes of padding follow `flag`, because `value` needs an address that is a
multiple of 4. The struct's alignment is 4, the largest among its fields, and
its size is rounded up to a multiple of that alignment, which adds three more
bytes at the end.

That trailing padding is what makes arrays of structs work. Each element starts
where the previous one ended, so a size that is a multiple of the alignment
keeps every element aligned.

Six bytes of data, twelve bytes in memory. Padding is the cost of a fixed field
order.

## No guarantees by default

Rust gives the equivalent struct no specified layout:

```rust,no_run
struct Sample {
    flag: bool,
    value: u32,
    tag: u8,
}
```

The compiler may place these three fields in any order, and it uses that freedom
to avoid the padding a fixed order would cost: with `value` first, the same data
fits in 8 bytes instead of 12. The choice isn't stable either. A different
compiler version may lay the same struct out differently, since nothing in the
language specifies it.

For Rust-only code that is a good trade, because the compiler picks a tighter
layout than a fixed order would and nothing outside the program depends on the
result. It stops working as soon as C is involved: the C side computes field
offsets from the header, and those offsets follow C's rules rather than whatever
the Rust compiler decided.

## What `repr` is for

`repr` is an attribute on a type definition that replaces the default layout
with a specified one. There are two reasons to reach for it:

- **The layout has to match a definition outside the program.** A struct C reads
  or writes, a value passed to a syscall, a header from a file format or a
  network packet. Something else already fixed the byte positions, and the type
  has to reproduce them.
- **The padding or alignment itself matters.** Removing padding to match a
  packed wire format, or raising alignment because hardware asks for it.

`repr` applies to `struct`, `enum`, and `union` definitions, and it describes
the whole type: there is no way to give an individual field its own
representation, and Rust has no equivalent of C's bitfields.

The variant you'll use most is `repr(C)`, which lays a type out by C's rules.
The others cover enum discriminants, single-field wrappers, and changes to
padding and alignment, and each one comes up together with the problem it
solves.

A type without a `repr` attribute has the default representation, sometimes
written `repr(Rust)`, which is the unspecified one described above.

Whatever variant you use, `repr` affects where the fields live in memory. It
doesn't change which fields the type has, or the values stored in them.

## Head to the exercise

The exercise has two halves. First, work out the size and alignment of a few
types on paper and write the numbers down as constants, which the tests compare
against what the compiler reports. Then take a `repr(C)` struct that carries 14
bytes of fields in 24 bytes of memory, and reorder its fields so it carries them
in 16.
