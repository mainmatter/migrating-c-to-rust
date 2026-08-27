use std::ffi::CString;

#[test]
fn formats_a_label() {
    let label = CString::new("rust").unwrap();

    assert_eq!(bm_lldb_gdb::shout(&label), Ok("RUST!".to_owned()));
}
