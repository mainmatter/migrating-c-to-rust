// Write Rust types that match these C declarations from `c_src/shapes.h` byte
// for byte:
//
//     struct Point {
//         double x;
//         double y;
//     };
//
//     struct Shape {
//         uint8_t kind;
//         char name[16];
//         struct Point origin;
//         const struct Shape *parent; /* may be NULL */
//         uint32_t id;
//     };
//
//     #pragma pack(1)
//     struct WireHeader {
//         uint8_t version;
//         uint32_t len;
//     };
//     #pragma pack()
//
//     typedef int Fd;
//
// Keep the field names from the header: the tests look them up by name. They
// compare your types against the sizes, alignments, and field offsets the C
// compiler reports for the declarations above.
use std::ffi::c_int;

// TODO: mirror `struct Point`.
pub struct Point;

// TODO: mirror `struct Shape`. C may leave `parent` null, so pick a Rust type
// that says so: an `Option` around a non-null pointer.
pub struct Shape;

// TODO: mirror `struct WireHeader`. It is declared under `#pragma pack(1)`, so C
// gives it no padding at all.
pub struct WireHeader;

// TODO: `Fd` is a wrapper that C never sees: it passes a plain `int`. Make it a
// tuple struct around a `c_int`, with the attribute that guarantees the same
// layout *and* the same function-call ABI as that field.
//
// `close_fd` below takes an `Fd`, so the crate won't build until `Fd` is a type
// C can accept. Note that the compiler can't tell the two candidate attributes
// apart here: it accepts either, and the ABI guarantee is the reason to prefer
// one of them.
pub struct Fd;

/// Closes a file descriptor. Here it only hands the number back.
#[unsafe(no_mangle)]
pub extern "C" fn close_fd(fd: Fd) -> c_int {
    fd.0
}

#[cfg(test)]
mod tests {
    use super::{Fd, Point, Shape, WireHeader};
    use std::mem::offset_of;

    // What the C compiler says about each type, from `c_src/layout.c`: size and
    // alignment first, then the offset of each field in declaration order.
    unsafe extern "C" {
        safe static POINT_LAYOUT: [usize; 4];
        safe static SHAPE_LAYOUT: [usize; 7];
        safe static WIRE_HEADER_LAYOUT: [usize; 4];
        safe static FD_LAYOUT: [usize; 2];
    }

    #[test]
    fn point_matches_c() {
        let rust = [
            size_of::<Point>(),
            align_of::<Point>(),
            offset_of!(Point, x),
            offset_of!(Point, y),
        ];
        assert_eq!(rust, POINT_LAYOUT);
    }

    #[test]
    fn shape_matches_c() {
        let rust = [
            size_of::<Shape>(),
            align_of::<Shape>(),
            offset_of!(Shape, kind),
            offset_of!(Shape, name),
            offset_of!(Shape, origin),
            offset_of!(Shape, parent),
            offset_of!(Shape, id),
        ];
        assert_eq!(rust, SHAPE_LAYOUT);
    }

    #[test]
    fn wire_header_matches_c() {
        let rust = [
            size_of::<WireHeader>(),
            align_of::<WireHeader>(),
            offset_of!(WireHeader, version),
            offset_of!(WireHeader, len),
        ];
        assert_eq!(rust, WIRE_HEADER_LAYOUT);
    }

    #[test]
    fn fd_matches_c() {
        let rust = [size_of::<Fd>(), align_of::<Fd>()];
        assert_eq!(rust, FD_LAYOUT);
    }
}
