# Error codes

C APIs commonly reserve integer values for success and different failures:

```c
#define BM_OK 0
#define BM_ERR_NOT_FOUND 1
#define BM_ERR_IO 2
#define BM_ERR_INVALID_INPUT 3

int remove_bookmark(struct Store *store, const char *url);
```

The integer does not identify which constants belong to this function, and C
will accept an unrelated integer in the same places. Correct propagation relies
on control flow and review rather than the type system.

## Give failures a type

In a safe Rust API, an enum defines the possible failures and `Result` connects
them to the operation:

```rust
#[derive(Debug, PartialEq, Eq)]
enum RemoveError {
    NotFound,
    Io,
}

# struct Store { urls: Vec<String> }
fn remove_bookmark(store: &mut Store, url: &str) -> Result<(), RemoveError> {
    let Some(index) = store.urls.iter().position(|stored| stored == url) else {
        return Err(RemoveError::NotFound);
    };
    store.urls.remove(index);
    Ok(())
}
```

The small in-memory implementation only produces `NotFound`. `Io` remains part
of the operation's error type because the complete implementation may persist
the changed store. The example does not need to manufacture an I/O failure to
show that callers must account for it.

A function that performs several fallible steps can use `?` to return early
without collapsing their error information:

```rust
# #[derive(Debug)] enum RemoveError { NotFound, Io }
# struct Store;
# fn remove_bookmark(_: &mut Store, _: &str) -> Result<(), RemoveError> { Ok(()) }
fn remove_pair(store: &mut Store, first: &str, second: &str) -> Result<(), RemoveError> {
    remove_bookmark(store, first)?;
    remove_bookmark(store, second)?;
    Ok(())
}
```

When a lower layer has a different error type, use `map_err` or implement `From`
to add the context needed by the public API. Do not convert every failure to one
generic status simply because the C version did.

## Map once at the C boundary

An unchanged C ABI still returns its specified numbers. Keep that mapping in one
adapter:

```rust,no_run
use std::ffi::{CStr, c_char};

# #[derive(Debug)] enum RemoveError { NotFound, Io }
# struct Store;
# fn remove_bookmark(_: &mut Store, _: &str) -> Result<(), RemoveError> { Ok(()) }
const BM_OK: i32 = 0;
const BM_ERR_NOT_FOUND: i32 = 1;
const BM_ERR_IO: i32 = 2;
const BM_ERR_INVALID_INPUT: i32 = 3;

/// # Safety
///
/// `store` must point to a valid, uniquely borrowed `Store`. `url` must point
/// to a readable, NUL-terminated byte string.
#[unsafe(export_name = "remove_bookmark")]
pub unsafe extern "C" fn remove_bookmark_ffi(
    store: *mut Store,
    url: *const c_char,
) -> i32 {
    let Some(store) = (unsafe { store.as_mut() }) else {
        return BM_ERR_INVALID_INPUT;
    };
    if url.is_null() {
        return BM_ERR_INVALID_INPUT;
    }

    // SAFETY: The pointer requirements are part of this function's contract,
    // and the null case was rejected above.
    let url = unsafe { CStr::from_ptr(url) };
    let Ok(url) = url.to_str() else {
        return BM_ERR_INVALID_INPUT;
    };

    match remove_bookmark(store, url) {
        Ok(()) => BM_OK,
        Err(RemoveError::NotFound) => BM_ERR_NOT_FOUND,
        Err(RemoveError::Io) => BM_ERR_IO,
    }
}
```

This does not require the Rust error enum to have a C representation. Its layout
is private because it never crosses the boundary. Conversely, if a raw numeric
status arrives from C, match its known values explicitly. Constructing a Rust
enum from an arbitrary integer can create an invalid value.

`bm`'s `BmResult` is a real example of this distinction. Its storage layer also
handles `ENOENT`: a missing file may represent an empty database, while a read
failure or corrupt file remains an error. The domain decides which outcomes are
ordinary; the fact that C reported them as integers does not.

## Head to the exercise

In `exercises/02_intermediate/05_02_error_codes`, replace the three numeric
outcomes of a bookmark-ID change with `Result<(), ChangeIdError>`. Preserve the
operation's behavior and leave the index unchanged when the old ID is missing or
the new ID is already assigned to another bookmark. Then complete the FFI
adapter that maps the Rust result back to the existing C status codes.
