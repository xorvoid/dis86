#pragma once

typedef struct variable    variable_t;
typedef struct symtab      symtab_t;
typedef struct symtab_iter symtab_iter_t;

struct __attribute__((aligned(16))) symtab_iter { char _opaque[32]; };

enum {
  VAR_TYPE_PARAM,
  VAR_TYPE_LOCAL,
  VAR_TYPE_GLOBAL,
};

struct variable
{
  int type;
  i16 off;
  int sz;
};

bool   variable_deduce(variable_t *v, operand_mem_t *mem);
char * variable_name(variable_t *v, char *buf, size_t buf_sz);
size_t variable_size_bytes(variable_t *v);

symtab_t * symtab_new(void);
void       symtab_delete(symtab_t *s);

bool         symtab_add(symtab_t *s, operand_mem_t *mem);
variable_t * symtab_find(symtab_t *s, operand_mem_t *mem);

void         symtab_iter_begin(symtab_iter_t *it, symtab_t *s);
variable_t * symtab_iter_next(symtab_iter_t *it);
