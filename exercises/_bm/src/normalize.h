#pragma once

#include <stddef.h>

#include "result.h"

#define BM_MAX_URL_LEN 2048
#define BM_MAX_TAG_LEN 64

/*
 * Lowercase the URL. Writes the normalized URL into `out`.
 */
BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);

/* Trim, lowercase, reject empty. Writes into `out` (capacity out_len). */
BmResult tag_normalize(const char *raw, char *out, size_t out_len);
