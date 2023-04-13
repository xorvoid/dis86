#include "decompile_private.h"
#include <stdalign.h>

static u16 size_in_bytes(int sz)
{
  switch (sz) {
    case SIZE_8:  return 1;
    case SIZE_16: return 2;
    case SIZE_32: return 4;
    default: FAIL("Unknown size type");
  }
}

bool sym_deduce(sym_t *s, operand_mem_t *m)
{
  i16 off = (i16)m->off;
  u16 len = size_in_bytes(m->sz);

  // Global?
  if (m->sreg == REG_DS && !m->reg1 && !m->reg2) {
    s->kind = SYM_KIND_GLOBAL;
    s->off = off;
    s->len = len;
    s->name = NULL;
    return true;
  }

  // Local var?
  if (m->sreg == REG_SS && m->reg1 == REG_BP && !m->reg2) {
    if (off < 0) {
      s->kind = SYM_KIND_LOCAL;
      s->off = off;
      s->len = len;
      s->name = NULL;
    } else {
      s->kind = SYM_KIND_PARAM;
      s->off = off;
      s->len = len;
      s->name = NULL;
    }
    return true;
  }

  return false;
}

size_t sym_size_bytes(sym_t *s)
{
  return s->len;
}

const char * sym_name(sym_t *s, char *buf, size_t buf_sz)
{
  if (s->name) {
    return s->name;
  }

  switch (s->kind) {
    case SYM_KIND_PARAM: {
      snprintf(buf, buf_sz, "_param_%04x", (u16)s->off);
    } break;
    case SYM_KIND_LOCAL: {
      snprintf(buf, buf_sz, "_local_%04x", (u16)-s->off);
    } break;
    case SYM_KIND_GLOBAL: {
      snprintf(buf, buf_sz, "G_data_%04x", (u16)s->off);
    } break;
    default: FAIL("Unknown sym kind: %d", s->kind);
  }
  return buf;
}

static bool sym_overlaps(sym_t *a, sym_t *b)
{
  // WLOG: Let a->off <= b->off
  if (b->off < a->off) {
    sym_t *tmp = a;
    a = b;
    b = tmp;
  }

  i16 end = (i16)a->off + a->len;

  return
    a->kind == b->kind &&
    b->off < end;
}

symbols_t * symbols_new(void)
{
  symbols_t *s = calloc(1, sizeof(symbols_t));
  s->globals = symtab_new();
  s->params  = symtab_new();
  s->locals  = symtab_new();
  return s;
}

void symbols_delete(symbols_t *s)
{
  symtab_delete(s->globals);
  symtab_delete(s->params);
  symtab_delete(s->locals);
}

#define SYMTAB_MAX_SIZE 128

struct symtab
{
  // TODO: REPLACE WITH A HASHTABLE
  size_t n_var;
  sym_t var[SYMTAB_MAX_SIZE];
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

static void symtab_add_merge(symtab_t *s, sym_t *sym)
{
  for (size_t i = 0; i < s->n_var; i++) {
    sym_t *cand = &s->var[i];
    if (!sym_overlaps(sym, cand)) continue;

    // Overlaps: grow to encapsulate both!

    i16 new_start = MIN(sym->off, cand->off);
    i16 new_end   = MAX(sym->off + sym->len, cand->off + cand->len);
    int new_len   = new_end - new_start;

    // Update sym
    sym->off = new_start;
    sym->len = new_len;

    // Remove the candidate (avoid duplicates)
    s->var[i] = s->var[--s->n_var];
    i--;
  }

  assert(s->n_var < ARRAY_SIZE(s->var));
  s->var[s->n_var++] = *sym;
}

static sym_t * symtab_find(symtab_t *s, sym_t *deduced_sym)
{
  for (size_t i = 0; i < s->n_var; i++) {
    sym_t *cand = &s->var[i];
    if (sym_overlaps(deduced_sym, cand)) return cand;
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

sym_t * symtab_iter_next(symtab_iter_t *_it)
{
  iter_impl_t *it = (iter_impl_t*)_it;
  if (it->idx >= it->s->n_var) return NULL;
  return &it->s->var[it->idx++];
}

bool symbols_insert_deduced(symbols_t *s, sym_t *deduced_sym)
{
  switch (deduced_sym->kind) {
    case SYM_KIND_PARAM:  symtab_add_merge(s->params,  deduced_sym); break;
    case SYM_KIND_LOCAL:  symtab_add_merge(s->locals,  deduced_sym); break;
    case SYM_KIND_GLOBAL: {
      // Globals are special in that we don't merge them in. We require that globals
      // are set up via a config file. So, here, we simply verify that our deduced
      // symbol cooresponds to some pre-configured global
      if (!symtab_find(s->globals, deduced_sym)) {
        //static char buf[128];
        //const char *name = sym_name(deduced_sym, buf, ARRAY_SIZE(buf));
        //FAIL("Failed to find global for '%s'", name);
        return false;
      }
    } break;
    default: FAIL("Unknown symbol kind: %d", deduced_sym->kind);
  }

  return true;
}

sym_t * symbols_find(symbols_t *s, operand_mem_t *mem)
{
  sym_t deduced_sym[1];
  if (!sym_deduce(deduced_sym, mem)) return NULL;

  switch (deduced_sym->kind) {
    case SYM_KIND_PARAM:  return symtab_find(s->params,  deduced_sym);
    case SYM_KIND_LOCAL:  return symtab_find(s->locals,  deduced_sym);
    case SYM_KIND_GLOBAL: return symtab_find(s->globals, deduced_sym);
    default: FAIL("Unknown symbol kind: %d", deduced_sym->kind);
  }
}

void symbols_add_global(symbols_t *s, const char *name, u16 offset, u16 len)
{
  sym_t sym[1] = {{}};
  sym->kind = SYM_KIND_GLOBAL;
  sym->off  = (i16)offset;
  sym->len  = len;
  sym->name = name;

  symtab_t *symtab = s->globals;
  assert(symtab->n_var < ARRAY_SIZE(symtab->var));
  symtab->var[symtab->n_var++] = *sym;
}
