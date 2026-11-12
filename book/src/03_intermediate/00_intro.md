# Chapter 3: Intermediate

From now on we will be porting a "real" C application: the `bm` bookmark manager
CLI. We will learn how to approach real-world codebases, structure our approach,
and debug our code when it breaks.

In this chapter we'll pick our first module to rewrite, learn how C and Rust
allocators (don't) mix, debug across the language boundary with `lldb`/`gdb`,
harden our `unsafe` code with Clippy lints, replace C-isms with Rust-isms, catch
undefined behavior with Miri, and benchmark the port against the C baseline.

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

The exercise for this page asks you two questions only `bm` can answer.
