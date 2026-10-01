// The Rust side of these C declarations:
//
//     #define EVENT_STARTED  0
//     #define EVENT_EXITED   1
//     #define EVENT_PROGRESS 2
//
//     struct Event {
//         int kind;
//         union {
//             int32_t code;  /* EVENT_EXITED */
//             float ratio;   /* EVENT_PROGRESS */
//         } data;
//     };

use std::ffi::c_int;

pub const EVENT_STARTED: c_int = 0;
pub const EVENT_EXITED: c_int = 1;
pub const EVENT_PROGRESS: c_int = 2;

#[repr(C)]
#[derive(Clone, Copy)]
pub union CEventData {
    pub code: i32,
    pub ratio: f32,
}

#[repr(C)]
pub struct CEvent {
    pub kind: c_int,
    pub data: CEventData,
}

/// What the rest of our Rust code works with.
pub enum Event {
    Started,
    Exited(i32),
    Progress(f32),
}

impl From<&Event> for CEvent {
    fn from(event: &Event) -> Self {
        // TODO: set the tag for each variant, and write its data into the
        // matching union field. `Started` has no data, but `data` still needs
        // a value.
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn started() {
        let event = CEvent::from(&Event::Started);
        assert_eq!(event.kind, EVENT_STARTED);
    }

    #[test]
    fn exited() {
        let event = CEvent::from(&Event::Exited(-2));
        assert_eq!(event.kind, EVENT_EXITED);
        // SAFETY: the tag says `code` is the active field.
        assert_eq!(unsafe { event.data.code }, -2);
    }

    #[test]
    fn progress() {
        let event = CEvent::from(&Event::Progress(0.25));
        assert_eq!(event.kind, EVENT_PROGRESS);
        // SAFETY: the tag says `ratio` is the active field.
        assert_eq!(unsafe { event.data.ratio }, 0.25);
    }
}
