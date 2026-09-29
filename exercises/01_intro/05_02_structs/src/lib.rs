// The types below have to match these C declarations byte for byte:
//
//     struct Record {
//         uint8_t  kind;
//         uint64_t id;
//         uint16_t flags;
//     };
//
//     #pragma pack(1)
//     struct PackedHeader {
//         uint8_t  kind;
//         uint32_t len;
//     };
//
//     /* A file descriptor, passed around as a plain int. */
//     typedef int Fd;
//     int close_fd(Fd fd);
//
// The fields are already written out for you. What's missing is the `repr`
// attribute on each type: the tests check the size, alignment, and field
// offsets a C compiler would produce.
use std::ffi::c_int;

// TODO: make `Record` follow C's layout rules.
pub struct Record {
    pub kind: u8,
    pub id: u64,
    pub flags: u16,
}

// TODO: `PackedHeader` is declared under `#pragma pack(1)`, so C gives it no
// padding at all.
pub struct PackedHeader {
    pub kind: u8,
    pub len: u32,
}

// TODO: `Fd` is a wrapper that C never sees: it passes a plain `int`. Give it
// the attribute that guarantees the same layout *and* the same function-call
// ABI as the field it wraps.
//
// `close_fd` below takes an `Fd`, so the crate won't build until `Fd` is a type
// C can accept. Note that the compiler can't tell the two candidate attributes
// apart here: it accepts either, and the ABI guarantee is the reason to prefer
// one of them.
pub struct Fd(pub c_int);

/// Closes a file descriptor. Here it only hands the number back.
#[unsafe(no_mangle)]
pub extern "C" fn close_fd(fd: Fd) -> c_int {
    fd.0
}

#[cfg(test)]
mod tests {
    use super::{Fd, PackedHeader, Record};
    use std::ffi::c_int;
    use std::mem::offset_of;

    #[test]
    fn record_matches_the_c_layout() {
        assert_eq!(offset_of!(Record, kind), 0);
        assert_eq!(offset_of!(Record, id), 8);
        assert_eq!(offset_of!(Record, flags), 16);
        assert_eq!(align_of::<Record>(), 8);
        assert_eq!(size_of::<Record>(), 24);
    }

    #[test]
    fn packed_header_has_no_padding() {
        assert_eq!(offset_of!(PackedHeader, kind), 0);
        assert_eq!(offset_of!(PackedHeader, len), 1);
        assert_eq!(align_of::<PackedHeader>(), 1);
        assert_eq!(size_of::<PackedHeader>(), 5);
    }

    #[test]
    fn fd_is_just_an_int() {
        assert_eq!(size_of::<Fd>(), size_of::<c_int>());
        assert_eq!(align_of::<Fd>(), align_of::<c_int>());
    }
}
