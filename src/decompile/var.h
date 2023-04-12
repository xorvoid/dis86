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
  u16 num;
};

bool   variable_deduce(variable_t *v, operand_mem_t *mem);
char * variable_name(variable_t *v, char *buf, size_t buf_sz);

symtab_t * symtab_new(void);
void       symtab_delete(symtab_t *s);

variable_t * symtab_lookup_or_create(symtab_t *s, operand_mem_t *mem, bool *_created);

void         symtab_iter_begin(symtab_iter_t *it, symtab_t *s);
variable_t * symtab_iter_next(symtab_iter_t *it);
