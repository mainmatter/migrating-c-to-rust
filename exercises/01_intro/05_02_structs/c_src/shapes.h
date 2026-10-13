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

typedef int Fd;
