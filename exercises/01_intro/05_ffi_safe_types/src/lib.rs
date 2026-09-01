pub struct Tag {
    pub name: String,
}

#[no_mangle]
pub extern "C" fn bm_tag_describe(tag: Tag) -> String {
    // `Tag` and `String` are both types that aren't FFI-safe. We need to replace
    // them with types that are.
    // hint: it's probably time for pointers. Note that the C harness passes
    // `Tag` by value, so it stays a struct: it's the field that has to change.
    // Check the c_test/test_tag_describe.c file for how they expect to call `bm_tag_describe`!
    tag.name.clone()
}
