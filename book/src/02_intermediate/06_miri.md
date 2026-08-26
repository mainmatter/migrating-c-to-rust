# Miri

In the first chapter we introduced the "firewall" rules that keep FFI interfaces sane and you've learned 
about lint rules to enforce these earlier too. Lints and compiler settings suffer from not being able to observe
behaviour though. They can only read your code as the compiler does and infer from that.

We'll now introduce another class of tooling to complement the *static analysis*: *dynamic analysis*. In particular `Miri`. 

Miri is an interpreter for rustc's MIR (mid-level intermediate representation). It runs your code, checking each operation against the Rust Abstract Machine. Miri is especially useful for `unsafe` Rust since it can catch things like incorrect initialization, use after free, borrowing rule violations, etc. A perfect tool for the kinds of systems code we tend to translate from C to Rust.

## What Miri Catches

When interpreting your Rust code Miri tracks the initialization state and provenance of each allocation. It will flag if you read outside the bounds of your allocation, before its correctly initialized (unless you use `MaybeUninit`) or after its freed. Miri will also catch data races including weak-memory reorderings and aliasing violations (writing through a mutable reference while other code still holds references to the allocation too) such as can happen with complex unsafe code. Additionally it will also catch memory leaks: if you have not freed all allocations when a test ends e.g. through `Box::into_raw`/`CString::into_raw` without a matching `from_raw` Miri will fail the test.

## The Downsides

Miri is by design an interpreter, meaning it can only ever see code paths that actually executed. Codepaths never exercised by your test suite will not be visibile to Miri! This also means non-deterministic bugs need luck: A bug that only happens on a specific thread interleaving will remain flaky. [^1] Miri is also very slow, expect ~3000–7000× in some cases. 

> ## Quick Miri Cheat Sheet
> 
> Using miri is just like using cargo regularly, but with `miri` prefixed to the commands. 
> For example to run unit tests: `cargo miri test -p <crate>`. You can also run binaries, examples, etc.
> 
> To pass along flags to miri itself use the `MIRIFLAGS` env var.
>
> The first `cargo miri` invocation builds a Miri-specific sysroot, which takes a minute or two. Run `cargo miri setup` once ahead of time to get that out of the way.
>
> Quite often you will need to tweak your code and test cases to run under miri performantly (or at all). For example, 
> you may want to reduce test iterations when running under Miri or disable certain incompatible tests. For this, the `miri` cfg attribute exists. You can write `#[cfg(miri)]` on expressions to enable them _only_ when executing under Miri.
> For incompatible tests you can write `#[cfg_attr(miri, ignore)]`.
>
> Note that Miri (being an interpreter) also accepts any `--target` triple which is handy to emaulate certain hardware features such as big-endian or 32-bit.

## Miri And C

Miri is a pure Rust interpreter and cannot run native libraries. The Miri team hand-wrote shims for common ones such as libc and others which are builtin to Miri, but for the long tail of native libraries there is no support. Unfortunately we are working right in this long tail with our mixed C-Rust codebase... So for us the question isn't "can Miri check my mixed codebase" (it cannot), it is "how do I get it to check as much as possible". 

The strategy we most often employ is "thin glue, thick core": pure-Rust core with `unsafe extern "C"` functions (the uncheckable part) being extremely simple adapters only. This means the vast majority of the code we actually care about is trivially Miri checkable already and FFI functions can be validated using lints, reviews and other tools if necessary.

The only complicating factor is Rust calling into C, e.g. our Rust code calling a not-yet translated module, or some native library which we do not intend to port at all. For this we use hand-written shims (just like the Miri team did) that swap out the C functions for Rust stubs when running under Miri. Keep in mind the stub doesn't need to fully implement the C code at all, it simply needs to return the right values (error codes, out-params, buffer of certain lengths). One neat side effect is that "stubbing out a C function" forces you to confront the details of a function that we often gloss over when simply calling it which often lead us to a much better understanding of the code as a result, a very worthwhile exercise!

You would usually have something like this:

```rust
#[cfg(miri)]
mod stubs {
    #[unsafe(no_mangle)]
    unsafe extern "C" fn SomeCFunction() {
        // ... some stub here ...
    }
}
```

In a regular build we would link against the `.so` library and get the `SomeCFunction` symbol that way, under Miri we stub out the symbol pointing it at a simple pure Rust function. The stubs live in their own module because Rust won't let a definition and the `extern` declaration of `SomeCFunction` share a name.

One word of caution though: Miri passing with Rust stubs *obviously* does not mean your codebase with the full C functions is correct. These stubs - written correctly - can give you confidence that the final product will also be correct, but do not substitue full program tests. It is easy to accidentally test your stubs behvaiour more than the actual C codes.


## Future Features

Miri has very recently gained support for running code directly from native libraries: `MIRIFLAGS="-Zmiri-native-lib=path/to/libbm.so"` loads the .so and directly calls the symbols in it. This is a promising direction, but for now quite limited. It does not support Windows, supports only int/pointer args & returns  (no structs by value) and by design cannot check most Rust invariants: Memory shared with C stops being tracked for init and provenance. On Linux there is even more experimental support for native tracing that attempts to trace which bytes C touched and keep tracking for all others, but most of this is not ready for action quite yet.

## Head to the exercise

Head to the exercise in `exercises/02_intermediate/06_miri`. It contains a small
Rust wrapper around `tag_normalize` from `tag.c`, a module we haven't ported
yet. Its tests pass with a plain `cargo test` but fail under miri.

[^1]: the [`-Zmiri-many-seeds`](https://github.com/rust-lang/miri/#testing-multiple-different-executions) option can be used to execute multiple different executions at the same time which helps with catching flakes more often. Miri also has the `-Zmiri-address-reuse-rate`, `-Zmiri-address-reuse-cross-thread-rate`, `-Zmiri-preemption-rate`, and `-Zmiri-compare-exchange-weak-failure-rate` options that help with catching threading bugs more often. But generally its better to turn to tools like [`loom`](https://github.com/tokio-rs/loom) for proper concurrency testing.
