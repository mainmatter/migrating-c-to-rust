#pragma once

#include <stddef.h>

#include "result.h"

#define BM_MAX_URL_LEN 2048
#define BM_MAX_TAG_LEN 64

/* Lowercase a URL and reject tabs. Writes into the caller-owned buffer. */
BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);

/* Trim, lowercase, and reject empty tags. Writes into the caller-owned buffer.
 */
BmResult tag_normalize(const char *raw, char *out, size_t out_len);
