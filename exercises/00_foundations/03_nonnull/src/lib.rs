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
    // TODO: turn the raw pointer into an `Option<NonNull<Counter>>`, then read
    // the value only when there is a counter.
    todo!()
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
