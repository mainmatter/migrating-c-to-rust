// A C library reports events through these declarations:
//
//     enum Level { LEVEL_DEBUG, LEVEL_INFO, LEVEL_WARN, LEVEL_ERROR };
//
//     #define EVENT_EXITED   0
//     #define EVENT_PROGRESS 1
//
//     struct Event {
//         int kind;
//         union {
//             struct { int32_t code; } exited;
//             struct { float ratio; } progress;
//         } data;
//     };
//
// The Rust mirrors of those declarations are written for you. What's missing
// are the two conversions that keep values C sends us out of Rust enums until
// we've checked them.

use std::ffi::c_int;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

pub const EVENT_EXITED: c_int = 0;
pub const EVENT_PROGRESS: c_int = 1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Exited {
    pub code: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Progress {
    pub ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union EventData {
    pub exited: Exited,
    pub progress: Progress,
}

/// The C struct, mirrored field for field.
#[repr(C)]
pub struct CEvent {
    pub kind: c_int,
    pub data: EventData,
}

/// What the rest of our Rust code works with.
#[derive(Debug, PartialEq)]
pub enum Event {
    Exited { code: i32 },
    Progress { ratio: f32 },
}

impl TryFrom<c_int> for Level {
    /// The value C sent, when it isn't one of the four levels.
    type Error = c_int;

    fn try_from(value: c_int) -> Result<Self, c_int> {
        // TODO: turn the integer into a `Level`, and report unknown values as
        // an error instead of producing an enum that holds one.
        todo!()
    }
}

impl TryFrom<&CEvent> for Event {
    /// The `kind` C sent, when it isn't one of the known kinds.
    type Error = c_int;

    fn try_from(event: &CEvent) -> Result<Self, c_int> {
        // TODO: match on `event.kind` to find out which union field is the
        // active one, then read it. Reading a union field is `unsafe`, so each
        // read needs a `// SAFETY:` comment saying why the field is the right
        // one.
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{CEvent, EVENT_EXITED, EVENT_PROGRESS, Event, EventData, Exited, Level, Progress};

    #[test]
    fn known_levels_are_converted() {
        assert_eq!(Level::try_from(0), Ok(Level::Debug));
        assert_eq!(Level::try_from(3), Ok(Level::Error));
    }

    #[test]
    fn unknown_levels_are_rejected() {
        assert_eq!(Level::try_from(4), Err(4));
        assert_eq!(Level::try_from(-1), Err(-1));
    }

    #[test]
    fn reads_the_active_union_field() {
        let exited = CEvent {
            kind: EVENT_EXITED,
            data: EventData {
                exited: Exited { code: -2 },
            },
        };
        let progress = CEvent {
            kind: EVENT_PROGRESS,
            data: EventData {
                progress: Progress { ratio: 0.25 },
            },
        };

        assert_eq!(Event::try_from(&exited), Ok(Event::Exited { code: -2 }));
        assert_eq!(
            Event::try_from(&progress),
            Ok(Event::Progress { ratio: 0.25 })
        );
    }

    #[test]
    fn unknown_kinds_are_rejected() {
        let unknown = CEvent {
            kind: 7,
            data: EventData {
                exited: Exited { code: 0 },
            },
        };

        assert_eq!(Event::try_from(&unknown), Err(7));
    }
}
