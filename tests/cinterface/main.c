#include <stdio.h>

static int res[45];

int main() {
  extern unsigned long long MaNgLe_Fib_rec();
  extern unsigned long long MaNgLe_Factorial();
  for (int i = 1; i <= 30; i++) {
    printf("fac(%d)=%lld ", i, MaNgLe_Factorial(i));
    printf("fib(%d)=%lld \n", i, MaNgLe_Fib_rec(i));
  }
  return 0;
}
