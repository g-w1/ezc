#include <stdio.h>

int main() {
  extern long MaNgLe_another_test();
  long res = MaNgLe_another_test(1, 2, 3, 4, 5, 6);
  printf("%ld\n", res);
  return 0;
}
