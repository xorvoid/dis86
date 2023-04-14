#include "decompile_private.h"

enum {
  INFO_TYPE_UNKNOWN = 0,
  INFO_TYPE_OP,
  INFO_TYPE_FUNC,
  INFO_TYPE_RFUNC,
  INFO_TYPE_LIT,
};

typedef struct info info_t;
struct info
{
  int type;
  union {
    const char *op;
    const char *func;
    const char *rfunc;
    const char *lit;
  } u;
};

static info_t instr_info(dis86_instr_t *instr)
{
  info_t info = {};

#define OPERATOR(s)  do { info.type = INFO_TYPE_OP;    info.u.op    = s; } while(0)
#define FUNCTION(s)  do { info.type = INFO_TYPE_FUNC;  info.u.func  = s; } while(0)
#define RFUNCTION(s) do { info.type = INFO_TYPE_RFUNC; info.u.rfunc = s; } while(0)
#define LITERAL(s)   do { info.type = INFO_TYPE_LIT;   info.u.lit   = s; } while(0)

  int type = -1;
  const char *op = NULL;
  const char *func = NULL;
  const char *lit = NULL;

  switch (instr->opcode) {
    case OP_AAA:                                     break;
    case OP_AAS:                                     break;
    case OP_ADC:                                     break;
    case OP_ADD:    OPERATOR("+=");                  break;
    case OP_AND:    OPERATOR("&=");                  break;
    case OP_CALL:                                    break;
    case OP_CALLF:                                   break;
    case OP_CBW:                                     break;
    case OP_CLC:                                     break;
    case OP_CLD:                                     break;
    case OP_CLI:                                     break;
    case OP_CMC:                                     break;
    case OP_CMP:                                     break;
    case OP_CMPS:                                    break;
    case OP_CWD:                                     break;
    case OP_DAA:                                     break;
    case OP_DAS:                                     break;
    case OP_DEC:    OPERATOR("-= 1");                break;
    case OP_DIV:                                     break;
    case OP_ENTER:                                   break;
    case OP_HLT:                                     break;
    case OP_IMUL:                                    break;
    case OP_IN:                                      break;
    case OP_INC:    OPERATOR("+= 1");                break;
    case OP_INS:                                     break;
    case OP_INT:                                     break;
    case OP_INTO:                                    break;
    case OP_INVAL:                                   break;
    case OP_IRET:                                    break;
    case OP_JA:                                      break;
    case OP_JAE:                                     break;
    case OP_JB:                                      break;
    case OP_JBE:                                     break;
    case OP_JCXZ:                                    break;
    case OP_JE:                                      break;
    case OP_JG:                                      break;
    case OP_JGE:                                     break;
    case OP_JL:                                      break;
    case OP_JLE:                                     break;
    case OP_JMP:                                     break;
    case OP_JMPF:                                    break;
    case OP_JNE:                                     break;
    case OP_JNO:                                     break;
    case OP_JNP:                                     break;
    case OP_JNS:                                     break;
    case OP_JO:                                      break;
    case OP_JP:                                      break;
    case OP_JS:                                      break;
    case OP_LAHF:                                    break;
    case OP_LDS:    FUNCTION("LOAD_SEG_OFF");        break;
    case OP_LEA:                                     break;
    case OP_LEAVE:  LITERAL("SP = BP; BP = POP();"); break;
    case OP_LES:    FUNCTION("LOAD_SEG_OFF");        break;
    case OP_LODS:                                    break;
    case OP_LOOP:                                    break;
    case OP_LOOPE:                                   break;
    case OP_LOOPNE:                                  break;
    case OP_MOV:    OPERATOR("=");                   break;
    case OP_MOVS:                                    break;
    case OP_MUL:                                     break;
    case OP_NEG:                                     break;
    case OP_NOP:                                     break;
    case OP_NOT:                                     break;
    case OP_OR:     OPERATOR("|=");                  break;
    case OP_OUT:                                     break;
    case OP_OUTS:                                    break;
    case OP_POP:    RFUNCTION("POP");                break;
    case OP_POPA:                                    break;
    case OP_POPF:                                    break;
    case OP_PUSH:   FUNCTION("PUSH");                break;
    case OP_PUSHA:                                   break;
    case OP_PUSHF:                                   break;
    case OP_RCL:                                     break;
    case OP_RCR:                                     break;
    case OP_RET:    LITERAL("RETURN_NEAR()");        break;
    case OP_RETF:   LITERAL("RETURN_FAR()");         break;
    case OP_ROL:                                     break;
    case OP_ROR:                                     break;
    case OP_SAHF:                                    break;
    case OP_SAR:                                     break;
    case OP_SBB:                                     break;
    case OP_SCAS:                                    break;
    case OP_SHL:    OPERATOR("<<=");                 break;
    case OP_SHR:    OPERATOR(">>=");                 break;
    case OP_STC:                                     break;
    case OP_STD:                                     break;
    case OP_STI:                                     break;
    case OP_STOS:                                    break;
    case OP_SUB:    OPERATOR("-=");                  break;
    case OP_TEST:                                    break;
    case OP_XCHG:                                    break;
    case OP_XLAT:                                    break;
    case OP_XOR:    OPERATOR("^=");                  break;
    default: FAIL("Unknown Instruction: %d", instr->opcode);
  }
  return info;

#undef OPERATOR
#undef FUNCTION
#undef RFUNCTION
#undef LITERAL
}

