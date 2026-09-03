use core::ffi::c_int;

unsafe extern "C" {
    // TODO: add an entry to this `extern "C" {}` block to allow our tests
    // below to invoke the `bm_add` C function defined in `c_src/bm_legacy.*`.
    pub fn bm_add(a: c_int, b: c_int) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::bm_add;

    #[test]
    fn test_add() {
        assert_eq!(unsafe { bm_add(2, 3) }, 5);
        assert_eq!(unsafe { bm_add(0, 0) }, 0);
    }
}
