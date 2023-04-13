#include "decompile_private.h"

#define OPERAND_IMM_ZERO ({\
  operand_t o = {};\
  o.type = OPERAND_TYPE_IMM;\
  o.u.imm.sz = SIZE_16;\
  o.u.imm.val = 0;\
  o; })

static int code_c_type[] = {
#define ELT(_1, _2, ty, _4) ty,
  INSTR_OP_ARRAY(ELT)
#undef ELT
};

static const char *code_c_str[] = {
#define ELT(_1, _2, _3, s) s,
  INSTR_OP_ARRAY(ELT)
#undef ELT
};

static const char *cmp_oper(int opcode, int *sign)
{
  const char *oper = NULL;
  *sign = 0;

  switch (opcode) {
    case OP_JB:  oper = "<";  break;
    case OP_JBE: oper = "<="; break;
    case OP_JA:  oper = ">";  break;
    case OP_JAE: oper = ">="; break;
    case OP_JE:  oper = "=="; break;
    case OP_JNE: oper = "!="; break;
    case OP_JL:  oper = "<";  *sign = 1; break;
    case OP_JLE: oper = "<="; *sign = 1; break;
    case OP_JG:  oper = ">";  *sign = 1; break;
    case OP_JGE: oper = ">="; *sign = 1; break;
  }

  return oper;
}

static size_t extract_expr(expr_t *expr, dis86_instr_t *ins, size_t n_ins)
{
  dis86_instr_t * next_ins = n_ins > 1 ? ins+1 : NULL;

  int type = code_c_type[ins->opcode];
  const char *str = code_c_str[ins->opcode];

  // Special handling for cmp+jmp
  if (ins->opcode == OP_CMP && next_ins) {
    int sign = 0;
    const char *oper = cmp_oper(next_ins->opcode, &sign);
    if (oper) {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      assert(ins->operand[1].type != OPERAND_TYPE_NONE);

      expr->kind = EXPR_KIND_BRANCH;
      expr_branch_t *k = expr->k.branch;
      k->operator = oper;
      k->signed_cmp = sign;
      k->oper1 = ins->operand[0];
      k->oper2 = ins->operand[1];
      k->target = branch_destination(next_ins);

      expr->n_ins = 2;
      expr->ins   = ins;
      return expr->n_ins;
    }
  }

  // Special handling for or reg,reg + je / jne
  if (ins->opcode == OP_OR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id &&
      next_ins) {

    const char *oper = NULL;
    switch (next_ins->opcode) {
      case OP_JE:  oper = "=="; break;
      case OP_JNE: oper = "!="; break;
    }
    if (oper) {
      expr->kind = EXPR_KIND_BRANCH;
      expr_branch_t *k = expr->k.branch;
      k->operator = oper;
      k->signed_cmp = false;
      k->oper1 = ins->operand[0];
      k->oper2 = OPERAND_IMM_ZERO;
      k->target = branch_destination(next_ins);

      expr->n_ins = 2;
      expr->ins   = ins;
      return expr->n_ins;
    }
  }

  // Special handling for xor r,r shorthand for zeroing
  if (ins->opcode == OP_XOR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id) {

    expr->kind = EXPR_KIND_OPERATOR;
    expr_operator_t *k = expr->k.operator;
    k->operator = "=";
    k->oper1 = ins->operand[0];
    k->oper2 = OPERAND_IMM_ZERO;

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  switch (type) {
    case CODE_C_UNKNOWN: {
      expr->kind = EXPR_KIND_UNKNOWN;
    } break;
    case CODE_C_OPERATOR: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      expr->kind = EXPR_KIND_OPERATOR;
      expr_operator_t *k = expr->k.operator;
      k->operator = str;
      k->oper1    = ins->operand[0];
      k->oper2    = ins->operand[1];
    } break;
    case CODE_C_FUNCTION: {
      expr->kind = EXPR_KIND_FUNCTION;
      expr_function_t *k = expr->k.function;
      k->func_name = str;
      memset(&k->ret, 0, sizeof(k->ret));
      memcpy(k->args, ins->operand, sizeof(k->args));
    } break;
    case CODE_C_RFUNCTION: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      expr->kind = EXPR_KIND_FUNCTION;
      expr_function_t *k = expr->k.function;
      k->func_name = str;
      memcpy(&k->ret, &ins->operand[0], sizeof(operand_t));
      memcpy(k->args, &ins->operand[1], sizeof(ins->operand)-sizeof(operand_t));
      memset(&k->args[ARRAY_SIZE(k->args)-1], 0, sizeof(operand_t));
    } break;
    case CODE_C_LITERAL: {
      expr->kind = EXPR_KIND_LITERAL;
      expr_literal_t *k = expr->k.literal;
      k->text = str;
    } break;
    default: {
      FAIL("Unknown code type: %d\n", type);
    } break;
  }

  expr->n_ins = 1;
  expr->ins   = ins;

  return expr->n_ins;
}

meh_t * meh_new(dis86_instr_t *ins, size_t n_ins)
{
  meh_t *m = calloc(1, sizeof(meh_t));

  while (n_ins) {
    assert(m->expr_len < ARRAY_SIZE(m->expr_arr));

    expr_t *expr = &m->expr_arr[m->expr_len];
    size_t consumed = extract_expr(expr, ins, n_ins);
    assert(consumed <= n_ins);
    m->expr_len++;

    ins += consumed;
    n_ins -= consumed;
  }

  return m;
}

void meh_delete(meh_t *m)
{
  free(m);
}
