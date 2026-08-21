#pragma once

#include <stddef.h>

#include "result.h"

/*
 * Split argv-style "+rust" / "-python" words into add/rm lists. Default
 * (no prefix) is add. Output arrays and inner strings are heap-allocated;
 * free with tag_list_free.
 */
BmResult parse_tag_args(int argc, char **argv, char ***out_add, size_t *n_add,
                        char ***out_rm, size_t *n_rm);

void tag_list_free(char **tags, size_t n);

/* Append unique tags from src into *dst. */
BmResult tag_set_union(char ***dst, size_t *n_dst, const char *const *src,
                       size_t n_src);

/* Remove tags listed in src from *dst, in place. */
BmResult tag_set_diff(char ***dst, size_t *n_dst, const char *const *src,
                      size_t n_src);
