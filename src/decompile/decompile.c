#include "dis86_private.h"
#include "instr_tbl.h"
#include "str.h"
#include "config.h"

#define MAX_LABELS 256

typedef struct labels labels_t;
struct labels
{
  u32 addr[MAX_LABELS];
  size_t n_addr;
};

// FIXME: O(n) search
static bool is_label(labels_t *labels, u32 addr)
{
  for (size_t i = 0; i < labels->n_addr; i++) {
    if (labels->addr[i] == addr) return true;
  }
  return false;
}

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

static u32 branch_destination(dis86_instr_t *ins)
{
  i16 rel = 0;
  switch (ins->opcode) {
    case OP_JO:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNO: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JB:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JAE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JE:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JBE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JA:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JS:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNS: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JP:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JNP: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JL:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JGE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JLE: rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JG:  rel = (i16)ins->operand[0].u.rel.val; break;
    case OP_JMP: rel = (i16)ins->operand[0].u.rel.val; break;
    default: return 0;
  }

  u16 effective = ins->addr + ins->n_bytes + rel;
  return effective;
}

static void find_labels(labels_t *labels, dis86_instr_t *ins_arr, size_t n_ins)
{
  labels->n_addr = 0;

  for (size_t i = 0; i < n_ins; i++) {
    dis86_instr_t *ins = &ins_arr[i];
    u16 dst = branch_destination(ins);
    if (!dst) continue;

    assert(labels->n_addr < ARRAY_SIZE(labels->addr));
    labels->addr[labels->n_addr++] = dst;
  }
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
        case SIZE_8:  str_fmt(s, "*PTR_8("); break;
        case SIZE_16: str_fmt(s, "*PTR_16("); break;
        case SIZE_32: str_fmt(s, "*PTR_32("); break;
      }
      str_fmt(s, "%s, ", as_upper(reg_name(m->sreg)));
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

