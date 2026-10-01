// Two functions over a pointer and a length, the way C passes arrays around:
//
//     int32_t sum(const int32_t *items, size_t len);
//     void scale(int32_t *items, size_t len, int32_t factor);
//
// The tests pass `NULL` with a length of 0 to say "no items", which is normal
// in C and is exactly the case `slice::from_raw_parts` does not allow. Handle it
// before you build a slice.
//
// If a first attempt ends in `unsafe precondition(s) violated` and takes the
// whole test run down with it, that is this rule being enforced at runtime.

/// Returns the sum of the `len` `i32`s starting at `items`, or 0 if there are
/// none.
///
/// # Safety
///
/// If `len` is non-zero, `items` must be non-null, aligned, and point to `len`
/// initialized `i32`s that stay alive and unmodified for the duration of the
/// call.
pub unsafe fn sum(items: *const i32, len: usize) -> i32 {
    // TODO: turn `items` and `len` into a `&[i32]`, then sum it.
    todo!()
}

/// Multiplies each of the `len` `i32`s starting at `items` by `factor`, in
/// place.
///
/// # Safety
///
/// If `len` is non-zero, `items` must be non-null, aligned, and point to `len`
/// initialized `i32`s that stay alive for the duration of the call and that
/// nothing else accesses during it.
pub unsafe fn scale(items: *mut i32, len: usize, factor: i32) {
    // TODO: turn `items` and `len` into a `&mut [i32]`, then scale each item.
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{scale, sum};

    #[test]
    fn sums_the_items() {
        let items = [1, 2, 3, -4];

        let total = unsafe { sum(items.as_ptr(), items.len()) };

        assert_eq!(total, 2);
    }

    #[test]
    fn scales_the_items() {
        let mut items = [1, 9, -4];

        unsafe { scale(items.as_mut_ptr(), items.len(), 3) };

        assert_eq!(items, [3, 27, -12]);
    }

    #[test]
    fn handles_a_null_pointer_with_zero_length() {
        assert_eq!(unsafe { sum(std::ptr::null(), 0) }, 0);
        unsafe { scale(std::ptr::null_mut(), 0, 3) };
    }

    #[test]
    fn handles_a_valid_pointer_with_zero_length() {
        let mut items: [i32; 0] = [];

        assert_eq!(unsafe { sum(items.as_ptr(), 0) }, 0);
        unsafe { scale(items.as_mut_ptr(), 0, 3) };
    }
}
