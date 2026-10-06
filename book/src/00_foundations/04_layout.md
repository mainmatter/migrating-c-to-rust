# Memory layout: size, alignment, and `repr`

When C and Rust share data, both sides have to agree on its layout: how many
bytes a value takes, and where each field sits inside them. C types derive their
default layout rules from the platform ABI. Rust types, by default, have an
unspecified layout optimized by the compiler. This section explains how to
ensure that size and alignment between C and Rust match predictably across
language boundaries.

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
hardware: misaligned loads and stores can be much slower than aligned accesses,
sometimes requiring the compiler to emit multiple instructions; on some
architectures they even raise a fault. Rust doesn't distinguish between
architectures here: accessing a value through a misaligned pointer is undefined
behavior on **every** target.[^1]

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

## How a struct gets its size and alignment

A struct's size and alignment are calculated from its fields. The fields are
placed one after the other, but each one has to start at an offset that is a
multiple of its own alignment, so a field may start a few bytes after the
previous one ends. The struct as a whole gets the alignment of its most-aligned
field, and its size is rounded up to a multiple of that alignment, so that the
next value in an array starts aligned too.

Here is a C struct and its layout:

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

`flag` sits at offset 0. `value` needs an offset that is a multiple of 4, so it
starts at 4. `tag` follows at 8. The struct's alignment is 4, from `value`, so
its size is rounded up to 12.

The bytes a struct skips to keep its fields aligned are called _padding_: the
three after `flag` and the three after `tag`. That's 6 bytes of data in 12 bytes
of memory.

## No guarantees by default

Rust gives the equivalent struct no specified layout:

```rust,no_run
struct Sample {
    flag: bool,
    value: u32,
    tag: u8,
}
```

The compiler may place these three fields in any order. With `value` first, the
same data requires less padding and fits in 8 bytes instead of 12.

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

[^1]: When the data really is misaligned, such as a struct read straight out of
    a network packet, reach for
    [`ptr::read_unaligned`](https://doc.rust-lang.org/std/ptr/fn.read_unaligned.html)
    and its `write_unaligned` counterpart. They copy the bytes without ever
    forming a misaligned reference, at the cost of the slower access the
    hardware needs for it.
