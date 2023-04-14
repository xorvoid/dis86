#include "decompile_private.h"

#define VALUE_IMM(_val) ({\
  value_t v = {};\
  v.type = VALUE_TYPE_IMM;\
  v.u.imm->value = _val;     \
  v; })

static bool cmp_oper(int opcode, operator_t *out)
{
  const char *oper = NULL;
  int sign = 0;

  switch (opcode) {
    case OP_JB:  oper = "<";  break;
    case OP_JBE: oper = "<="; break;
    case OP_JA:  oper = ">";  break;
    case OP_JAE: oper = ">="; break;
    case OP_JE:  oper = "=="; break;
    case OP_JNE: oper = "!="; break;
    case OP_JL:  oper = "<";  sign = 1; break;
    case OP_JLE: oper = "<="; sign = 1; break;
    case OP_JG:  oper = ">";  sign = 1; break;
    case OP_JGE: oper = ">="; sign = 1; break;
    default: return false;
  }

  out->oper = oper;
  out->sign = sign;
  return true;
}

static size_t extract_expr_special(expr_t *expr, config_t *cfg, symbols_t *symbols,
                                   dis86_instr_t *ins, size_t n_ins)
{
  dis86_instr_t * next_ins = n_ins > 1 ? ins+1 : NULL;

  // Special handling for cmp+jmp
  if (ins->opcode == OP_CMP && next_ins) {
    operator_t oper[1];
    if (cmp_oper(next_ins->opcode, oper)) {
      assert(ins->operand[0].type != OPERAND_TYPE_NONE);
      assert(ins->operand[1].type != OPERAND_TYPE_NONE);

      expr->kind = EXPR_KIND_BRANCH_COND;
      expr_branch_cond_t *k = expr->k.branch_cond;
      k->operator = *oper;
      k->left = value_from_operand(&ins->operand[0], symbols);
      k->right = value_from_operand(&ins->operand[1], symbols);
      k->target = branch_destination(next_ins);

      return 2;
    }
  }

  // Special handling for or reg,reg + je / jne
  if (ins->opcode == OP_OR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id &&
      next_ins) {

    operator_t oper[1] = {{}};
    switch (next_ins->opcode) {
      case OP_JE:  oper->oper = "=="; break;
      case OP_JNE: oper->oper = "!="; break;
    }
    if (oper->oper) {
      expr->kind = EXPR_KIND_BRANCH_COND;
      expr_branch_cond_t *k = expr->k.branch_cond;
      k->operator = *oper;
      k->left = value_from_operand(&ins->operand[0], symbols);
      k->right = VALUE_IMM(0);
      k->target = branch_destination(next_ins);

      return 2;
    }
  }

  // Special handling for xor r,r shorthand for zeroing
  if (ins->opcode == OP_XOR &&
      ins->operand[0].type == OPERAND_TYPE_REG &&
      ins->operand[1].type == OPERAND_TYPE_REG &&
      ins->operand[0].u.reg.id == ins->operand[1].u.reg.id) {

    expr->kind = EXPR_KIND_OPERATOR2;
    expr_operator2_t *k = expr->k.operator2;
    k->operator.oper = "=";
    k->operator.sign = 0;
    k->dest = value_from_operand(&ins->operand[0], symbols);
    k->src = VALUE_IMM(0);

    return 1;
  }

  // Special handling for uncond jmp
  if (ins->opcode == OP_JMP) {
    expr->kind = EXPR_KIND_BRANCH;
    expr_branch_t *k = expr->k.branch;
    k->target = branch_destination(ins);

    return 1;
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

    return 1;
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

    return 1;
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

    expr->kind = EXPR_KIND_OPERATOR3;
    expr_operator3_t *k = expr->k.operator3;
    k->operator.oper = "-";
    k->operator.sign = 0;
    k->dest = value_from_operand(&ins->operand[0], symbols);
    k->left = value_from_symref(symbols_find_reg(symbols, mem->reg1));
    k->right = value_from_imm(-(i16)mem->off);

    return 1;
  }

  return 0;
}

static size_t _impl_operator1(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                              const char *_oper, int _sign)
{
  assert(ins->operand[0].type != OPERAND_TYPE_NONE);

  expr->kind = EXPR_KIND_OPERATOR1;
  expr_operator1_t *k = expr->k.operator1;
  k->operator.oper = _oper;
  k->operator.sign = _sign;
  k->dest     = value_from_operand(&ins->operand[0], symbols);

  return 1;
}

