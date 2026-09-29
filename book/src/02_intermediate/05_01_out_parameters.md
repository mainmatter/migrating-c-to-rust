# Replacing C-isms with Rust-isms

A literal port can preserve shapes that exist because of C's type system rather
than because of the program's domain. The code may work, but its Rust API still
asks callers to follow conventions that Rust can express directly.

The sections that follow separate five common transformations. Each one starts
with a recognizable C shape, identifies the information hidden in that shape,
and moves it into a Rust type or iterator operation, starting here with
out-parameters.

These are changes to the safe Rust implementation. If C code still calls the
module, its ABI remains in place as a thin adapter. The adapter validates and
converts C values, calls the safe API, and maps the result back to the existing
C contract.

## Out-parameters

Some C functions return values by writing them through pointer arguments. These
arguments are called out-parameters. For example, this function splits a
duration into minutes and seconds:

```c
void split_duration(uint32_t total_seconds,
                    uint32_t *out_minutes,
                    uint32_t *out_seconds);
```

`total_seconds` is an input. The function writes its two results through
`out_minutes` and `out_seconds`. Those pointers are output channels rather than
ordinary mutable state.

C functions can return only one value directly, so APIs commonly use
out-parameters for additional results. Rust functions can return tuples and
structs, which usually express those results without requiring the caller to
provide empty destinations first.

## Put outputs in the return type

A literal Rust translation preserves those destinations:

```rust
fn split_duration(total_seconds: u32, out_minutes: &mut u32, out_seconds: &mut u32) {
    *out_minutes = total_seconds / 60;
    *out_seconds = total_seconds % 60;
}
```

The caller must create two variables before it can call the function, even
though those variables have no useful value yet. The signature also describes
the mechanism inherited from C rather than the values being computed.

Return both values instead:

```rust
fn split_duration(total_seconds: u32) -> (u32, u32) {
    (total_seconds / 60, total_seconds % 60)
}
```

The result is now initialized as one operation and can be destructured where it
is used:

```rust
# fn split_duration(total_seconds: u32) -> (u32, u32) {
#     (total_seconds / 60, total_seconds % 60)
# }
let (minutes, seconds) = split_duration(125);
assert_eq!((minutes, seconds), (2, 5));
```

One required output becomes a direct return value. A few closely related values
can become a tuple. Prefer a named result struct when there are many values, the
ordering would be unclear, or the result is passed through several APIs.

## Preserve an existing C boundary

Existing C callers still need the original signature. Keep the out-parameters in
a thin compatibility adapter and use the tuple inside Rust:

```rust,no_run
fn split_duration(total_seconds: u32) -> (u32, u32) {
    (total_seconds / 60, total_seconds % 60)
}

/// # Safety
///
/// `out_minutes` and `out_seconds` must be valid, writable pointers.
#[unsafe(export_name = "split_duration")]
pub unsafe extern "C" fn split_duration_ffi(
    total_seconds: u32,
    out_minutes: *mut u32,
    out_seconds: *mut u32,
) {
    let (minutes, seconds) = split_duration(total_seconds);

    // SAFETY: The caller must satisfy the pointer requirements above.
    unsafe {
        out_minutes.write(minutes);
        out_seconds.write(seconds);
    }
}
```

The C ABI remains unchanged, but the out-parameter shape stops at the boundary.
The calculation and its Rust callers use an ordinary return value.

Some C APIs allow an output pointer to be `NULL` when the caller does not need
that output. In that case, the adapter checks each pointer before writing it.
That does not necessarily make the computed value optional: the safe Rust
function can still return all of its results.

## Recognize values split across channels

An out-parameter can combine with the ordinary return value to describe one
logical result. `bm` contains this example:

```c
size_t bm_cursor_tags(BmCursor *cursor, const char ***out_tags);
```

The return value is the number of tags and `out_tags` receives a pointer to the
first tag. Together they describe a borrowed sequence. Once the cursor is a safe
Rust type, its API can return that sequence as one value:

```rust,ignore
fn tags(&self) -> &[Tag];
```

The same reasoning applies to other paired outputs: identify the value the
pieces represent, then choose the Rust return type that represents it directly.
The compatibility adapter is responsible for splitting that value back into the
fields expected by C.

## Head to the exercise

In `exercises/02_intermediate/05_01_out_parameters`, replace two page-location
out-parameters with a `(page, slot)` return value. Then complete the C adapter
without changing its `void bm_page_location(...)` contract. The exercise has no
failure path or status codes: it focuses only on returning values instead of
writing them through pointers.
