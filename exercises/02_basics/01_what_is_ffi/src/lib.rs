use core::ffi::c_int;

unsafe extern "C" {
    // Declares the `bm_add` C function defined in `c_src/bm_legacy.*`, so the
    // tests below can call it.
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
