/// Swaps the `i32`s that `a` and `b` point to.
///
/// # Safety
///
/// `a` and `b` must be non-null, aligned, and point to two different
/// initialized `i32`s that nothing else accesses during the call.
pub unsafe fn swap_values(a: *mut i32, b: *mut i32) {
    // TODO: swap the two values, reading and writing through the pointers.
    todo!()
}

#[cfg(test)]
mod tests {
    use super::swap_values;

    #[test]
    fn swaps_two_values() {
        let mut a = 1;
        let mut b = 2;

        unsafe { swap_values(&mut a, &mut b) };

        assert_eq!((a, b), (2, 1));
    }

    #[test]
    fn swapping_twice_restores_the_values() {
        let mut a = 10;
        let mut b = 20;

        unsafe { swap_values(&mut a, &mut b) };
        unsafe { swap_values(&mut a, &mut b) };

        assert_eq!((a, b), (10, 20));
    }
}
