// A C library over single bytes, declared in `c_src/chars.h`:
//
//     char to_lower(char byte);
//     int  is_letter(char byte);

use std::ffi::{c_char, c_int};

// The bindings are already correct, no need to change them.
unsafe extern "C" {
    pub fn to_lower(byte: c_char) -> c_char;
    pub fn is_letter(byte: c_char) -> c_int;
}

/// Lowercases an ASCII letter, and returns any other byte unchanged.
pub fn lower(byte: u8) -> u8 {
    // This code was written to compile on another platform. See what happens
    // when you remove the `as i8` cast.
    //
    // TODO: make it build for both targets.
    // SAFETY: `to_lower` accepts any `char` value and has no other requirements.
    unsafe { to_lower(byte as c_char) as u8 }

}

/// Returns whether `byte` is an ASCII letter.
pub fn is_ascii_letter(byte: u8) -> bool {
    // SAFETY: `is_letter` accepts any `char` value and has no other requirements.
    unsafe { is_letter(byte as c_char) != 0 }

}

#[cfg(test)]
mod tests {
    use super::*;

    // The C side, called the way C would call it.
    #[test]
    fn lowercases_a_c_char() {
        let byte: c_char = 0x41; // 'A'

        let lowered = unsafe { to_lower(byte) };

        assert_eq!(lowered, 0x61); // 'a'
    }

    #[test]
    fn lowercases_a_byte() {
        assert_eq!(lower(b'A'), b'a');
        assert_eq!(lower(b'a'), b'a');
        assert_eq!(lower(b'7'), b'7');
    }

    #[test]
    fn leaves_a_byte_above_127_alone() {
        // 0xC3 is the first byte of "é" in UTF-8. It is not a letter as far as
        // C is concerned, and it is negative when read as a signed `char`.
        assert_eq!(lower(0xC3), 0xC3);
        assert!(!is_ascii_letter(0xC3));
    }

    #[test]
    fn recognizes_ascii_letters() {
        assert!(is_ascii_letter(b'q'));
        assert!(is_ascii_letter(b'Q'));
        assert!(!is_ascii_letter(b'7'));
    }
}
