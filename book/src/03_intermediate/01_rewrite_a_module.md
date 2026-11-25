# How to rewrite a module

In the previous chapter you learned the FFI building blocks: `extern` blocks,
generated bindings, FFI-safe types, and how to design a boundary that acts as a
firewall. Now we'll put them to work on a real codebase. Meet `bm`, a small
bookmark manager CLI written in C.

## Take `bm` for a spin

Before porting any of it, build `bm` and use it for a minute. It's easier to
reason about a module once you've seen what the program does with it:

```sh
cd exercises/_bm
make
./bm
```

Without arguments, `bm` prints its usage. Add a couple of bookmarks, then look
them up again:

```sh
./bm add https://www.rust-lang.org/ --tag rust --tag lang
./bm add https://en.cppreference.com/w/c --tag c --tag reference
./bm find rust
./bm list
```

`find` prints matching bookmarks, one per line, and `list` prints how many
bookmarks carry each tag. Your bookmarks live in `bookmarks.bm` in the current
directory. The `BM_DB` environment variable points `bm` at a different database.
`fixtures/sample.bm` has about 50 entries to explore:

```sh
BM_DB=fixtures/sample.bm ./bm find blog
BM_DB=fixtures/sample.bm ./bm list
```

`fixtures/corrupt.bm` shows the error path instead. Its fourth line is the first
URL again with a different case, so loading stops at a duplicate and `bm`
refuses to open the database:

```sh
BM_DB=fixtures/corrupt.bm ./bm list
```

Stick to `find` and `list` on the fixtures: `add`, `del`, and `tag` rewrite the
database they're pointed at. `make clean` removes the binary and your own
`bookmarks.bm`.

## The source code

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

We'll start with `normalize`. It has no project dependencies, and both of its
functions write into a buffer the caller supplies, so no memory changes owner at
the boundary. You have also ported half of it already: `bm_normalize_url` is the
function you made safe in exercise 2.11.

## Preserve the existing contract

When possible, an incremental migration replaces the module's _object file_
without changing the interface used by the remaining C code. The existing C
header describes the ABI our Rust implementation must initially satisfy.
`normalize.h` declares two functions:

```c
BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);
BmResult tag_normalize(const char *raw, char *out, size_t out_len);
```

We drop all of `normalize.o` from the link, so the Rust replacement must export
every function it provided, under the same symbol names and with the C ABI:

```rust,no_run
use std::ffi::c_char;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tag_normalize(
    raw: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> i32 {
    todo!()
}
```

Leave one out and `bm` no longer links. The rest of the C code doesn't know, and
doesn't need to know, that the implementation behind the symbols changed. The
linker resolves the same symbol names as before; it just finds them in our Rust
static library instead of the old object file.

That is a migration constraint, not necessarily the interface we want in the
long term. Some C headers expose globals, macros, shared data structures, or
ownership assumptions that cannot be reproduced cleanly in Rust. In those cases,
we can introduce a small compatibility layer and improve the interface
separately.

## Reuse what you already ported

`bm_normalize_url` already exists in Rust: it's your solution to exercise 2.11.
Copying it would leave two versions to keep in sync, so the exercise crate
depends on the 2.11 crate instead:

```toml
[dependencies]
bm_high_quality_ffi_apis = { path = "../../02_basics/11_high_quality_ffi_apis" }
```

Its `#[unsafe(no_mangle)]` function only ends up in your static library if your
code uses the crate. Cargo hands every dependency to `rustc`, and `rustc` only
links the ones the code refers to. A `use` is enough:

```rust,ignore
pub use bm_high_quality_ffi_apis::bm_normalize_url;
```

Your version is stricter than the C one: it rejects characters such as spaces
and double quotes, which C accepted. That is a change in behavior: a database
that the C build saved with such a URL no longer opens. The only such URL in the
fixtures comes after the line where `corrupt.bm` stops loading, so the
comparison below still passes. In a real migration you'd make that call on
purpose, and tell your users.

`tag_normalize` is new. Port it as it is, quirks included: the goal of this step
is a drop-in replacement. Improvements come after the swap, one reviewed change
at a time.

## Link it into `bm`

Cargo must first produce the right kind of artifact:

```toml
[lib]
crate-type = ["staticlib"]
```

Then, from the repository root, build `bm` from every C file except
`normalize.c`, with the flags of its `Makefile`, and add the archive in its
place:

```sh
cargo build -p bm_normalize
cd exercises/_bm
cc -Wall -Wextra -Werror -O0 -g -std=c11 -Isrc \
  src/bookmark.c src/cli.c src/index.c src/storage.c src/tag.c src/util.c \
  ../../target/debug/libbm_normalize.a -o ../../target/bm-rust
```

```text
before:  cli.o, index.o, ..., normalize.o (C)
after:   cli.o, index.o, ..., libbm_normalize.a (Rust)
```

## Verifying behavior parity

A rewrite must not change observable behavior. With both builds of `bm` at hand,
run them on the same input and compare what they print:

```sh
make
diff <(BM_DB=fixtures/sample.bm ./bm list) \
     <(BM_DB=fixtures/sample.bm ../../target/bm-rust list)
```

No output means no difference. Comparing two implementations on the same inputs
is called differential testing. Pick the commands with care: `list` only reaches
`bm_normalize_url`, because loading a database normalizes every URL. Tags come
through `tag_normalize` only in `add --tag` and `tag`.

The exercise's tests do this for you. They build both versions of `bm`, run
`find`, `list`, `add --tag`, and `tag` on copies of the fixtures, and compare
stdout, stderr, exit codes, and the database each one leaves behind.

## Tips

If the linker complains about an undefined symbol, inspect the symbols your Rust
static library exports. The command depends on your toolchain and platform:

- Unix-like systems: `nm target/debug/lib<crate>.a`
- LLVM toolchains: `llvm-nm target/debug/lib<crate>.a`
- Windows with MSVC: `dumpbin /symbols target\debug\<crate>.lib`

A missing `#[unsafe(no_mangle)]` is a common cause. So is a dependency that your
code never uses. On macOS, symbol names start with an underscore, so look for
`_tag_normalize`.

## Head to the exercise

The exercise in `exercises/03_intermediate/01_rewrite_a_module` replaces
`normalize.c`. Configure the crate to produce a static library, bring in
`bm_normalize_url` from exercise 2.11, and port `tag_normalize`. The tests then
link the real `bm` against your library and compare it with the all-C build.

The exercise links your solution to exercise 2.11 as it is in your copy of the
repository, so solve that one first. If you change it later, this exercise
checks the new version.
