#include "dis86_private.h"
#include "instr_tbl.h"
#include "str.h"

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

static const char *as_upper(const char *s)
{
  static char buf[256];

  size_t len = strlen(s);
  if (len+1 >= sizeof(buf)) FAIL("String too long!");

  for (size_t i = 0; i < len+1; i++) {
    char c = s[i];
    if ('a' <= c && c <= 'z') c += ('A' - 'a');
    buf[i] = c;
  }

  return buf;
}

static const char *lookup_name(operand_mem_t *m)
{
  static char buf[128];
  i16 off = (i16)m->off;

  // Data section?
  if (m->sreg == REG_DS && !m->reg1 && !m->reg2) {
    sprintf(buf, "_data_%04x", (u16)off);
    return buf;
  }

  // Local var?
  if (m->sreg == REG_SS && m->reg1 == REG_BP && !m->reg2) {
    if (off < 0) {
      sprintf(buf, "_local_%04x", (u16)-off);
    } else {
      sprintf(buf, "_param_%04x", (u16)off);
    }
    return buf;
  }


  return NULL;
}

static void print_operand_code_c(str_t *s, dis86_instr_t *ins, operand_t *o)
{
  switch (o->type) {
    case OPERAND_TYPE_REG: str_fmt(s, "%s", as_upper(reg_name(o->u.reg.id))); break;
    case OPERAND_TYPE_MEM: {
      operand_mem_t *m = &o->u.mem;
      const char *var_name = lookup_name(m);
      if (var_name) {
        str_fmt(s, "%s", var_name);
        break; // all done
      }
      switch (m->sz) {
        case SIZE_8:  str_fmt(s, "*(u8*)"); break;
        case SIZE_16: str_fmt(s, "*(u16*)"); break;
        case SIZE_32: str_fmt(s, "*(u32*)"); break;
      }
      str_fmt(s, "(%s:", as_upper(reg_name(m->sreg)));
      if (!m->reg1 && !m->reg2) {
        if (m->off) str_fmt(s, "0x%x", m->off);
      } else {
        if (m->reg1) str_fmt(s, "%s", as_upper(reg_name(m->reg1)));
        if (m->reg2) str_fmt(s, "+%s", as_upper(reg_name(m->reg2)));
        if (m->off) {
          i16 disp = (i16)m->off;
          if (disp >= 0) str_fmt(s, "+0x%x", (u16)disp);
          else           str_fmt(s, "-0x%x", (u16)-disp);
        }
      }
      str_fmt(s, ")");
    } break;
    case OPERAND_TYPE_IMM: str_fmt(s, "0x%x", o->u.imm.val); break;
    case OPERAND_TYPE_REL: {
      u16 effective = ins->addr + ins->n_bytes + o->u.rel.val;
      str_fmt(s, "0x%x", effective);
    } break;
      //case OPERAND_TYPE_FAR: break;
    default: FAIL("INVALID OPERAND TYPE: %d", o->type);
  }
}

char *dis86_decompile(dis86_t *d, dis86_instr_t *ins_arr, size_t n_ins)
{
  char buf[32];

  str_t ret_s[1];
  str_init(ret_s);

  str_fmt(ret_s, "void func(void)\n");
  str_fmt(ret_s, "{\n");

  for (size_t i = 0; i < n_ins; i++) {
    str_t s[1];
    str_init(s);

    dis86_instr_t *ins = &ins_arr[i];

    int type = code_c_type[ins->opcode];
    const char *str = code_c_str[ins->opcode];

    switch (type) {
      case CODE_C_UNKNOWN:   str_fmt(s, "UNKNOWN();"); break;
      case CODE_C_OPERATOR: {
        assert(ins->operand[0].type != OPERAND_TYPE_NONE);
        print_operand_code_c(s, ins, &ins->operand[0]);
        str_fmt(s, " %s ", str);
        if (ins->operand[1].type != OPERAND_TYPE_NONE) {
          print_operand_code_c(s, ins, &ins->operand[1]);
        }
        str_fmt(s, ";");
      } break;
      case CODE_C_FUNCTION: {
        str_fmt(s, "%s(", str);
          for (size_t i = 0; i < ARRAY_SIZE(ins->operand); i++) {
            operand_t *o = &ins->operand[i];
            if (o->type == OPERAND_TYPE_NONE) break;
            if (i != 0) str_fmt(s, ", ");
            print_operand_code_c(s, ins, o);
          }
        str_fmt(s, ");", str);
      } break;
      case CODE_C_RFUNCTION: {
        assert(ins->operand[0].type != OPERAND_TYPE_NONE);
        print_operand_code_c(s, ins, &ins->operand[0]);
        str_fmt(s, " = %s(", str);
          for (size_t i = 1; i < ARRAY_SIZE(ins->operand); i++) {
            operand_t *o = &ins->operand[i];
            if (o->type == OPERAND_TYPE_NONE) break;
            if (i != 1) str_fmt(s, ", ");
            print_operand_code_c(s, ins, o);
          }
        str_fmt(s, ");", str);
      } break;
      case CODE_C_LITERAL: {
        str_fmt(s, "%s", str);
      } break;
      default:
        FAIL("Unknown code type: %d\n", type);
    }

    const char *cs = str_to_cstr(s);
    const char *as = dis86_print_intel_syntax(d, ins, false);
    if (i != 0) str_fmt(ret_s, "\n");
    str_fmt(ret_s, "  %-30s // %s", cs, as);
    free((void*)as);
    free((void*)cs);
  }

  str_fmt(ret_s, "}\n");

  return str_to_cstr(ret_s);
}
