# Slices in FFI

C has no slice type. A function that works on an array takes a pointer to the
first element and, if you're lucky, the number of elements as a separate
argument:

```c
int sum(const int *items, size_t len);
```

Rust's slices are the same pair of pointer and length, packed into one value
with guarantees attached. A `&[i32]` always points to `len` valid, initialized
`i32`s, and the borrow checker makes sure they stay that way while you use them.
Turning C's pair into a slice is one of the most common operations in FFI code,
and one of the easiest to get subtly wrong.

## Slices from C

`std::slice::from_raw_parts` turns a pointer and a length into a slice, and
`from_raw_parts_mut` does the same for a mutable one. The
[contract](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) is longer
than most people expect:

- The pointer must be **non-null and aligned**, even when the length is 0.
- It must point to `len` consecutive, **initialized** values of type `T`, all
  within a **single allocation**.
- The memory must **not be mutated** while the slice is in use. For
  `from_raw_parts_mut`, it must not be accessed through any other pointer at
  all.
- The total size, `len * size_of::<T>()`, must be at most `isize::MAX` bytes.
- The **lifetime** of the slice is whatever the caller picks, so you have to tie
  it to something that keeps the memory alive.

The first rule is the one that catches people. In C, passing `NULL` with a
length of 0 is a normal way to say "no items". For `from_raw_parts`, it's
undefined behavior, because a slice's pointer must never be null, not even an
empty one. So handle the empty case before you build a slice:

```rust,no_run
use std::slice;

/// # Safety
///
/// If `len` is non-zero, `ptr` must be non-null and aligned, and point to `len`
/// initialized `T`s that stay alive and unmodified for `'a`.
pub unsafe fn slice_from_c<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        return &[];
    }
    // SAFETY: `len` is non-zero, so the caller guarantees the whole contract.
    unsafe { slice::from_raw_parts(ptr, len) }
}
```

Everything else stays the caller's responsibility. Nothing in Rust can check
whether C's length is honest: if C says 10 and allocated 8, you read past the
end, and the `# Safety` section is the only place that says whose job it was to
prevent it.

Debug builds check some of these rules at runtime and abort with
`unsafe precondition(s) violated` when C passes `NULL`. Release builds skip
those checks, and the same call can appear to work. Don't count on the debug
check: write the explicit one anyway.

## Slices to C

Handing a slice to C is simpler. `as_ptr` and `len` give you the two halves, and
`as_mut_ptr` gives you a `*mut T` for C functions that write into the buffer:

```c
void scale_all(int32_t *items, size_t len, int32_t factor);
```

```rust,no_run
# // Stands in for the C implementation, so that this example links.
# #[unsafe(export_name = "scale_all")]
# unsafe extern "C" fn scale_all_impl(items: *mut i32, len: usize, factor: i32) {
#     let items = unsafe { std::slice::from_raw_parts_mut(items, len) };
#     items.iter_mut().for_each(|item| *item *= factor);
# }
unsafe extern "C" {
    fn scale_all(items: *mut i32, len: usize, factor: i32);
}

let mut items = [1_i32, 2, 3];
// SAFETY: the pointer and length describe `items`, which isn't accessed
// elsewhere during the call.
unsafe { scale_all(items.as_mut_ptr(), items.len(), 2) };
```

Two things to watch for. First, check which unit the C function counts its
length in. `scale_all` counts elements, like `len()` does, but `memcpy` and the
rest of `string.h` count bytes. Second, the pointer of an empty slice is
dangling: non-null, but not pointing to any memory. C code that follows the
length never touches it, but C code that checks `ptr != NULL` instead of
`len != 0` might.

This only lends the memory to C for the duration of the call. Handing ownership
over is a different matter, which
[Allocation, allocators, and mixing them](../02_intermediate/02_allocators.md)
covers.

## Head to the exercise

The exercise asks you to implement two functions that take a pointer and a
length the way C would pass them: one that reads the items, and one that
modifies them in place. The tests call them with normal input, and with both
ways of saying "no items": a null pointer with a length of 0, and a valid
pointer with a length of 0.
