#include "shout.h"

#include <ctype.h>
#include <string.h>

int bm_shout(char *out, size_t out_len, const char *label) {
  size_t label_len = strlen(label);

  if (out_len < label_len + 2) {
    return -1;
  }

  for (size_t i = 0; i < label_len; i++) {
    out[i] = (char)toupper((unsigned char)label[i]);
  }

  out[label_len] = '!';
  out[label_len + 1] = '\0';
  return 0;
}
