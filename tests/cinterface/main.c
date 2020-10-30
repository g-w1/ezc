#include <stdio.h>

extern unsigned long long Fib_rec();
extern unsigned long long Factorial();

int main() {
  for (int i = 1; i <= 30; i++) {
    printf("fac(%d)=%lld ", i, Factorial(i));
    printf("fib(%d)=%lld \n", i, Fib_rec(i));
  }
  return 0;
}
