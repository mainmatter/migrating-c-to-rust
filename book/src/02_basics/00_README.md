# Chapter 2: Basics

In this chapter we'll cover the FFI foundations: what FFI is, how we use it to
interoperate between C and Rust, and the common pitfalls.

We start with the mechanics of declaring and exporting functions, then work
through the kinds of data that cross the boundary: primitive types, structs,
enums and unions, slices, and opaque handles. Next we look at why hand-written
bindings drift out of sync with the C they describe, and at the tools that
generate them for you: `bindgen` for Rust bindings to C, and `cheadergen` for C
headers from Rust. The chapter ends with the rules we follow to make an FFI
boundary trustworthy: validating everything that comes in, and designing an API
that is small and documented well enough to keep correct.

## Exercises

The exercises for this chapter are located in `exercises/02_basics`.
