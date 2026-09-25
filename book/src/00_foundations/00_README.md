# Chapter 0: Unsafe Rust foundations

Calling C from Rust, or Rust from C, means handing raw memory from one language
to the other. Neither compiler can check what happens on the other side, so it
falls to you to know what the language guarantees about memory, what it assumes,
and what happens when those assumptions don't hold.

In this chapter we'll cover those building blocks:

- what `unsafe` means and what it does and doesn't change.
- how raw pointers differ from references and what each one guarantees.
- how `NonNull` and `Option` let the type system track null pointers.
- how size, alignment and padding work, and what `repr` is for.
- how to describe memory that isn't a valid value yet or that changes behind
  your back.

## Exercises

The exercises for this section are located in `exercises/00_foundations`
