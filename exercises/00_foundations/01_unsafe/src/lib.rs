/// Returns the last byte of `bytes`, or `None` if `bytes` is empty.
pub fn last_byte(bytes: &[u8]) -> Option<u8> {
    // TODO: return the last byte, reading it with `get_unchecked`.
    //
    // `get_unchecked` skips the bounds check that indexing does, which makes it
    // an `unsafe` function: you have to call it from an `unsafe` block, and it
    // is on you to pass an index that is in bounds.
    todo!()
}

#[cfg(test)]
mod tests {
    use super::last_byte;

    #[test]
    fn returns_the_last_byte() {
        assert_eq!(last_byte(&[1, 2, 3]), Some(3));
        assert_eq!(last_byte(&[7]), Some(7));
    }

    #[test]
    fn returns_none_for_an_empty_slice() {
        assert_eq!(last_byte(&[]), None);
    }
}
