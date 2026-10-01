//! Replace numeric error codes with typed Rust errors.

/// Status codes used by the unchanged C API, declared there as:
///
/// ```c
/// typedef enum { BM_OK = 0, BM_ERR_NOT_FOUND = 1, BM_ERR_DUPLICATE = 2 } BmStatus;
/// ```
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BmStatus {
    Ok = 0,
    NotFound = 1,
    Duplicate = 2,
}

pub struct BookmarkIndex {
    pub ids: Vec<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeIdError {
    NotFound,
    Duplicate,
}

pub fn change_bookmark_id(
    index: &mut BookmarkIndex,
    old_id: u32,
    new_id: u32,
) -> Result<(), ChangeIdError> {
    // TODO 1 OF 2: Change `old_id` in place. Return `NotFound` if it does not
    // exist and `Duplicate` if another bookmark already has `new_id`.
    let _ = (index, old_id, new_id);
    Err(ChangeIdError::NotFound)
}

/// Map the typed Rust result back to the C API's status codes.
///
/// # Safety
///
/// `index` must point to a valid, uniquely borrowed `BookmarkIndex`.
#[unsafe(export_name = "bm_change_bookmark_id")]
pub unsafe extern "C" fn bm_change_bookmark_id_ffi(
    index: *mut BookmarkIndex,
    old_id: u32,
    new_id: u32,
) -> BmStatus {
    // TODO 2 OF 2: Borrow `index`, call `change_bookmark_id`, and map its
    // result to a `BmStatus`.
    let _ = (index, old_id, new_id);
    BmStatus::NotFound
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> BookmarkIndex {
        BookmarkIndex {
            ids: vec![10, 20, 30],
        }
    }

    #[test]
    fn safe_api_uses_typed_errors() {
        let mut index = index();

        assert_eq!(change_bookmark_id(&mut index, 20, 42), Ok(()));
        assert_eq!(index.ids, [10, 42, 30]);

        assert_eq!(
            change_bookmark_id(&mut index, 99, 50),
            Err(ChangeIdError::NotFound)
        );
        assert_eq!(
            change_bookmark_id(&mut index, 42, 10),
            Err(ChangeIdError::Duplicate)
        );
        assert_eq!(index.ids, [10, 42, 30]);
    }

    #[test]
    fn ffi_adapter_maps_the_status_codes() {
        let mut index = index();

        // SAFETY: `index` is a live, uniquely borrowed `BookmarkIndex`.
        unsafe {
            assert_eq!(bm_change_bookmark_id_ffi(&mut index, 20, 42), BmStatus::Ok);
            assert_eq!(
                bm_change_bookmark_id_ffi(&mut index, 99, 50),
                BmStatus::NotFound
            );
            assert_eq!(
                bm_change_bookmark_id_ffi(&mut index, 42, 10),
                BmStatus::Duplicate
            );
        }
        assert_eq!(index.ids, [10, 42, 30]);
    }
}
