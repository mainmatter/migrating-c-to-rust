/// Returns the last byte of `bytes`, or `None` if `bytes` is empty.
pub fn last_byte(bytes: &[u8]) -> Option<u8> {
    if bytes.is_empty() {
        return None;
    }
    // SAFETY: `bytes` is not empty, so `bytes.len() - 1` is in bounds.
    Some(unsafe { *bytes.get_unchecked(bytes.len() - 1) })
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
