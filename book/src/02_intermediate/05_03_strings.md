# Strings

C and Rust use different representations for strings.

A traditional C string is a sequence of bytes followed by a NUL byte (`\0`). A
`const char *` usually carries no separate length: functions find the end by
scanning for that terminator. The bytes before it aren't necessarily UTF-8.

Rust's `&str` and `String` contain valid UTF-8 and carry their byte length. A
`&str` borrows its contents, while a `String` owns them. Neither representation
needs a trailing NUL byte.

Rust provides two more types for traditional C strings:

- `CStr` is a borrowed, NUL-terminated sequence of bytes;
- `CString` is an owned, NUL-terminated sequence of bytes.

Passing a string across the boundary therefore takes two steps: establish the C
string representation, then decide whether its bytes are valid Rust text.

## Borrow a C string as `CStr`

Use `CStr::from_ptr` to borrow a traditional C string:

```rust,no_run
use std::ffi::{CStr, c_char};

/// # Safety
///
/// A non-null `tag` must point to a live, NUL-terminated byte string.
unsafe extern "C" fn tag_byte_len(tag: *const c_char) -> usize {
    if tag.is_null() {
        return 0;
    }

    // SAFETY: The caller supplies a live C string and null was handled above.
    let tag = unsafe { CStr::from_ptr(tag) };
    tag.count_bytes()
}
```

`from_ptr` is `unsafe` because Rust can't determine how much memory is readable
or whether a NUL terminator will be found. The resulting `&CStr` borrows the
original allocation and can't outlive it.

`count_bytes()` reports the number of bytes before the terminator. Nothing so
far has checked or assumed that those bytes are UTF-8. `to_bytes()` exposes
those bytes when the API defines the value as arbitrary data. If the value is
text, perform the encoding conversion explicitly.

## Convert between `CStr` and Rust text

`CStr::to_str` validates UTF-8 and returns a borrowed `&str`, or an error if the
bytes aren't valid UTF-8. Turn the `&str` into a `String` when the Rust code
needs its own copy. `to_string_lossy()` replaces invalid sequences, which
silently changes the text; use it only when that's the API's stated policy.

Going the other way requires a NUL-terminated representation. `CString::new`
creates one from Rust text:

```rust
use std::ffi::CString;

let tag = String::from("rust");
let c_tag = CString::new(tag)?;
let borrowed = c_tag.as_c_str();
let pointer = borrowed.as_ptr();
# let _ = pointer;
# Ok::<(), std::ffi::NulError>(())
```

The conversion is fallible because a Rust string may contain an interior NUL.
That byte would terminate a C string early. The pointer from `as_ptr()` is valid
only while `c_tag` is alive, so don't save a pointer obtained from a temporary
`CString`.

## Expose a Rust string as a pointer and length

A C API can also describe text using a returned pointer and a length
out-parameter:

```c
const uint8_t *bm_category_label(uint32_t category, size_t *out_len);
```

A borrowed Rust string already has both pieces. The adapter returns its pointer
and writes its byte length through the out-parameter:

```rust
fn category_label(category: u32) -> &'static str {
    match category {
        1 => "article",
        2 => "reference",
        _ => "other",
    }
}

/// # Safety
///
/// `out_len` must be a valid, writable pointer.
#[unsafe(export_name = "bm_category_label")]
unsafe extern "C" fn category_label_ffi(category: u32, out_len: *mut usize) -> *const u8 {
    let label = category_label(category);

    // SAFETY: The caller must provide a writable output pointer.
    unsafe { out_len.write(label.len()) };
    label.as_ptr()
}
```

The pointer isn't NUL-terminated. C must use the written length instead of
`strlen` and must not mutate the bytes. The length is a number of bytes, not
Unicode characters.

This example returns a view of a static string, so no allocation or matching
destructor is required. When the `&str` borrows from an input or another value
instead, C must not retain the returned pointer after that value stops being
valid.

## Keep the boundary explicit

For each string parameter, specify:

- whether it's a traditional C string or a pointer-and-length view;
- whether `NULL` is accepted and how it differs from an empty string;
- whether the bytes must be UTF-8;
- how long a returned or borrowed pointer remains valid.

Use `CStr` and `CString` for traditional C strings, `&str` and `String` for Rust
text, and byte slices when the value isn't text. Convert between them at the
boundary instead of treating their pointers as interchangeable.

## Head to the exercise

In `exercises/02_intermediate/05_03_strings`, accept a traditional C string,
convert it through `CStr` to `&str`, and return a borrowed tag name as a pointer
with its byte length written to an out-parameter. The returned pointer points
into the input, so it's valid only while that input string is alive.
