#include <stdio.h>

int main() {
  extern long MaNgLe_another_test();
  printf("%ld\n", MaNgLe_another_test(1, 11, 11, 12, 12));
  return 0;
}
