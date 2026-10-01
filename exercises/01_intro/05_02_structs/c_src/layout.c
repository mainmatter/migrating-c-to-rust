/* Reports how the C compiler lays out the types in `shapes.h`, so the tests can
 * compare your Rust types against the real thing. You don't need to change
 * anything here. */

#include <stdalign.h>
#include <stddef.h>

#include "shapes.h"

const size_t POINT_LAYOUT[] = {
    sizeof(struct Point),
    alignof(struct Point),
    offsetof(struct Point, x),
    offsetof(struct Point, y),
};

const size_t SHAPE_LAYOUT[] = {
    sizeof(struct Shape),           alignof(struct Shape),
    offsetof(struct Shape, kind),   offsetof(struct Shape, name),
    offsetof(struct Shape, origin), offsetof(struct Shape, parent),
    offsetof(struct Shape, id),
};

const size_t WIRE_HEADER_LAYOUT[] = {
    sizeof(struct WireHeader),
    alignof(struct WireHeader),
    offsetof(struct WireHeader, version),
    offsetof(struct WireHeader, len),
};

const size_t FD_LAYOUT[] = {
    sizeof(Fd),
    alignof(Fd),
};
