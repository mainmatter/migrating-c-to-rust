//! Setup check: is a C compiler available?
//!
//! There is nothing to change in this exercise. `build.rs` compiles a tiny C
//! file and links it into this crate, and the test below calls into it. If the
//! test passes, your C toolchain works and you are ready for the chapter.
//!
//! If it fails, revisit the requirements in the course's welcome page. Don't
//! worry about the `extern` block and the `unsafe` call below yet: chapter 2
//! covers both in detail.

unsafe extern "C" {
    fn setup_check_answer() -> i32;
}

/// Returns the value computed by the C side.
pub fn answer() -> i32 {
    // SAFETY: `setup_check_answer` takes no arguments, touches no memory and
    // always returns a plain `int`.
    unsafe { setup_check_answer() }
}

#[cfg(test)]
mod tests {
    use super::answer;

    #[test]
    fn calls_into_c() {
        assert_eq!(answer(), 42);
    }
}
