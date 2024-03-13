#pragma once

#define NAME_MAX 128

typedef struct sym         sym_t;
typedef struct symref      symref_t;
typedef struct symbols     symbols_t;
typedef struct symtab      symtab_t;
typedef struct symtab_iter symtab_iter_t;

struct __attribute__((aligned(16))) symtab_iter { char _opaque[32]; };

enum {
  SYM_KIND_REGISTER,
  SYM_KIND_PARAM,
  SYM_KIND_LOCAL,
  SYM_KIND_GLOBAL,
};

struct sym
{
  int          kind;
  i16          off;
  u16          len;              // in bytes
  const char * name;             // optional (default name is constructed otherwise)
};

bool         sym_deduce(sym_t *v, operand_mem_t *mem);
bool         sym_deduce_reg(sym_t *sym, int reg_id);
const char * sym_name(sym_t *v, char *buf, size_t buf_sz);
size_t       sym_size_bytes(sym_t *v);

struct symbols
{
  symtab_t * registers;
  symtab_t * globals;
  symtab_t * params;
  symtab_t * locals;
};

symbols_t * symbols_new(void);
void        symbols_delete(symbols_t *s);
bool        symbols_insert_deduced(symbols_t *s, sym_t *deduced_sym);
symref_t    symbols_find_ref(symbols_t *s, sym_t *deduced_sym);
symref_t    symbols_find_mem(symbols_t *s, operand_mem_t *mem);
symref_t    symbols_find_reg(symbols_t *s, int reg_id);
void        symbols_add_global(symbols_t *s, const char *name, u16 offset, u16 len);

struct symref
{
  sym_t * symbol;  // NULL if the ref doesn't point anywhere
  u16     off;     // offset into this symbol
  u16     len;     // length from the offset
};

bool symref_matches(symref_t *a, symref_t *b);

symtab_t * symtab_new(void);
void       symtab_delete(symtab_t *s);

void    symtab_iter_begin(symtab_iter_t *it, symtab_t *s);
sym_t * symtab_iter_next(symtab_iter_t *it);
