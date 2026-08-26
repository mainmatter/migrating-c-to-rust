#include "storage.h"

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bookmark.h"
#include "index.h"
#include "internal.h"
#include "normalize.h"
#include "util.h"

#define LINE_BUF (BM_MAX_URL_LEN + 4096)

/* ------------------------------------------------------------------ */
/* Load.                                                              */
/* ------------------------------------------------------------------ */

static BmResult parse_tags(char *p, char ***out_tags, size_t *out_n) {
  char **tags = NULL;
  size_t n = 0, cap = 0;

  while (*p) {
    char *comma = strchr(p, ',');
    if (comma)
      *comma = '\0';
    if (*p) {
      if (n == cap) {
        size_t nc = cap == 0 ? 4 : cap * 2;
        char **nt = realloc(tags, nc * sizeof(char *));
        if (!nt)
          goto oom;
        tags = nt;
        cap = nc;
      }
      tags[n] = bm_strdup(p);
      if (!tags[n])
        goto oom;
      n++;
    }
    if (!comma)
      break;
    p = comma + 1;
  }

  *out_tags = tags;
  *out_n = n;
  return BM_OK;

oom:
  for (size_t i = 0; i < n; i++)
    free(tags[i]);
  free(tags);
  return BM_ERR_IO;
}

BmResult storage_load(const char *path, BmDb *db) {
  FILE *f = fopen(path, "r");
  if (!f) {
    if (errno == ENOENT)
      return BM_OK; /* fresh database */
    return BM_ERR_IO;
  }

  char line[LINE_BUF];
  size_t lineno = 0;
  while (fgets(line, sizeof line, f)) {
    lineno++;
    size_t len = strlen(line);
    if (len > 0 && line[len - 1] == '\n')
      line[--len] = '\0';
    if (len == 0)
      continue;

    char *tab = strchr(line, '\t');
    char **tags = NULL;
    size_t n_tags = 0;
    const char *url;

    if (tab) {
      *tab = '\0';
      url = line;
      BmResult r = parse_tags(tab + 1, &tags, &n_tags);
      if (r != BM_OK) {
        fclose(f);
        return r;
      }
    } else {
      url = line;
    }

    BmResult r = bm_add(db, url, (const char *const *)tags, n_tags);
    for (size_t i = 0; i < n_tags; i++)
      free(tags[i]);
    free(tags);

    if (r != BM_OK) {
      fprintf(stderr, "bm: %s:%zu: %s\n", path, lineno, bm_result_to_str(r));
      fclose(f);
      return BM_ERR_CORRUPT;
    }
  }

  if (ferror(f)) {
    fclose(f);
    return BM_ERR_IO;
  }
  fclose(f);
  return BM_OK;
}

/* ------------------------------------------------------------------ */
/* Save.                                                              */
/* ------------------------------------------------------------------ */

static int write_bookmark(FILE *f, const Bookmark *b) {
  if (fputs(b->url, f) == EOF)
    return -1;
  for (size_t i = 0; i < b->n_tags; i++) {
    if (fputc(i == 0 ? '\t' : ',', f) == EOF)
      return -1;
    if (fputs(b->tags[i], f) == EOF)
      return -1;
  }
  if (fputc('\n', f) == EOF)
    return -1;
  return 0;
}

BmResult storage_save(const BmDb *db, const char *path) {
  /* Atomic save: write to <path>.tmp, then rename. */
  char tmp[1024];
  int n = snprintf(tmp, sizeof tmp, "%s.tmp", path);
  if (n < 0 || (size_t)n >= sizeof tmp)
    return BM_ERR_IO;

  FILE *f = fopen(tmp, "w");
  if (!f)
    return BM_ERR_IO;

  for (size_t i = 0; i < db->cap; i++) {
    Bookmark *b = db->entries[i];
    if (!b || b == BM_TOMBSTONE)
      continue;
    if (write_bookmark(f, b) < 0) {
      fclose(f);
      remove(tmp);
      return BM_ERR_IO;
    }
  }

  if (fclose(f) != 0) {
    remove(tmp);
    return BM_ERR_IO;
  }
  if (rename(tmp, path) != 0) {
    remove(tmp);
    return BM_ERR_IO;
  }
  return BM_OK;
}