char *dis86_decompile( dis86_t *                  d,
                       dis86_decompile_config_t * opt_cfg,
                       const char *               func_name,
                       dis86_instr_t *            ins_arr,
                       size_t                     n_ins )
{
  // Default config?
  dis86_decompile_config_t * default_cfg = NULL;
  dis86_decompile_config_t * cfg = opt_cfg;
  if (!cfg) {
    default_cfg = config_default_new();
    cfg = default_cfg;
  }

  config_print(cfg);
  exit(87);

  char buf[32];
  const char *cs, *as;

  labels_t labels[1];
  find_labels(labels, ins_arr, n_ins);

  str_t ret_s[1];
  str_init(ret_s);
  str_fmt(ret_s, "void %s(void)\n", func_name);
  str_fmt(ret_s, "{\n");

  for (size_t i = 0; i < n_ins; i++) {
    str_t s[1];
    str_init(s);

    dis86_instr_t *ins = &ins_arr[i];
    if (is_label(labels, (u32)ins->addr)) {
      str_fmt(ret_s, "\n label_%08x:\n", (u32)ins->addr);
    }

    // Special handling for cmp+jmp
    if (ins->opcode == OP_CMP) {
      const char *oper = NULL;
      int sign = 0;
      if (i+1 < n_ins) {
        dis86_instr_t *next_ins = &ins_arr[i+1];
        switch (next_ins->opcode) {
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
        }
        if (oper) {
          u32 dest = branch_destination(next_ins);
          str_fmt(s, "if (");
          if (sign) str_fmt(s, "(i16)");
          assert(ins->operand[0].type != OPERAND_TYPE_NONE);
          print_operand_code_c(s, ins, &ins->operand[0]);
          str_fmt(s, " %s ", oper);
          if (sign) str_fmt(s, "(i16)");
          assert(ins->operand[1].type != OPERAND_TYPE_NONE);
          print_operand_code_c(s, ins, &ins->operand[1]);
          str_fmt(s, ") goto label_%08x;", dest);

          as = dis86_print_intel_syntax(d, ins, false);
          str_fmt(ret_s, "  %-50s // %s\n", "", as);
          free((void*)as);

          cs = str_to_cstr(s);
          as = dis86_print_intel_syntax(d, next_ins, false);
          str_fmt(ret_s, "  %-50s // %s\n", cs, as);
          free((void*)as);
          free((void*)cs);

          i++; // advance one extra
          continue;
        }
      }
    }

    // Special handling for or reg,reg + je / jne
    if (ins->opcode == OP_OR &&
        ins->operand[0].type == OPERAND_TYPE_REG &&
        ins->operand[1].type == OPERAND_TYPE_REG &&
        ins->operand[0].u.reg.id == ins->operand[1].u.reg.id
    ) {
      const char *oper = NULL;
      if (i+1 < n_ins) {
        dis86_instr_t *next_ins = &ins_arr[i+1];
        switch (next_ins->opcode) {
          case OP_JE:  oper = "=="; break;
          case OP_JNE: oper = "!="; break;
        }
        if (oper) {
          u32 dest = branch_destination(next_ins);
          str_fmt(s, "if (");
          print_operand_code_c(s, ins, &ins->operand[0]);
          str_fmt(s, " %s 0) goto label_%08x;", oper, dest);

          as = dis86_print_intel_syntax(d, ins, false);
          str_fmt(ret_s, "  %-50s // %s\n", "", as);
          free((void*)as);

          cs = str_to_cstr(s);
          as = dis86_print_intel_syntax(d, next_ins, false);
          str_fmt(ret_s, "  %-50s // %s\n", cs, as);
          free((void*)as);
          free((void*)cs);

          i++; // advance one extra
          continue;
        }
      }
    }

    if (ins->opcode == OP_JMP) {
      u32 dest = branch_destination(ins);
      str_fmt(s, "goto label_%08x;", dest);

      cs = str_to_cstr(s);
      as = dis86_print_intel_syntax(d, ins, false);
      str_fmt(ret_s, "  %-50s // %s\n", cs, as);
      free((void*)as);
      free((void*)cs);

      continue;
    }

    if (ins->opcode == OP_LDS || ins->opcode == OP_LES) {
      str_fmt(s, "LOAD_SEG_OFF(");
      print_operand_code_c(s, ins, &ins->operand[0]);
      str_fmt(s, ", ");
      print_operand_code_c(s, ins, &ins->operand[1]);
      str_fmt(s, ", ");
      print_operand_code_c(s, ins, &ins->operand[2]);
      str_fmt(s, ");");

      cs = str_to_cstr(s);
      as = dis86_print_intel_syntax(d, ins, false);
      str_fmt(ret_s, "  %-50s // %s\n", cs, as);
      free((void*)as);
      free((void*)cs);

      continue;
    }

    if (ins->opcode == OP_CALLF) {
      if (ins->operand[0].type == OPERAND_TYPE_FAR) {
        operand_far_t *far = &ins->operand[0].u.far;
        segoff_t addr = {far->seg, far->off};
        const char *name = config_lookup_func(cfg, addr);
        if (name) {
          str_fmt(s, "CALL_FUNC(%s);", name);
        } else {
          str_fmt(s, "CALL_FAR(0x%04x, 0x%04x);", far->seg, far->off);
        }
      }
      // HAX
      else {
        str_fmt(s, "UNKNOWN_CALL_FAR()");
      }

      cs = str_to_cstr(s);
      as = dis86_print_intel_syntax(d, ins, false);
      str_fmt(ret_s, "  %-50s // %s\n", cs, as);
      free((void*)as);
      free((void*)cs);

      continue;
    }

    if (ins->opcode == OP_LEA) {
      assert(ins->operand[0].type == OPERAND_TYPE_REG);
      operand_reg_t *reg = &ins->operand[0].u.reg;

      assert(ins->operand[1].type == OPERAND_TYPE_MEM);
      operand_mem_t *mem = &ins->operand[1].u.mem;
      assert(mem->sz == SIZE_16);
      assert(mem->reg1);
      assert(!mem->reg2);
      assert(mem->off);

      str_fmt(s, "%s", as_upper(reg_name(reg->id)));
      str_fmt(s, " = %s - 0x%x;", as_upper(reg_name(mem->reg1)), -(i16)mem->off);

      cs = str_to_cstr(s);
      as = dis86_print_intel_syntax(d, ins, false);
      str_fmt(ret_s, "  %-50s // %s\n", cs, as);
      free((void*)as);
      free((void*)cs);

      continue;
    }

    if (ins->opcode == OP_IMUL) {
      assert(ins->operand[0].type == OPERAND_TYPE_REG);
      operand_reg_t *reg1 = &ins->operand[0].u.reg;
      assert(ins->operand[1].type == OPERAND_TYPE_REG);
      operand_reg_t *reg2 = &ins->operand[1].u.reg;
      assert(ins->operand[2].type == OPERAND_TYPE_IMM);
      operand_imm_t *imm = &ins->operand[2].u.imm;

      str_fmt(s, "%s = (i16)%s * (i16)0x%x;", as_upper(reg_name(reg1->id)),
              as_upper(reg_name(reg2->id)), imm->val);

      cs = str_to_cstr(s);
      as = dis86_print_intel_syntax(d, ins, false);
      str_fmt(ret_s, "  %-50s // %s\n", cs, as);
      free((void*)as);
      free((void*)cs);

      continue;
    }

    /////////////////
    // GENERIC

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

    cs = str_to_cstr(s);
    as = dis86_print_intel_syntax(d, ins, false);
    str_fmt(ret_s, "  %-50s // %s\n", cs, as);
    free((void*)as);
    free((void*)cs);
  }

  str_fmt(ret_s, "}\n");

  if (default_cfg) config_delete(default_cfg);
  return str_to_cstr(ret_s);
}
