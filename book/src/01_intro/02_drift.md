# When hand-written bindings drift

In the previous exercise you wrote an `extern "C"` block binding to C code. It
worked great, but can you imagine doing that for _hundreds_, maybe _thousands_,
of functions and types? There's a subtler problem, too. Hand-written bindings
will _inevitably_ drift out of sync with the C header, especially at scale.

Neither the C nor the Rust compiler can detect this, because each operates in
its own small universe, called a _compilation unit_. A compilation unit is the
single chunk of work that flows through the various stages of the compiler. In
C/C++ this would typically be a `.c/.cpp` file and in Rust that is typically a
single crate. The compiler parses, typechecks, and optimizes the compilation
unit, then produces an _intermediate_ object file. Once all the compilation
units that make up your project are built, the intermediate object files are
gathered and passed to the _linker_, which produces the final executable or
library.

```text
┌─────────────────────┐          ┌─────────────────────┐
│   Rust source       │          │   C source          │
└──────────┬──────────┘          └──────────┬──────────┘
           │ rustc                          │ cc
           ▼                                ▼
┌───────────────────────┐        ┌───────────────────────┐
│ compile               │        │ compile               │
│ (parse → typecheck →  │        │ (parse → typecheck →  │
│  optimize  → codegen) │        │  optimize  → codegen) │
└──────────┬────────────┘        └──────────┬────────────┘
           │ object file                    │ object file
           └───────────────┬────────────────┘
                           ▼
                ┌─────────────────────┐
                │   link              │
                │   (ld / lld)        │
                └──────────┬──────────┘
                           ▼
                ┌─────────────────────┐
                │   final binary      │
                └─────────────────────┘
```

This model is great for compilation performance, because we can process many of
these compilation units _in parallel_. There is a catch: they _must not_ share
information, since that would destroy the parallelism. Each compilation _must_
be its own self-contained universe.

And even if we had a mechanism to share information between compilation units,
we would need to make it language-agnostic so that a C compilation unit and a
Rust compilation unit can interoperate. How would that even work with wildly
different type systems?[^1]

So compilers have resorted to _manual escape hatches_ like the `extern "C"`
block you wrote. You as the programmer promise to the compiler that a function
with a given name and a given signature will exist at link time, and the
compiler takes your word for it.

## Possible consequences

The consequences of drift between the two versions can be dire: if C expects
`int` but Rust expects `float`, the integer bit patterns are reinterpreted as
floats. The result is almost always nonsensical.

It can be worse: passing too many or too few arguments can result in either
overwriting important information on the stack or reading garbage from it.

Here is an especially insidious case. The function, as we have established, is
the following:

```c
int bm_add(int a, int b);
```

but now in the Rust code, we expect it to return 64-bit integers instead of
32-bit integers:

```rust
unsafe extern "C" {
    fn bm_add(a: c_longlong, b: c_longlong) -> c_longlong;
}
```

How will this example fail? If you try it yourself, you will notice that it
doesn't. The reason is that modern CPUs pass function arguments in registers and
these registers are always _word-sized_, meaning 64-bit on 64-bit machines. This
means that internally even 32-bit ints are passed as 64-bit integers instead.

You can see this in Compiler Explorer here:
[The correct version](https://godbolt.org/z/PzrM4a575) passes `a` and `b` in the
`a0` and `a1` registers (the `li a0, 1` and `li a1, 2` lines do the loading) and
passes the return value in the `a0` register. Those registers are 64-bit
registers though and so you can see the
[incorrect `c_longlong`-expecting version](https://godbolt.org/z/E97EeoEP6)
**just works** because the registers were 64-bit anyway.

Now, if your 64-bit numbers stay below the 32-bit max value, everything just
happens to work. But it is _very_ fragile. As you can see here,
[when we compile to a 32-bit target instead](https://godbolt.org/z/bGfocb43f),
everything breaks and we end up adding `0` to `a` instead.

This example is particularly scary, because for 99% of inputs and deployment
configurations the mistake is virtually consequence-free. The addition will
continue to work as expected. But as soon as the input is unusual, the
deployment target is different, _or you add code that will make the compiler
change the generated code even a bit_, this will be a bug that takes you weeks
to troubleshoot in the worst case.

## Head to the exercise

<!-- BRIDGE GAP (Jonas): the prose above frames drift as the hand-written *Rust
     binding* going stale. The exercise inverts this: the binding stays fixed
     and correct-looking while the *C implementation* drifts underneath it.
     Consider a sentence here noting the mismatch (and thus the bug) is
     identical either way -- doing it from the C side just keeps the Rust crate
     compiling so you can actually run it and watch the fallout. -->

Play with the different "drifted" C implementations of `bm_add` to see what
happens on your machine. Feel free to also play around a bit with the Compiler
Explorer playgrounds to see how different miscompilations manifest in the
generated assembly.

[^1]: Yes, the LLVM bitcode embedded by toolchains for LTO (Rust's
    `-Clto=thin -Cembed-bitcode=yes` and Clang's `-flto=thin`) _does_ carry the
    information to catch problems like this at link time and it _is_
    cross-language, but linkers do not generally validate it. (The `wasm-ld`
    linker does, but only for Wasm.) The reasons are many, the most important
    being that validating it would break existing compiler optimizations. You
    _could_ write custom LLVM-bitcode parsing tooling to check this if you
    wanted.
