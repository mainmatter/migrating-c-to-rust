# Bit flags

C represents a set of independent options by assigning each option a bit:

```c
#define FIND_URLS             (1u << 0)
#define FIND_TAGS             (1u << 1)
#define FIND_CASE_INSENSITIVE (1u << 2)

size_t find_bookmarks(const struct Bookmark *bookmarks, size_t len,
                      const char *query, uint32_t flags);
```

Callers combine options with `|` and test them with `&`. The representation is
compact and crosses an ABI easily, but `uint32_t` accepts unrelated constants,
unknown bits, and combinations the operation may not support.

## Name the set of valid bits

The `bitflags` crate generates a transparent newtype, constants, bitwise
operators, and set-like methods:

```rust,ignore
use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct FindFlags: u32 {
        const URLS = 1 << 0;
        const TAGS = 1 << 1;
        const CASE_INSENSITIVE = 1 << 2;
    }
}
```

The storage width matches the C header, and `#[repr(transparent)]` gives the
wrapper the layout of that `u32`. Within Rust, `contains` requires every bit in
its argument, while `intersects` asks whether any bit overlaps. `insert` and
`remove` mutate a set, and `bits()` returns the raw representation for C.

Named bits do not automatically make every combination meaningful. If a search
must inspect at least URLs or tags, validate that separately:

```rust,ignore
# use bitflags::bitflags;
# bitflags! { struct FindFlags: u32 {
#   const URLS = 1; const TAGS = 2; const CASE_INSENSITIVE = 4;
# } }
fn validate(flags: FindFlags) -> Result<FindFlags, &'static str> {
    if !flags.intersects(FindFlags::URLS | FindFlags::TAGS) {
        return Err("select URLS, TAGS, or both");
    }
    Ok(flags)
}
```

## Choose an unknown-bit policy

Converting a raw C integer requires an explicit compatibility decision:

- `from_bits(raw)` rejects any unknown bit;
- `from_bits_truncate(raw)` discards unknown bits;
- `from_bits_retain(raw)` preserves them.

Rejecting is appropriate when executing an unknown option could silently do the
wrong thing. Retaining can be useful for a value that must round-trip to a newer
library even though this version does not interpret every bit. Truncating is a
real behavior choice; it is not a neutral default.

A strict C adapter can therefore convert and validate before entering the safe
implementation:

```rust,ignore
# use bitflags::bitflags;
# bitflags! { #[derive(Clone, Copy)] struct FindFlags: u32 {
#   const URLS = 1; const TAGS = 2; const CASE_INSENSITIVE = 4;
# } }
# fn validate(flags: FindFlags) -> Result<FindFlags, ()> {
#   flags.intersects(FindFlags::URLS | FindFlags::TAGS).then_some(flags).ok_or(())
# }
fn flags_from_c(raw: u32) -> Result<FindFlags, ()> {
    FindFlags::from_bits(raw).ok_or(()).and_then(validate)
}
```

The safe search function accepts only `FindFlags`; only the boundary knows that
C supplied an untyped integer.

## Head to the exercise

In `exercises/02_intermediate/05_04_bit_flags`, update a set of bookmark output
fields using separate enable and disable masks. Detect conflicting changes with
typed flags, then strictly convert three raw C masks, apply the update, and
return the resulting raw bits.
