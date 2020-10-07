#include <stdio.h>

int main() {
  //  extern long MaNgLe_another_test();
  extern unsigned long long MaNgLe_fib();
  for (int i = 1; i <= 10; i++) {
    printf("%lld ", MaNgLe_fib(i));
  }
  printf("\n");
  return 0;
}
