use std::num::NonZero;

#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub struct Session {
    pub id: NonZero<u32>,
    pub flags: u32,
}

/// Stands in for the C implementation.
///
/// Writes a `Session` into `out` and returns `true`. When `fail` is set, it
/// returns `false` and writes nothing at all.
///
/// # Safety
///
/// `out` must point to memory with the size and alignment of a `Session`.
pub unsafe extern "C" fn open_session(out: *mut Session, fail: bool) -> bool {
    if fail {
        return false;
    }
    // SAFETY: the caller guarantees that `out` has the size and alignment of a
    // `Session`, and `write` doesn't read whatever was there before.
    unsafe {
        out.write(Session {
            id: NonZero::new(7).unwrap(),
            flags: 3,
        })
    };
    true
}

/// Opens a session, or returns `None` when the C side reports failure.
pub fn load_session(fail: bool) -> Option<Session> {
    // TODO: reserve memory for a `Session` without claiming it holds one yet,
    // hand a pointer to it to `open_session`, and only treat it as a `Session`
    // once the call says it was filled in.
    todo!()
}

#[cfg(test)]
mod tests {
    use super::{Session, load_session};
    use std::num::NonZero;

    #[test]
    fn returns_the_session_the_c_side_wrote() {
        let session = load_session(false);

        assert_eq!(
            session,
            Some(Session {
                id: NonZero::new(7).unwrap(),
                flags: 3,
            })
        );
    }

    #[test]
    fn returns_none_when_the_call_fails() {
        assert_eq!(load_session(true), None);
    }
}
