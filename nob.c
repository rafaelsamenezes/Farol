#define NOB_IMPLEMENTATION
#include "nob.h"

#define BUILD_FOLDER "build/"
#define SRC_FOLDER "src/"

#define STRING_INTERNER_IMPL
#include "src/string_interner.h"


uint64_t run_tests() {
  uint64_t errors = 0;
  errors += string_interner_tests();
  return errors;
}


int main(int argc, char **argv) {
  NOB_GO_REBUILD_URSELF(argc, argv);

  if (argc == 2 && !strcmp(argv[1], "test")) {
    uint64_t errors = run_tests();
    printf("\nIdentified %lu failures\n", errors);
    return errors;
  }
  
  if (!nob_mkdir_if_not_exists(BUILD_FOLDER))
    return 1;
  Nob_Cmd cmd = {0};

  nob_cmd_append(&cmd, "cc", "-Wall", "-Wextra", "-o", BUILD_FOLDER "farol",
                 SRC_FOLDER "main.c");

  if (!nob_cmd_run(&cmd))
    return 1;
}
