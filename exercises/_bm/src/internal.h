#pragma once

#include <stddef.h>
#include <stdint.h>

#include "bookmark.h"
#include "result.h"

/* Internal definitions shared by index.c and storage.c. */

extern Bookmark *const BM_TOMBSTONE;

#define BM_INITIAL_CAP 16

struct BmDb {
  char *path;
  Bookmark **entries; /* NULL = empty, BM_TOMBSTONE = deleted */
  size_t cap;
  size_t len;
  size_t used;
  uint32_t next_id;
  int dirty;
};
