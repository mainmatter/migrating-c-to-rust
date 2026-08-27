#pragma once

#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

#include "result.h"

/* strdup with NULL passthrough. */
char *bm_strdup(const char *s);

/* Lowercase ASCII in place. */
void bm_strlower(char *s);

/*
 * BM_FMT_ERR(out, fmt, ...) — heap-allocate a formatted error string into
 * `out` (a `char *` lvalue) and evaluate to NULL.
 *
 *   return BM_FMT_ERR(err, "bad url: %s", input);
 */
#define BM_FMT_ERR(out, ...)                                                   \
  ({                                                                           \
    int _bm_n = snprintf(NULL, 0, __VA_ARGS__);                                \
    (out) = (_bm_n >= 0) ? (char *)malloc((size_t)_bm_n + 1) : NULL;           \
    if (out)                                                                   \
      snprintf((out), (size_t)_bm_n + 1, __VA_ARGS__);                         \
    (void *)0;                                                                 \
  })
