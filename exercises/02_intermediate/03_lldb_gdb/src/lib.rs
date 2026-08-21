//! Debug this crate with `rust-lldb` or `rust-gdb`.
//!
//! `cargo run -p bm_lldb_gdb` crashes in the C helper. Start by setting a
//! breakpoint on `bm_shout`, then inspect its arguments and the mixed stack.
//! The C header is the source of truth for the function signature.

use std::ffi::{CStr, CString, c_char};

unsafe extern "C" {
    fn bm_shout(label: *const c_char, out: *mut c_char, out_len: usize) -> i32;
}

pub fn shout(label: &CStr) -> Result<String, i32> {
    let mut out = [0_u8; 64];

    // SAFETY: `label` is NUL-terminated and `out` points to 64 writable bytes.
    let status = unsafe { bm_shout(label.as_ptr(), out.as_mut_ptr().cast::<c_char>(), out.len()) };

    if status != 0 {
        return Err(status);
    }

    // SAFETY: `bm_shout` writes a NUL-terminated string when it returns 0.
    let out = unsafe { CStr::from_ptr(out.as_ptr().cast::<c_char>()) };
    Ok(out.to_string_lossy().into_owned())
}

pub fn demo() -> Result<String, i32> {
    let label = CString::new("you did it!").expect("a string literal contains no NUL");
    shout(&label)
}
