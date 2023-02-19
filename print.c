#include "dis86_private.h"
#include "str.h"

void print_operand_intel_syntax(str_t *s, operand_t *o)
{
  switch (o->type) {
    case OPERAND_TYPE_REG: str_fmt(s, "%s", reg_name(o->u.reg.id)); break;
    case OPERAND_TYPE_IMM: str_fmt(s, "0x%x", o->u.imm.val); break;
    default: FAIL("INVALID OPERAND TYPE: %d", o->type);
  }
}

char *dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes, bool with_detail)
{
  str_t s[1];
  str_init(s);

  str_fmt(s, "%s", instr_op_mneumonic(ins->opcode));
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) break;
    if (i == 0) str_fmt(s, "  ");
    else str_fmt(s, ", ");
    print_operand_intel_syntax(s, o);
  }

  return str_to_cstr(s);
}

char *dis86_print_c_code(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes)
{
  UNIMPL();
}
