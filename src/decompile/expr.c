#include "decompile_private.h"

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
  k->target = ins->addr + ins->n_bytes + (i16)ins->operand[0].u.rel.val;

  return 1;
}

/* static size_t _impl_abstract_loop(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins) */
/* { */
/*   STOPPED_HERE; */
/* } */

static size_t _impl_call_far(expr_t *expr, config_t *cfg, symbols_t *symbols, dis86_instr_t *ins)
{
  // FIXME BROKEN!!!
  if (ins->operand[0].type != OPERAND_TYPE_FAR) {
    expr->kind = EXPR_KIND_UNKNOWN;
    return 1;
  }

  operand_far_t *far = &ins->operand[0].u.far;
  segoff_t addr = {far->seg, far->off};
  bool remapped = config_seg_remap(cfg, &addr.seg);
  config_func_t *func = config_func_lookup(cfg, addr);

  expr->kind = EXPR_KIND_CALL;
  expr_call_t *k = expr->k.call;
  k->addr.type  = ADDR_TYPE_FAR;
  k->addr.u.far = addr;
  k->remapped   = remapped;
  k->func       = func;

  return 1;
}

static size_t _impl_call_near(u16 seg, expr_t *expr,
                              config_t *cfg, symbols_t *symbols, dis86_instr_t *ins)
{
  if (ins->operand[0].type != OPERAND_TYPE_REL) {
    expr->kind = EXPR_KIND_UNKNOWN;
    return 1;
  }

  size_t effective = (u16)(ins->addr + ins->n_bytes + ins->operand[0].u.rel.val);
  //printf("effective: 0x%x | seg: 0x%x\n", (u32)effective, seg);
  assert(16*(size_t)seg <= effective && effective < 16*(size_t)seg + (1<<16));
  u16 off = effective - 16*(size_t)seg;

  segoff_t addr = {seg, off};
  config_func_t *func = config_func_lookup(cfg, addr);

  expr->kind = EXPR_KIND_CALL;
  expr_call_t *k = expr->k.call;
  k->addr.type   = ADDR_TYPE_NEAR;
  k->addr.u.near = effective;
  k->remapped    = false;
  k->func        = func;

  return 1;
}

static size_t _impl_load_effective_addr(expr_t *expr, symbols_t *symbols, dis86_instr_t *ins)
{
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

#define OPERATOR1(_oper, _sign)  _impl_operator1(expr, symbols, ins, _oper, _sign)
#define OPERATOR2(_oper, _sign)  _impl_operator2(expr, symbols, ins, _oper, _sign)
#define OPERATOR3(_oper, _sign)  _impl_operator3(expr, symbols, ins, _oper, _sign)
#define ABSTRACT(_name)          _impl_abstract(expr, symbols, ins, _name)
#define ABSTRACT_RET(_name)      _impl_abstract_ret(expr, symbols, ins, _name)
#define ABSTRACT_FLAGS(_name)    _impl_abstract_flags(expr, symbols, ins, _name)
#define ABSTRACT_JUMP(_op)       _impl_abstract_jump(expr, symbols, ins, _op)
//#define ABSTRACT_LOOP()          _impl_abstract_loop(expr, symbols, ins)
#define CALL_FAR()               _impl_call_far(expr, cfg, symbols, ins)
#define CALL_NEAR()              _impl_call_near(seg, expr, cfg, symbols, ins)
#define LOAD_EFFECTIVE_ADDR()    _impl_load_effective_addr(expr, symbols, ins)

static size_t extract_expr(u16 seg, expr_t *expr, config_t *cfg, symbols_t *symbols,
                           dis86_instr_t *ins, size_t n_ins)
{
  switch (ins->opcode) {
    case OP_AAA:    break;
    case OP_AAS:    break;
    case OP_ADC:    break;
    case OP_ADD:    return OPERATOR2("+=", 0);
    case OP_AND:    return OPERATOR2("&=", 0);
    case OP_CALL:   return CALL_NEAR();
    case OP_CALLF:  return CALL_FAR();
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
    case OP_JMP: {
      expr->kind = EXPR_KIND_BRANCH;
      expr_branch_t *k = expr->k.branch;
      k->target = branch_destination(ins);
      return 1;
    } break;
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
    case OP_LEA:    return LOAD_EFFECTIVE_ADDR();
    case OP_LEAVE:  return ABSTRACT("LEAVE");
      //case OP_LEAVE:  LITERAL("SP = BP; BP = POP();
    case OP_LES:    return ABSTRACT("LOAD_SEG_OFF");
    case OP_LODS:   break;
    case OP_LOOP:   break; //return ABSTRACT_LOOP();
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

value_t expr_destination(expr_t *expr)
{
  switch (expr->kind) {
    case EXPR_KIND_UNKNOWN:       FAIL("EXPR_KIND_UNKNOWN UNSUPPORTED");
    case EXPR_KIND_NONE:          return VALUE_NONE;
    case EXPR_KIND_OPERATOR1:     return expr->k.operator1->dest;
    case EXPR_KIND_OPERATOR2:     return expr->k.operator2->dest;
    case EXPR_KIND_OPERATOR3:     return expr->k.operator3->dest;
    case EXPR_KIND_ABSTRACT:      return expr->k.abstract->ret;
    case EXPR_KIND_BRANCH_COND:   return VALUE_NONE;
    case EXPR_KIND_BRANCH_FLAGS:  return expr->k.branch_flags->flags;
    case EXPR_KIND_BRANCH:        return VALUE_NONE;
    case EXPR_KIND_CALL:          return VALUE_NONE;  // ??? is this true ?? Should this be AX:DX ??
    default: FAIL("Unkown expression kind: %d", expr->kind);
  }
}

meh_t * meh_new(config_t *cfg, symbols_t *symbols, u16 seg, dis86_instr_t *ins, size_t n_ins)
{
  meh_t *m = calloc(1, sizeof(meh_t));

  while (n_ins) {
    assert(m->expr_len < ARRAY_SIZE(m->expr_arr));

    expr_t *expr = &m->expr_arr[m->expr_len];
    size_t consumed = extract_expr(seg, expr, cfg, symbols, ins, n_ins);
    assert(consumed <= n_ins);
    expr->ins = ins;
    expr->n_ins = consumed;
    m->expr_len++;

    ins += consumed;
    n_ins -= consumed;
  }

  transform_pass_xor_rr(m);
  transform_pass_cmp_jmp(m);
  transform_pass_or_jmp(m);
  transform_pass_synthesize_calls(m);

  return m;
}

void meh_delete(meh_t *m)
{
  free(m);
}
