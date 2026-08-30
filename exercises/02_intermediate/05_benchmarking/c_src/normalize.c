#include "normalize.h"

#include <ctype.h>
#include <string.h>

BmResult bm_normalize_url(const char *raw, char *out, size_t out_len) {
  (void)out_len;

  /* Lowercase the URL into `out`. */
  size_t i = 0;
  while (raw[i] != '\0') {
    out[i] = (char)tolower((unsigned char)raw[i]);
    i++;
  }
  out[i] = '\0';

  /* Reject embedded '\t' — it's our on-disk separator. */
  if (strchr(out, '\t'))
    return BM_ERR_INVALID_URL;

  return BM_OK;
}
