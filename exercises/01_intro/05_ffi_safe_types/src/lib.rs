pub struct Tag {
    pub name: String,
}

#[no_mangle]
pub extern "C" fn bm_tag_describe(tag: Tag) -> String {
    // `Tag` and `String` are both no FFI safe types :/ we need to replace them with types that are.
    // hint: its probably time for pointers.
    // Check the c_test/test_tag_describe.c file for how they expect to call `bm_tag_describe`!
    tag.name.clone()
}
