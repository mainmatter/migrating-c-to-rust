#pragma once

/* Result codes for the public bm_* API. */

typedef enum {
  BM_OK = 0,
  BM_ERR_NOT_FOUND,
  BM_ERR_DUPLICATE,
  BM_ERR_INVALID_URL,
  BM_ERR_IO,
  BM_ERR_CORRUPT,
} BmResult;
