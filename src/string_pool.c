#include "string_pool.h"

#include <assert.h>
#include <stdlib.h>
#include <string.h>


// NOTE: CBMC/ESBMC lets the user define any string of arbitrary size.
// This does not seems performatic and neither practical. Let's just
// do a big allocation and call it a day.

#define MAX_STR_LEN 128

// 4MB of strings oughta be enough 
#define STR_POOL_ALLOC 4 * 1024 * 1024
#define STR_POOL_LENGTH STR_POOL_ALLOC / MAX_STR_LEN

string_pool initialize_string_pool() {
  return (string_pool) {.strings=malloc(STR_POOL_ALLOC), .reserved=STR_POOL_LENGTH, .length = 0};
}

uint32_t add_string_pool(string_pool * const pool, const char *const str) {
  assert(pool->length < pool->reserved);
  strncpy(&pool->strings[MAX_STR_LEN * pool->length], str, MAX_STR_LEN);
  return pool->length++;
}

char *get_string_pool(const string_pool *const pool, uint32_t id) {
  assert(id < pool->length );
  return &pool->strings[MAX_STR_LEN*id];
}

void free_string_pool(string_pool *const pool) {
  free(pool->strings);
  pool->reserved = 0;
  pool->length = 0;
}

#include <stdio.h>
void string_pool_test() {
  string_pool pool = initialize_string_pool();

  const char str1[] = "My str 1";
  const char str2[] = "My str 2";
  const char str3[] = "My str 3";

  printf("[string_pool] Checking addition\n");
  assert(add_string_pool(&pool, str1) == 0);
  assert(add_string_pool(&pool, str3) == 1);
  assert(add_string_pool(&pool, str2) == 2);

  printf("[string_pool] Checking elements\n");

  printf("[string_pool] %s\n", get_string_pool(&pool, 0));
  printf("[string_pool] %s\n", get_string_pool(&pool, 1));
  printf("[string_pool] %s\n", get_string_pool(&pool, 2));
  assert(!strcmp(str1, get_string_pool(&pool, 0)));
  assert(!strcmp(str3, get_string_pool(&pool, 1)));
  assert(!strcmp(str2, get_string_pool(&pool, 2)));
  
  free_string_pool(&pool); 
}
