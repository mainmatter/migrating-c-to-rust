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

## Exercises

The exercises for this chapter are located in `exercises/01_foundations`.
