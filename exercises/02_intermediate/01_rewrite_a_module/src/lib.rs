//! Replace a tiny C module without changing its header or caller.
//!
//! There are two TODOs:
//!
//! 1. configure this crate as a `staticlib` in `Cargo.toml`;
//! 2. export `bm_version` with the ABI and symbol name promised by
//!    `c_test/bm_version.h`.
//!
//! `wr` first builds an all-C baseline from `bm_version.c` and the unchanged C
//! caller. It then omits `bm_version.c` and links that same caller against this
//! crate. The implementation is intentionally trivial: the exercise is about
//! replacing an object file, not porting an algorithm.

/// Return the version of the `bm` interface.
// TODO 2 OF 2: Use the C ABI and export the exact `bm_version` symbol.
pub fn bm_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_current_version() {
        assert_eq!(bm_version(), 1);
    }
}
