// The Rust binding in `src/lib.rs` always declares `bm_add` as
//
//     int bm_add(int a, int b);
//
// Comment out the correct definition and uncomment one of the drifted ones
// below. Rerun `cargo test` after each swap and see what happens on your
// machine.

// the correct version -- the test passes
int bm_add(int a, int b) { return a + b; }

// // drifted: reinterprets the arguments (and result) as floats
// float bm_add(float a, float b) { return a + b; }

// // drifted: one parameter too few -- the second argument is silently ignored
// int bm_add(int a) { return a; }

// // drifted: 64-bit addition instead of 32-bit. Notice this one *still passes*
// // on your machine probably!
// long long bm_add(long long a, long long b) { return a + b; }
