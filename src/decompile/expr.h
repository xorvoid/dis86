#pragma once

typedef struct meh              meh_t;
typedef struct expr             expr_t;
typedef struct expr_operator1   expr_operator1_t;
typedef struct expr_operator2   expr_operator2_t;
typedef struct expr_operator3   expr_operator3_t;
typedef struct expr_abstract    expr_abstract_t;
typedef struct expr_branch_cond expr_branch_cond_t;
typedef struct expr_branch_flags expr_branch_flags_t;
typedef struct expr_branch      expr_branch_t;
typedef struct expr_call        expr_call_t;

enum {
  ADDR_TYPE_FAR,
  ADDR_TYPE_NEAR,
};

typedef struct addr addr_t;
struct addr
{
  int type;
  union {
    segoff_t far;
    u16      near;
  } u;
};

typedef struct operator operator_t;
struct operator
{
  const char * oper;
  int          sign;
};

enum {
  EXPR_KIND_UNKNOWN,
  EXPR_KIND_OPERATOR1,
  EXPR_KIND_OPERATOR2,
  EXPR_KIND_OPERATOR3,
  EXPR_KIND_ABSTRACT,
  EXPR_KIND_BRANCH_COND,
  EXPR_KIND_BRANCH_FLAGS,
  EXPR_KIND_BRANCH,
  EXPR_KIND_CALL,
};

struct expr_operator1
{
  operator_t   operator;
  value_t      dest;
};

struct expr_operator2
{
  operator_t   operator;
  value_t      dest;
  value_t      src;
};

struct expr_operator3
{
  operator_t   operator;
  value_t      dest;
  value_t      left;
  value_t      right;
};

struct expr_abstract
{
  const char * func_name;
  value_t      ret;
  u16          n_args;
  value_t      args[3];
};

struct expr_branch_cond
{
  operator_t   operator;
  value_t      left;
  value_t      right;
  u32          target;
};

struct expr_branch_flags
{
  const char * op; // FIXME
  value_t      flags;
  u32          target;
};

struct expr_branch
{
  u32 target;
};

struct expr_call
{
  addr_t       addr;
  bool         remapped;
  const char * name; // optional
};

struct expr
{
  int kind;
  union {
    expr_operator1_t    operator1[1];
    expr_operator2_t    operator2[1];
    expr_operator3_t    operator3[1];
    expr_abstract_t     abstract[1];
    expr_branch_cond_t  branch_cond[1];
    expr_branch_flags_t branch_flags[1];
    expr_branch_t       branch[1];
    expr_call_t         call[1];
  } k;

  size_t          n_ins;
  dis86_instr_t * ins;
};

#define EXPR_MAX 4096
struct meh
{
  size_t expr_len;
  expr_t expr_arr[EXPR_MAX];
};

meh_t * meh_new(config_t *cfg, symbols_t *symbols, dis86_instr_t *ins, size_t n_ins);
void    meh_delete(meh_t *m);
