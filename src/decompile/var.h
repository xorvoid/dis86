#pragma once

enum {
  VAR_TYPE_PARAM,
  VAR_TYPE_LOCAL,
  VAR_TYPE_GLOBAL,
};

typedef struct variable variable_t;
struct variable
{
  int type;
  u16 num;
};

bool   variable_deduce(variable_t *v, operand_mem_t *mem);
char * variable_name(variable_t *v, char *buf, size_t buf_sz);

typedef struct symtab symtab_t;

symtab_t * symtab_new(void);
void       symtab_delete(symtab_t *s);

variable_t * symtab_lookup_or_create(symtab_t *s, operand_mem_t *mem, bool *_created);
