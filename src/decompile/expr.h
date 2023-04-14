#pragma once

typedef struct meh              meh_t;
typedef struct expr             expr_t;
typedef struct expr_operator    expr_operator_t;
typedef struct expr_operator3   expr_operator3_t;
typedef struct expr_function    expr_function_t;
typedef struct expr_literal     expr_literal_t;
typedef struct expr_branch_cond expr_branch_cond_t;
typedef struct expr_branch      expr_branch_t;
typedef struct expr_call        expr_call_t;
typedef struct expr_lea         expr_lea_t;

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

enum {
  EXPR_KIND_UNKNOWN,
  EXPR_KIND_OPERATOR,
  EXPR_KIND_OPERATOR3,
  EXPR_KIND_FUNCTION,
  EXPR_KIND_LITERAL,
  EXPR_KIND_BRANCH_COND,
  EXPR_KIND_BRANCH,
  EXPR_KIND_CALL,
  EXPR_KIND_LEA,
};

struct expr_operator
{
  // TODO: REMOVE dis86 instr operands
  const char * operator;
  operand_t    oper1;           // required
  operand_t    oper2;           // optional
};

struct expr_operator3
{
  const char * operator;
  int          sign;
  value_t      dest;
  value_t      left;
  value_t      right;
};

struct expr_function
{
  // TODO: REMOVE dis86 instr operands
  const char * func_name;
  value_t      ret;
  u16          n_args;
  value_t      args[3];
};

struct expr_literal
{
  const char *text;
};

struct expr_branch_cond
{
  const char * operator;
  int          signed_cmp;
  value_t      left;
  value_t      right;
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

struct expr_lea
{
  // TODO: REMOVE dis86 instr operands
  value_t dest;               // required
  int     addr_base_reg;      // required
  u16     addr_offset;        // required
};

struct expr
{
  int kind;
  union {
    expr_operator_t    operator[1];
    expr_operator3_t   operator3[1];
    expr_function_t    function[1];
    expr_literal_t     literal[1];
    expr_branch_cond_t branch_cond[1];
    expr_branch_t      branch[1];
    expr_call_t        call[1];
    expr_lea_t         lea[1];
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
