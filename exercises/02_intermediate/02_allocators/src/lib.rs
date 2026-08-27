//! # Exercise: crossing the FFI boundary with owned allocations
//!
//! This exercise covers the first two strategies from the allocator chapter.
//! You only need to edit these two files:
//!
//! 1. `src/use_rust_allocator_from_c.rs`
//! 2. `src/use_c_allocator_from_rust.rs`
//!
//! Each file contains exactly two numbered `TODO`s. Replace those four
//! `todo!()` calls without changing the function signatures.
//!
//! Everything in `src/given.rs`, `src/tests.rs`, `c_src`, and `c_test` is
//! provided support or verification code. You are welcome to read it, but you
//! should not change it.
//!
//! Run `wr` from the repository root to check your solution.

mod given;

// START HERE: implement the two FFI strategies in the following modules.
mod use_c_allocator_from_rust;
mod use_rust_allocator_from_c;

#[cfg(test)]
mod tests;
