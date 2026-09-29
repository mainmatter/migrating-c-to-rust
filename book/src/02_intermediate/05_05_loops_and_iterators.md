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

This is already an idiomatic improvement. It removes range bookkeeping and makes
every loop iteration provide the value the body actually uses. A `for` loop is a
perfectly good final form when its body contains several operations or exit
paths.

Use the form that matches how the collection should be accessed:

- `iter()` yields shared references;
- `iter_mut()` yields mutable references;
- `into_iter()` consumes the collection and yields its elements;
- `enumerate()` pairs each element with an index when the index is genuinely
  part of the calculation.

## Idiomatic Rust port

The tag loop computes whether any tag matches. The `any` operation states that
directly:

```rust
# struct Bookmark { tags: Vec<String> }
# fn matches(bookmark: &Bookmark, query: &str) -> bool {
bookmark.tags.iter().any(|tag| tag.contains(query))
# }
```

`iter()` supplies borrowed tags. The closure describes the condition for one
tag, and `any()` produces the boolean result. It stops after the first match, so
the control flow is the same as the early `return` in both loops above.

This gives a useful sequence for rewriting a loop:

1. describe what the loop computes;
2. replace accidental indexing with iteration over values;
3. use an iterator operation when one directly names that computation.

## How an iterator chain runs

An iterator expression has a source, may have one or more adapters, and ends in
an operation that consumes the iterator:

```text
source → adapters → consumer
```

For example:

```rust
# let tags = ["rust", "c", "ffi"];
let lengths = tags
    .iter()                    // source: borrowed tags
    .filter(|tag| tag.len() > 1) // adapter: retain some tags
    .map(|tag| tag.len())      // adapter: transform each tag
    .collect::<Vec<_>>();      // consumer: build the result
# assert_eq!(lengths, vec![4, 3]);
```

Adapters such as `filter` and `map` are lazy: they describe work but do not
traverse the collection by themselves. A consumer such as `any`, `count`, `sum`,
or `collect` drives the traversal. The chain above allocates the returned `Vec`,
but it does not allocate an intermediate collection between `filter` and `map`.

## Match the operation to the purpose

Common loop results have direct iterator operations:

| What the loop computes        | Iterator form              |
| ----------------------------- | -------------------------- |
| Whether a match exists        | `any()`                    |
| Whether every item matches    | `all()`                    |
| The first matching value      | `find()`                   |
| The first matching index      | `position()`               |
| The number of matches         | `filter().count()`         |
| A transformed collection      | `map().collect()`          |
| A filtered transformed result | `filter().map().collect()` |

These names describe the result rather than the mechanics used to obtain it. For
example, a loop that returns the index of the first matching bookmark becomes
`position`:

```rust
# struct Bookmark { url: String }
fn position_of_url(bookmarks: &[Bookmark], url: &str) -> Option<usize> {
    bookmarks
        .iter()
        .position(|bookmark| bookmark.url == url)
}
```

## Build a pipeline one step at a time

Suppose the result should contain the URLs of all bookmarks with a particular
tag. An ordinary `for` loop makes the behavior explicit:

```rust
# struct Bookmark { url: String, tags: Vec<String> }
fn tagged_urls<'a>(bookmarks: &'a [Bookmark], tag: &str) -> Vec<&'a str> {
    let mut urls = Vec::new();
    for bookmark in bookmarks {
        if bookmark.tags.iter().any(|item| item == tag) {
            urls.push(bookmark.url.as_str());
        }
    }
    urls
}
```

The same calculation can be separated into three operations:

1. iterate over borrowed bookmarks;
2. retain bookmarks containing the tag;
3. obtain their URLs and collect them.

```rust
# struct Bookmark { url: String, tags: Vec<String> }
fn tagged_urls<'a>(bookmarks: &'a [Bookmark], tag: &str) -> Vec<&'a str> {
    bookmarks
        .iter()
        .filter(|bookmark| bookmark.tags.iter().any(|item| item == tag))
        .map(|bookmark| bookmark.url.as_str())
        .collect()
}
```

The nested `any()` asks one boolean question about each bookmark. Even if a
bookmark contains the same tag twice, `filter` sees one boolean and the URL is
collected only once.

## Preserve behavior deliberately

Changing a loop's form must not change its observable behavior:

- `any()` and `all()` short-circuit;
- `find()` and `position()` return the first match;
- `filter()` and `map()` preserve input order;
- `iter()` borrows values instead of moving them;
- adapters do not allocate intermediate collections;
- `collect()` allocates the requested output collection.

These properties are often more important during a migration than whether the
final expression is shorter.

## Keep a `for` loop when it is clearer

Not every loop should become a chain. A `for` loop may communicate the behavior
better when its body performs several side effects, has multiple kinds of early
exit, or updates related state. Iterator operations are useful when they name a
simple result, not as a goal by themselves.

## Text loops still need a representation choice

A pointer-bumping C loop over a string may be traversing encoded bytes rather
than text characters. Rust makes that choice explicit: `.bytes()` traverses the
UTF-8 encoding, while `.chars()` traverses Unicode scalar values.

```rust
let commas = "rust,ffi,c".bytes().filter(|&byte| byte == b',').count();
assert_eq!(commas, 2);

let characters = "café".chars().count();
assert_eq!(characters, 4);
```

## Keep raw pointers at the boundary

This refactoring begins after the FFI layer has validated raw pointers and
converted them into safe Rust slices, strings, or collections. Pointer
validation and slice construction stay in the boundary adapter; the iterator
chain operates only on safe Rust values.

## Head to the exercise

The starter code in `exercises/02_intermediate/05_05_loops_and_iterators` is
behaviorally correct, so its tests already pass. `wr` fails because Clippy finds
index-based loops whose indexes are not part of the calculation.

Refactor the four functions without changing their results, ordering,
short-circuiting, or borrowing behavior. They correspond to `any`, `position`,
`filter().count()`, and a `filter`/`map`/`collect` pipeline with a nested `any`.
