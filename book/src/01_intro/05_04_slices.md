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

## `slice::from_raw_parts`

The standard library function for the job is `std::slice::from_raw_parts`, with
`from_raw_parts_mut` for the mutable variant:

```rust,no_run
use std::slice;

/// # Safety
///
/// `items` must point to `len` initialized `i32`s.
pub unsafe fn sum(items: *const i32, len: usize) -> i32 {
    // SAFETY: the caller guarantees that `items` points to `len` `i32`s.
    let items = unsafe { slice::from_raw_parts(items, len) };
    items.iter().sum()
}
```

That `# Safety` section is incomplete, though. The
[full contract](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html) of
`from_raw_parts` is longer than most people expect:

- The pointer must be **non-null and aligned**, even when `len` is 0.
- It must point to `len` consecutive, **initialized** values of type `T`, all
  within a **single allocation**.
- The memory must **not be mutated** for as long as the slice is in use. For
  `from_raw_parts_mut`, it must not be accessed through any other pointer at
  all.
- The total size, `len * size_of::<T>()`, must be at most `isize::MAX` bytes.
- The **lifetime** of the returned slice is whatever the caller picks, the same
  way it is when you turn a raw pointer into a reference. You have to tie it to
  something that keeps the memory alive.

## The `NULL, 0` pitfall

The first rule is the one that catches people. In C, passing `NULL` with a
length of 0 is a completely normal way to say "no items", and plenty of C code
does it. For `from_raw_parts`, it's undefined behavior: a slice's pointer must
never be null, not even an empty one.

So don't pass C's pointer straight through. Handle the empty case separately,
and the rest with `from_raw_parts`:

```rust,no_run
use std::slice;

/// Turns a pointer and length from C into a slice.
///
/// # Safety
///
/// If `len` is non-zero, `ptr` must be non-null and aligned, and must point to
/// `len` initialized `T`s in a single allocation that stays alive and
/// unmodified for `'a`.
pub unsafe fn slice_from_c<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        return &[];
    }
    // SAFETY: `len` is non-zero, so the caller guarantees the whole contract.
    unsafe { slice::from_raw_parts(ptr, len) }
}
```

For `len == 0`, the function returns an empty slice literal, which Rust builds
from a dangling but valid pointer, so C's `NULL` never reaches `from_raw_parts`.
Everything else stays the caller's responsibility, and there's no way around
that: nothing in Rust can check whether C's length is honest. If C says 10 and
allocated 8, you read past the end, and the `# Safety` section is the only place
that says whose job it was to prevent it.

## Safety nets, and why you can't rely on them

Rust catches some of these mistakes for you. If the null pointer is visible at
compile time, the compiler rejects the call outright:

```text
error: calling this function with a null pointer is undefined behavior, even if
the result of the function is unused
```

Pointers from C are never visible at compile time, though. At runtime, debug
builds check some of `from_raw_parts`'s preconditions and abort with a message
like this:

```text
unsafe precondition(s) violated: slice::from_raw_parts requires the pointer to
be aligned and non-null, and the total size of the slice not to exceed
`isize::MAX`
```

Release builds skip those checks. The same call that aborts in a debug build
runs to completion in a release build, hands you an empty slice, and looks like
it worked. That's the dangerous part about undefined behavior: working as
expected is one of the possible outcomes, until the optimizer makes a different
choice. Treat the debug checks as a bonus, and write the explicit check anyway.

## The other direction

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
#     for item in items {
#         *item *= factor;
#     }
# }
unsafe extern "C" {
    fn scale_all(items: *mut i32, len: usize, factor: i32);
}

let mut items = [1_i32, 2, 3];
// SAFETY: the pointer and length describe `items`, which stays alive and isn't
// accessed elsewhere during the call.
unsafe { scale_all(items.as_mut_ptr(), items.len(), 2) };
assert_eq!(items, [2, 4, 6]);
```

Check which unit each C function expects for its length. `scale_all` counts
elements, which matches `len()`. Functions from `string.h`, such as `memcpy` and
`memset`, count bytes, and there you want `items.len() * size_of::<i32>()`.
Getting this wrong either processes a fraction of the data or runs off the end.

Some C APIs want a begin and end pointer instead of a pointer and a length.
`as_ptr_range` gives you exactly that pair.

All of these only lend the memory to C for the duration of the call. Handing
ownership over is a different operation: `Vec::into_raw_parts` gives you the
pointer, length, and capacity and stops Rust from freeing the memory, and
`Vec::from_raw_parts` takes it back. Which side frees that memory, and with
which allocator, then becomes a question you have to answer.

One more detail to watch for: the pointer of an empty slice is dangling. It's
non-null and aligned, but it doesn't point to any memory. C code that follows
the length will never dereference it, but C code that checks `ptr != NULL`
instead of `len != 0` might.

## Head to the exercise

The exercise asks you to implement two functions that take a pointer and a
length the way C would pass them. The tests call them with normal input, and
with both ways of saying "no items": a null pointer with a length of 0, and a
valid pointer with a length of 0.
