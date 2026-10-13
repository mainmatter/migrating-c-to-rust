#include "bm_version.h"

#include <assert.h>

int main(void) {
  assert(bm_version() == 1);
  return 0;
}
