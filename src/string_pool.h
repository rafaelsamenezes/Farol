#pragma once

#include <stdint.h>
typedef struct {
  char *strings;
  uint32_t reserved;
  uint32_t length;
} string_pool;

/**
 * Initializes a new string pool.
 *
 * @return A new `string_pool` structure with allocated memory for storing strings.
 */
string_pool initialize_string_pool();

/**
 * Adds a string to the given string pool.
 *
 * @param pool A pointer to the `string_pool` structure where the string will be added.
 * @param str The null-terminated string to add to the pool.
 * @return The ID of the added string in the pool, or an error code if the pool is full.
 */
uint32_t add_string_pool(string_pool * const pool, const char *const str);


/**
 * Retrieves a string from the string pool by its index.
 *
 * @param pool A pointer to the string pool structure.
 * @param id The index of the string to retrieve.
 * @return A pointer to the retrieved string, or NULL if the index is out of bounds.
 */
char *get_string_pool(const string_pool *const pool, uint32_t id);


/**
 * Frees the memory allocated for a string pool.
 *
 * @param pool A pointer to the string pool structure to be freed.
 */
void free_string_pool(string_pool * const pool);


