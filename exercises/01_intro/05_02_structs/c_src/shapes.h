#pragma once

#include <stdint.h>

struct Point {
  double x;
  double y;
};

struct Shape {
  uint8_t kind;
  char name[16];
  struct Point origin;
  const struct Shape *parent; /* may be NULL */
  uint32_t id;
};

#pragma pack(1)
struct WireHeader {
  uint8_t version;
  uint32_t len;
};
#pragma pack()

typedef int Fd;
