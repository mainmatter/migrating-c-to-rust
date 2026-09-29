// A C library over single bytes, declared in `c_src/chars.h`:
//
//     char to_lower(char byte);
//     int  is_letter(char byte);
//     void lower_bytes(char *bytes, size_t len);
//
// The bindings are already correct. The three wrappers below are the job: they
// speak `u8` and `bool`, so every value converts on its way across.
//
// `wr` type-checks this crate for two targets: `c_char` is `i8` on x86_64-unknown-linux-gnu, and `u8` on
// aarch64-unknown-linux-gnu so this will trigger when not properly casting to `c_char`.
use std::ffi::{c_char, c_int};

unsafe extern "C" {
    pub fn to_lower(byte: c_char) -> c_char;
    pub fn is_letter(byte: c_char) -> c_int;
    pub fn lower_bytes(bytes: *mut c_char, len: usize);
}

/// Lowercases an ASCII letter, and returns any other byte unchanged.
pub fn lower(byte: u8) -> u8 {
    // A colleague wrote this: the compiler asked for an `i8`, they gave it one,
    // the tests went green. See what happens when you remove the `as i8` cast.
    //
    // TODO: make it build for both targets.
    unsafe { to_lower(byte as i8) as u8 }
}

/// Returns whether `byte` is an ASCII letter.
pub fn is_ascii_letter(byte: u8) -> bool {
    // TODO: call `is_letter`, with the same care. C answers yes-or-no questions
    // with an `int`.
    todo!()
}

/// Lowercases the ASCII letters in `bytes`, in place.
pub fn lower_all(bytes: &mut [u8]) {
    // TODO: call `lower_bytes`. The slice stays a `[u8]`; only the pointer
    // changes type. `pointer::cast` takes that type from the signature, so it
    // can't pick the wrong one.
    todo!()
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

    #[test]
    fn lowercases_a_slice_in_place() {
        let mut bytes = *b"Ferris\xC3\xA9";

        lower_all(&mut bytes);

        assert_eq!(&bytes, b"ferris\xC3\xA9");
    }
}
