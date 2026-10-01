// Part one: work the numbers out on paper, then write them down below. The
// tests compare each one against what the compiler reports, so a wrong answer
// tells you which type to look at again.

#[repr(C)]
pub struct Pixel {
    pub alpha: u8,
    pub color: u32,
    pub layer: u16,
}

// TODO: replace each 0 with the right number.
pub const BOOL_SIZE: usize = 0;
pub const BOOL_ALIGN: usize = 0;
pub const U64_ARRAY_SIZE: usize = 0; // for `[u64; 3]`
pub const PIXEL_SIZE: usize = 0;
pub const PIXEL_ALIGN: usize = 0;

// Part two: `Header` follows C's rules, so its fields stay in the order you
// write them. It holds 14 bytes of fields and takes up 24, and a better order
// gets that down to 16.
//
// TODO: reorder the fields so the struct is as small as those rules allow.
// Keep every field, keep `repr(C)`, and don't change any types.
#[repr(C)]
pub struct Header {
    pub flag: bool,
    pub id: u64,
    pub kind: u8,
    pub len: u32,
}

#[cfg(test)]
mod tests {
    use super::{BOOL_ALIGN, BOOL_SIZE, Header, PIXEL_ALIGN, PIXEL_SIZE, Pixel, U64_ARRAY_SIZE};

    #[test]
    fn sizes_and_alignments_are_right() {
        assert_eq!(BOOL_SIZE, size_of::<bool>(), "size of bool");
        assert_eq!(BOOL_ALIGN, align_of::<bool>(), "alignment of bool");
        assert_eq!(U64_ARRAY_SIZE, size_of::<[u64; 3]>(), "size of [u64; 3]");
        assert_eq!(PIXEL_SIZE, size_of::<Pixel>(), "size of Pixel");
        assert_eq!(PIXEL_ALIGN, align_of::<Pixel>(), "alignment of Pixel");
    }

    #[test]
    fn header_wastes_no_space() {
        assert_eq!(align_of::<Header>(), 8);
        assert_eq!(size_of::<Header>(), 16);
    }

    #[test]
    fn header_still_has_every_field() {
        let header = Header {
            flag: true,
            id: 7,
            kind: 2,
            len: 512,
        };

        assert!(header.flag);
        assert_eq!(header.id, 7);
        assert_eq!(header.kind, 2);
        assert_eq!(header.len, 512);
    }
}
