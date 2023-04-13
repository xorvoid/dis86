#pragma once

typedef struct meh           meh_t;
typedef struct expr          expr_t;
typedef struct expr_operator expr_operator_t;
typedef struct expr_function expr_function_t;
typedef struct expr_literal  expr_literal_t;
typedef struct expr_branch   expr_branch_t;

enum {
  EXPR_KIND_UNKNOWN,
  EXPR_KIND_OPERATOR,
  EXPR_KIND_FUNCTION,
  EXPR_KIND_LITERAL,
  EXPR_KIND_BRANCH,
};

struct expr_operator
{
  // TODO: REMOVE dis86 instr operands
  const char * operator;
  operand_t    oper1;           // required
  operand_t    oper2;           // optional
};

struct expr_function
{
  // TODO: REMOVE dis86 instr operands
  const char * func_name;
  operand_t    ret;
  operand_t    args[OPERAND_MAX];
};

struct expr_literal
{
  const char *text;
};

struct expr_branch
{
  // TODO: REMOVE dis86 instr operands
  const char * operator;
  int          signed_cmp;
  operand_t    oper1;           // required
  operand_t    oper2;           // required
  u32          target;
};

struct expr
{
  int kind;
  union {
    expr_operator_t operator[1];
    expr_function_t function[1];
    expr_literal_t  literal[1];
    expr_branch_t   branch[1];
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

meh_t * meh_new(dis86_instr_t *ins, size_t n_ins);
void    meh_delete(meh_t *m);
