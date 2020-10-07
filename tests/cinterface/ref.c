#include <stdio.h>

int fib(int n) {
  int counter = 1;
  int a = 1;
  int b = 0;
  while (1) {
    a += b;
    counter++;
    if (counter > n) {
      return a;
    }
    b += a;
    counter++;
    if (counter > n) {
      return b;
    }
  }
}

int main() {
  for (int i = 1; i <= 50; i++) {
    printf("%d ", fib(i));
  }
  printf("\n");
  return 0;
}
