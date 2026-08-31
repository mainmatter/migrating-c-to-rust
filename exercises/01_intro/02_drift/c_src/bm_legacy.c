// The Rust binding in `src/lib.rs` always declares `bm_add` as
//
//     int bm_add(int a, int b);
//
// Let's see what happens with different C definitions for the same symbol!
// Uncomment one definition at a time and then run either `cargo test` in
// this folder or `wr --recheck` from the root of the repository to see
// what happens.
//
// The first compilation should fail with a linker error, since no definition
// exists for `bm_add` at link time.

// // A definition aligned with the Rust one. Tests should pass.
// int bm_add(int a, int b) { return a + b; }

// // Drifted! It interprets the arguments (and result) as floats rather than
// // integers.
// float bm_add(float a, float b) { return a + b; }

// // Drifted: one parameter instead of two.
// // The second argument is silently ignored!
// int bm_add(int a) { return a; }

// // Drifted: 64-bit addition instead of 32-bit.
// // This one *may still work* on your machine!
// long long bm_add(long long a, long long b) { return a + b; }
