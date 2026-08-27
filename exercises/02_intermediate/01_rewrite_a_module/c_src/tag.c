#include "tag.h"

#include <stdlib.h>
#include <string.h>

#include "normalize.h"
#include "util.h"

void tag_list_free(char **tags, size_t n) {
  if (!tags)
    return;
  for (size_t i = 0; i < n; i++)
    free(tags[i]);
  free(tags);
}

static BmResult list_push(char ***list, size_t *n, size_t *cap, const char *s) {
  if (*n == *cap) {
    size_t nc = (*cap == 0) ? 4 : (*cap) * 2;
    char **nl = realloc(*list, nc * sizeof(char *));
    if (!nl)
      return BM_ERR_IO;
    *list = nl;
    *cap = nc;
  }
  char *dup = bm_strdup(s);
  if (!dup)
    return BM_ERR_IO;
  (*list)[(*n)++] = dup;
  return BM_OK;
}

BmResult parse_tag_args(int argc, char **argv, char ***out_add, size_t *n_add,
                        char ***out_rm, size_t *n_rm) {
  char **add = NULL, **rm = NULL;
  size_t na = 0, nr = 0, ca = 0, cr = 0;

  for (int i = 0; i < argc; i++) {
    const char *w = argv[i];
    int is_remove = 0;
    if (*w == '-') {
      is_remove = 1;
      w++;
    } else if (*w == '+') {
      w++;
    }

    char buf[BM_MAX_TAG_LEN];
    BmResult r = tag_normalize(w, buf, sizeof buf);
    if (r != BM_OK)
      goto fail;

    r = is_remove ? list_push(&rm, &nr, &cr, buf)
                  : list_push(&add, &na, &ca, buf);
    if (r != BM_OK)
      goto fail;
  }

  *out_add = add;
  *n_add = na;
  *out_rm = rm;
  *n_rm = nr;
  return BM_OK;

fail:
  tag_list_free(add, na);
  tag_list_free(rm, nr);
  *out_add = NULL;
  *n_add = 0;
  *out_rm = NULL;
  *n_rm = 0;
  return BM_ERR_INVALID_URL;
}

static int has_tag(const char *const *list, size_t n, const char *t) {
  for (size_t i = 0; i < n; i++) {
    if (strcmp(list[i], t) == 0)
      return 1;
  }
  return 0;
}

BmResult tag_set_union(char ***dst, size_t *n_dst, const char *const *src,
                       size_t n_src) {
  for (size_t i = 0; i < n_src; i++) {
    if (has_tag((const char *const *)*dst, *n_dst, src[i]))
      continue;

    char **alloc = realloc(*dst, (*n_dst + 1) * sizeof(char *));
    if (!alloc)
      return BM_ERR_IO;
    *dst = alloc;

    char *dup = bm_strdup(src[i]);
    if (!dup)
      return BM_ERR_IO;
    (*dst)[(*n_dst)++] = dup;
  }
  return BM_OK;
}

BmResult tag_set_diff(char ***dst, size_t *n_dst, const char *const *src,
                      size_t n_src) {
  size_t w = 0;
  for (size_t i = 0; i < *n_dst; i++) {
    if (has_tag(src, n_src, (*dst)[i])) {
      free((*dst)[i]);
      (*dst)[i] = NULL; /* prevent double-free */
    } else {
      (*dst)[w++] = (*dst)[i];
    }
  }
  *n_dst = w;
  return BM_OK;
}
