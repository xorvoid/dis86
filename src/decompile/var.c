#include "decompile_private.h"
#include <stdalign.h>

static size_t size_in_bytes(int sz)
{
  switch (sz) {
    case SIZE_8:  return 1;
    case SIZE_16: return 2;
    case SIZE_32: return 4;
    default: FAIL("Unknown size type");
  }
}

static int bytes_to_size(size_t n)
{
  switch (n) {
    case 1: return SIZE_8;
    case 2: return SIZE_16;
    case 4: return SIZE_32;
    default: FAIL("Cannot reepresent %zu bytes as a size type", n);
  }
}

bool variable_deduce(variable_t *v, operand_mem_t *m)
{
  i16 off = (i16)m->off;

  // Global?
  if (m->sreg == REG_DS && !m->reg1 && !m->reg2) {
    v->type = VAR_TYPE_GLOBAL;
    v->off = off;
    v->sz = m->sz;
    return true;
  }

  // Local var?
  if (m->sreg == REG_SS && m->reg1 == REG_BP && !m->reg2) {
    if (off < 0) {
      v->type = VAR_TYPE_LOCAL;
      v->off = off;
      v->sz  = m->sz;
    } else {
      v->type = VAR_TYPE_PARAM;
      v->off = off;
      v->sz  = m->sz;
    }
    return true;
  }

  return false;
}

size_t variable_size_bytes(variable_t *v)
{
  return size_in_bytes(v->sz);
}

char * variable_name(variable_t *v, char *buf, size_t buf_sz)
{
  switch (v->type) {
    case VAR_TYPE_PARAM: {
      snprintf(buf, buf_sz, "_param_%04x", (u16)v->off);
    } break;
    case VAR_TYPE_LOCAL: {
      snprintf(buf, buf_sz, "_local_%04x", (u16)-v->off);
    } break;
    case VAR_TYPE_GLOBAL: {
      snprintf(buf, buf_sz, "G_data_%04x", (u16)v->off);
    } break;
    default: FAIL("Unknown variable type: %d", v->type);
  }
  return buf;
}

static bool variable_overlaps(variable_t *a, variable_t *b)
{
  // WLOG: Let a->off <= b->off
  if (b->off < a->off) {
    variable_t *tmp = a;
    a = b;
    b = tmp;
  }

  i16 end = (i16)a->off + size_in_bytes(a->sz);

  return
    a->type == b->type &&
    b->off < end;
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

bool symtab_add(symtab_t *s, operand_mem_t *mem)
{
  variable_t var[1];
  if (!variable_deduce(var, mem)) return false;

  for (size_t i = 0; i < s->n_var; i++) {
    variable_t *cand = &s->var[i];
    if (!variable_overlaps(var, cand)) continue;

    // Overlaps: grow to encapsulate both!

    i16 new_start = MIN(var->off, cand->off);
    i16 new_end   = MAX(var->off + size_in_bytes(var->sz), cand->off + size_in_bytes(cand->sz));
    int new_sz = bytes_to_size(new_end - new_start);

    // Update var
    var->off = new_start;
    var->sz  = new_sz;

    // Remove the candidate (avoid duplicates)
    s->var[i] = s->var[--s->n_var];
    i--;
  }

  assert(s->n_var < ARRAY_SIZE(s->var));
  s->var[s->n_var++] = *var;
  return true;
}

variable_t * symtab_find(symtab_t *s, operand_mem_t *mem)
{
  variable_t var[1];
  if (!variable_deduce(var, mem)) return NULL;

  for (size_t i = 0; i < s->n_var; i++) {
    variable_t *cand = &s->var[i];
    if (variable_overlaps(var, cand)) return cand;
  }

  return NULL;
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
