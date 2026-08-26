/*
 * CLI frontend: argv parsing and dispatch.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "bookmark.h"
#include "index.h"
#include "normalize.h"
#include "result.h"
#include "tag.h"
#include "util.h"

static void usage(FILE *f) {
  fputs("usage:\n"
        "  bm add  <url> [--tag <tag> ...]\n"
        "  bm del  <url>\n"
        "  bm find <query>\n"
        "  bm list\n"
        "  bm tag  <url> [+tag | -tag] ...\n"
        "\n"
        "BM_DB env var overrides the database path (default: bookmarks.bm).\n",
        f);
}

static const char *db_path(void) {
  const char *p = getenv("BM_DB");
  return p && *p ? p : "bookmarks.bm";
}

static void warn(const char *cmd, BmResult r) {
  fprintf(stderr, "bm %s: %s\n", cmd, bm_result_to_str(r));
}

/* ---- subcommands ---- */

static int cmd_add(BmDb *db, int argc, char **argv) {
  if (argc < 1) {
    usage(stderr);
    return 2;
  }
  const char *url = argv[0];

  char **tags = NULL;
  size_t n = 0, cap = 0;
  int rc = 0;

  for (int i = 1; i < argc; i++) {
    if (strcmp(argv[i], "--tag") != 0 || i + 1 >= argc) {
      fprintf(stderr, "bm add: unexpected argument: %s\n", argv[i]);
      rc = 2;
      goto done;
    }
    const char *raw = argv[++i];
    char buf[BM_MAX_TAG_LEN];
    BmResult r = tag_normalize(raw, buf, sizeof buf);
    if (r != BM_OK) {
      warn("add", r);
      rc = 1;
      goto done;
    }

    if (n == cap) {
      cap = cap ? cap * 2 : 4;
      char **nt = realloc(tags, cap * sizeof(char *));
      if (!nt) {
        warn("add", BM_ERR_IO);
        rc = 1;
        goto done;
      }
      tags = nt;
    }
    tags[n] = bm_strdup(buf);
    if (!tags[n]) {
      warn("add", BM_ERR_IO);
      rc = 1;
      goto done;
    }
    n++;
  }

  BmResult r = bm_add(db, url, (const char *const *)tags, n);
  if (r != BM_OK) {
    warn("add", r);
    rc = 1;
  }

done:
  for (size_t i = 0; i < n; i++)
    free(tags[i]);
  free(tags);
  return rc;
}

static int cmd_del(BmDb *db, int argc, char **argv) {
  if (argc != 1) {
    usage(stderr);
    return 2;
  }
  BmResult r = bm_del(db, argv[0]);
  if (r != BM_OK) {
    warn("del", r);
    return 1;
  }
  return 0;
}

static int cmd_find(BmDb *db, int argc, char **argv) {
  if (argc != 1) {
    usage(stderr);
    return 2;
  }

  BmCursor *c = bm_find(db, argv[0]);
  if (!c) {
    warn("find", BM_ERR_IO);
    return 1;
  }

  while (bm_cursor_next(c)) {
    const char *url = bm_cursor_url(c);
    const char **tags;
    size_t nt = bm_cursor_tags(c, &tags);

    fputs(url, stdout);
    for (size_t i = 0; i < nt; i++) {
      fputc(i == 0 ? '\t' : ',', stdout);
      fputs(tags[i], stdout);
    }
    fputc('\n', stdout);
  }
  bm_cursor_close(c);
  return 0;
}

static int cmd_list(BmDb *db, int argc, char **argv) {
  (void)argc;
  (void)argv;

  BmCursor *c = bm_list_tags(db);
  if (!c) {
    warn("list", BM_ERR_IO);
    return 1;
  }

  while (bm_cursor_next(c)) {
    printf("%s\t%zu\n", bm_cursor_url(c), bm_cursor_count(c));
  }
  bm_cursor_close(c);
  return 0;
}

static int cmd_tag(BmDb *db, int argc, char **argv) {
  if (argc < 2) {
    usage(stderr);
    return 2;
  }
  const char *url = argv[0];

  char **add = NULL, **rm = NULL;
  size_t na = 0, nr = 0;
  BmResult r = parse_tag_args(argc - 1, argv + 1, &add, &na, &rm, &nr);
  if (r != BM_OK) {
    warn("tag", r);
    return 1;
  }

  r = bm_tag(db, url, (const char *const *)add, na, (const char *const *)rm,
             nr);
  tag_list_free(add, na);
  tag_list_free(rm, nr);

  if (r != BM_OK) {
    warn("tag", r);
    return 1;
  }
  return 0;
}

/* ---- main ---- */

int main(int argc, char **argv) {
  if (argc < 2) {
    usage(stderr);
    return 2;
  }

  BmDb *db = NULL;
  BmResult r = bm_open(db_path(), &db);
  if (r != BM_OK) {
    fprintf(stderr, "bm: cannot open %s: %s\n", db_path(), bm_result_to_str(r));
    return 1;
  }

  const char *cmd = argv[1];
  int sub_argc = argc - 2;
  char **sub_argv = argv + 2;
  int rc;

  if (strcmp(cmd, "add") == 0)
    rc = cmd_add(db, sub_argc, sub_argv);
  else if (strcmp(cmd, "del") == 0)
    rc = cmd_del(db, sub_argc, sub_argv);
  else if (strcmp(cmd, "find") == 0)
    rc = cmd_find(db, sub_argc, sub_argv);
  else if (strcmp(cmd, "list") == 0)
    rc = cmd_list(db, sub_argc, sub_argv);
  else if (strcmp(cmd, "tag") == 0)
    rc = cmd_tag(db, sub_argc, sub_argv);
  else {
    fprintf(stderr, "bm: unknown command: %s\n", cmd);
    usage(stderr);
    rc = 2;
  }

  bm_close(db);
  return rc;
}
