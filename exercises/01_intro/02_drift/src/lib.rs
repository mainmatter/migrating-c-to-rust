use core::ffi::c_int;

unsafe extern "C" {
    // This is the hand-written binding. As far as Rust is concerned, *this* is
    // the signature of `bm_add` -- and it never changes in this exercise.
    fn bm_add(a: c_int, b: c_int) -> c_int;
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
