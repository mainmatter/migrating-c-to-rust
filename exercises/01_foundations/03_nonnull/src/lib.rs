#[derive(Debug, PartialEq, Eq)]
pub struct Counter {
    pub value: u32,
}

/// Reads the counter's value, or returns `None` if there is no counter.
///
/// # Safety
///
/// `counter` must be null, or point to a valid `Counter`.
pub unsafe fn counter_value(counter: *mut Counter) -> Option<u32> {
    std::ptr::NonNull::new(counter).map(|counter| {
        // SAFETY: `counter` is non-null, and the caller guarantees a non-null
        // pointer points to a valid `Counter`.
        unsafe { counter.as_ref() }.value
    })

}

#[cfg(test)]
mod tests {
    use super::{Counter, counter_value};

    #[test]
    fn reads_the_value() {
        let mut counter = Counter { value: 7 };

        assert_eq!(unsafe { counter_value(&mut counter) }, Some(7));
    }

    #[test]
    fn a_counter_holding_zero_still_has_a_value() {
        let mut counter = Counter { value: 0 };

        assert_eq!(unsafe { counter_value(&mut counter) }, Some(0));
    }

    #[test]
    fn a_null_pointer_has_no_value() {
        assert_eq!(unsafe { counter_value(std::ptr::null_mut()) }, None);
    }
}
