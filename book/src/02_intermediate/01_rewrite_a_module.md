# How to rewrite a module

In the previous chapter you learned the FFI building blocks: `extern` blocks,
generated bindings, FFI-safe types, and how to design a boundary that acts as a
firewall. Now we'll put them to work on a real codebase. Meet `bm`, a small
bookmark manager CLI written in C. Over the rest of this course we will migrate
it to Rust, one module at a time.

Here is an overview of the C source code:

```text
exercises/_bm/src/
├── bookmark.c    the Bookmark type
├── bookmark.h
├── cli.c         entry point, argument parsing
├── index.c       in-memory bookmark index
├── index.h
├── internal.h    definitions shared by index.c and storage.c
├── normalize.c   URL and tag normalization
├── normalize.h
├── result.h      shared result codes and error-string helper
├── storage.c     on-disk persistence of .bm files
├── storage.h
├── tag.c         tag parsing and matching
├── tag.h
├── util.c        string helpers
└── util.h
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
cli       → bookmark, index, normalize, result, tag, util
index     → bookmark, internal, normalize, storage, tag, util
storage   → bookmark, index, internal, normalize, util
tag       → normalize, util
bookmark  → util
normalize → (no project dependencies)
util      → (no project dependencies)
```

Two things are worth noting. `normalize` and `util` are the only real leaves:
every other module pulls in at least one of its siblings. And `index` and
`storage` include each other, so this is not the clean tree the file names
suggest. Tangles like that are normal in code that has been maintained for a
while, and they are exactly what you want to know before choosing where to
start.

The exercise uses an even smaller boundary than these production modules: a
one-function `bm_version` module. Its behavior is intentionally trivial so that
the work is limited to replacing a C object file with a Rust static library.
Later exercises add pointers, allocation, error handling, and API redesign.

## Preserve the existing contract

When possible, an incremental migration replaces the module's _object file_
without changing the interface used by the remaining C code. The existing C
header describes the ABI our Rust implementation must initially satisfy. The
exercise's header contains this declaration:

```c
#include <stdint.h>

uint32_t bm_version(void);
```

The Rust replacement must export the same function under the same symbol name
and use the C ABI:

```rust,no_run
#[unsafe(no_mangle)]
pub extern "C" fn bm_version() -> u32 {
    1
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
in Rust instead of the original one written in C. Cargo must first produce the
right kind of artifact:

```toml
[lib]
crate-type = ["staticlib"]
```

```text
before:  test_bm_version.o + bm_version.o (C)
after:   test_bm_version.o + libbm_version.a (Rust)
```

## Structuring the Rust side

For a substantial module, keep the two layers from the firewall pattern in
section 1.6: a thin `extern "C"` surface that converts C representations and a
safe Rust core that implements the behavior. `bm_version` takes no arguments and
returns a fixed-width integer, so in this case the exported function can remain
the entire implementation.

## Verifying behavior parity

A rewrite is only done when the observable behavior is unchanged. The exercise
uses the same C caller twice:

1. first it links the caller with `bm_version.c` and runs the all-C baseline;
2. then it omits `bm_version.c`, links the unchanged caller with the Rust
   archive, and runs it again.

A Rust unit test independently checks the function's return value. The C link is
what proves that the archive exposes the ABI and symbol promised by the header.

## Tips

If the linker complains about an undefined symbol, inspect the symbols your Rust
static library exports. The command depends on your toolchain and platform:

- Unix-like systems: `nm target/debug/lib<crate>.a`
- LLVM toolchains: `llvm-nm target/debug/lib<crate>.a`
- Windows with MSVC: `dumpbin /symbols target\debug\<crate>.lib`

A missing `#[unsafe(no_mangle)]` is a common cause.

While both implementations still exist, you can also run the same inputs through
the C and Rust versions and compare their results. This is called differential
testing. The exercise uses the simpler form of running the same assertions
against both implementations.

## Head to the exercise

The exercise in `exercises/02_intermediate/01_rewrite_a_module` replaces
`bm_version.c`. Configure the crate to produce a static library, then make the
provided Rust function satisfy `bm_version.h` without changing the header or the
C caller. There is no algorithm to port: the exercise focuses on the build
artifact, ABI, symbol name, and link step.
