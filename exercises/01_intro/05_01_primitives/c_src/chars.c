#include "chars.h"

char to_lower(char byte) {
  unsigned char b = (unsigned char)byte;
  if (b >= 'A' && b <= 'Z') {
    b += 32;
  }
  return (char)b;
}

int is_letter(char byte) {
  unsigned char b = (unsigned char)byte;
  return (b >= 'a' && b <= 'z') || (b >= 'A' && b <= 'Z');
}

void lower_bytes(char *bytes, size_t len) {
  for (size_t i = 0; i < len; i++) {
    bytes[i] = to_lower(bytes[i]);
  }
}
