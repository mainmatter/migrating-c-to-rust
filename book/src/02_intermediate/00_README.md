# Chapter 2: Intermediate

> Note: the following chapters are **not yet publicly available**!
>
> we're working hard on publishing the rest when we can. If you want early
> access, reach out!

From now on we will be porting a "real" C application: the `bm` bookmark manager
CLI. We will learn how to approach real-world codebases, structure our approach,
and debug our code when it breaks.

In this chapter we'll pick our first module to rewrite, learn how C and Rust
allocators (don't) mix, debug across the language boundary with `lldb`/`gdb`,
harden our `unsafe` code with clippy lints, benchmark the port against the C
baseline, and bind to C's global variables from Rust.

## Exercises

The exercises for this section are located in `exercises/02_intermediate`
