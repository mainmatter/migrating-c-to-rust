# The FFI boundary as a firewall: validate and narrow

By now you've written `extern "C"` functions in both directions, seen `bindgen`
generate Rust code and `cheadergen` generate C headers.

On paper that is everything you need, and it's tempting to go ahead and rewrite
your project now. Take your C API and translate it one-for-one into Rust.

The problem is that often you can't, and just as often you shouldn't. Real-world
C is complicated, idiosyncratic, and (let's admit it!) full of skeletons in the
closet. You're likely thinking about a Rust rewrite not just to address security
problems or performance issues; rewriting in Rust is a chance to clean up your
codebase: re-establish module boundaries, drop legacy assumptions, write the
module you wished you had.

At the same time big-bang rewrites (where you replace the entire codebase at
once) famously never work. Which leaves a problem: you have a clean Rust
codebase, a messy legacy codebase, and a need to keep both working together for
_some time_[^1].

During this transitional period, the FFI layer is where those worlds meet. It
has to bridge unsafe-everything-goes C and the borrow checker, and bridge your
new design and the old one.

Many people think FFI is about _type translation_. That's true as far as it
goes, but it's like saying programming is about typing words into a computer: it
misses the point. The _main_ job of your FFI interface is **establishing
confidence**. In a legacy codebase you rarely know with 100% certainty that
reality matches your assumptions: some caller might actually pass a null
pointer, or the `*const c_char` that used to be UTF-8 might now hold binary
data.

The FFI boundary is your chance to turn legacy uncertainty into known-good state
before it crosses into your new system. Uncertainty leaks. Your FFI must be a
vigilant firewall against it.

To this end, we've settled on a handful of rules that we now live by.

## Always validate your inputs

Validate every assumption at the FFI boundary. Null checks, length checks, UTF-8
checks. Panic or return an error if anything is even slightly off. In web
servers, the HTTP handler function is your primary interface to the "chaos
world" of the internet. Think of FFI functions the same way: they are your
primary interface to the legacy-C "chaos world".

You should also lean on Rust's type system so "forgot to check" isn't even an
option. For example: at Mainmatter, we encourage contributors to use
`Option<NonNull<T>>` instead of `*mut T` as much as possible, for the simple
reason that a `*mut T` can be dereferenced directly (`*ptr`), which triggers UB
if the pointer is null. You would have to add a manual `if ptr.is_null() {}`
check before every pointer dereference. With `Option<NonNull<T>>`, on the other
hand, the type system _forces_ you to handle the `None` case explicitly. Even
the laziest `.unwrap()` will result in a loud panic instead of potentially
silent UB.

```rust,compile_fail
// `*mut T` silently accepts null. You'll remember to check. Until you don't.
pub extern "C" fn bm_do_thing(input: *mut Thing) -> BmResult { /* … */
}

// `Option<NonNull<T>>` encodes null as `None` — the compiler forces the branch.
pub extern "C" fn bm_do_thing(input: Option<NonNull<Thing>>) -> BmResult {
    let Some(input) = input else {
        return BmResult::ErrInvalid;
    };
    // `input: NonNull<Thing>`, statically non-null.
    /* … */
}
```

## Don't overload primitives

Legacy C code has a habit of packing several meanings into one return type.
POSIX `read()` is a good example: `-1` means error (check `errno`), `0` means
end-of-file, anything positive is the byte count. It is _very easy_ to mess this
up. Rust gives you better tools: `bool` for yes/no, an enum for branching
outcomes, an out-parameter for the count. These types usually translate well
into C headers too, so use them.

```rust,compile_fail
#[repr(C)]
pub enum BmReadStatus { Ok, Eof, IoError }

pub extern "C" fn bm_read(/* … */, out_bytes: Option<NonNull<usize>>) -> BmReadStatus;
```

## Head to the exercise

We'll continue the porting work by looking at `bm_normalize_url`, in
`exercises/_bm/src/bookmark.c`. It will normalize a given URL by lowercasing it
and writing the normalized string into the provided buffer. The exercise already
contains our _"first draft"_ Rust translation: a naive transliteration from C to
Rust that has a number of issues.

Let's apply what we learned above and fix the implementation.

Hint: check the exercise's tests for the expected behavior of
`bm_normalize_url`.

[^1]: The exact amount of time of course varies with the scale of your codebase,
    but from hard-won experience: measure this in years. C codebase migrations
    have a funny habit of taking much, much longer than you think. As a rule of
    thumb: take your worst-case estimate, double it, and add a year. Really.
