use std::ffi::c_char;

#[no_mangle]
pub unsafe extern "C" fn bm_normalize_url(
    _url: *const c_char,
    _out: *mut c_char,
    _out_len: usize,
) -> i32 {
    // Replaces this stub with your exercise 06 solution.
    // You will then notice the failing clippy lints pointing you towards adding safety comments!
    // Note that during this exercise we do not test the "FFI Tax" rule because that would involve
    // a much larger, more complicated example to show any slowdowns.

    todo!()
}
