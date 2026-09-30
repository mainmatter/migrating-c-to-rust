# Chapter 0: Unsafe Rust foundations

Safe Rust comes with a guarantee: if the program compiles, it has no undefined
behavior. The compiler can only promise that because it checks the rules behind
it, such as where a reference may point, which values a type may hold, and who
may mutate what and when. None of those checks reach across a language boundary.
Rust cannot see what a C function does with a pointer, and C knows nothing about
the rules Rust expects to hold, so it falls to you to know what each language
guarantees about memory, what it assumes, and what happens when those
assumptions don't hold.

This chapter covers the building blocks:

- what `unsafe` means and what it does and doesn't change.
- how raw pointers differ from references and what each one guarantees.
- how `NonNull` and `Option` let the type system track null pointers.
- how size, alignment, and padding work, and what `repr` is for.
- how to describe memory that isn't a valid value yet or that changes behind
  your back.

## Exercises

The exercises for this chapter are located in `exercises/00_foundations`.
