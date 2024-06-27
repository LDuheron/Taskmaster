#include <signal.h>
#include <stdio.h>

void handler(int sig) { printf("sig = %d\n", sig); }

int main(void) {
  signal(3, handler);
  signal(9, handler);
  signal(15, handler);
  while (1)
    ;
  return 0;
}