#define OPERAND_IMM_ZERO ({\
  operand_t o = {};\
  o.type = OPERAND_TYPE_IMM;\
  o.u.imm.sz = SIZE_16;\
  o.u.imm.val = 0;\
  o; })

#define VALUE_IMM_ZERO ({\
  value_t v = {};\
  v.type = VALUE_TYPE_IMM;\
  v.u.imm->value = 0;     \
  v; })

/* static int code_c_type[] = { */
/* #define ELT(_1, _2, ty, _4) ty, */
/*   INSTR_OP_ARRAY(ELT) */
/* #undef ELT */
/* }; */

/* static const char *code_c_str[] = { */
/* #define ELT(_1, _2, _3, s) s, */
/*   INSTR_OP_ARRAY(ELT) */
/* #undef ELT */
/* }; */

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

static size_t extract_expr(expr_t *expr, config_t *cfg, symbols_t *symbols,
                           dis86_instr_t *ins, size_t n_ins)
{
  dis86_instr_t * next_ins = n_ins > 1 ? ins+1 : NULL;

  info_t info = instr_info(ins);

  // Special handling for cmp+jmp
  if (ins->opcode == OP_CMP && next_ins) {
    int sign = 0;
    const char *oper = cmp_oper(next_ins->opcode, &sign);
    if (oper) {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      assert(ins->operand[1].type != OPERAND_TYPE_NONE);

      expr->kind = EXPR_KIND_BRANCH_COND;
      expr_branch_cond_t *k = expr->k.branch_cond;
      k->operator = oper;
      k->signed_cmp = sign;
      k->left = value_from_operand(&ins->operand[0], symbols);
      k->right = value_from_operand(&ins->operand[1], symbols);
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
      expr->kind = EXPR_KIND_BRANCH_COND;
      expr_branch_cond_t *k = expr->k.branch_cond;
      k->operator = oper;
      k->signed_cmp = false;
      k->left = value_from_operand(&ins->operand[0], symbols);
      k->right = VALUE_IMM_ZERO;
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

  // Special handling for uncond jmp
  if (ins->opcode == OP_JMP) {
    expr->kind = EXPR_KIND_BRANCH;
    expr_branch_t *k = expr->k.branch;
    k->target = branch_destination(ins);

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  // Special handling for callf
  if (ins->opcode == OP_CALLF && ins->operand[0].type == OPERAND_TYPE_FAR) {
    operand_far_t *far = &ins->operand[0].u.far;
    segoff_t addr = {far->seg, far->off};
    bool remapped = config_seg_remap(cfg, &addr.seg);
    const char *name = config_func_lookup(cfg, addr);

    expr->kind = EXPR_KIND_CALL;
    expr_call_t *k = expr->k.call;
    k->addr.type  = ADDR_TYPE_FAR;
    k->addr.u.far = addr;
    k->remapped   = remapped;
    k->name       = name;

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  // Special handling for call
  if (ins->opcode == OP_CALL) {
    assert(ins->operand[0].type == OPERAND_TYPE_REL);
    u16 effective = ins->addr + ins->n_bytes + ins->operand[0].u.rel.val;

    expr->kind = EXPR_KIND_CALL;
    expr_call_t *k = expr->k.call;
    k->addr.type   = ADDR_TYPE_NEAR;
    k->addr.u.near = effective;
    k->remapped    = false;
    k->name        = NULL;

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  // Special handling for imul
  if (ins->opcode == OP_IMUL) {
    assert(ins->operand[0].type != OPERAND_TYPE_NONE);
    assert(ins->operand[1].type != OPERAND_TYPE_NONE);
    assert(ins->operand[2].type != OPERAND_TYPE_NONE);

    expr->kind = EXPR_KIND_OPERATOR3;
    expr_operator3_t *k = expr->k.operator3;
    k->operator = "*";
    k->sign     = 1;
    k->dest    = value_from_operand(&ins->operand[0], symbols);
    k->left    = value_from_operand(&ins->operand[1], symbols);
    k->right   = value_from_operand(&ins->operand[2], symbols);

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  // Special handling for lea
    if (ins->opcode == OP_LEA) {
    assert(ins->operand[0].type != OPERAND_TYPE_NONE);

    assert(ins->operand[1].type == OPERAND_TYPE_MEM);
    operand_mem_t *mem = &ins->operand[1].u.mem;
    assert(mem->sz == SIZE_16);
    assert(mem->reg1);
    assert(!mem->reg2);
    assert(mem->off);


    expr->kind = EXPR_KIND_LEA;
    expr_lea_t *k = expr->k.lea;
    k->dest          = value_from_operand(&ins->operand[0], symbols);
    k->addr_base_reg = mem->reg1;
    k->addr_offset   = mem->off;

    expr->n_ins = 1;
    expr->ins   = ins;
    return expr->n_ins;
  }

  switch (info.type) {
    case INFO_TYPE_UNKNOWN: {
      expr->kind = EXPR_KIND_UNKNOWN;
    } break;
    case INFO_TYPE_OP: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      expr->kind = EXPR_KIND_OPERATOR;
      expr_operator_t *k = expr->k.operator;
      k->operator = info.u.op;
      k->oper1    = ins->operand[0];
      k->oper2    = ins->operand[1];
    } break;
    case INFO_TYPE_FUNC: {
      expr->kind = EXPR_KIND_FUNCTION;
      expr_function_t *k = expr->k.function;
      k->func_name = info.u.func;
      k->ret = VALUE_NONE;
      k->n_args = 0;
      assert(ARRAY_SIZE(k->args) <= ARRAY_SIZE(ins->operand));
      for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
        operand_t *o = &ins->operand[i];
        if (o->type == OPERAND_TYPE_NONE) break;
        k->args[k->n_args++] = value_from_operand(o, symbols);
      }
    } break;
    case INFO_TYPE_RFUNC: {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      expr->kind = EXPR_KIND_FUNCTION;
      expr_function_t *k = expr->k.function;
      k->func_name = info.u.rfunc;
      k->ret = value_from_operand(&ins->operand[0], symbols);
      k->n_args = 0;
      assert(ARRAY_SIZE(k->args) <= ARRAY_SIZE(ins->operand));
      for (size_t i = 1; i < ARRAY_SIZE(ins->operand); i++) {
        operand_t *o = &ins->operand[i];
        if (o->type == OPERAND_TYPE_NONE) break;
        k->args[k->n_args++] = value_from_operand(o, symbols);
      }
    } break;
    case INFO_TYPE_LIT: {
      expr->kind = EXPR_KIND_LITERAL;
      expr_literal_t *k = expr->k.literal;
      k->text = info.u.lit;
    } break;
    default: {
      FAIL("Unknown code type: %d\n", info.type);
    } break;
  }

  expr->n_ins = 1;
  expr->ins   = ins;

  return expr->n_ins;
}

meh_t * meh_new(config_t *cfg, symbols_t *symbols, dis86_instr_t *ins, size_t n_ins)
{
  meh_t *m = calloc(1, sizeof(meh_t));

  while (n_ins) {
    assert(m->expr_len < ARRAY_SIZE(m->expr_arr));

    expr_t *expr = &m->expr_arr[m->expr_len];
    size_t consumed = extract_expr(expr, cfg, symbols, ins, n_ins);
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
