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

#include <assert.h>


void test_initialize_string_pool() {
    string_pool pool = initialize_string_pool();
    assert(pool.strings != NULL);
    assert(pool.reserved == STR_POOL_LENGTH);
    assert(pool.length == 0);
    free_string_pool(&pool);
}

void test_add_string_pool() {
    string_pool pool = initialize_string_pool();
    const char str1[] = "My str 1";
    uint32_t id = add_string_pool(&pool, str1);
    assert(id == 0);
    assert(strcmp(get_string_pool(&pool, id), str1) == 0);
    free_string_pool(&pool);
}

void test_get_string_pool() {
    string_pool pool = initialize_string_pool();
    const char str1[] = "My str 1";
    add_string_pool(&pool, str1);
    assert(strcmp(get_string_pool(&pool, 0), str1) == 0);
    free_string_pool(&pool);
}

void test_free_string_pool() {
    string_pool pool = initialize_string_pool();
    const char str1[] = "My str 1";
    add_string_pool(&pool, str1);
    free_string_pool(&pool);
    assert(pool.strings == NULL);
    assert(pool.reserved == 0);
    assert(pool.length == 0);
}

void run_tests() {
    test_initialize_string_pool();
    test_add_string_pool();
    test_get_string_pool();
    test_free_string_pool();
}

void run_buffer_integration_tests() {
    string_pool pool = initialize_string_pool();
    
    const char *str1 = "My str 1";
    const char *str2 = "My str 2";
    const char *str3 = "My str 3";

    uint32_t id1 = add_string_pool(&pool, str1);
    uint32_t id2 = add_string_pool(&pool, str2);
    uint32_t id3 = add_string_pool(&pool, str3);

    assert(id1 == 0);
    assert(id2 == 1);
    assert(id3 == 2);

    assert(strcmp(get_string_pool(&pool, id1), str1) == 0);
    assert(strcmp(get_string_pool(&pool, id2), str2) == 0);
    assert(strcmp(get_string_pool(&pool, id3), str3) == 0);

    free_string_pool(&pool);
}

#include "test.h"
void string_pool_harness() {
    // Initialize variables
    string_pool pool = initialize_string_pool();
    
    // Nondeterministic input strings
    char str1[50];
    str1[49] = '\0';

    char str2[50];
    str2[49] = '\0';

    char str3[50];
    str3[49] = '\0';

    // Nondeterministic string pool operations
    uint32_t id1 = add_string_pool(&pool, str1);
    uint32_t id2 = add_string_pool(&pool, str2);
    uint32_t id3 = add_string_pool(&pool, str3);

    // Assume valid indices
    __VERIFIER_assume(id1 < pool.reserved);
    __VERIFIER_assume(id2 < pool.reserved);
    __VERIFIER_assume(id3 < pool.reserved);

    // Retrieve and assert strings
    char *retrieved_str1 = get_string_pool(&pool, id1);
    char *retrieved_str2 = get_string_pool(&pool, id2);
    char *retrieved_str3 = get_string_pool(&pool, id3);

    __VERIFIER_assert(strcmp(retrieved_str1, str1) == 0);
    __VERIFIER_assert(strcmp(retrieved_str2, str2) == 0);
    __VERIFIER_assert(strcmp(retrieved_str3, str3) == 0);

    // Free the string pool
    free_string_pool(&pool);
}

