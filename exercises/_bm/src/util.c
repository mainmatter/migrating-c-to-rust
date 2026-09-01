#include "util.h"

#include <ctype.h>
#include <string.h>

char *bm_strdup(const char *s) {
  if (!s)
    return NULL;
  size_t n = strlen(s);
  char *r = malloc(n + 1);
  if (!r)
    return NULL;
  memcpy(r, s, n + 1);
  return r;
}

void bm_strlower(char *s) {
  if (!s)
    return;
  for (; *s; s++)
    *s = (char)tolower((unsigned char)*s);
}

const char *bm_result_to_str(BmResult r) {
  switch (r) {
  case BM_OK:
    return "ok";
  case BM_ERR_NOT_FOUND:
    return "not found";
  case BM_ERR_DUPLICATE:
    return "duplicate";
  case BM_ERR_INVALID_URL:
    return "invalid URL";
  case BM_ERR_IO:
    return "I/O error";
  case BM_ERR_CORRUPT:
    return "corrupt database";
  case BM_ERR_INVALID_BUFFER:
    return "invalid buffer";
  case BM_ERR_BUFFER_TOO_SMALL:
    return "buffer too small";
  }
  return "unknown error";
}
