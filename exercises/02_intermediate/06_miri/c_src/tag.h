#pragma once

#include <stddef.h>

/* A slice of the legacy bookmark-manager C API (see
 * exercises/_bm/src/normalize.h). This is the not-yet-ported module our Rust
 * code still has to call. */

typedef enum {
  BM_OK = 0,
  BM_ERR_NOT_FOUND,
  BM_ERR_DUPLICATE,
  BM_ERR_INVALID_URL,
  BM_ERR_IO,
  BM_ERR_CORRUPT,
  BM_ERR_INVALID_BUFFER,
  BM_ERR_BUFFER_TOO_SMALL,
} BmResult;

#define BM_MAX_TAG_LEN 64

/*
 * Normalize a tag: skip leading whitespace, copy up to the next whitespace or
 * ',' into `out` lowercased and NUL-terminated (capacity `out_len`, including
 * the NUL). Returns BM_ERR_INVALID_URL for NULL arguments, an empty result, or
 * a tag that does not fit.
 */
BmResult tag_normalize(const char *raw, char *out, size_t out_len);
