# How to rewrite a module

In the previous chapter you learned the FFI building blocks: `extern` blocks,
generated bindings, FFI-safe types, and how to design a boundary that acts as a
firewall. Now we'll put them to work on a real codebase. Meet `bm`, a small
bookmark manager CLI written in C. Over the rest of this course we will migrate
it to Rust, one module at a time.

Here is an overview of the C source code:

```text
exercises/_bm/src/
├── bookmark.c   the Bookmark type
├── cli.c        entry point, argument parsing
├── index.c      in-memory bookmark index
├── normalize.c  URL and tag normalization
├── result.h     shared result codes and error-string helper
├── storage.c    on-disk persistence of .bm files
├── tag.c        tag parsing and matching
└── util.c       string helpers
```

## Picking the first module

There is no universally best place to start. A leaf module is often a good
choice because it has few dependencies. The program's entry point can also work
if it mostly calls other modules through simple interfaces. Sometimes the best
boundary does not match an existing source file at all, and it is worth
extracting a small module before porting it.

The right choice depends on the codebase, but a few guidelines help:

- Prefer a small, clearly defined interface.
- Minimize the number of C functions and data structures the Rust code must use.
- Avoid shared global state and complicated ownership rules in the first port.
- Choose code with useful tests, so you can compare behavior before and after.
- Keep the change small enough to review and, if necessary, revert.

```text
cli      → index, tag, normalize
index    → bookmark, storage, tag, normalize, util
storage  → bookmark, normalize, util
tag      → normalize, util
bookmark → util
normalize → (no project dependencies)
util      → (no project dependencies)
```

For `bm`, we chose to start with `normalize.c`. It exposes two functions: one
normalizes URLs and the other normalizes tags. Both write into buffers supplied
by the caller, so no memory changes ownership at the FFI boundary. Why not
`util.c` or `bookmark.c`? Both seem suitable too. `util.c` is also small, but it
is mostly thin wrappers around C string and allocation functions. `bookmark.c`
contains more substantial logic, but it allocates values that must later be
freed across the language boundary. We cover that problem in the next section.

## Preserve the existing contract

When possible, an incremental migration replaces the module's _object file_
without changing the interface used by the remaining C code. The existing C
header (`normalize.h` in this example) describes the ABI our Rust implementation
must initially satisfy. We re-implement every function the header declares in
Rust and export it under the same symbol name using `#[unsafe(no_mangle)]` and
`extern "C"`:

```rust,compile_fail
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bm_normalize_url(
    url: Option<NonNull<c_char>>,
    out: Option<NonNull<c_char>>,
    out_len: usize,
) -> BmResult {
    // ...
}
```

The rest of the C code doesn't know, and doesn't need to know, that the
implementation behind the symbol changed. The linker resolves the same symbol
names as before; it just finds them in our Rust static library instead of the
old object file.

That is a migration constraint, not necessarily the interface we want in the
long term. Some C headers expose globals, macros, shared data structures, or
ownership assumptions that cannot be reproduced cleanly in Rust. In those cases,
we can introduce a small compatibility layer and improve the interface
separately.

After that, all you have to do is link against the replacement library written
in Rust instead of the original one written in C.

```text
before:  cli.o, index.o, normalize.o (C), ...
after:   cli.o, index.o, libnormalize.a (Rust), ...
```

## Structuring the Rust side

Inside the crate we keep two layers, following the firewall pattern from Chapter
1.6:

1. a thin `extern "C"` surface that validates raw pointers and converts C types
   at the boundary, and
2. a safe, idiomatic core that does the actual work with `&str`, `String`, and
   `Result`.

The safe core is where all new logic lives, and it's plain Rust: unit-testable
with `cargo test`, no `unsafe` in sight. The FFI layer should stay boring.

## Verifying behavior parity

A rewrite is only done when the observable behavior is unchanged. We have two
safety nets:

- the module's existing C test suite, which now links against our Rust
  implementation and must keep passing, and
- new Rust unit tests against the safe core, which will outlive the C tests.

## Tips

If the linker complains about an undefined symbol, inspect the symbols your Rust
static library exports. The command depends on your toolchain and platform:

- Unix-like systems: `nm target/debug/lib<crate>.a`
- LLVM toolchains: `llvm-nm target/debug/lib<crate>.a`
- Windows with MSVC: `dumpbin /symbols target\debug\<crate>.lib`

A missing `#[unsafe(no_mangle)]` is a common cause.

While both implementations still exist, you can also run the same inputs through
the C and Rust versions and compare their return values and output buffers. This
is called differential testing. It catches small differences in behavior that
the existing tests may not cover.
