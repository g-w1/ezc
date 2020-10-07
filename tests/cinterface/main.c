#include <stdio.h>

int main() {
  extern unsigned long long MaNgLe_fib();
  for (int i = 1; i <= 50; i++) {
    printf("%lld ", MaNgLe_fib(i));
  }
  printf("\n");
  return 0;
}
