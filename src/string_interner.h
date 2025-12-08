#ifndef STRING_INTERNER_H
#define STRING_INTERNER_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
  char *str;
  size_t length;
} intern_entry;

typedef struct {
  intern_entry *entries;
  size_t capacity;
  size_t length;
} string_interner;

uint64_t string_interner_tests();

string_interner *interner_create(void);
void interner_destroy(string_interner *it);
uint64_t interner_intern(string_interner *it, const char *key);

#ifdef STRING_INTERNER_IMPL

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

string_interner *interner_create(void) {
  string_interner *it = (string_interner *)malloc(sizeof(*it));
  if (!it)
    return NULL;
  it->capacity = 16;
  it->length = 0;
  it->entries = (intern_entry *)calloc(16, sizeof(intern_entry));
  return it->entries ? it : (free(it), (string_interner *)NULL);
}

void interner_destroy(string_interner *it) {
  for (size_t i = 0; i < it->capacity; ++i)
    free((char *)it->entries[i].str);
  free(it->entries);
  free(it);
}

uint64_t interner_intern(string_interner *it, const char *key) {
  // Do we have this string? Maybe this is a good place for a bloom filter
  for (int i = 0; i < it->length; i++) {
    if (strcmp(key, it->entries[i].str) == 0)
      return i;
  }

  if (it->capacity == it->length) {
    it->capacity *= 2;
    it->entries = (intern_entry *)realloc(it->entries,
                                          sizeof(intern_entry) * it->capacity);
  }

  size_t length = strlen(key);
  it->entries[it->length].str = (char *)malloc(length + 1);
  it->entries[it->length].length = length;
  memcpy(it->entries[it->length].str, key, length + 1);

  return it->length++;
}

uint64_t string_interner_tests() {
  uint64_t errors = 0;

  printf("String interner suite...\n");

  {
    printf("- Initial state... ");
    string_interner *test = interner_create();

    if (test->length) {
      printf("FAIL\n");
      errors++;
    } else
      printf("OK\n");

    interner_destroy(test);
  }

  {
    printf("- First string... ");
    string_interner *test = interner_create();

    uint64_t id = interner_intern(test, "hello");

    if (id != 0) {
      printf("FAIL\n");
      errors++;
    } else {
      printf("OK\n");
    }

    interner_destroy(test);
  }

  {
    printf("- A few strings... ");
    string_interner *test = interner_create();

    uint64_t id = interner_intern(test, "hello");
    uint64_t id2 = interner_intern(test, "hello");

    if (id != 0 || id != id2) {
      printf("FAIL\n");
      errors++;
    } else {
      printf("OK\n");
    }

    interner_destroy(test);
  }

  {
    printf("- Expanding... ");
    string_interner *test = interner_create();

    char str[] = "a";
    for (int i = 0; i < 64; i++) {
      interner_intern(test, str);
      str[0]++;
    }

    _Bool check1 = test->length == 64;

    if (!check1) {
      printf("FAIL\n");
      errors++;
    } else {
      printf("OK\n");
    }

    interner_destroy(test);
  }

  return errors;
}

#endif
#endif
