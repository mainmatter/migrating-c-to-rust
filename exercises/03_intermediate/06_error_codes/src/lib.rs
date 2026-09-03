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
    let position = index
        .ids
        .iter()
        .position(|&id| id == old_id)
        .ok_or(ChangeIdError::NotFound)?;
    if new_id != old_id && index.ids.contains(&new_id) {
        return Err(ChangeIdError::Duplicate);
    }
    index.ids[position] = new_id;
    Ok(())

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
    // SAFETY: the caller guarantees `index` points to a valid `BookmarkIndex`
    // that nothing else borrows during the call.
    let index = unsafe { &mut *index };

    match change_bookmark_id(index, old_id, new_id) {
        Ok(()) => BmStatus::Ok,
        Err(ChangeIdError::NotFound) => BmStatus::NotFound,
        Err(ChangeIdError::Duplicate) => BmStatus::Duplicate,
    }

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
