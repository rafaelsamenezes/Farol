#include <stdlib.h>

#pragma once

// Irep are mostly just a few characters
#define MAX_STR_LENGTH 1024

typedef struct irep {
  size_t id;

  size_t *sub_expressions;
  size_t sub_expression_length;

  struct irep *named_sub_expressions;
  size_t named_sub_expression_length;

  struct irep *comment_sub_expressions;
  size_t comment_sub_expression_length;
} irep;


typedef struct irep_container {
  size_t length;
  char *strings;
  irep *ireps;
} ic;


ic create_irep_container(const size_t length);
void destroy_irep_container(ic container);

void parse_binary_file(ic *ic);
