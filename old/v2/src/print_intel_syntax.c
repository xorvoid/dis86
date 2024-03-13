#include "dis86_private.h"
#include "str.h"

static void print_operand_intel_syntax(str_t *s, dis86_instr_t *ins, operand_t *o)
{
  switch (o->type) {
    case OPERAND_TYPE_REG: str_fmt(s, "%s", reg_name(o->u.reg.id)); break;
    case OPERAND_TYPE_MEM: {
      operand_mem_t *m = &o->u.mem;
      switch (m->sz) {
        case SIZE_8:  str_fmt(s, "BYTE PTR "); break;
        case SIZE_16: str_fmt(s, "WORD PTR "); break;
        case SIZE_32: str_fmt(s, "DWORD PTR "); break;
      }
      str_fmt(s, "%s:", reg_name(m->sreg));
      if (!m->reg1 && !m->reg2) {
        if (m->off) str_fmt(s, "0x%x", m->off);
      } else {
        str_fmt(s, "[");
        if (m->reg1) str_fmt(s, "%s", reg_name(m->reg1));
        if (m->reg2) str_fmt(s, "+%s", reg_name(m->reg2));
        if (m->off) {
          i16 disp = (i16)m->off;
          if (disp >= 0) str_fmt(s, "+0x%x", (u16)disp);
          else           str_fmt(s, "-0x%x", (u16)-disp);
        }
        str_fmt(s, "]");
      }
    } break;
    case OPERAND_TYPE_IMM: str_fmt(s, "0x%x", o->u.imm.val); break;
    case OPERAND_TYPE_REL: {
      u16 effective = ins->addr + ins->n_bytes + o->u.rel.val;
      str_fmt(s, "0x%x", effective);
    } break;
    case OPERAND_TYPE_FAR: str_fmt(s, "0x%x:0x%x", o->u.far.seg, o->u.far.off); break;
    default: FAIL("INVALID OPERAND TYPE: %d", o->type);
  }
}

char *dis86_print_intel_syntax(dis86_t *d, dis86_instr_t *ins, bool with_detail)
{
  str_t s[1];
  str_init(s);

  if (with_detail) {
    str_fmt(s, "%8zx:\t", ins->addr);
    for (size_t i = 0; i < ins->n_bytes; i++) {
      u8 b = binary_byte_at(d->b, ins->addr + i);
      str_fmt(s, "%02x ", b);
    }
    size_t used = ins->n_bytes * 3;
    size_t remain = (used <= 21) ? 21 - used : 0;
    str_fmt(s, "%*s\t", (int)remain, " ");
  }

  if (ins->rep == REP_NE) str_fmt(s, "repne ");
  else if (ins->rep == REP_E)  str_fmt(s, "rep ");

  str_fmt(s, "%-5s", instr_op_mneumonic(ins->opcode));

  int n_operands = 0;
  for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
    operand_t *o = &ins->operand[i];
    if (o->type == OPERAND_TYPE_NONE) break;
    if ((int)(1<<i) & ins->intel_hidden) continue;
    if (n_operands == 0) str_fmt(s, "  ");
    else str_fmt(s, ",");
    print_operand_intel_syntax(s, ins, o);
    n_operands++;
  }

  /* remove any trailing space */
  str_rstrip(s);

  return str_to_cstr(s);
}

char *dis86_print_c_code(dis86_t *d, dis86_instr_t *ins, size_t addr, size_t n_bytes)
{
  UNIMPL();
}
