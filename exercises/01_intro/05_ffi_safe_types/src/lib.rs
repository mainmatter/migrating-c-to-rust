use std::ffi::c_char;

#[repr(C)]
pub struct Tag {
    pub name: *mut c_char,
}

// From this exercise on, `wr` regenerates the C header for you before it builds
// the harness, so you don't have to run `cheadergen generate` by hand again.
#[unsafe(no_mangle)]
pub extern "C" fn bm_tag_describe(tag: Tag) -> *const c_char {
    // `Tag` and `String` are both types that aren't FFI-safe. We need to replace
    // them with types that are.
    // hint: it's probably time for pointers. Note that the C harness passes
    // `Tag` by value, so it stays a struct: it's the field that has to change.
    tag.name.cast_const()
}
