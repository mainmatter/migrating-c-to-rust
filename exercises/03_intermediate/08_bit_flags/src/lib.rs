//! Update bookmark-output fields through typed flags.

use bitflags::bitflags;

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct BookmarkFields: u32 {
        const TITLE = 1 << 0;
        const URL = 1 << 1;
        const TAGS = 1 << 2;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldsError {
    UnknownBits,
    ConflictingChanges,
}

/// Apply one update while preserving fields mentioned in neither change set.
pub fn update_fields(
    current: BookmarkFields,
    enable: BookmarkFields,
    disable: BookmarkFields,
) -> Result<BookmarkFields, FieldsError> {
    if enable.intersects(disable) {
        return Err(FieldsError::ConflictingChanges);
    }
    let mut fields = current;
    fields.insert(enable);
    fields.remove(disable);
    Ok(fields)

}

/// Strictly convert raw C masks, apply the update, and return raw result bits.
pub fn update_fields_from_c(current: u32, enable: u32, disable: u32) -> Result<u32, FieldsError> {
    let from_c = |raw| BookmarkFields::from_bits(raw).ok_or(FieldsError::UnknownBits);
    update_fields(from_c(current)?, from_c(enable)?, from_c(disable)?).map(|fields| fields.bits())


}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enables_disables_and_preserves_fields() {
        let current = BookmarkFields::TITLE | BookmarkFields::URL;

        assert_eq!(
            update_fields(current, BookmarkFields::TAGS, BookmarkFields::TITLE),
            Ok(BookmarkFields::URL | BookmarkFields::TAGS)
        );
    }

    #[test]
    fn rejects_conflicting_changes() {
        assert_eq!(
            update_fields(
                BookmarkFields::TITLE,
                BookmarkFields::URL | BookmarkFields::TAGS,
                BookmarkFields::TAGS,
            ),
            Err(FieldsError::ConflictingChanges)
        );
    }

    #[test]
    fn converts_raw_masks_and_rejects_unknown_bits() {
        let current = (BookmarkFields::TITLE | BookmarkFields::URL).bits();

        assert_eq!(
            update_fields_from_c(
                current,
                BookmarkFields::TAGS.bits(),
                BookmarkFields::TITLE.bits(),
            ),
            Ok((BookmarkFields::URL | BookmarkFields::TAGS).bits())
        );

        let unknown = 1 << 8;
        assert_eq!(
            update_fields_from_c(unknown, 0, 0),
            Err(FieldsError::UnknownBits)
        );
        assert_eq!(
            update_fields_from_c(0, unknown, 0),
            Err(FieldsError::UnknownBits)
        );
        assert_eq!(
            update_fields_from_c(0, 0, unknown),
            Err(FieldsError::UnknownBits)
        );
    }
}
