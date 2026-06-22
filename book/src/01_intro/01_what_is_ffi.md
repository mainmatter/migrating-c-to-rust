# What Is FFI? How Do Languages Communicate?

Programming language design has taken many different paths over the years. We
have C-like languages that are compiled all the way down to machine code, but we
also have interpreted languages, lazy languages, languages that compile to a
portable bytecode, garbage collected languages, and so much more.

As software engineers we do not want to duplicate work unnecessarily. So what
happens when work exists in a language other than the one you're using?

When developing in - for example - Python, we don't want to port all the code we
need to Python just to use it. Instead, we rely on infrastructure that lets us
call functions and use types from other programming languages.

This infrastructure, which bridges different type systems (or lack of type
systems), execution models, and code-organization concepts, is called a _Foreign
Function Interface_ (abbreviated _FFI_ from here on out). Most modern languages
have some way to interoperate with others through FFI. The exact syntax and
options vary by language, but they almost always have one thing in common: they
represent functions as if they were C functions.

This is called the "C ABI" (C Application Binary Interface) or "C Calling
Convention" and as the name suggests is a convention on how to call functions
(which CPU registers hold what arguments, which register(s) hold return values,
when to spill to the stack, etc.)[^1]. Notice that this is just a convention
that the industry agreed on over time. The C ABI is predictable, simple, and has
been stable for decades and therefore became the de-facto standard for language
interoperability.

## Calling an FFI Function

Let's say for example we have a Rust program that needs to call the `time`
function from `libc` (a C static library)[^2]. We would use the following
construct:

```rust
unsafe extern "C" {
    fn time(time: *mut time_t) -> time_t
}
```

Before we dissect the syntax though: you may have already noticed that nowhere
in this snippet do we ask for `libc`. Why is that?

This is because object file formats (ELF on Linux, Mach-O on macOS, PE on
Windows) are all quite old and therefore quite simplistic. They have _one global
namespace_ called a _Symbol Table_ that all functions (and statics) share. So
you cannot say "call `time` from `libc`", you can only say "call a function
named `time`".

When a compiler builds a program, all function calls reference the function _by
symbol_ ("call function named `X`"). To make this actually executable, we need
to replace every symbol reference with the actual address of the function. This
happens _after_ compilation during the _linking_ step, where a separate program
called the _Linker_ will aggregate all object files that make up your final
program, lay them out on disk and then resolve these symbol names.

To call `time` from `libc` we therefore need Rust to emit a reference to the
`time` symbol and make sure the `libc` file is also passed to the linker.

This solves the problem of _what_ to call, but we still need to figure out _how_
to call that function: How many parameters does the function accept? What are
the types of these parameters? How many return values does it return? Remember
the symbol references above are plain string names. They do not carry any
information about the function's argument types or return types so Rustc has no
way of figuring out the types itself.

This is why we need to tell it about `time`'s signature through a so-called
"extern block" (or sometimes an "extern C block") above. This block _declares_
items that are not _defined_ in the current crate. Each item we declare is a
promise to the compiler: "this is the correct signature of this symbol, trust
me". We commonly refer to it as a **binding**.

Get the signature wrong and your Rust program will pass garbage to the FFI
function without any way to check this at compile-time. The exact implementation
won't be known until link-time, much later than the compiler's type-checking
pass. This is why bindings are marked `unsafe`: you as the programmer have to
ensure signatures are correct.

This is the foundation of all Foreign Function Interfaces in Rust. In later
exercises we will see how to make this much safer and more ergonomic.

## Head to the exercise

Head to the exercise, where you'll write this block for a `bm_add` function
implemented in C.

[^1]: Technically, there is no "single" calling convention. Every architecture
    defines its own "C Calling Convention". For example, the RISC-V C Calling
    Convention is defined
    [here](https://riscv.org/wp-content/uploads/2024/12/riscv-calling.pdf) and
    lays out the sizes of C primitives and how arguments and return values are
    passed from and to functions. x86 architectures have _many_ different
    calling conventions: the Microsoft x64 calling convention and the System V
    ABI are the most common, but many calling conventions exist to e.g. improve
    calling performance (`fastcall`, `regcall`, and more). When we say "the C
    calling convention" we usually mean "the C calling convention commonly used
    on this OS+architecture combination".

[^2]: Yes, generally speaking `libc` is distributed not as a static but as a
    _dynamic_ library which is a completely different way of linking and calling
    functions. We'll cover this in a later chapter in detail.
