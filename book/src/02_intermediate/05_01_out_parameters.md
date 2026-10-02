# Out-parameters

A literal port can preserve shapes that exist because of C's type system rather
than because of the program's domain. The code may work, but its Rust API still
asks callers to follow conventions that Rust can express directly.

The following sections cover five common transformations, starting here with
out-parameters. Each one starts with a recognizable C shape, identifies the
information hidden in it, and moves that information into a Rust type or an
iterator operation.

## What an out-parameter is

Some C functions return values by writing them through pointer arguments. These
arguments are called out-parameters. For example, this function splits a
duration into minutes and seconds:

```c
void split_duration(uint32_t total_seconds,
                    uint32_t *out_minutes,
                    uint32_t *out_seconds);
```

`total_seconds` is an input. The function writes its two results through
`out_minutes` and `out_seconds`, because a C function can return only one value
directly.

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

Rust can return both values, as a tuple or a struct:

```rust
fn split_duration(total_seconds: u32) -> (u32, u32) {
    (total_seconds / 60, total_seconds % 60)
}
```

## Preserve an existing C boundary

Existing C callers still need the original signature. Keep the out-parameters in
a thin compatibility adapter that calls the Rust `split_duration` above and
writes its two results through the pointers:

```rust,no_run
# fn split_duration(total_seconds: u32) -> (u32, u32) {
#     (total_seconds / 60, total_seconds % 60)
# }
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

Some C APIs allow an output pointer to be `NULL` when the caller doesn't need
that output. In that case, the adapter checks each pointer before writing
through it.

## Recognize values split across channels

An out-parameter can combine with the ordinary return value to describe one
logical result. `bm` contains this example:

```c
size_t bm_cursor_tags(BmCursor *cursor, const char ***out_tags);
```

The return value is the number of tags, and `out_tags` receives a pointer to the
first tag. Together they describe a borrowed sequence. Once the cursor is a safe
Rust type, its API can return that sequence as one value:

```rust,ignore
fn tags(&self) -> &[Tag];
```

The same reasoning applies to other paired outputs: identify the value the
pieces represent, then choose the Rust return type that represents it directly.
The compatibility adapter splits that value back into the return value and
out-parameter C expects.

## Head to the exercise

In `exercises/02_intermediate/05_01_out_parameters`, implement `page_location`
so that it returns a bookmark's page and slot as a `(page, slot)` tuple instead
of writing them through out-parameters. Then complete the C adapter without
changing its `void bm_page_location(...)` contract.
