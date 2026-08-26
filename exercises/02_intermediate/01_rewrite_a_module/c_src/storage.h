#pragma once

#include "result.h"

typedef struct BmDb BmDb;

/*
 * On-disk persistence.
 *
 * One bookmark per line:
 *
 *     <url>\t<tag1>,<tag2>,...\n
 *
 * No header, no length prefix. Missing file == empty database.
 */

BmResult storage_load(const char *path, BmDb *db);
BmResult storage_save(const BmDb *db, const char *path);
