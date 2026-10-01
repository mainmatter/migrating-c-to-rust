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