static size_t _impl_operator2(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                              const char *_oper, int _sign)
{
  assert(ins->operand[0].type != OPERAND_TYPE_NONE);
  assert(ins->operand[1].type != OPERAND_TYPE_NONE);

  expr->kind = EXPR_KIND_OPERATOR2;
  expr_operator2_t *k = expr->k.operator2;
  k->operator.oper = _oper;
  k->operator.sign = _sign;
  k->dest     = value_from_operand(&ins->operand[0], symbols);
  k->src      = value_from_operand(&ins->operand[1], symbols);

  return 1;
}

static size_t _impl_operator3(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                              const char *_oper, int _sign)
{
  assert(ins->operand[0].type != OPERAND_TYPE_NONE);
  assert(ins->operand[1].type != OPERAND_TYPE_NONE);
  assert(ins->operand[2].type != OPERAND_TYPE_NONE);

  expr->kind = EXPR_KIND_OPERATOR3;
  expr_operator3_t *k = expr->k.operator3;
  k->operator.oper = _oper;
  k->operator.sign = _sign;
  k->dest     = value_from_operand(&ins->operand[0], symbols);
  k->left     = value_from_operand(&ins->operand[1], symbols);
  k->right    = value_from_operand(&ins->operand[2], symbols);

  return 1;
}

static size_t _impl_abstract(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                             const char *_name)
{
  expr->kind = EXPR_KIND_ABSTRACT;
  expr_abstract_t *k = expr->k.abstract;
  k->func_name = _name;
  k->ret = VALUE_NONE;
  k->n_args = 0;

  assert(ARRAY_SIZE(k->args) <= ARRAY_SIZE(ins->operand));
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) break;
    k->args[k->n_args++] = value_from_operand(o, symbols);
  }

  return 1;
}

static size_t _impl_abstract_ret(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                                 const char *_name)
{
  assert(ins->operand[0].type != OPERAND_TYPE_NONE);

  expr->kind = EXPR_KIND_ABSTRACT;
  expr_abstract_t *k = expr->k.abstract;
  k->func_name = _name;
  k->ret = value_from_operand(&ins->operand[0], symbols);
  k->n_args = 0;

  assert(ARRAY_SIZE(k->args) <= ARRAY_SIZE(ins->operand));
  for (size_t i = 1; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) break;
    k->args[k->n_args++] = value_from_operand(o, symbols);
  }

  return 1;
}

static size_t _impl_abstract_flags(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                                   const char *_name)
{
  expr->kind = EXPR_KIND_ABSTRACT;
  expr_abstract_t *k = expr->k.abstract;
  k->func_name = _name;
  k->ret = value_from_symref(symbols_find_reg(symbols, REG_FLAGS));
  k->n_args = 0;

  assert(ARRAY_SIZE(k->args) <= ARRAY_SIZE(ins->operand));
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) break;
    k->args[k->n_args++] = value_from_operand(o, symbols);
  }

  return 1;
}

static size_t _impl_abstract_jump(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins,
                                  const char *_operation)
{
  assert(ins->operand[0].type == OPERAND_TYPE_REL);
  assert(ins->operand[1].type == OPERAND_TYPE_NONE);

  expr->kind = EXPR_KIND_BRANCH_FLAGS;
  expr_branch_flags_t *k = expr->k.branch_flags;
  k->op     = _operation;
  k->flags  = value_from_symref(symbols_find_reg(symbols, REG_FLAGS));
  k->target = ins->addr + ins->n_bytes + ins->operand[0].u.rel.val;

  return 1;
}

#define OPERATOR1(_oper, _sign)  _impl_operator1(expr, symbols, ins, _oper, _sign)
#define OPERATOR2(_oper, _sign)  _impl_operator2(expr, symbols, ins, _oper, _sign)
#define OPERATOR3(_oper, _sign)  _impl_operator3(expr, symbols, ins, _oper, _sign)
#define ABSTRACT(_name)          _impl_abstract(expr, symbols, ins, _name)
#define ABSTRACT_RET(_name)      _impl_abstract_ret(expr, symbols, ins, _name)
#define ABSTRACT_FLAGS(_name)    _impl_abstract_flags(expr, symbols, ins, _name)
#define ABSTRACT_JUMP(_op)       _impl_abstract_jump(expr, symbols, ins, _op)

