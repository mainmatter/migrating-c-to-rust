//! Setup check: are the FFI tools for this chapter installed?
//!
//! There is nothing to change in this exercise. It exercises the two tools this
//! chapter introduces, in both directions across the boundary:
//!
//! - `build.rs` runs `bindgen` (which needs libclang) to generate Rust bindings
//!   for a C function, and the unit test below calls it.
//! - `tests/cheadergen.rs` runs the `cheadergen` CLI on this crate and checks
//!   that the generated C header declares [`setup_check_double`].
//!
//! If both tests pass, you're ready for the chapter. If either fails, its
//! message tells you what's missing; the course's welcome page lists the
//! requirements and how to install them.

// The bindings are only used by the tests, so non-test builds see them as dead.
#[allow(dead_code)]
mod sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

/// Doubles `x`. Exported to C so that `cheadergen` has something to declare.
#[unsafe(no_mangle)]
pub extern "C" fn setup_check_double(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod tests {
    use super::sys::setup_check_add;

    #[test]
    fn calls_c_through_bindgen() {
        // SAFETY: `setup_check_add` only adds two integers.
        assert_eq!(unsafe { setup_check_add(2, 3) }, 5);
    }
}
