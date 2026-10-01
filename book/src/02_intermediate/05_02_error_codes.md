# Error codes

C APIs commonly report the outcome of an operation with an enum, with one
variant for success and one for each kind of failure:

```c
typedef enum {
    BM_OK = 0,
    BM_ERR_NOT_FOUND = 1,
    BM_ERR_IO = 2,
    BM_ERR_INVALID_INPUT = 3,
} BmStatus;

BmStatus remove_bookmark(struct Store *store, const char *url);
```

The return type names the codes, but a C enum is still just an integer. C
converts it to and from `int` freely and doesn't stop a function from returning
a code that makes no sense for it, or one that isn't a variant at all. Correct
propagation relies on control flow and review rather than the type system.

## Give failures a type

In the safe Rust API, `BM_OK` becomes `Ok(())`, and each failure becomes a
variant of an error enum:

```rust,ignore
enum RemoveError {
    NotFound,
    Io,
}

fn remove_bookmark(store: &mut Store, url: &str) -> Result<(), RemoveError>;
```

`BM_ERR_INVALID_INPUT` has no variant. It reports a null pointer or a URL that
isn't valid UTF-8, and the safe function can't receive either of those, because
the adapter below rejects them first. Don't merge the remaining failures into
one generic error because the C version shared a return type.

## Map once at the C boundary

An unchanged C ABI still returns its specified numbers. Keep that mapping in one
adapter:

```rust,no_run
use std::ffi::{CStr, c_char};

# #[derive(Debug)] enum RemoveError { NotFound, Io }
# struct Store;
# fn remove_bookmark(_: &mut Store, _: &str) -> Result<(), RemoveError> { Ok(()) }
#[repr(C)]
pub enum BmStatus {
    Ok = 0,
    NotFound = 1,
    Io = 2,
    InvalidInput = 3,
}

/// # Safety
///
/// `store` must point to a valid, uniquely borrowed `Store`. `url` must point
/// to a readable, NUL-terminated byte string.
#[unsafe(export_name = "remove_bookmark")]
pub unsafe extern "C" fn remove_bookmark_ffi(
    store: *mut Store,
    url: *const c_char,
) -> BmStatus {
    let Some(store) = (unsafe { store.as_mut() }) else {
        return BmStatus::InvalidInput;
    };
    if url.is_null() {
        return BmStatus::InvalidInput;
    }

    // SAFETY: The pointer requirements are part of this function's contract,
    // and the null case was rejected above.
    let url = unsafe { CStr::from_ptr(url) };
    let Ok(url) = url.to_str() else {
        return BmStatus::InvalidInput;
    };

    match remove_bookmark(store, url) {
        Ok(()) => BmStatus::Ok,
        Err(RemoveError::NotFound) => BmStatus::NotFound,
        Err(RemoveError::Io) => BmStatus::Io,
    }
}
```

`BmStatus` mirrors the C enum. `#[repr(C)]` gives it the representation the C
compiler picks for the same enum, so the two agree on the values they exchange.
Returning it to C is sound, because a Rust enum always holds one of its
variants. The Rust error enum doesn't need a C representation: its layout stays
private because it never crosses the boundary. Conversely, if a raw numeric
status arrives from C, match its known values explicitly. Constructing a Rust
enum from an arbitrary integer can create an invalid value.

`bm`'s `BmResult` follows this pattern. Its storage layer also shows that not
every C failure code is a Rust error: `ENOENT` means a missing file, which may
be an empty database, while a read failure or corrupt file remains an error. The
domain decides which outcomes are ordinary; the fact that C reported them as
integers doesn't.

## Head to the exercise

In `exercises/02_intermediate/05_02_error_codes`, implement `change_bookmark_id`
so that it reports its failures as a `Result<(), ChangeIdError>` instead of C
status codes, leaving the index unchanged when it fails. Then complete the FFI
adapter that maps that result back to the C status codes.
