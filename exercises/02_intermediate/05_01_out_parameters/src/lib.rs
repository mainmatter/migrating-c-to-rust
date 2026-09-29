//! Replace two C out-parameters with one Rust return value.

pub const BM_BOOKMARKS_PER_PAGE: usize = 20;

/// Return the zero-based page and slot for `bookmark_index`.
pub fn page_location(bookmark_index: usize) -> (usize, usize) {
    // TODO 1 OF 2: Return `(page, slot)` using `BM_BOOKMARKS_PER_PAGE`.
    let _ = bookmark_index;
    (0, 0)
}

/// Preserve the C header while delegating the calculation to `page_location`.
///
/// # Safety
///
/// `out_page` and `out_slot` must be valid, writable pointers.
#[unsafe(export_name = "bm_page_location")]
pub unsafe extern "C" fn bm_page_location_ffi(
    bookmark_index: usize,
    out_page: *mut usize,
    out_slot: *mut usize,
) {
    // TODO 2 OF 2: Call `page_location` and write both returned values to the
    // output pointers.
    let _ = (bookmark_index, out_page, out_slot);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_api_returns_page_and_slot() {
        assert_eq!(page_location(0), (0, 0));
        assert_eq!(page_location(19), (0, 19));
        assert_eq!(page_location(20), (1, 0));
        assert_eq!(page_location(47), (2, 7));
    }

    #[test]
    fn adapter_writes_both_outputs() {
        let mut page = usize::MAX;
        let mut slot = usize::MAX;

        // SAFETY: Both output pointers refer to live, writable `usize` values.
        unsafe { bm_page_location_ffi(47, &mut page, &mut slot) };

        assert_eq!((page, slot), (2, 7));
    }
}
