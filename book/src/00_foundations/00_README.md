# Chapter 0: Unsafe Rust foundations

Safe Rust comes with a guarantee: if the program compiles, it has no undefined
behavior. The borrow checker and the type checker enforce the rules for
references, valid values, and mutation that make that guarantee possible. Those
checks stop at the language boundary, though: Rust cannot borrow-check C, and C
does not enforce Rust's invariants. It falls to you to know what each language
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
