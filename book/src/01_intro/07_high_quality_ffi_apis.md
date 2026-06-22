# High-quality FFI APIs

We previously introduced two "mechanical" rules: always validating your input
and not overloading primitives. Now we will introduce two more rules that are a
bit more conceptual but have proven themselves _very_ useful.

But don't let that fool you, the following rules are just as important as the
previous two, if not more so.

## Documentation, documentation, documentation

Some invariants you can check at runtime. Many you can't — who owns this
pointer, whether the string is copied or borrowed, whether `free` has run
before, what each error variant means. Write them all down.[^1] Rust's
`# Safety` convention (being just a comment) works with `extern "C"` functions
too! `cheadergen` will emit them as C doc comments in the header files:

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

- Delegate upholding its local invariant to the surrounding function's safety
  comment. In this case it MUST reference a numbered invariant
- OR it must exhaustively explain why the local invariant is upheld by the code
  itself.

This way we make sure that all invariants are either upheld by the function
itself or correctly documented as a responsibility of the caller.

At the moment this is checked by discipline and PR reviews but in the future
this may soon find its way into Rust/Clippy proper.[^2]

## Mind the FFI tax

Every exposed function is API surface you'll maintain forever, an `unsafe`
contract to keep correct, and a per-call cost the compiler can't optimise away.
Cross-language LTO can inline across, at the cost of a real setup burden. The
cheapest FFI function is the one you didn't expose. Expose coarse operations —
`bm_thing_update_with(...)` over one setter per field — and treat the boundary
as a small set of verbs, not a mirror of your internal struct.

## Head to the exercise

Head to the exercise where we will update our exercise 06 solution with the new
rule(s) we have learned.

[^1]: You may (jadedly) say that no one ever reads comments and you may actually
    be correct. BUT, with the rise of LLMs _something_ does actually "read"
    them. We've found that LLMs unsurprisingly struggle quite a bit with the
    nuanced, unspoken invariants of FFI code. Turning as many of these unspoken
    invariants into spoken ones helps you get better mileage out of these tools.

[^2]: There are a couple of related proposals, all in the "pre-RFC" stage. The
    most interesting one being
    https://github.com/safer-rust/safety-tags/blob/main/pre-RFC.md
