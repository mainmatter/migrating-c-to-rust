# Dynamic analysis with Miri

In Chapter 1 we introduced the "firewall" rules that keep FFI interfaces sane,
and in the previous section we looked at lint rules that help enforce them.
Lints and compiler settings can't observe behavior, though. They can only read
your code as the compiler does, and infer from that.

We'll now introduce another class of tooling to complement static analysis:
dynamic analysis with Miri.

Miri is an interpreter for rustc's MIR (mid-level intermediate representation).
It runs your code, checking each operation against the Rust Abstract Machine.
Miri is especially useful for `unsafe` Rust, since it catches incorrect
initialization, use-after-free, and borrowing-rule violations: a good fit for
the kind of systems code we tend to translate from C to Rust. Do note that Miri
requires a nightly compiler. This repository's `rust-toolchain.toml` already
pins one with the `miri` component included, so the exercises work without any
extra setup.

## What Miri catches

When interpreting your Rust code, Miri tracks the initialization state and
provenance of each allocation. It will flag reads outside the bounds of your
allocation, before it's correctly initialized (unless you use `MaybeUninit`), or
after it's freed. Miri also catches data races, including weak-memory
reorderings, and the aliasing violations that complex unsafe code tends to
produce, such as writing through a mutable reference while other code still
holds references to the same allocation. It also catches memory leaks: if a test
ends without freeing every allocation, for example through `Box::into_raw` or
`CString::into_raw` without a matching `from_raw`, Miri will fail the test.

## The downsides

Miri is by design an interpreter, meaning it can only ever see code paths that
actually execute. Code paths never exercised by your test suite are invisible to
Miri. This also means non-deterministic bugs need luck: a bug that only happens
on a specific thread interleaving will remain flaky.[^1] Miri is also slow:
expect a 3,000–7,000× slowdown in some cases.

> ## Quick Miri cheat sheet
>
> Using Miri is just like using Cargo, but with `miri` inserted into the
> commands. For example, to run unit tests: `cargo miri test -p <crate>`. You
> can also run binaries, examples, etc.
>
> To pass flags to Miri itself, use the `MIRIFLAGS` environment variable.
>
> The first `cargo miri` invocation builds a Miri-specific sysroot, which takes
> a minute or two. Run `cargo miri setup` once ahead of time to get that out of
> the way.
>
> Quite often you will need to tweak your code and test cases to run under Miri
> at reasonable speed, or at all. For example, you may want to reduce test
> iterations when running under Miri or disable certain incompatible tests. For
> this, the `miri` cfg attribute exists. You can write `#[cfg(miri)]` on
> expressions to enable them _only_ when executing under Miri. For incompatible
> tests you can write `#[cfg_attr(miri, ignore)]`.
>
> Note that Miri (being an interpreter) also accepts any `--target` triple which
> is handy to test certain target features such as big-endian or 32-bit without
> running an actual emulator of some sort.

## Miri and C

Miri is a pure Rust interpreter and cannot run native libraries. The Miri team
hand-wrote shims for common ones such as libc, which are built into Miri, but
for the long tail of native libraries there is no support. Unfortunately, our
mixed C-Rust codebase sits right in that long tail. So the question isn't "can
Miri check my mixed codebase?" (it cannot), but "how much of it can I get Miri
to check?"

The strategy we most often employ is "thin glue, thick core": pure-Rust core
with `unsafe extern "C"` functions (the uncheckable part) being extremely simple
adapters only. This means the vast majority of the code we actually care about
is already Miri-checkable, and FFI functions can be validated using lints,
reviews, and other tools if necessary.

The only complicating factor is Rust calling into C: our Rust code calling a
not-yet-translated module, for example, or some native library we do not intend
to port at all. For this we use hand-written shims (just like the Miri team did)
that swap out the C functions for Rust stubs when running under Miri. Keep in
mind that the stub doesn't need to implement the C code at all; it simply needs
to return the right values: error codes, out-params, and buffers of a given
length. One neat side effect is that "stubbing out a C function" forces you to
confront the details of a function that we gloss over when we simply call it.
That usually leads to a much better understanding of the code, which makes it a
worthwhile exercise in itself.

You would usually have something like this:

```rust,no_run
#[cfg(miri)]
mod stubs {
    #[unsafe(no_mangle)]
    unsafe extern "C" fn SomeCFunction() {
        // ... some stub here ...
    }
}
```

In a regular build we would link against the `.so` library and get the
`SomeCFunction` symbol that way. Under Miri we stub out the symbol, pointing it
at a simple pure-Rust function. The stubs live in their own module because Rust
won't let a definition and the `extern` declaration of `SomeCFunction` share a
name.

One word of caution, though: Miri passing with Rust stubs does not mean your
codebase with the full C functions is correct. These stubs, written correctly,
can give you confidence that the final product will also be correct, but they
are not a substitute for full-program tests. It is easy to accidentally test
your stubs' behavior more than the actual C code's.

## Future features

Miri has gained support for running code directly from native libraries:
`MIRIFLAGS="-Zmiri-native-lib=path/to/libbm.so"` loads the .so and directly
calls the symbols in it. This is a promising direction, but for now it is
limited. It does not support Windows, supports only integer and pointer
arguments and returns (no structs by value), and by design cannot check most
Rust invariants: memory shared with C stops being tracked for init and
provenance. On Linux there is even more experimental support for native tracing
that attempts to trace which bytes C touched and keep tracking for all others,
but none of it is ready for production use yet.

## Head to the exercise

The exercise is in `exercises/02_intermediate/06_miri`. It contains a small Rust
wrapper around `tag_normalize` from `tag.c`, a module we haven't ported yet. Its
tests pass with a plain `cargo test` but fail under Miri.

[^1]: The
    [`-Zmiri-many-seeds`](https://github.com/rust-lang/miri/#testing-multiple-different-executions)
    option runs several executions at once, which helps catch flakes more often.
    Miri also has the `-Zmiri-address-reuse-rate`,
    `-Zmiri-address-reuse-cross-thread-rate`, `-Zmiri-preemption-rate`, and
    `-Zmiri-compare-exchange-weak-failure-rate` options that help with catching
    threading bugs more often. But generally it's better to turn to tools like
    [`loom`](https://github.com/tokio-rs/loom) for proper concurrency testing.