static size_t extract_expr(expr_t *expr, config_t *cfg, symbols_t *symbols,
                           dis86_instr_t *ins, size_t n_ins)
{
  size_t consumed = extract_expr_special(expr, cfg, symbols, ins, n_ins);
  if (consumed) return consumed;

  switch (ins->opcode) {
    case OP_AAA:    break;
    case OP_AAS:    break;
    case OP_ADC:    break;
    case OP_ADD:    return OPERATOR2("+=", 0);
    case OP_AND:    return OPERATOR2("&=", 0);
    case OP_CALL:   break;
    case OP_CALLF:  break;
    case OP_CBW:    break;
    case OP_CLC:    break;
    case OP_CLD:    break;
    case OP_CLI:    break;
    case OP_CMC:    break;
    case OP_CMP:    return ABSTRACT_FLAGS("CMP");
    case OP_CMPS:   break;
    case OP_CWD:    break;
    case OP_DAA:    break;
    case OP_DAS:    break;
    case OP_DEC:    return OPERATOR1("-= 1", 0);
    case OP_DIV:    break;
    case OP_ENTER:  break;
    case OP_HLT:    break;
    case OP_IMUL:   return OPERATOR3("*", 1);
    case OP_IN:     break;
    case OP_INC:    return OPERATOR1("+= 1", 0);
    case OP_INS:    break;
    case OP_INT:    break;
    case OP_INTO:   break;
    case OP_INVAL:  break;
    case OP_IRET:   break;
    case OP_JA:     return ABSTRACT_JUMP("JA");
    case OP_JAE:    return ABSTRACT_JUMP("JAE");
    case OP_JB:     return ABSTRACT_JUMP("JB");
    case OP_JBE:    return ABSTRACT_JUMP("JBE");
    case OP_JCXZ:   break;
    case OP_JE:     return ABSTRACT_JUMP("JE");
    case OP_JG:     return ABSTRACT_JUMP("JG");
    case OP_JGE:    return ABSTRACT_JUMP("JGE");
    case OP_JL:     return ABSTRACT_JUMP("JL");
    case OP_JLE:    return ABSTRACT_JUMP("JLE");
    case OP_JMP:    break;
    case OP_JMPF:   break;
    case OP_JNE:    return ABSTRACT_JUMP("JNE");
    case OP_JNO:    break;
    case OP_JNP:    break;
    case OP_JNS:    break;
    case OP_JO:     break;
    case OP_JP:     break;
    case OP_JS:     break;
    case OP_LAHF:   break;
    case OP_LDS:    return ABSTRACT("LOAD_SEG_OFF");
    case OP_LEA:    break;
    case OP_LEAVE:  return ABSTRACT("LEAVE");
      //case OP_LEAVE:  LITERAL("SP = BP; BP = POP();
    case OP_LES:    return ABSTRACT("LOAD_SEG_OFF");
    case OP_LODS:   break;
    case OP_LOOP:   break;
    case OP_LOOPE:  break;
    case OP_LOOPNE: break;
    case OP_MOV:    return OPERATOR2("=", 0);
    case OP_MOVS:   break;
    case OP_MUL:    break;
    case OP_NEG:    break;
    case OP_NOP:    break;
    case OP_NOT:    break;
    case OP_OR:     return OPERATOR2("|=", 0);
    case OP_OUT:    break;
    case OP_OUTS:   break;
    case OP_POP:    return ABSTRACT_RET("POP");
    case OP_POPA:   break;
    case OP_POPF:   break;
    case OP_PUSH:   return ABSTRACT("PUSH");
    case OP_PUSHA:  break;
    case OP_PUSHF:  break;
    case OP_RCL:    break;
    case OP_RCR:    break;
    case OP_RET:    return ABSTRACT("RETURN_NEAR");
    case OP_RETF:   return ABSTRACT("RETURN_FAR");
    case OP_ROL:    break;
    case OP_ROR:    break;
    case OP_SAHF:   break;
    case OP_SAR:    break;
    case OP_SBB:    break;
    case OP_SCAS:   break;
    case OP_SHL:    return OPERATOR2("<<=", 0);
    case OP_SHR:    return OPERATOR2(">>=", 0);
    case OP_STC:    break;
    case OP_STD:    break;
    case OP_STI:    break;
    case OP_STOS:   break;
    case OP_SUB:    return OPERATOR2("-=", 0);
    case OP_TEST:   return ABSTRACT_FLAGS("TEST");
    case OP_XCHG:   break;
    case OP_XLAT:   break;
    case OP_XOR:    return OPERATOR2("^=", 0);
    default: FAIL("Unknown Instruction: %d", ins->opcode);
  }

  // If we reach this point, the instruction wasn't mapped to any expression: UNKNOWN
  expr->kind = EXPR_KIND_UNKNOWN;
  return 1;
}

meh_t * meh_new(config_t *cfg, symbols_t *symbols, dis86_instr_t *ins, size_t n_ins)
{
  meh_t *m = calloc(1, sizeof(meh_t));

  while (n_ins) {
    assert(m->expr_len < ARRAY_SIZE(m->expr_arr));

    expr_t *expr = &m->expr_arr[m->expr_len];
    size_t consumed = extract_expr(expr, cfg, symbols, ins, n_ins);
    assert(consumed <= n_ins);
    expr->ins = ins;
    expr->n_ins = consumed;
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
