#include <stdio.h>

#define STRING_INTERNER_IMPL
#include "string_interner.h"

int main(int argc, char **argv) {
  printf("Hello Farol!\n");

  printf("Running tests\n");
  string_interner_tests();
  
  return 0;
}
