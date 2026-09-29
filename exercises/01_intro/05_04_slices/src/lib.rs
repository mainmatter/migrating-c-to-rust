// Two functions over a pointer and a length, the way C passes arrays around:
//
//     int32_t sum(const int32_t *items, size_t len);
//     bool max(const int32_t *items, size_t len, int32_t *out_max);
//
// The Rust versions below keep C's arguments, but `max` returns `Option<i32>`
// instead of writing through an out-parameter. Replacing out-parameters with
// return values is section 2.5.1's subject; here the job is the pointer and the
// length.
//
// Both callers in the test suite pass `NULL` with a length of 0 to say "no
// items", which is normal in C and is exactly the case `slice::from_raw_parts`
// does not allow. Handle it before you build a slice.
//
// If a first attempt ends in `unsafe precondition(s) violated` and takes the
// whole test run down with it, that is this rule being enforced at runtime: a
// slice's pointer may not be null even when its length is 0.

/// Returns the sum of the `len` `i32`s starting at `items`, or 0 if there are
/// none.
///
/// # Safety
///
/// If `len` is non-zero, `items` must be non-null, aligned, and point to `len`
/// initialized `i32`s in a single allocation that stays alive and unmodified
/// for the duration of the call.
pub unsafe fn sum(items: *const i32, len: usize) -> i32 {
    // TODO: turn `items` and `len` into a `&[i32]`, then sum it.
    todo!()
}

/// Returns the largest of the `len` `i32`s starting at `items`, or `None` if
/// there are none.
///
/// # Safety
///
/// The same as [`sum`].
pub unsafe fn max(items: *const i32, len: usize) -> Option<i32> {
    // TODO: same idea as `sum`.
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{max, sum};

    #[test]
    fn sums_the_items() {
        let items = [1, 2, 3, -4];

        let total = unsafe { sum(items.as_ptr(), items.len()) };

        assert_eq!(total, 2);
    }

    #[test]
    fn finds_the_largest_item() {
        let items = [1, 9, -4];

        let largest = unsafe { max(items.as_ptr(), items.len()) };

        assert_eq!(largest, Some(9));
    }

    #[test]
    fn handles_a_null_pointer_with_zero_length() {
        assert_eq!(unsafe { sum(std::ptr::null(), 0) }, 0);
        assert_eq!(unsafe { max(std::ptr::null(), 0) }, None);
    }

    #[test]
    fn handles_a_valid_pointer_with_zero_length() {
        let items: [i32; 0] = [];

        assert_eq!(unsafe { sum(items.as_ptr(), 0) }, 0);
        assert_eq!(unsafe { max(items.as_ptr(), 0) }, None);
    }
}
