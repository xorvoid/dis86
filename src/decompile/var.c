#include "decompile_private.h"
#include <stdalign.h>

bool variable_deduce(variable_t *v, operand_mem_t *m)
{
  i16 off = (i16)m->off;

  // Global?
  if (m->sreg == REG_DS && !m->reg1 && !m->reg2) {
    v->type = VAR_TYPE_GLOBAL;
    v->num = (u16)off;
    return true;
  }

  // Local var?
  if (m->sreg == REG_SS && m->reg1 == REG_BP && !m->reg2) {
    if (off < 0) {
      v->type = VAR_TYPE_LOCAL;
      v->num = (u16)-off;
    } else {
      v->type = VAR_TYPE_PARAM;
      v->num = (u16)off;
    }
    return true;
  }

  return false;
}

char * variable_name(variable_t *v, char *buf, size_t buf_sz)
{
  switch (v->type) {
    case VAR_TYPE_PARAM: {
      snprintf(buf, buf_sz, "_param_%04x", v->num);
    } break;
    case VAR_TYPE_LOCAL: {
      snprintf(buf, buf_sz, "_local_%04x", v->num);
    } break;
    case VAR_TYPE_GLOBAL: {
      snprintf(buf, buf_sz, "G_data_%04x", v->num);
    } break;
    default: FAIL("Unknown variable type: %d", v->type);
  }
  return buf;
}

static bool variable_match(variable_t *a, variable_t *b)
{
  return
    a->type == b->type &&
    a->num == b->num;
}

#define MAX_VARIABLES 128

struct symtab
{
  // TODO: REPLACE WITH A HASHTABLE
  size_t n_var;
  variable_t var[MAX_VARIABLES];
};

symtab_t * symtab_new(void)
{
  symtab_t *s = calloc(1, sizeof(symtab_t));
  s->n_var = 0;

  return s;
}

void symtab_delete(symtab_t *s)
{
  free(s);
}

variable_t * symtab_lookup_or_create(symtab_t *s, operand_mem_t *mem, bool *_created)
{
  if (_created) *_created = false;

  variable_t var[1];
  if (!variable_deduce(var, mem)) return NULL;

  for (size_t i = 0; i < s->n_var; i++) {
    variable_t *cand = &s->var[i];
    if (variable_match(var, cand)) return cand;
  }

  // not found, create it
  assert(s->n_var < ARRAY_SIZE(s->var));
  s->var[s->n_var++] = *var;

  if (_created) *_created = true;
  return &s->var[s->n_var-1];
}

typedef struct iter_impl iter_impl_t;
struct __attribute__((aligned(16))) iter_impl
{
  symtab_t * s;
  size_t     idx;
  char       _extra[16];
};
static_assert(sizeof(iter_impl_t) == sizeof(symtab_iter_t), "");
static_assert(alignof(iter_impl_t) == alignof(symtab_iter_t), "");

void symtab_iter_begin(symtab_iter_t *_it, symtab_t *s)
{
  iter_impl_t *it = (iter_impl_t*)_it;
  it->s = s;
  it->idx = 0;
}

variable_t * symtab_iter_next(symtab_iter_t *_it)
{
  iter_impl_t *it = (iter_impl_t*)_it;
  if (it->idx >= it->s->n_var) return NULL;
  return &it->s->var[it->idx++];
}
