# Loops and iterators

A C loop often does several jobs at once: it traverses memory, controls when
iteration stops, and computes a result. A literal Rust port can preserve all of
that machinery even after the data has become a safe Rust collection.

Consider this loop from `bm`:

```c
for (size_t i = 0; i < b->n_tags; i++) {
    if (strstr(b->tags[i], query))
        return 1;
}
return 0;
```

## Initial Rust port

A direct Rust translation looks like this:

```rust
# struct Bookmark { tags: Vec<String> }
# fn matches(bookmark: &Bookmark, query: &str) -> bool {
for index in 0..bookmark.tags.len() {
    if bookmark.tags[index].contains(query) {
        return true;
    }
}
false
# }
```

The calculation never uses the numeric value of `index`; it only needs the tag
stored there. Rust can iterate over those values directly:

```rust
# struct Bookmark { tags: Vec<String> }
# fn matches(bookmark: &Bookmark, query: &str) -> bool {
for tag in &bookmark.tags {
    if tag.contains(query) {
        return true;
    }
}
false
# }
```

The index existed only because C needs one to reach each element. A `for` loop
over the values is a good final form, especially when its body does several
things.

## Idiomatic Rust port

The tag loop computes whether any tag matches. The `any` operation states that
directly:

```rust
# struct Bookmark { tags: Vec<String> }
# fn matches(bookmark: &Bookmark, query: &str) -> bool {
bookmark.tags.iter().any(|tag| tag.contains(query))
# }
```

`any()` stops after the first match, so the control flow is the same as the
early `return` in the C loop. Keep that kind of behavior in mind when you port a
loop: short-circuiting, ordering, and whether values are borrowed or moved
should all stay the same.

## Pointer-bumping loops

The other common C shape walks a string by moving a pointer forward until it
reaches the NUL terminator:

```c
size_t n = 0;
for (const char *p = s; *p; p++) {
    if (*p == ',')
        n++;
}
```

A literal port keeps the raw pointer, which means every read and every step is
`unsafe`, and the loop is only correct as long as the string really ends in a
NUL:

```rust
# use std::ffi::c_char;
# let s = c"rust,ffi,c".as_ptr();
let mut n = 0;
let mut p: *const c_char = s;
// SAFETY: `s` points to a live, NUL-terminated string, and the loop stops at
// the terminator.
while unsafe { *p } != 0 {
    if unsafe { *p } == b',' as c_char {
        n += 1;
    }
    p = unsafe { p.add(1) };
}
# assert_eq!(n, 2);
```

Borrow the C string as a `&CStr` instead. `to_bytes()` gives you the bytes
before the terminator as a slice, so there is no terminator to look for and no
pointer to move. The loop becomes an iterator over those bytes:

```rust
# let c_str = c"rust,ffi,c";
let n = c_str.to_bytes().iter().filter(|&&byte| byte == b',').count();
# assert_eq!(n, 2);
```

## Keep raw pointers at the boundary

This refactoring begins after the FFI layer has validated raw pointers and
converted them into safe Rust slices, strings, or collections. Pointer
validation and slice construction stay in the boundary adapter; the iterator
chain operates only on safe Rust values.

## Head to the exercise

The starter code in `exercises/02_intermediate/05_05_loops_and_iterators` is
behaviorally correct, so its tests already pass. `wr` fails because Clippy finds
index-based loops whose indexes aren't part of the calculation.

Refactor the four functions without changing their results, ordering,
short-circuiting, or borrowing behavior. They correspond to `any`, `position`,
`filter().count()`, and a `filter`/`map`/`collect` pipeline with a nested `any`.
