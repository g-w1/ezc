#include <stdio.h>

int main() {
  extern unsigned long long MaNgLe_fib();
  for (int i = 1; i <= 50; i++) {
    printf("fib(%d)=%lld ", i, MaNgLe_fib(i));
  }
  return 0;
}
