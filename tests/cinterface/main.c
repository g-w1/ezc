#include <stdio.h>

static int res[45];

int main() {
  extern unsigned long long MaNgLe_Fib_rec();
  for (int i = 1; i <= 45; i++) {
    printf("fib(%d)=%lld \n", i, MaNgLe_Fib_rec(i));
  }
  return 0;
}
