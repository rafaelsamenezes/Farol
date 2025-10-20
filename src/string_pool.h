#pragma once

#include <stdint.h>
typedef struct {
  char *strings;
  uint32_t reserved;
  uint32_t length;
} string_pool;

string_pool initialize_string_pool();
uint32_t add_string_pool(string_pool * const pool, const char *const str);
char* get_string_pool(const string_pool * const pool, uint32_t id);
void free_string_pool(string_pool * const pool);

void string_pool_test();

