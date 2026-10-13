#pragma once

#include <stddef.h>

#include "result.h"

#define BM_MAX_URL_LEN 2048

BmResult bm_normalize_url(const char *raw, char *out, size_t out_len);
