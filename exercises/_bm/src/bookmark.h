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

#define BM_MAX_URL_LEN 2048

/*
 * Lowercase the URL. Writes the normalized URL into `out`.
 */
BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);
