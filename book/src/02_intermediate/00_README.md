# Chapter 2: Intermediate

From now on we will be porting a "real" C application: the `bm` bookmark manager
CLI. We will learn how to approach real-world codebases, structure our approach,
and debug our code when it breaks.

In this chapter we'll pick our first module to rewrite, learn how C and Rust
allocators (don't) mix, debug across the language boundary with `lldb`/`gdb`,
harden our `unsafe` code with Clippy lints, replace C-isms with Rust-isms, catch
undefined behavior with Miri, and benchmark the port against the C baseline.

## Exercises

The exercises for this section are located in `exercises/02_intermediate`
