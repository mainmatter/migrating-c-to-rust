# Chapter 3: Rewriting a C module

This chapter moves from the FFI building blocks to rewriting C code in Rust. We
will learn how to approach real-world codebases, structure our approach, and
debug our code when it breaks.

Each section has its own small exercise. Most of them borrow names from the same
imaginary program, a bookmark manager written in C, but each exercise stands on
its own.

In this chapter we'll pick our first module to rewrite, learn how C and Rust
allocators (don't) mix, debug across the language boundary with `lldb`/`gdb`,
harden our `unsafe` code with Clippy lints, replace C-isms with Rust-isms, catch
undefined behavior with Miri, and benchmark the port against the C baseline.

## Exercises

The exercises for this chapter are located in `exercises/03_intermediate`
