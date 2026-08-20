#pragma once

#include <stddef.h>

typedef struct {
  char *url;
  char **tags;
  size_t n_tags;
} Bookmark;

/*
 * Allocate a Bookmark and copies of all its strings with C's allocator.
 * Returns NULL when an input is invalid or an allocation fails.
 */
Bookmark *bookmark_new(const char *url, const char *const *tags, size_t n_tags);

/* Release a Bookmark through the same C allocation API. NULL is allowed. */
void bookmark_free(Bookmark *bookmark);

/* Number of live Bookmarks, used by the exercise's verification code. */
size_t bookmark_live_count(void);
