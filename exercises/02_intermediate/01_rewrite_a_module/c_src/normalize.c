#include "normalize.h"

#include <ctype.h>
#include <string.h>

BmResult bm_normalize_url(const char *raw, char *out, size_t out_len) {
  if (!raw || !out || out_len == 0)
    return BM_ERR_INVALID_URL;

  size_t len = strlen(raw);
  if (len + 1 > out_len || strchr(raw, '\t'))
    return BM_ERR_INVALID_URL;

  for (size_t i = 0; i < len; i++)
    out[i] = (char)tolower((unsigned char)raw[i]);
  out[len] = '\0';
  return BM_OK;
}

BmResult tag_normalize(const char *raw, char *out, size_t out_len) {
  if (!raw || !out || out_len == 0)
    return BM_ERR_INVALID_URL;

  while (*raw && isspace((unsigned char)*raw))
    raw++;

  size_t n = 0;
  while (*raw && !isspace((unsigned char)*raw) && *raw != ',' && *raw != '\t') {
    if (n + 1 >= out_len)
      return BM_ERR_INVALID_URL;
    out[n++] = (char)tolower((unsigned char)*raw);
    raw++;
  }
  out[n] = '\0';

  return n == 0 ? BM_ERR_INVALID_URL : BM_OK;
}
