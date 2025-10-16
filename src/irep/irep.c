#include "irep.h"
#include <assert.h>


/******************/
/* IREP UTILITIES */
/******************/

ic create_irep_container(const size_t length) {
  ic container = {length, calloc(length, sizeof(char)* MAX_STR_LENGTH), calloc(length, sizeof(irep))};
  assert(container.strings);
  assert(container.ireps);
  return container;
}

void destroy_irep_container(ic container) {
  free(container.strings);
  for (size_t i = 0; i < container.length; i++) {
    irep ir = container.ireps[i];
    if(ir.named_sub_expression_length)
      free(container.ireps[i].named_sub_expressions);
    if(ir.comment_sub_expression_length)
      free(container.ireps[i].comment_sub_expressions);
  }  
}

/***************/
/* IREP READER */
/***************/





/****************/
/* IREP WRITTER */
/****************/
