#pragma once

#include <stddef.h>

#include "result.h"

typedef struct {
  char *url;
  char **tags;
  size_t n_tags;
} Bookmark;

/*
 * Allocate a new Bookmark. Returns NULL on allocation failure.
 */
Bookmark *bookmark_new(const char *url, const char *const *tags, size_t n_tags);

void bookmark_free(Bookmark *b);
