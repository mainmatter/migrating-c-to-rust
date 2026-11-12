# Chapter 1: Unsafe Rust foundations

Safe Rust comes with a guarantee: if the program compiles, it has no undefined
behavior. The compiler enforces the rules for references, valid values, and
mutation that make that guarantee possible. Those checks stop at the language
boundary, though. Rust cannot borrow-check C, and C does not enforce Rust's
invariants. It falls to you to know what each language guarantees about memory,
what it assumes, and what happens when those assumptions don't hold.

This chapter covers the building blocks: what `unsafe` means and what it does
and doesn't change, how raw pointers differ from references, how `NonNull` and
`Option` let the type system track null pointers, and how size, alignment, and
padding work.

## Check your setup

Later chapters compile C code alongside Rust, so this page's exercise makes sure
that works on your machine before you need it. It compiles a tiny C file, links
it into a Rust crate, and calls it from a test. There's nothing to write: if the
test passes, you're ready. If it fails, revisit the requirements in the
[welcome page](../welcome.md).
