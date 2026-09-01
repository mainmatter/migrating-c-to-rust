# High-quality FFI APIs

We previously introduced two "mechanical" rules: always validate your inputs,
and don't overload primitives. The two that follow are more conceptual. Don't
let that fool you: they are just as important as the first two, if not more so.

## Documentation, documentation, documentation

Some invariants you can check at runtime. Many you can't: who owns this pointer,
whether the string is copied or borrowed, whether `free` has already run, what
each error variant means. Write them all down.[^1] Rust's `# Safety` convention,
being just a comment, works with `extern "C"` functions too, and `cheadergen`
emits those sections as C doc comments in the header files:

```rust
/// Normalize a URL into the caller's buffer.
///
/// # Ownership
/// `url` and `out` are borrowed for the call; the caller continues to own both.
///
/// # Safety
/// 1. `url`, if non-null, must be a [valid] pointer to a NUL-terminated UTF-8 byte sequence.
/// 2. `out`, must be a [valid], non-null pointer to a writable buffer of at least `out_len` bytes.
///
/// # Errors
/// - `BmResult::ErrInvalidUrl` if `url` is null or not valid UTF-8.
/// - `BmResult::ErrBufferTooSmall` if the result wouldn't fit in `out_len` bytes.
///
/// [valid]:
#[no_mangle]
pub unsafe extern "C" fn bm_normalize_url(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> BmResult {
    let Some(out) = out else {
        //...
    };

    // Safety: caller ensured `out` points to at least `out_len` bytes (2.)
    unsafe { slice::from_raw_parts_mut(out.as_ptr(), out_len) };
}
```

Note how we number safety invariants and force every inline safety comment to
either:

- delegate its local invariant to the surrounding function's safety comment, in
  which case it must reference a numbered invariant; or
- explain exhaustively why the code itself upholds the local invariant.

This way we make sure that all invariants are either upheld by the function
itself or correctly documented as a responsibility of the caller.

At the moment, the numbered-invariant convention is checked by discipline and PR
review. Clippy can require `# Safety` documentation and comments on `unsafe`
blocks, but it cannot verify that a local justification actually upholds an
invariant. There are proposals for tools that could make this kind of
traceability machine-checkable in the future.[^2]

## Mind the FFI tax

Every exposed function is API surface you'll maintain forever, an `unsafe`
contract to keep correct, and a per-call cost the compiler can't optimize away.
Cross-language LTO can inline across the boundary, at the cost of a real setup
burden. The cheapest FFI function is the one you didn't expose. Prefer coarse
operations, such as `bm_thing_update_with(...)`, over one setter per field, and
treat the boundary as a small set of verbs, not a mirror of your internal
struct.

## Head to the exercise

You'll update your solution to exercise 1.6 with the two rules from this
section.

[^1]: You may say, jadedly, that no one ever reads comments, and you may be
    right. But with the rise of LLMs, _something_ does read them. We've found
    that LLMs struggle with the nuanced, unspoken invariants of FFI code, which
    is not surprising. Turning as many of these unspoken invariants into spoken
    ones helps you get better mileage out of these tools.

[^2]: There are a couple of related proposals, all in the "pre-RFC" stage. The
    most interesting is the
    [safety-tags pre-RFC](https://github.com/safer-rust/safety-tags/blob/main/pre-RFC.md).
